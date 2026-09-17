use codex_tools::JsonSchema;
use codex_tools::ResponsesApiTool;
use codex_tools::ToolSpec;
use serde_json::Value;
use serde_json::json;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandToolOptions {
    pub allow_login_shell: bool,
    pub exec_permission_approvals_enabled: bool,
    /// Advertise only the parameters a non-interactive run can act on.
    ///
    /// Measured: this tool's spec is the single largest influence on how much of a file the model
    /// asks `read` for, and it acts whether or not the tool is ever called - the spec rides the
    /// fixed prefix of every request. Replacing the whole spec with OpenCode's three-parameter
    /// `bash` took reads from the whole file to 45% of it and the median window from 220 lines to
    /// 80, six replays across two independent batches landing in 0.39-0.45 with no overlap against
    /// a base at 0.85-1.00. Nothing smaller reproduced it: OpenCode's description on our schema
    /// moved 0.79 / 1.00 / 1.00, dropping `max_output_tokens` alone moved nothing, and dropping the
    /// tool entirely moved nothing. `_sim/probe.py`, arms `suf+swap:exec_command=bash` and below.
    ///
    /// Nothing is lost by not advertising them. Every field of `ExecCommandArgs` except `cmd` is
    /// `#[serde(default)]` (`unified_exec.rs:30-53`), so a parameter left out of the schema simply
    /// takes its default, and the handler is untouched.
    pub lean_parameters: bool,
}

#[cfg(test)]
pub fn create_exec_command_tool(options: CommandToolOptions) -> ToolSpec {
    create_exec_command_tool_with_environment_id(
        options,
        /*include_environment_id*/ false,
        /*include_shell_parameter*/ true,
        /*include_windows_shell_guidance*/ cfg!(windows),
    )
}

pub(crate) fn create_exec_command_tool_with_environment_id(
    options: CommandToolOptions,
    include_environment_id: bool,
    include_shell_parameter: bool,
    include_windows_shell_guidance: bool,
) -> ToolSpec {
    if options.lean_parameters {
        return lean_exec_command_tool(include_environment_id, include_windows_shell_guidance);
    }

    let yield_time_ms_description = if cfg!(windows) {
        "Maximum time to wait before returning a session ID for a still-running command. Commands that finish sooner return immediately. For ordinary commands, omit this parameter to use the 10000 ms default. Effective range on Windows is 10000-30000 ms."
    } else {
        "Wait before yielding output. Defaults to 10000 ms; effective range is 250-30000 ms."
    };
    let mut properties = BTreeMap::from([
        (
            "cmd".to_string(),
            JsonSchema::string(Some("Shell command to execute.".to_string())),
        ),
        (
            "workdir".to_string(),
            JsonSchema::string(Some(
                "Working directory for the command. Defaults to the turn cwd."
                    .to_string(),
            )),
        ),
        (
            "tty".to_string(),
            JsonSchema::boolean(Some(
                "True allocates a PTY for the command; false or omitted uses plain pipes."
                    .to_string(),
            )),
        ),
        (
            "yield_time_ms".to_string(),
            JsonSchema::number(Some(yield_time_ms_description.to_string())),
        ),
        (
            "max_output_tokens".to_string(),
            // Not "token budget": on two measured runs the model read that phrase as its *context*
            // budget - "we have token budget 19k only" - and rationed its `read` windows against a
            // ceiling that belongs to one command's output. The window was three times larger.
            JsonSchema::number(Some(
                "Cap on this one command's output, in tokens. Defaults to 10000; larger requests may be capped by policy. It bounds this command only and says nothing about how much conversation context remains.".to_string(),
            )),
        ),
    ]);
    if include_shell_parameter {
        properties.insert(
            "shell".to_string(),
            JsonSchema::string(Some(
                "Shell binary to launch. Defaults to the user's default shell.".to_string(),
            )),
        );
    }
    if options.allow_login_shell {
        properties.insert(
            "login".to_string(),
            JsonSchema::boolean(Some(
                "True runs the shell with -l/-i semantics; false disables them. Defaults to true."
                    .to_string(),
            )),
        );
    }
    if include_environment_id {
        properties.insert(
            "environment_id".to_string(),
            JsonSchema::string(Some(
                "Environment id from <environment_context>. Omit to use the primary environment."
                    .to_string(),
            )),
        );
    }
    properties.extend(create_approval_parameters(
        options.exec_permission_approvals_enabled,
    ));

    ToolSpec::Function(ResponsesApiTool {
        name: "exec_command".to_string(),
        description: if include_windows_shell_guidance {
            format!(
                "Runs a command in a PTY, returning output or a session ID for ongoing interaction.\n\n{}\n\n{}",
                file_work_guidance(),
                windows_shell_guidance()
            )
        } else {
            format!(
                "Runs a command in a PTY, returning output or a session ID for ongoing interaction.\n\n{}",
                file_work_guidance()
            )
        },
        strict: false,
        defer_loading: None,
        parameters: JsonSchema::object(
            properties,
            Some(vec!["cmd".to_string()]),
            Some(false.into()),
        ),
        output_schema: Some(unified_exec_output_schema()),
    })
}

