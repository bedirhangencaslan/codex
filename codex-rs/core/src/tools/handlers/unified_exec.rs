use crate::sandboxing::SandboxPermissions;
use crate::shell::Shell;
use crate::shell::ShellType;
use crate::shell::get_shell;
use crate::shell::get_shell_by_model_provided_path;
use crate::tools::context::ToolInvocation;
use crate::tools::context::ToolOutput;
use crate::tools::context::ToolPayload;
use crate::tools::hook_names::HookToolName;
use crate::tools::registry::PostToolUsePayload;
use codex_exec_server::Environment;
use codex_protocol::models::AdditionalPermissionProfile;
use codex_shell_command::bash::try_parse_shell;
use codex_tools::UnifiedExecShellMode;
use serde::Deserialize;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

#[cfg(test)]
use crate::tools::handlers::parse_arguments;

mod exec_command;
mod write_stdin;

pub use exec_command::ExecCommandHandler;
pub(crate) use exec_command::ExecCommandHandlerOptions;
pub use write_stdin::WriteStdinHandler;

#[derive(Debug, Deserialize)]
pub(crate) struct ExecCommandArgs {
    pub(crate) cmd: String,
    #[serde(default)]
    shell: Option<String>,
    #[serde(default)]
    login: Option<bool>,
    #[serde(default = "default_tty")]
    tty: bool,
    #[serde(default = "default_exec_yield_time_ms")]
    yield_time_ms: u64,
    #[serde(default)]
    timeout_ms: Option<u64>,
    #[serde(default)]
    max_output_tokens: Option<usize>,
    #[serde(default)]
    sandbox_permissions: Option<SandboxPermissions>,
    #[serde(default)]
    additional_permissions: Option<AdditionalPermissionProfile>,
    #[serde(default)]
    justification: Option<String>,
    #[serde(default)]
    prefix_rule: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct ExecCommandEnvironmentArgs {
    #[serde(default)]
    environment_id: Option<String>,
    // Keep this raw until after environment selection; relative paths must be
    // resolved against the selected environment cwd, not the process cwd.
    #[serde(default)]
    workdir: Option<String>,
}

fn default_exec_yield_time_ms() -> u64 {
    10_000
}

fn default_write_stdin_yield_time_ms() -> u64 {
    250
}

fn default_tty() -> bool {
    false
}

#[derive(Debug)]
pub(crate) struct ResolvedCommand {
    pub(crate) command: Vec<String>,
    pub(crate) shell_type: ShellType,
}

fn post_unified_exec_tool_use_payload(
    invocation: &ToolInvocation,
    result: &dyn ToolOutput,
) -> Option<PostToolUsePayload> {
    let ToolPayload::Function { .. } = &invocation.payload else {
        return None;
    };

    let tool_input = result.post_tool_use_input(&invocation.payload)?;
    let tool_use_id = result.post_tool_use_id(&invocation.call_id);
    let tool_response = result.post_tool_use_response(&tool_use_id, &invocation.payload)?;
    Some(PostToolUsePayload {
        tool_name: HookToolName::bash(),
        tool_use_id,
        tool_input,
        tool_response,
    })
}

pub(crate) fn get_command(
    args: &ExecCommandArgs,
    session_shell: Arc<Shell>,
    shell_mode: &UnifiedExecShellMode,
    allow_login_shell: bool,
) -> Result<ResolvedCommand, String> {
    let use_login_shell = match args.login {
        Some(true) if !allow_login_shell => {
            return Err(
                "login shell is disabled by config; omit `login` or set it to false.".to_string(),
            );
        }
        Some(use_login_shell) => use_login_shell,
        None => allow_login_shell,
    };

    match shell_mode {
        UnifiedExecShellMode::Direct => {
            let model_shell = args
                .shell
                .as_ref()
                .map(|shell_str| get_shell_by_model_provided_path(&PathBuf::from(shell_str)))
                .or_else(|| heredoc_fallback_shell(&args.cmd, session_shell.as_ref()));
            let shell = model_shell.as_ref().unwrap_or(session_shell.as_ref());
            Ok(ResolvedCommand {
                command: shell.derive_exec_args(&args.cmd, use_login_shell),
                shell_type: shell.shell_type,
            })
        }
        UnifiedExecShellMode::ZshFork(zsh_fork_config) => {
            if args.shell.is_some() {
                return Err(
                    "`shell` is not supported for local zsh-fork exec; omit `shell` to use zsh-fork, or target a remote environment where `shell` is supported.".to_string(),
                );
            }

            Ok(ResolvedCommand {
                command: vec![
                    zsh_fork_config.shell_zsh_path.to_string_lossy().to_string(),
                    if use_login_shell { "-lc" } else { "-c" }.to_string(),
                    args.cmd.clone(),
                ],
                shell_type: ShellType::Zsh,
            })
        }
    }
}

/// Runs a here-document under bash when the model did not name a shell itself.
///
/// PowerShell reserves `<<` as a redirection operator, so a here-document fails to parse before
/// the command runs and no PowerShell invocation can be asking for one. The model already passes
/// `shell` for most here-documents; this covers the rest instead of handing back a syntax error.
/// It stays out of the way whenever the model chose a shell, and falls through to the session
/// shell when no usable bash is installed.
fn heredoc_fallback_shell(cmd: &str, session_shell: &Shell) -> Option<Shell> {
    if session_shell.shell_type != ShellType::PowerShell || !contains_heredoc(cmd) {
        return None;
    }

    let bash = get_shell(ShellType::Bash)?;
    if is_windows_bash_shim(&bash.shell_path) {
        return None;
    }
    Some(bash)
}

/// Whether `cmd` redirects a here-document into a command.
///
/// Asks the bash grammar the workspace already ships rather than scanning for `<<`: only a parse
/// separates a redirection from the same two characters inside a quoted body, and
/// `Set-Content out.cpp -Value 'std::cout << x'` is a valid PowerShell command that has to keep
/// running there. A script the grammar cannot parse simply reports no here-document, which leaves
/// the command in the session shell.
fn contains_heredoc(cmd: &str) -> bool {
    let Some(tree) = try_parse_shell(cmd) else {
        return false;
    };

    let mut stack = vec![tree.root_node()];
    while let Some(node) = stack.pop() {
        if node.kind() == "heredoc_redirect" {
            return true;
        }

        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            stack.push(child);
        }
    }

