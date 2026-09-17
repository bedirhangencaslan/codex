//! Unified Exec: interactive process execution orchestrated with approvals + sandboxing.
//!
//! Responsibilities
//! - Manages interactive processes (create, reuse, buffer output with caps).
//! - Supports completion-only calls that terminate on timeout or cancellation.
//! - Uses the shared ToolOrchestrator to handle approval, sandbox selection, and
//!   retry semantics in a single, descriptive flow.
//! - Spawns the PTY from a sandbox-transformed `ExecRequest`; on sandbox denial,
//!   retries without sandbox when policy allows (no re‑prompt thanks to caching).
//! - Uses the shared `is_likely_sandbox_denied` heuristic to keep denial messages
//!   consistent with other exec paths.
//!
//! Flow at a glance (open process)
//! 1) Build a small request `{ command, cwd }`.
//! 2) Orchestrator: approval (bypass/cache/prompt) → select sandbox → run.
//! 3) Runtime: transform `SandboxTransformRequest` -> `ExecRequest` -> spawn PTY.
//! 4) If denial, orchestrator retries with `SandboxType::None`.
//! 5) Process handle is returned with streaming output + metadata.
//!
//! This keeps policy logic and user interaction centralized while the PTY/process
//! concerns remain isolated here. The implementation is split between:
//! - `process.rs`: PTY process lifecycle + output buffering.
//! - `process_state.rs`: shared exit/failure state for local and remote processes.
//! - `process_manager.rs`: orchestration (approvals, sandboxing, reuse) and request handling.

use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::OnceLock;
use std::sync::Weak;
use std::time::Duration;
use std::time::SystemTime;

use codex_exec_server::FileSystemSandboxContext;
use codex_network_proxy::NetworkProxy;
use codex_protocol::ThreadId;
use codex_protocol::models::AdditionalPermissionProfile;
use codex_protocol::models::PermissionProfile;
use codex_tools::UnifiedExecShellMode;
use codex_utils_output_truncation::TruncationPolicy;
use codex_utils_path_uri::PathUri;
use rand::Rng;
use rand::rng;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use crate::sandboxing::SandboxPermissions;
use crate::session::session::Session;
use crate::session::step_context::StepContext;
use crate::session::turn_context::TurnContext;
use crate::session::turn_context::TurnEnvironment;
use crate::shell::ShellType;
use crate::tools::network_approval::DeferredNetworkApproval;
use codex_core_plugins::PluginMetricsSidecar;

mod async_watcher;
mod errors;
mod head_tail_buffer;
mod oneshot;
mod process;
mod process_manager;
mod process_state;
mod shell_snapshot;
mod stdin_approval;

pub(crate) fn set_deterministic_process_ids_for_tests(enabled: bool) {
    process_manager::set_deterministic_process_ids_for_tests(enabled);
}

/// Directory name under `<codex_home>/tmp` holding output the model's budget cut.
pub(crate) const TOOL_OUTPUT_SUBDIR: &str = "tool-output";

/// How long a spill directory outlives the session that wrote it.
///
/// These files exist for the model to read back inside the same session, so nothing needs them
/// afterwards. The window is only wide enough that a crashed session's files are still there if
/// someone goes looking for them the same day.
const TOOL_OUTPUT_MAX_AGE: Duration = Duration::from_secs(24 * 60 * 60);

static TOOL_OUTPUT_SWEPT: OnceLock<()> = OnceLock::new();