/// `exec_command` with the three parameters a non-interactive run can act on, and a description
/// written the way OpenCode writes `bash`'s.
///
/// The seven that go: `shell` and `login` (the environment's own shell is the right one),
/// `tty` and `yield_time_ms` (a resumable session needs a caller who will come back to it),
/// `max_output_tokens` (the cap belongs to the harness, and this model has twice been seen reading
/// a per-command output cap as its *context* budget), and `sandbox_permissions` / `justification` /
/// `prefix_rule` (an escalation nobody is present to approve). All still deserialize; none is
/// offered.
///
/// The opening sentence is kept verbatim because `one_shot_exec_command_spec` rewrites it by
/// matching on it (`unified_exec/exec_command.rs:459-463`).
fn lean_exec_command_tool(
    include_environment_id: bool,
    include_windows_shell_guidance: bool,
) -> ToolSpec {
    let mut properties = BTreeMap::from([
        (
            "cmd".to_string(),
            JsonSchema::string(Some("Shell command to execute.".to_string())),
        ),
        (
            "workdir".to_string(),
            JsonSchema::string(Some(
                "Working directory for the command. Defaults to the turn cwd.".to_string(),
            )),
        ),
        (
            "timeout_ms".to_string(),
            JsonSchema::number(Some(
                "Maximum command runtime. Defaults to 10000 ms.".to_string(),
            )),
        ),
    ]);
    if include_environment_id {
        properties.insert(
            "environment_id".to_string(),
            JsonSchema::string(Some(
                "Environment id from <environment_context>. Omit to use the primary environment."
                    .to_string(),
            )),
        );
    }

    let description = if include_windows_shell_guidance {
        format!(
            "Runs a command in a PTY, returning output or a session ID for ongoing interaction.\n\n{}\n\n{}",
            lean_usage_guidance(),
            windows_shell_guidance()
        )
    } else {
        format!(
            "Runs a command in a PTY, returning output or a session ID for ongoing interaction.\n\n{}",
            lean_usage_guidance()
        )
    };

    ToolSpec::Function(ResponsesApiTool {
        name: "exec_command".to_string(),
        description,
        strict: false,
        defer_loading: None,
        parameters: JsonSchema::object(
            properties,
            Some(vec!["cmd".to_string()]),
            Some(false.into()),
        ),
        output_schema: Some(unified_exec_output_schema()),
    })
}

/// OpenCode's `bash` description, section for section, with our tool names and our shell.
///
/// Kept deliberately: its anti-shell list is the part that was already measured to collapse shell
/// file work from 79,052 characters to 635, and it is more specific than OpenCode's because it
/// names the cmdlets. What is new is everything around it - the shell notes, the pre-flight steps,
/// the usage notes and the git section - which is the shape the measurement moved, not any single
/// sentence in it.
fn lean_usage_guidance() -> &'static str {
    r#"Be aware: OS: win32, Shell: powershell

All commands run in the turn's working directory by default. Use the `workdir` parameter if you need to run a command in a different directory. AVOID changing directories inside the command - use `workdir` instead.

IMPORTANT: This tool is for terminal operations such as `git`, `cargo`, `npm`, and `docker`. Do NOT use it for file operations - reading, writing, editing, searching, or finding files. Use the dedicated tools for those instead.

# Windows PowerShell (5.1) shell notes
- Use `cmd1; if ($?) { cmd2 }` to chain dependent commands; `&&` and `||` do not exist in this shell.
- Use double quotes for interpolated strings (`"Hello $name"`), single quotes for verbatim strings.
- Prefer full cmdlet names like `Get-ChildItem`, `Set-Content`, `Remove-Item`, and `New-Item` over aliases.
- Use `$(...)` for subexpressions. Use `@(...)` for array expressions.
- To call a native executable whose path contains spaces, use the call operator: `& "path/to/exe" args`.
- An argument to a native program loses its inner double quotes: `python -c 'print("hi")'` arrives as `print(hi)`. Quote it the other way round: `python -c "print('hi')"`. A payload that needs double quotes of its own cannot be escaped into one: `\"` ends the string rather than escaping the quote. Pipe it in instead: `@'` on its own line, the script, then `'@ | python -`.
- Here-documents do not exist; `<<` is a reserved operator that was never implemented. The here-string `@'...'@` is a different construct; it does exist, and nothing inside it is interpreted.
- Escape special characters with the PowerShell backtick character.