    false
}

/// Whether `path` is one of Windows' own `bash.exe` shims rather than a real bash.
///
/// `System32\bash.exe` and the `WindowsApps` execution alias both launch WSL, which has its own
/// filesystem: the Windows cwd this command runs in, and the paths it carries, mean nothing there.
/// Shell lookup takes whichever `bash` comes first on PATH, so on a machine that orders those
/// ahead of Git for Windows the here-document would quietly run against the wrong filesystem.
/// Decline, and let it fail in the session shell the way it did before this fallback existed —
/// only a bash that shares the filesystem is an improvement.
fn is_windows_bash_shim(path: &Path) -> bool {
    const WINDOWS_SHIM_DIRS: [&str; 4] = ["system32", "sysnative", "syswow64", "windowsapps"];

    let normalized = path
        .to_string_lossy()
        .to_ascii_lowercase()
        .replace('\\', "/");
    WINDOWS_SHIM_DIRS
        .iter()
        .any(|dir| normalized.contains(&format!("/{dir}/")))
}

pub(crate) fn shell_mode_for_environment(
    turn_shell_mode: &UnifiedExecShellMode,
    environment: &Environment,
) -> UnifiedExecShellMode {
    if environment.is_remote() {
        UnifiedExecShellMode::Direct
    } else {
        turn_shell_mode.clone()
    }
}

#[cfg(test)]
#[path = "unified_exec_tests.rs"]
mod tests;