/// Where this session keeps the untruncated output of commands whose results were cut.
///
/// Under `codex_home` rather than the workspace, for two reasons that are both load-bearing:
/// `<workspace>/.suffice` is read-only by default even before it exists, so the write would
/// fail; and a harness log inside a user's repository turns up in their `git status`.
/// `<codex_home>/tmp` is the established home for this kind of state - `arg0` already keeps its
/// scratch there and sweeps it - and `read` reaches it under every profile in normal use,
/// because `workspace-write` narrows writes, not reads.
///
/// Returns `None` when the turn's policy would refuse to read the file, in which case no notice
/// is printed either. A path the model cannot open is worse than no path: it is the same failure
/// as telling the model to run `rg` on a host that has no `rg`, where the harness advertised a
/// capability the environment did not have and the model spent a request finding out.
pub(crate) fn tool_output_spill_dir(
    codex_home: &Path,
    thread_id: ThreadId,
    sandbox: &FileSystemSandboxContext,
    cwd: &Path,
) -> Option<PathBuf> {
    let root = codex_home.join("tmp").join(TOOL_OUTPUT_SUBDIR);
    // Once per process, on the first command that could spill. `arg0` sweeps its own scratch at
    // startup but does not depend on this crate, and a sweep here costs one directory read on a
    // path that is about to be written anyway.
    TOOL_OUTPUT_SWEPT.get_or_init(|| sweep_stale_tool_output(&root, TOOL_OUTPUT_MAX_AGE));
    let dir = root.join(thread_id.to_string());
    let profile = PermissionProfile::try_from(sandbox.permissions.clone()).ok()?;
    let policy = profile.file_system_sandbox_policy();
    (policy.has_full_disk_read_access() || policy.can_read_path_with_cwd(&dir, cwd)).then_some(dir)
}

/// Removes spill directories left behind by sessions that are no longer running.
///
/// Deleting at the end of a session cannot be the only mechanism, because a crash never reaches
/// it - that is how OpenCode's equivalent directory reached 63 GB on a user's machine with a
/// seven-day sweep that was never wired up. This runs at startup and bounds the worst case at
/// one day's worth of one session, whatever happens to the process.
fn sweep_stale_tool_output(root: &Path, max_age: Duration) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    let now = SystemTime::now();
    for entry in entries.flatten() {
        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        if !metadata.is_dir() {
            continue;
        }
        let stale = metadata
            .modified()
            .ok()
            .and_then(|modified| now.duration_since(modified).ok())
            .is_some_and(|age| age > max_age);
        if stale {
            let _ = std::fs::remove_dir_all(entry.path());
        }
    }
}

pub(crate) use errors::UnifiedExecError;
pub(crate) use process::NoopSpawnLifecycle;
#[cfg(unix)]
pub(crate) use process::SpawnLifecycle;
pub(crate) use process::SpawnLifecycleHandle;
pub(crate) use process::UnifiedExecProcess;
pub(crate) use stdin_approval::TerminalPermissions;
pub(crate) use stdin_approval::TerminalSandboxSource;

pub(crate) const MIN_YIELD_TIME_MS: u64 = 250;
pub(crate) const WINDOWS_INITIAL_EXEC_YIELD_TIME_FLOOR_MS: u64 = 10_000;
// Minimum yield time for an empty `write_stdin`.
pub(crate) const MIN_EMPTY_YIELD_TIME_MS: u64 = 5_000;
pub(crate) const MAX_YIELD_TIME_MS: u64 = 30_000;
pub(crate) const DEFAULT_MAX_BACKGROUND_TERMINAL_TIMEOUT_MS: u64 = 300_000;
pub(crate) const DEFAULT_MAX_OUTPUT_TOKENS: usize = 10_000;
pub(crate) const UNIFIED_EXEC_OUTPUT_MAX_BYTES: usize = 1024 * 1024; // 1 MiB
pub(crate) const UNIFIED_EXEC_OUTPUT_MAX_TOKENS: usize = UNIFIED_EXEC_OUTPUT_MAX_BYTES / 4;
pub(crate) const MAX_UNIFIED_EXEC_PROCESSES: usize = 64;

pub(crate) struct UnifiedExecContext {
    pub session: Arc<Session>,
    pub step_context: Arc<StepContext>,
    pub cancellation_token: CancellationToken,
    pub call_id: String,
}

impl UnifiedExecContext {
    pub fn new(
        session: Arc<Session>,
        step_context: Arc<StepContext>,
        cancellation_token: CancellationToken,
        call_id: String,
    ) -> Self {
        Self {
            session,
            step_context,
            cancellation_token,
            call_id,
        }
    }
}