Before executing the command, please follow these steps:

1. Directory Verification:
   - If the command will create new directories or files, first use `Test-Path -LiteralPath <parent>` to verify the parent directory exists and is the correct location
   - For example, before creating `foo\bar`, first use `Test-Path -LiteralPath "foo"` to check that `foo` exists and is the intended parent directory

2. Command Execution:
   - Always quote file paths that contain spaces with double quotes (e.g., Remove-Item -LiteralPath "path with spaces\file.txt")
   - Examples of proper quoting:
     - New-Item -ItemType Directory -Path "My Documents" (correct)
     - New-Item -ItemType Directory -Path My Documents (incorrect - path is split)
     - & "path with spaces\script.ps1" (correct)
     - path with spaces\script.ps1 (incorrect - path is split and not invoked)
   - After ensuring proper quoting, execute the command.
   - Capture the output of the command.

Usage notes:
  - The `cmd` argument is required.
  - You can specify an optional `timeout_ms`. If not specified, commands time out after 10000 ms.
  - Output is capped and truncated at that point, and the full output is written to a file whose path the response names. Use `read` with `offset`/`limit` on that path to see a section of it, or `grep` to search it; do NOT use `Select-Object -First`, `Select-Object -Last`, or other commands that trim output, because the whole of it is in that file already. Do NOT pull bulk file text through here to work around the cap either: `read` and `grep` return the same content in windows you choose. Whatever comes back through here stays in context for every later request, so a listing or a bulk read taken this way is paid for again on each one.
  - Avoid using this tool with the file and content commands listed below unless explicitly instructed, or when one of them is truly necessary for the task. Instead, always prefer using the dedicated tools for these commands:
    - File search: Use `glob` (NOT `Get-ChildItem`, `ls`, or `find`)
    - Content search: Use `grep` (NOT `Select-String` or `rg`)
    - Read files: Use `read` (NOT `Get-Content`, `cat`, `head`, or `tail`)
    - Edit or create a file whose content you are writing yourself: Use `apply_patch` (NOT `Set-Content`, `sed`, or `awk`). A program that computes its own output - a generator, a formatter - writes its file itself and needs none of this.
    - Communication: Output text directly (NOT `Write-Output`/`Write-Host`)
  - Reach for the shell on file work only for what the dedicated tools do not do.
  - When issuing multiple commands:
    - If the commands are independent and can run in parallel, make multiple `exec_command` calls in a single message.
    - If the commands depend on each other and must run sequentially, use `cmd1; if ($?) { cmd2 }` rather than `;` alone, because PowerShell does not stop at the first failure and reports only the last statement's exit code.
    - Use `;` only when you need to run commands sequentially but don't care if earlier commands fail.
    - DO NOT use newlines to separate commands (newlines are ok in quoted strings).
  - AVOID changing directories inside the command. Use the `workdir` parameter to change directories instead.
    <good-example>
    Use workdir="project\subdir" with cmd: cargo test
    </good-example>
    <bad-example>
    Set-Location -LiteralPath "project\subdir"; if ($?) { cargo test }
    </bad-example>

# Git and GitHub
- Only commit, amend, push, or create PRs when explicitly requested.
- Before committing, inspect `git status`, `git diff`, and `git log --oneline -10`; stage only intended files and never commit secrets.
- Write a concise commit message that matches the repo style.
- Do not update git config, skip hooks, use interactive `-i`, force-push, or create empty commits unless explicitly requested.
- If a commit fails or hooks reject it, fix the issue and create a new commit; do not amend the failed commit.
- Before creating a PR, inspect status, diff, remote tracking, recent commits, and the diff from the base branch.
- Review all commits included in the PR, not just the latest commit.
- Use `gh` for GitHub tasks, including PRs, issues, checks, and releases; return the PR URL when done."#
}

pub fn create_write_stdin_tool() -> ToolSpec {
    let properties = BTreeMap::from([
        (
            "session_id".to_string(),
            JsonSchema::number(Some(
                "Identifier of the running unified exec session.".to_string(),
            )),
        ),
        (
            "chars".to_string(),
            JsonSchema::string(Some(
                "Bytes to write to stdin. Defaults to empty, which polls without writing.".to_string(),
            )),
        ),
        (
            "yield_time_ms".to_string(),
            JsonSchema::number(Some(
                "Wait before yielding output. Non-empty writes default to 250 ms and cap at 30000 ms; empty polls wait 5000-300000 ms by default.".to_string(),
            )),
        ),
        (
            "max_output_tokens".to_string(),
            // Not "token budget": on two measured runs the model read that phrase as its *context*
            // budget - "we have token budget 19k only" - and rationed its `read` windows against a
            // ceiling that belongs to one command's output. The window was three times larger.
            JsonSchema::number(Some(
                "Cap on this one command's output, in tokens. Defaults to 10000; larger requests may be capped by policy. It bounds this command only and says nothing about how much conversation context remains.".to_string(),
            )),
        ),
    ]);

    ToolSpec::Function(ResponsesApiTool {
        name: "write_stdin".to_string(),
        description:
            "Writes characters to an existing unified exec session and returns recent output."
                .to_string(),
        strict: false,
        defer_loading: None,
        parameters: JsonSchema::object(
            properties,
            Some(vec!["session_id".to_string()]),
            Some(false.into()),
        ),
        output_schema: Some(unified_exec_output_schema()),
    })
}

pub fn create_request_permissions_tool(description: String) -> ToolSpec {
    let properties = BTreeMap::from([
        (
            "reason".to_string(),
            JsonSchema::string(Some(
                "Optional short explanation for why additional permissions are needed.".to_string(),
            )),
        ),
        (
            "environment_id".to_string(),
            JsonSchema::string(Some(
                "Environment id from <environment_context>. Omit to use the primary environment."
                    .to_string(),
            )),
        ),
        ("permissions".to_string(), permission_profile_schema()),
    ]);

    ToolSpec::Function(ResponsesApiTool {
        name: "request_permissions".to_string(),
        description,
        strict: false,
        defer_loading: None,
        parameters: JsonSchema::object(
            properties,
            Some(vec!["permissions".to_string()]),
            Some(false.into()),
        ),
        output_schema: None,
    })
}

pub fn request_permissions_tool_description() -> String {
    "Request additional filesystem or network permissions from the user and wait for the client to grant a subset of the requested permission profile. Use environment_id to target a specific attached environment; omit it to use the primary environment. Relative filesystem paths resolve against the selected environment cwd. Granted permissions apply automatically to later shell-like commands in the current turn, or for the rest of the session if the client approves them at session scope."
        .to_string()
}

fn unified_exec_output_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "chunk_id": {
                "type": "string",
                "description": "Chunk identifier included when the response reports one."
            },
            "wall_time_seconds": {
                "type": "number",
                "description": "Elapsed wall time spent waiting for output in seconds."
            },
            "exit_code": {
                "type": "number",
                "description": "Process exit code when the command finished during this call."
            },
            "session_id": {
                "type": "number",
                "description": "Session identifier to pass to write_stdin when the process is still running."
            },
            "original_token_count": {
                "type": "number",
                "description": "Approximate token count before output truncation."
            },
            "output": {
                "type": "string",
                "description": "Command output text, possibly truncated."
            }
        },
        "required": ["wall_time_seconds", "output"],
        "additionalProperties": false
    })
}

fn create_approval_parameters(
    exec_permission_approvals_enabled: bool,
) -> BTreeMap<String, JsonSchema> {
    let mut sandbox_permission_values = vec![json!("use_default")];
    if exec_permission_approvals_enabled {
        sandbox_permission_values.push(json!("with_additional_permissions"));
    }
    sandbox_permission_values.push(json!("require_escalated"));
    let sandbox_permissions_description = if exec_permission_approvals_enabled {
        "Per-command sandbox override. Defaults to `use_default`; use `with_additional_permissions` with `additional_permissions`, or `require_escalated` for unsandboxed execution."
    } else {
        "Per-command sandbox override. Defaults to `use_default`; use `require_escalated` for unsandboxed execution."
    };

    let mut properties = BTreeMap::from([
        (
            "sandbox_permissions".to_string(),
            JsonSchema::string_enum(
                sandbox_permission_values,
                Some(sandbox_permissions_description.to_string()),
            ),
        ),
        (
            "justification".to_string(),
            JsonSchema::string(Some(
                "User-facing approval question for `require_escalated`; omit otherwise.".to_string(),
            )),
        ),
        (
            "prefix_rule".to_string(),
            JsonSchema::array(JsonSchema::string(/*description*/ None), Some(
                    r#"Reusable approval prefix for `cmd`, only with `sandbox_permissions: "require_escalated"`; for example ["git", "pull"]."#.to_string(),
                )),
        ),
    ]);

    if exec_permission_approvals_enabled {
        let mut additional_permissions = permission_profile_schema();
        additional_permissions.description = Some(
            "Sandboxed filesystem or network access for this command; only with `sandbox_permissions: \"with_additional_permissions\"`."
                .to_string(),
        );
        properties.insert("additional_permissions".to_string(), additional_permissions);
    }

    properties
}