#[derive(Debug)]
pub(crate) struct ExecCommandRequest {
    pub command: Vec<String>,
    pub shell_type: ShellType,
    pub hook_command: String,
    pub process_id: i32,
    pub yield_time_ms: u64,
    pub max_output_tokens: Option<usize>,
    pub cwd: PathUri,
    pub sandbox_cwd: PathUri,
    pub turn_environment: TurnEnvironment,
    pub shell_mode: UnifiedExecShellMode,
    pub network: Option<NetworkProxy>,
    pub tty: bool,
    pub sandbox_permissions: SandboxPermissions,
    pub additional_permissions: Option<AdditionalPermissionProfile>,
    pub additional_permissions_preapproved: bool,
    pub justification: Option<String>,
    pub prefix_rule: Option<Vec<String>>,
    /// Resolved by the handler, where the turn's environment and policy are both in hand.
    pub spill_dir: Option<PathBuf>,
}

#[derive(Debug)]
pub(crate) struct WriteStdinRequest<'a> {
    pub process_id: i32,
    pub input: &'a str,
    pub yield_time_ms: u64,
    pub max_output_tokens: Option<usize>,
    pub truncation_policy: TruncationPolicy,
    pub interaction_event: Option<WriteStdinInteractionEvent<'a>>,
    /// Resolved by the handler, where the turn's environment and policy are both in hand.
    pub spill_dir: Option<PathBuf>,
}

pub(crate) struct WriteStdinInteractionEvent<'a> {
    pub session: &'a Arc<Session>,
    pub turn: &'a Arc<TurnContext>,
}

impl std::fmt::Debug for WriteStdinInteractionEvent<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("WriteStdinInteractionEvent")
    }
}

#[derive(Default)]
pub(crate) struct ProcessStore {
    processes: HashMap<i32, ProcessEntry>,
    reserved_process_ids: HashSet<i32>,
}

impl ProcessStore {
    fn remove(&mut self, process_id: i32) -> Option<ProcessEntry> {
        self.reserved_process_ids.remove(&process_id);
        self.processes.remove(&process_id)
    }
}

pub(crate) struct UnifiedExecProcessManager {
    process_store: Mutex<ProcessStore>,
    max_write_stdin_yield_time_ms: u64,
}

impl UnifiedExecProcessManager {
    pub(crate) fn new(max_write_stdin_yield_time_ms: u64) -> Self {
        Self {
            process_store: Mutex::new(ProcessStore::default()),
            max_write_stdin_yield_time_ms: max_write_stdin_yield_time_ms
                .max(MIN_EMPTY_YIELD_TIME_MS),
        }
    }
}

impl Default for UnifiedExecProcessManager {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_BACKGROUND_TERMINAL_TIMEOUT_MS)
    }
}

struct ProcessEntry {
    process: Arc<UnifiedExecProcess>,
    plugin_metrics_sidecar: Option<SharedPluginMetricsSidecar>,
    call_id: String,
    process_id: i32,
    cwd: PathUri,
    initial_exec_command_active: Arc<std::sync::atomic::AtomicBool>,
    hook_command: String,
    tty: bool,
    environment_id: String,
    permissions: TerminalPermissions,
    network_approval: Option<DeferredNetworkApproval>,
    session: Weak<Session>,
    last_used: tokio::time::Instant,
}

type SharedPluginMetricsSidecar = Arc<std::sync::Mutex<Option<PluginMetricsSidecar>>>;

fn take_plugin_metrics_sidecar(
    sidecar: &SharedPluginMetricsSidecar,
) -> Option<PluginMetricsSidecar> {
    sidecar
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take()
}

pub(crate) fn clamp_yield_time(yield_time_ms: u64) -> u64 {
    let yield_time_ms = if cfg!(windows) {
        yield_time_ms.max(WINDOWS_INITIAL_EXEC_YIELD_TIME_FLOOR_MS)
    } else {
        yield_time_ms
    };
    yield_time_ms.clamp(MIN_YIELD_TIME_MS, MAX_YIELD_TIME_MS)
}

pub(crate) fn resolve_max_tokens(max_tokens: Option<usize>) -> usize {
    max_tokens.unwrap_or(DEFAULT_MAX_OUTPUT_TOKENS)
}

pub(crate) fn format_output_omission_marker(omitted_bytes: usize) -> String {
    format!("... {omitted_bytes} bytes omitted ...")
}

pub(crate) fn generate_chunk_id() -> String {
    let mut rng = rng();
    (0..6)
        .map(|_| format!("{:x}", rng.random_range(0..16)))
        .collect()
}

#[cfg(test)]
#[cfg(unix)]
#[path = "process_tests.rs"]
mod process_tests;
#[cfg(test)]
#[cfg(unix)]
#[path = "mod_tests.rs"]
mod tests;