fn permission_profile_schema() -> JsonSchema {
    let mut schema = JsonSchema::object(
        BTreeMap::from([
            ("network".to_string(), network_permissions_schema()),
            ("file_system".to_string(), file_system_permissions_schema()),
        ]),
        /*required*/ None,
        Some(false.into()),
    );
    schema.description = Some("Filesystem or network access request.".to_string());
    schema
}

fn network_permissions_schema() -> JsonSchema {
    let mut schema = JsonSchema::object(
        BTreeMap::from([(
            "enabled".to_string(),
            JsonSchema::boolean(Some(
                "True requests network access; false or omitted requests none.".to_string(),
            )),
        )]),
        /*required*/ None,
        Some(false.into()),
    );
    schema.description = Some("Network access request.".to_string());
    schema
}

fn file_system_permissions_schema() -> JsonSchema {
    let mut schema = JsonSchema::object(
        BTreeMap::from([
            (
                "read".to_string(),
                JsonSchema::array(
                    JsonSchema::string(/*description*/ None),
                    Some(
                        "Absolute paths to grant read access; omit when none are needed."
                            .to_string(),
                    ),
                ),
            ),
            (
                "write".to_string(),
                JsonSchema::array(
                    JsonSchema::string(/*description*/ None),
                    Some(
                        "Absolute paths to grant write access; omit when none are needed."
                            .to_string(),
                    ),
                ),
            ),
        ]),
        /*required*/ None,
        Some(false.into()),
    );
    schema.description = Some("Filesystem access request.".to_string());
    schema
}

/// The shell is the cheapest wrong answer for file work: its output arrives whole, under this
/// tool's own cap rather than `read`'s per-file budget, and it stays in context for the rest of the
/// session. Measured on the 44-file reading task, the model reached for `rg`, then `Get-Content
/// -TotalCount`, then `Select-String`, and pulled 79 KB of file text through here while `read`,
/// `glob` and `grep` sat unused - the tool descriptions offered them, but nothing said the shell was
/// the wrong door. OpenCode's `bash` description says exactly that, by command name, and its model
/// does not make this trade. This is that paragraph, with our tool names.
fn file_work_guidance() -> &'static str {
    r#"IMPORTANT: This tool is for terminal operations such as `git`, `cargo`, `npm`, and `docker`. Do NOT use it for file operations - reading, writing, editing, searching, or finding files. Use the dedicated tools for those instead:
- File search: use `glob` (NOT `Get-ChildItem`, `ls`, or `find`)
- Content search: use `grep` (NOT `Select-String` or `rg`)
- Read files: use `read` (NOT `Get-Content`, `cat`, `head`, or `tail`)
- Edit or create files: use `apply_patch` (NOT `Set-Content`, `sed`, `awk`, or output redirection)
Reach for the shell on file work only for what the dedicated tools do not do. Whatever comes back through here stays in context for every later request, so a listing or a bulk read taken this way is paid for again on each one.
If output is capped, the full output is written to a file and the response names its path; read a section of it with `read` and `offset`/`limit`, or search it with `grep`."#
}

fn windows_shell_guidance() -> &'static str {
    r#"Windows safety rules:
- Do not compose destructive filesystem commands across shells. Do not enumerate paths in PowerShell and then pass them to `cmd /c`, batch builtins, or another shell for deletion or moving. Use one shell end-to-end, prefer native PowerShell cmdlets such as `Remove-Item` / `Move-Item` with `-LiteralPath`, and avoid string-built shell commands for file operations.
- Before any recursive delete or move on Windows, verify the resolved absolute target paths stay within the intended workspace or explicitly named target directory. Never issue a recursive delete or move against a computed path if the final target has not been checked.
- When using `Start-Process` to launch a background helper or service, pass `-WindowStyle Hidden` unless the user explicitly asked for a visible interactive window. Use visible windows only for interactive tools the user needs to see or control."#
}

#[cfg(test)]
#[path = "shell_spec_tests.rs"]
mod tests;