#[cfg(test)]
mod spill_tests {
    use super::TOOL_OUTPUT_MAX_AGE;
    use super::sweep_stale_tool_output;
    use super::tool_output_spill_dir;
    use codex_exec_server::FileSystemSandboxContext;
    use codex_protocol::ThreadId;
    use codex_protocol::protocol::SandboxPolicy;
    use codex_utils_absolute_path::AbsolutePathBuf;
    use codex_utils_path_uri::PathUri;
    use std::time::Duration;

    fn context_for(policy: SandboxPolicy, cwd: &std::path::Path) -> FileSystemSandboxContext {
        let cwd = AbsolutePathBuf::try_from(cwd.to_path_buf()).expect("absolute cwd");
        FileSystemSandboxContext::from_legacy_sandbox_policy(policy, PathUri::from(cwd))
            .expect("sandbox context from policy")
    }

    /// The load-bearing claim of the whole mechanism: the model can open what we point it at.
    ///
    /// Spilling to a directory the turn's policy refuses to read would be worse than not spilling
    /// at all, so this asserts the reachability rather than trusting the reading of
    /// `permissions.rs` that motivated putting the file under `codex_home`.
    #[test]
    fn the_profiles_in_normal_use_can_read_what_is_spilled() {
        let home = tempfile::tempdir().expect("codex home");
        let workspace = tempfile::tempdir().expect("workspace");
        let thread_id = ThreadId::default();

        for (label, policy) in [
            ("workspace-write", SandboxPolicy::new_workspace_write_policy()),
            ("read-only", SandboxPolicy::new_read_only_policy()),
        ] {
            let dir = tool_output_spill_dir(
                home.path(),
                thread_id,
                &context_for(policy, workspace.path()),
                workspace.path(),
            );
            let dir = dir.unwrap_or_else(|| panic!("{label} should be able to read the spill dir"));
            assert!(
                dir.starts_with(home.path()),
                "{label} spilled outside the codex home: {}",
                dir.display()
            );
            assert!(
                !dir.starts_with(workspace.path()),
                "{label} spilled into the workspace, where `.suffice` is read-only by default"
            );
        }
    }

    /// The window is varied instead of the directory's timestamp, because backdating an mtime on
    /// a directory needs a crate this workspace does not carry. It exercises the same comparison
    /// from both sides: one call where the directory is inside the window and one where it is
    /// not.
    #[test]
    fn the_sweep_removes_what_is_stale_and_keeps_what_is_not() {
        let root = tempfile::tempdir().expect("tempdir");
        let session = root.path().join("crashed-session");
        std::fs::create_dir_all(session.join("nested")).expect("session dir");
        std::fs::write(session.join("tool_a.txt"), b"output").expect("spill file");
        std::thread::sleep(Duration::from_millis(/*millis*/ 20));

        sweep_stale_tool_output(root.path(), TOOL_OUTPUT_MAX_AGE);
        assert!(
            session.exists(),
            "a directory written moments ago is not stale under a one-day window"
        );

        sweep_stale_tool_output(root.path(), Duration::from_nanos(/*nanos*/ 1));
        assert!(
            !session.exists(),
            "past the window the whole directory goes, nested files included"
        );
    }

    #[test]
    fn the_sweep_leaves_loose_files_alone() {
        let root = tempfile::tempdir().expect("tempdir");
        let stray = root.path().join("not-a-session.txt");
        std::fs::write(&stray, b"someone else's file").expect("stray file");
        std::thread::sleep(Duration::from_millis(/*millis*/ 20));

        // Only per-session directories are ours to delete. Anything else under this root was put
        // there by something we do not know about, and recursive deletion is not the place to
        // guess.
        sweep_stale_tool_output(root.path(), Duration::from_nanos(/*nanos*/ 1));

        assert!(stray.exists());
    }

    #[test]
    fn the_sweep_is_quiet_when_there_is_nothing_to_sweep() {
        let root = tempfile::tempdir().expect("tempdir");
        // The common case: no command has ever been truncated, so the root was never created.
        sweep_stale_tool_output(&root.path().join("never-written"), TOOL_OUTPUT_MAX_AGE);
    }
}
