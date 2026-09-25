//! `glob` and `grep`, built to match OpenCode's tools mechanically, not just in wording.
//!
//! The first version of these walked the environment's filesystem and matched with a hand-rolled
//! `globset`. That was wrong in three ways that all cost money, and all three were invisible until
//! OpenCode's own bundle was read:
//!
//! 1. **`.gitignore` was ignored.** OpenCode shells out to `rg`, which honours it. `target/` is not
//!    a hidden directory, so a glob in a Rust repo walked every build artefact.
//! 2. **The pattern meant something else.** `rg --glob=*.rs` matches a basename at any depth, the
//!    way `.gitignore` does; a `globset` with `literal_separator(true)` matches only the root.
//! 3. **The limits and the wording differed** - 200 results against OpenCode's 100, and
//!    "No matches found" where OpenCode says "No files found".
//!
//! All three are fixed by using `ignore`, which is ripgrep's own walker, with the caller's pattern
//! installed as an `Override` exactly as `rg --glob=` installs it.
//!
//! `grep` now prefers the bundled `rg` binary and keeps this walker as its fallback, so it does
//! spawn a process - see `search_rg`. `glob` still never does.
//!
//! Why these tools exist at all: on a 44-file reading task the model asked itself eleven times
//! whether search output counts as having read a file - *"tool command is not model reading? It
//! sees output"*, *"It technically reads every file"* - and each time resolved it by reading
//! anyway, paying for the search and the read. The prompt now answers that question; these tools
//! are what it answers it in favour of.

use codex_exec_server::ExecutorFileSystem;
use codex_exec_server::FileSystemSandboxContext;
use codex_file_system::MAX_WALK_DEPTH;
use codex_file_system::MAX_WALK_DIRECTORIES;
use codex_file_system::MAX_WALK_ENTRIES;
use codex_file_system::WalkEntryKind;
use codex_file_system::WalkOptions;
use codex_install_context::InstallContext;
use codex_protocol::models::PermissionProfile;
use codex_protocol::permissions::FileSystemSandboxPolicy;
use codex_utils_path_uri::PathUri;
use ignore::WalkBuilder;
use ignore::overrides::OverrideBuilder;
use regex_lite::Regex;
use serde_json::Value;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;

use crate::function_tool::FunctionCallError;
use crate::tools::context::FunctionToolOutput;
use crate::tools::context::ToolInvocation;
use crate::tools::context::ToolOutput;
use crate::tools::context::ToolPayload;
use crate::tools::context::boxed_tool_output;
use crate::tools::handlers::resolve_tool_environment;
use crate::tools::handlers::search_rg;
use crate::tools::handlers::search_spec::GLOB_TOOL_NAME;
use crate::tools::handlers::search_spec::GREP_TOOL_NAME;
use crate::tools::handlers::search_spec::SearchToolOptions;
use crate::tools::handlers::search_spec::create_glob_tool;
use crate::tools::handlers::search_spec::create_grep_tool;
use crate::tools::registry::CoreToolRuntime;
use crate::tools::registry::ToolExecutor;
use codex_tools::ToolName;
use codex_tools::ToolSpec;

/// OpenCode's own result limits: `glob` and `grep` each stop at 100.
const MAX_GLOB_PATHS: usize = 100;
pub(super) const MAX_GREP_MATCHES: usize = 100;
/// OpenCode caps a matched line at 2000 characters and appends `...`.
pub(super) const MAX_MATCH_LINE_BYTES: usize = 2_000;
/// Files above this are not searched. OpenCode passes no `--max-filesize`, but it also never holds
/// a whole file: `rg` streams. This reads, so a bound is needed; 1 MiB is well past any source file
/// and short of a bundle.
const MAX_SEARCHED_FILE_BYTES: usize = 1 << 20;
/// Divergence from OpenCode, deliberately: it has no byte cap on grep output because its results go
/// back as their own tool message, while ours share a `function_call_output` the harness will cut
/// middle-out if it grows. A hundred matches of ordinary source lines land far below this; the cap
/// only binds on pathological input, and it announces itself in OpenCode's own words.
const MAX_GREP_OUTPUT_BYTES: usize = 48_000;
const MAX_GLOB_OUTPUT_BYTES: usize = 24_000;
/// How many files are walked before the walk gives up and says so.
const MAX_WALKED_FILES: usize = 50_000;

// ---------------------------------------------------------------------------
// Shared
// ---------------------------------------------------------------------------

fn string_argument(arguments: &str, key: &str) -> Result<Option<String>, FunctionCallError> {
    let value: Value = serde_json::from_str(arguments).map_err(|error| {
        FunctionCallError::RespondToModel(format!("arguments must be a JSON object: {error}"))
    })?;
    match value.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(text)) if text.is_empty() => Ok(None),
        Some(Value::String(text)) => Ok(Some(text.clone())),
        Some(_) => Err(FunctionCallError::RespondToModel(format!(
            "`{key}` must be a string"
        ))),
    }
}

fn function_arguments(payload: ToolPayload) -> Result<String, FunctionCallError> {
    match payload {
        ToolPayload::Function { arguments } => Ok(arguments),
        _ => Err(FunctionCallError::RespondToModel(
            "search handler received unsupported payload".to_string(),
        )),
    }
}

pub(super) fn clip_to_bytes(text: &str, max: usize) -> &str {
    if text.len() <= max {
        return text;
    }
    let mut end = max;
    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }
    &text[..end]
}

/// A result path as the model should see it: relative to the turn's cwd, absolute when the file
/// lies outside it.
///
/// Measured over 1,490 real result paths from the recorded runs, an absolute path averages 87
/// characters against 32 for the same path written relative to the cwd - 23,300 tokens across that
/// corpus, paid again on every later request. Nothing is lost by shortening it: the model's own
/// citations are already cwd-relative, and `read` resolves what it is handed against the same cwd
/// (`read.rs`, `turn_environment.cwd().join(...)`), so a relative path can be pasted straight back.
///
/// Falling back to the absolute form is deliberate rather than an error case - a file above the cwd
/// has no relative name worth printing, and a wrong short path is worse than a long right one.
/// `None` asks for the absolute form outright; see [`display_base`].
pub(super) fn display_path(cwd: Option<&Path>, file: &str) -> String {
    cwd.and_then(|cwd| Path::new(file).strip_prefix(cwd).ok())
        .map(|rest| rest.to_string_lossy().into_owned())
        .filter(|rest| !rest.is_empty())
        .unwrap_or_else(|| file.to_string())
}

/// What result paths are shortened against, or `None` when they have to stay absolute.
///
/// A relative path is only unambiguous while there is one environment to resolve it against. Once
/// the tools advertise `environment_id`, `read` resolves what it is handed against the *primary*
/// environment's cwd unless the model repeats the id, so a relative path that came from another
/// environment would quietly read a different file - or the same name in the wrong tree. An
/// absolute path resolves identically in every environment, which is why it stays in that mode:
/// the shorter form is worth tokens, not a silently wrong file.
fn display_base<'a>(options: &SearchToolOptions, cwd: &'a Path) -> Option<&'a Path> {
    (!options.include_environment_id).then_some(cwd)
}

/// The turn's read policy, asked one path at a time.
///
/// `read` reaches the disk through `ExecutorFileSystem`, which applies this policy on its behalf.
/// Neither `grep` engine does: the fast one spawns ripgrep, which the filesystem abstraction has no
/// way to contain, and the fallback calls `std::fs::read` directly (`scan_in_process`). So the
/// policy is carried to them here instead, and a file it refuses is dropped *before* its matches
/// are counted rather than filtered out of the rendering afterwards - `Found N matches` must not
/// count something the model is never shown.
///
/// This is a guard, not a fix for a live leak. Every profile in normal use grants full-disk *read*
/// access - `workspace-write` narrows writes, not reads - so [`Self::Unrestricted`] is the usual
/// answer and every check is a discriminant test. It earns its place on a profile that does narrow
/// reads, and on the day one of those becomes the default.
#[derive(Clone)]
pub(super) enum ReadGuard {
    /// This profile does not narrow reads, so there is nothing to check.
    Unrestricted,
    /// Ask `policy`, resolving its relative entries against `cwd`.
    Policy {
        policy: Box<FileSystemSandboxPolicy>,
        cwd: PathBuf,
    },
    /// The profile did not parse as one of this host's. Nothing outside the cwd is allowed, which
    /// is the conservative reading: a search of the workspace still works and a search of anywhere
    /// else is refused rather than guessed at.
    CwdOnly { cwd: PathBuf },
}

impl ReadGuard {
    fn new(sandbox: &FileSystemSandboxContext, cwd: &Path) -> Self {
        // This used to be a `try_from` with a `CwdOnly` fallback for a profile that did not
        // parse. Upstream made the conversion infallible, so that arm is now unreachable
        // from here - kept on the enum because it is still the right answer for a caller
        // that cannot resolve a profile at all.
        let profile = PermissionProfile::from(sandbox.permissions.clone());
        let policy = profile.file_system_sandbox_policy();
        if policy.has_full_disk_read_access() {
            return Self::Unrestricted;
        }
        Self::Policy {
            policy: Box::new(policy),
            cwd: cwd.to_path_buf(),
        }
    }

    pub(super) fn allows(&self, path: &Path) -> bool {
        match self {
            Self::Unrestricted => true,
            Self::Policy { policy, cwd } => policy.can_read_local_path_with_cwd(path, cwd),
            Self::CwdOnly { cwd } => path.starts_with(cwd),
        }
    }
}

/// Resolve the `path` argument against the turn's cwd, or use the cwd itself.
fn search_root(cwd: &PathUri, path: Option<&str>) -> Result<PathUri, FunctionCallError> {
    match path {
        None => Ok(cwd.clone()),
        Some(raw) => cwd.join(raw).map_err(|error| {
            FunctionCallError::RespondToModel(format!("invalid path `{raw}`: {error}"))
        }),
    }
}

struct Walked {
    files: Vec<String>,
    truncated: bool,
}

/// The walk `rg` would do: `.gitignore` honoured, `.git` excluded, the caller's pattern installed as
/// an override so it means what it means on an `rg --glob=` command line.
///
/// `include_hidden` mirrors the one place OpenCode's two tools differ from each other: its grep
/// passes `--hidden` unconditionally, its glob passes it never.
fn walk_local(
    root: PathBuf,
    pattern: Option<String>,
    include_hidden: bool,
) -> Result<Walked, String> {
    let mut overrides = OverrideBuilder::new(&root);
    if let Some(pattern) = &pattern {
        overrides
            .add(pattern)
            .map_err(|error| format!("invalid pattern `{pattern}`: {error}"))?;
    }
    overrides
        .add("!**/.git/**")
        .map_err(|error| format!("internal override failed: {error}"))?;
    let overrides = overrides
        .build()
        .map_err(|error| format!("invalid pattern: {error}"))?;

    let mut builder = WalkBuilder::new(&root);
    builder
        // `ignore`'s `hidden(true)` means "skip hidden", which is the inverse of `--hidden`.
        .hidden(!include_hidden)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .parents(true)
        .follow_links(false)
        .max_depth(Some(MAX_WALK_DEPTH))
        .overrides(overrides);

    let mut files = Vec::new();
    let mut truncated = false;
    for entry in builder.build() {
        let Ok(entry) = entry else { continue };
        if !entry.file_type().is_some_and(|kind| kind.is_file()) {
            continue;
        }
        if files.len() >= MAX_WALKED_FILES {
            truncated = true;
            break;
        }
        files.push(entry.path().to_string_lossy().into_owned());
    }
    // Deterministic order: `rg` streams in walk order, which is filesystem-dependent, and a search
    // whose result reshuffles between identical calls cannot be diffed.
    files.sort();
    Ok(Walked { files, truncated })
}

/// Fallback for an environment whose files are not on this host. It has no `.gitignore` support -
/// the filesystem abstraction exposes none - so it is strictly worse, and it is only reached when
/// the local walker cannot see the root at all.
async fn walk_remote(
    filesystem: &dyn ExecutorFileSystem,
    sandbox: Option<&FileSystemSandboxContext>,
    root: &PathUri,
) -> Result<Walked, FunctionCallError> {
    let outcome = filesystem
        .walk(
            root,
            WalkOptions {
                max_depth: MAX_WALK_DEPTH,
                max_directories: MAX_WALK_DIRECTORIES,
                max_entries: MAX_WALK_ENTRIES,
                follow_directory_symlinks: false,
                prune_hidden_directories: true,
            },
            sandbox,
        )
        .await
        .map_err(|error| {
            FunctionCallError::RespondToModel(format!(
                "unable to search {}: {error}",
                root.inferred_native_path_string()
            ))
        })?;
    let mut files: Vec<String> = outcome
        .entries
        .into_iter()
        .filter(|entry| entry.kind == WalkEntryKind::File)
        .map(|entry| entry.path.inferred_native_path_string())
        .collect();
    files.sort();
    Ok(Walked {
        files,
        truncated: outcome.truncated,
    })
}

/// Walk `root`, preferring ripgrep's walker when the root is on this host.
async fn walk(
    filesystem: &dyn ExecutorFileSystem,
    sandbox: Option<&FileSystemSandboxContext>,
    root: &PathUri,
    pattern: Option<String>,
    include_hidden: bool,
) -> Result<Walked, FunctionCallError> {
    let local = root.to_path_buf();
    if local.is_dir() {
        return tokio::task::spawn_blocking(move || walk_local(local, pattern, include_hidden))
            .await
            .map_err(|error| {
                FunctionCallError::RespondToModel(format!("search was interrupted: {error}"))
            })?
            .map_err(FunctionCallError::RespondToModel);
    }
    walk_remote(filesystem, sandbox, root).await
}

// ---------------------------------------------------------------------------
// glob
// ---------------------------------------------------------------------------

#[derive(Default)]
pub struct GlobHandler {
    options: SearchToolOptions,
}

impl GlobHandler {
    pub(crate) fn new(options: SearchToolOptions) -> Self {
        Self { options }
    }

    async fn handle_call(
        &self,
        invocation: ToolInvocation,
    ) -> Result<Box<dyn ToolOutput>, FunctionCallError> {
        let ToolInvocation {
            step_context,
            turn,
            payload,
            cancellation_token,
            ..
        } = invocation;
        let arguments = function_arguments(payload)?;
        let pattern = string_argument(&arguments, "pattern")?.ok_or_else(|| {
            FunctionCallError::RespondToModel("`pattern` is required".to_string())
        })?;
        let path = string_argument(&arguments, "path")?;
        let environment_id = string_argument(&arguments, "environment_id")?;

        let Some(turn_environment) =
            resolve_tool_environment(&step_context.environments, environment_id.as_deref())?
        else {
            return Err(FunctionCallError::RespondToModel(
                "glob is unavailable in this session".to_string(),
            ));
        };
        let sandbox = turn_environment.sandbox_context(/*additional_permissions*/ None);
        let sandbox = Some(&sandbox);
        let filesystem = turn_environment.environment.get_filesystem();
        let filesystem = filesystem.as_ref();

        let root = search_root(turn_environment.cwd(), path.as_deref())?;
        // OpenCode refuses a file here rather than silently searching its parent.
        if root.to_path_buf().is_file() {
            return Err(FunctionCallError::RespondToModel(format!(
                "glob path must be a directory: {}",
                root.inferred_native_path_string()
            )));
        }
        // A profile that narrows reads is enforced by the sandbox, not here: the listing runs as
        // `rg --files` inside the turn's sandbox, so a directory the profile denies is never
        // entered. Otherwise the walk is the unconfined one, exactly as before.
        let walked = if !turn_environment.environment.is_remote()
            && sandbox.is_some_and(FileSystemSandboxContext::should_read_from_sandbox)
        {
            let cwd = turn_environment.cwd().to_abs_path().map_err(|error| {
                FunctionCallError::RespondToModel(format!("glob failed: {error}"))
            })?;
            let stdout = search_rg::run_in_turn_sandbox(
                &InstallContext::current().rg_command(),
                search_rg::rg_files_args(&pattern, &root.to_path_buf()),
                &cwd,
                &turn,
                turn_environment,
                &cancellation_token,
            )
            .await?;
            let mut files: Vec<String> = stdout
                .lines()
                .filter(|line| !line.is_empty())
                .map(str::to_string)
                .collect();
            files.sort();
            let truncated = files.len() > MAX_WALKED_FILES;
            files.truncate(MAX_WALKED_FILES);
            Walked { files, truncated }
        } else {
            walk(
                filesystem,
                sandbox,
                &root,
                Some(pattern),
                /*include_hidden*/ false,
            )
            .await?
        };

        // `glob`'s answer is nothing but paths, so the path length *is* the result size: measured
        // over 3,210 real result paths it averages 90 characters absolute against 36 relative, and
        // across the recorded runs that is 605,228 of `glob`'s 1,000,532 billed tokens. Grep does
        // the same at its own collection point.
        let cwd = turn_environment.cwd().to_path_buf();
        let base = display_base(&self.options, &cwd);
        let files: Vec<String> = walked
            .files
            .iter()
            .map(|file| display_path(base, file))
            .collect();

        Ok(boxed_tool_output(FunctionToolOutput::from_text(
            render_glob(&files, walked.truncated),
            Some(true),
        )))
    }
}

/// OpenCode's layout: one path per line, then its own truncation sentence.
///
/// The paths were absolute, as OpenCode's are. They are now written relative to the turn's cwd -
/// see [`display_path`] for the measurement and [`display_base`] for when they are not.
///
/// `walk_truncated` has no OpenCode counterpart - `rg` has no walk cap - but a silently short
/// answer is worse than a small divergence, so the same sentence covers it.
fn render_glob(matches: &[String], walk_truncated: bool) -> String {
    if matches.is_empty() {
        return "No files found".to_string();
    }
    let mut out = String::new();
    let mut shown = 0usize;
    for path in matches.iter().take(MAX_GLOB_PATHS) {
        if out.len() + path.len() + 1 > MAX_GLOB_OUTPUT_BYTES {
            break;
        }
        out.push_str(path);
        out.push('\n');
        shown += 1;
    }
    if shown < matches.len() || walk_truncated {
        out.push_str(&format!(
            "\n(Results are truncated: showing first {shown} results. Consider using a more \
             specific path or pattern.)\n"
        ));
    }
    out
}

impl ToolExecutor<ToolInvocation> for GlobHandler {
    fn tool_name(&self) -> ToolName {
        ToolName::plain(GLOB_TOOL_NAME)
    }

    fn spec(&self) -> ToolSpec {
        create_glob_tool(self.options)
    }

    fn supports_parallel_tool_calls(&self) -> bool {
        true
    }

    fn handle<'a>(&'a self, invocation: ToolInvocation) -> codex_tools::ToolExecutorFuture<'a>
    where
        ToolInvocation: 'a,
    {
        Box::pin(self.handle_call(invocation))
    }
}

impl CoreToolRuntime for GlobHandler {}

// ---------------------------------------------------------------------------
// grep
// ---------------------------------------------------------------------------

#[derive(Default)]
pub struct GrepHandler {
    options: SearchToolOptions,
}

pub(super) struct FileMatches {
    pub(super) display: String,
    /// Every match in this file, counted whether or not its line survived `MAX_GREP_MATCHES`.
    ///
    /// The scan already reads every line of every walked file - the cap gates the push, not the
    /// loop - so this costs nothing to keep, and it is the only thing that makes a saturated
    /// result usable: without it a file whose matches all landed past the cap is dropped from
    /// `per_file` entirely and the model never learns it matched.
    pub(super) matched: usize,
    pub(super) lines: Vec<(usize, String)>,
}

/// Which cap ended the result, if any. All three used to render the same sentence, which told the
/// model that something was missing but not what, nor what to do about it.
#[derive(Clone, Copy, PartialEq, Eq)]
enum GrepLimit {
    None,
    /// `MAX_GREP_MATCHES`: more matches exist than the result may carry.
    Matches,
    /// `MAX_GREP_OUTPUT_BYTES`: the rendered lines would not fit.
    Bytes,
    /// The search ended before it had seen everything - the in-process walk gave up at
    /// `MAX_WALKED_FILES`, or ripgrep was cut short - so even the counts are a floor.
    Walk,
}

impl GrepHandler {
    pub(crate) fn new(options: SearchToolOptions) -> Self {
        Self { options }
    }

    async fn handle_call(
        &self,
        invocation: ToolInvocation,
    ) -> Result<Box<dyn ToolOutput>, FunctionCallError> {
        let ToolInvocation {
            step_context,
            turn,
            payload,
            cancellation_token,
            ..
        } = invocation;
        let arguments = function_arguments(payload)?;
        let pattern = string_argument(&arguments, "pattern")?.ok_or_else(|| {
            FunctionCallError::RespondToModel("`pattern` is required".to_string())
        })?;
        let path = string_argument(&arguments, "path")?;
        let include = string_argument(&arguments, "include")?;
        let environment_id = string_argument(&arguments, "environment_id")?;

        // Compiled even when ripgrep is going to do the matching, and the error text is still this
        // one. ripgrep's engine accepts a larger language than `regex-lite` - `\p{Greek}`, a
        // Unicode-aware `\w` - and letting it through would mean a pattern is valid or invalid
        // depending on whether ripgrep happened to be reachable. Both engines accept exactly what
        // the smaller one accepts until that is changed on purpose.
        let regex = Regex::new(&pattern).map_err(|error| {
            FunctionCallError::RespondToModel(format!("invalid regex `{pattern}`: {error}"))
        })?;

        let Some(turn_environment) =
            resolve_tool_environment(&step_context.environments, environment_id.as_deref())?
        else {
            return Err(FunctionCallError::RespondToModel(
                "grep is unavailable in this session".to_string(),
            ));
        };
        let sandbox_context = turn_environment.sandbox_context(/*additional_permissions*/ None);
        let sandbox = Some(&sandbox_context);
        let filesystem = turn_environment.environment.get_filesystem();
        let filesystem = filesystem.as_ref();

        let requested = search_root(turn_environment.cwd(), path.as_deref())?;
        let cwd = turn_environment.cwd().to_path_buf();
        let base = display_base(&self.options, &cwd);
        let target = requested.to_path_buf();
        let single_file = single_file_target(&requested);

        // Refused up front rather than per file, so a search of somewhere unreadable says so
        // instead of returning an honest-looking `Found 0 matches`.
        let read_guard = ReadGuard::new(&sandbox_context, &cwd);
        if !read_guard.allows(&target) {
            return Err(FunctionCallError::RespondToModel(format!(
                "cannot search {}: reading it is not permitted in this session",
                display_path(base, &target.to_string_lossy())
            )));
        }

        // A profile that narrows reads is enforced by the sandbox, not here: ripgrep runs inside the
        // turn's sandbox, so the operating system refuses every file the profile denies (glob
        // denies included) and ripgrep never reads one. No unconfined fallback exists on this path.
        if !turn_environment.environment.is_remote() && sandbox_context.should_read_from_sandbox() {
            let request = search_rg::RgRequest {
                program: InstallContext::current().rg_command(),
                cwd: cwd.clone(),
                display_base: base.map(Path::to_path_buf),
                target: target.clone(),
                pattern: pattern.clone(),
                include: single_file.is_none().then_some(include.clone()).flatten(),
                read_guard: read_guard.clone(),
            };
            let cwd = turn_environment.cwd().to_abs_path().map_err(|error| {
                FunctionCallError::RespondToModel(format!("grep failed: {error}"))
            })?;
            let stdout = search_rg::run_in_turn_sandbox(
                &request.program,
                search_rg::rg_args(&request),
                &cwd,
                &turn,
                turn_environment,
                &cancellation_token,
            )
            .await?;
            let scan = search_rg::collect_json(&stdout, base, &read_guard);
            return Ok(boxed_tool_output(FunctionToolOutput::from_text(
                render_grep(scan.found, &scan.per_file, scan.stopped_early),
                Some(true),
            )));
        }

        // ripgrep needs the files on this host: it is a process, and a remote environment reaches
        // its filesystem over `ExecutorFileSystem`, which has no way to run anything. `is_remote`
        // alone is not enough - a local environment can still be backed by a filesystem that is not
        // this host's, which is what `walk` already probes for - so both have to hold.
        let rg_reachable = !turn_environment.environment.is_remote() && target.exists();
        let scan = if rg_reachable {
            let request = search_rg::RgRequest {
                program: InstallContext::current().rg_command(),
                cwd: cwd.clone(),
                display_base: base.map(Path::to_path_buf),
                target: target.clone(),
                pattern: pattern.clone(),
                // A glob filters a directory walk, so it has nothing to filter once one file is
                // named. The in-process branch below drops it for the same reason.
                include: single_file.is_none().then_some(include.clone()).flatten(),
                // ripgrep runs unsandboxed, so what it walks into is filtered on the way back.
                read_guard: read_guard.clone(),
            };
            let cancellation = cancellation_token.clone();
            tokio::task::spawn_blocking(move || search_rg::run_rg(&request, &cancellation))
                .await
                .map_err(|error| {
                    FunctionCallError::RespondToModel(format!("grep was interrupted: {error}"))
                })??
        } else {
            None
        };

        let (found, per_file, stopped_early) = match scan {
            Some(scan) => (scan.found, scan.per_file, scan.stopped_early),
            None => {
                let (files, truncated) = match single_file {
                    Some(file) => (vec![file], false),
                    None => {
                        // `--hidden` is unconditional for OpenCode's grep.
                        let walked = walk(
                            filesystem, sandbox, &requested, include, /*include_hidden*/ true,
                        )
                        .await?;
                        (walked.files, walked.truncated)
                    }
                };
                let (found, per_file) = scan_in_process(files, base, &regex, &read_guard);
                (found, per_file, truncated)
            }
        };

        Ok(boxed_tool_output(FunctionToolOutput::from_text(
            render_grep(found, &per_file, stopped_early),
            Some(true),
        )))
    }
}

/// The one file `path` named, or `None` when it named a directory to walk.
///
/// A `path` that names a file means that file. It used to mean that file's parent directory: 40 of
/// the 82 recorded `grep` calls passed a file and were answered with the whole directory it sat in,
/// so 550 of the 857 matches that came back were from files the model had not asked about, and
/// nothing in the result said the scope had widened. `glob` refuses a file outright rather than
/// widening, but refusing is wrong here - a single-file grep is the call the model makes most.
///
/// `include` is a filter for a directory walk, so it does not apply once one file is named.
fn single_file_target(requested: &PathUri) -> Option<String> {
    requested
        .to_path_buf()
        .is_file()
        .then(|| requested.inferred_native_path_string())
}

/// Searches `files` in this process, line by line.
///
/// This is the fallback engine. It reads each file whole, decodes it lossily and runs the pattern
/// over every line, which is why it is no longer the first choice: on this repository a root-level
/// search walks 6,881 files and 69.6 MB, serially, and `std::fs::read` blocks the runtime while it
/// does. It stays because `rg` cannot always be reached - a remote environment has no host to spawn
/// it on, and a source build resolves `rg` to a bare name that may not be on `PATH`.
///
/// The cap gates the push, not the loop: `matched` counts every hit so a file whose matches all
/// landed past `MAX_GREP_MATCHES` still appears in the count map.
fn scan_in_process(
    files: Vec<String>,
    cwd: Option<&Path>,
    regex: &Regex,
    read_guard: &ReadGuard,
) -> (usize, Vec<FileMatches>) {
    let mut found = 0usize;
    let mut per_file: Vec<FileMatches> = Vec::new();
    for path in files {
        // `walk_local` is ripgrep's walker, which knows nothing about the sandbox, and the read
        // below is a plain `std::fs::read`. Skipped the same way an unreadable file is: silently,
        // and before anything about it is counted.
        if !read_guard.allows(Path::new(&path)) {
            continue;
        }
        let Ok(bytes) = std::fs::read(&path) else {
            continue;
        };
        if bytes.len() > MAX_SEARCHED_FILE_BYTES || bytes.contains(&0) {
            continue;
        }
        let text = String::from_utf8_lossy(&bytes);
        let mut matched = 0usize;
        let mut lines = Vec::new();
        for (index, line) in text.lines().enumerate() {
            if !regex.is_match(line) {
                continue;
            }
            found += 1;
            matched += 1;
            if found <= MAX_GREP_MATCHES {
                let clipped = if line.len() > MAX_MATCH_LINE_BYTES {
                    format!("{}...", clip_to_bytes(line, MAX_MATCH_LINE_BYTES))
                } else {
                    line.to_string()
                };
                lines.push((index + 1, clipped));
            }
        }
        // Not `!lines.is_empty()`: a file whose matches all fell past the cap has no lines to
        // show and still belongs in the count map.
        if matched > 0 {
            per_file.push(FileMatches {
                display: display_path(cwd, &path),
                matched,
                lines,
            });
        }
    }
    (found, per_file)
}

/// OpenCode's own layout, down to the blank line after each match and the header that says
/// "matches" whatever the count is.
///
/// Unsaturated results render exactly as they always did. A saturated one no longer returns a
/// hundred lines and an apology: on the measured `impl2` runs every truncated grep was abandoned
/// whole - 13 to 27 kB bought nothing and was then carried to the end of the run - because a
/// partial line dump tells the model neither what it missed nor where. `render_map` answers both
/// from counts the scan already had.
fn render_grep(found: usize, per_file: &[FileMatches], stopped_early: bool) -> String {
    if found == 0 {
        // OpenCode's empty-result sentinel says "files", not "matches". Copied as it is.
        return "No files found".to_string();
    }
    let saturated = found >= MAX_GREP_MATCHES;
    let mut out = format!("Found {found} matches\n");
    let mut stopped = false;
    if !saturated {
        // Repetition is folded away first, before anything is measured, because it is the only
        // reduction here that costs nothing: the text moves, no `file:line` is lost. A result
        // that fits after folding never reaches the tiers below.
        let (shared, rest) = group_shared_lines(per_file);
        for line in &shared {
            let block = render_shared_line(line);
            if out.len() + block.len() > MAX_GREP_OUTPUT_BYTES {
                stopped = true;
                break;
            }
            out.push_str(&block);
        }
        if !stopped {
            for file in rest {
                let mut block = format!("{}:\n", file.display);
                for (line_no, text) in &file.lines {
                    // A line already written once above is not written again here.
                    if shared.iter().any(|line| line.text == text.as_str()) {
                        continue;
                    }
                    block.push_str(&format!("  Line {line_no}: {text}\n\n"));
                }
                if out.len() + block.len() > MAX_GREP_OUTPUT_BYTES {
                    stopped = true;
                    break;
                }
                out.push_str(&block);
            }
        }
    }
    let limit = if stopped_early {
        GrepLimit::Walk
    } else if saturated {
        GrepLimit::Matches
    } else if stopped {
        GrepLimit::Bytes
    } else {
        GrepLimit::None
    };
    if limit == GrepLimit::None {
        return out;
    }
    render_capped(found, per_file, limit)
}

/// One shared line, written once with every place it was found.
fn render_shared_line(line: &SharedLine<'_>) -> String {
    let mut out = format!(
        "{} files share this line:\n  {}\n  in: ",
        line.places.len(),
        line.text
    );
    let places: Vec<String> = line
        .places
        .iter()
        .map(|(display, line_no)| format!("{display}:{line_no}"))
        .collect();
    out.push_str(&places.join(", "));
    out.push_str("\n\n");
    out
}

/// What a capped result returns, in the most detailed form that fits.
///
/// The old behaviour stepped straight from the full listing to per-file counts the moment either
/// cap was touched, which threw away every line of text over one threshold being crossed. There
/// is a form in between - the `file:line` coordinates without the text - and for this fork it is
/// not a consolation prize: the model's job on the `wire` task is to write `file.rs:778`
/// citations, so coordinates alone can be the whole answer and save the narrowing call that a
/// count map would have cost.
///
/// Sized arithmetically rather than by rendering each form and measuring: `coordinate_bytes`
/// walks the matches once and adds up what the text would be, so only the tier that wins is ever
/// built.
fn render_capped(found: usize, per_file: &[FileMatches], limit: GrepLimit) -> String {
    let header = capped_header(found, per_file.len(), limit);
    // A coordinate list with no coordinates is the count map with its own cap removed, so the
    // tier is only worth taking when the scan actually captured some line numbers. It refuses
    // itself here rather than degrading into a worse copy of the form below it.
    let has_coordinates = per_file.iter().any(|file| !file.lines.is_empty());
    if has_coordinates && header.len() + coordinate_bytes(per_file) <= MAX_MAP_OUTPUT_BYTES {
        return render_coordinates(header, per_file, limit);
    }
    render_map(found, per_file, limit)
}

/// The bytes `render_coordinates` would produce for the coordinate list, without producing it.
fn coordinate_bytes(per_file: &[FileMatches]) -> usize {
    per_file
        .iter()
        .map(|file| {
            let numbers: usize = file
                .lines
                .iter()
                .map(|(line_no, _)| decimal_width(*line_no) + 2)
                .sum();
            // `  <display>: ` plus the numbers plus a newline.
            file.display.len() + 5 + numbers
        })
        .sum()
}

fn decimal_width(value: usize) -> usize {
    let mut width = 1;
    let mut rest = value / 10;
    while rest > 0 {
        width += 1;
        rest /= 10;
    }
    width
}

/// Every match as a `file: line, line, line` coordinate, with no text.
fn render_coordinates(header: String, per_file: &[FileMatches], limit: GrepLimit) -> String {
    let mut out = header;
    out.push_str("Lines per file:\n");
    // The same file cap the map has, for the same reason: `MAX_MAP_OUTPUT_BYTES` bounds the size
    // but not the shape, and twenty thousand one-line entries is a dump however many bytes it is.
    for file in per_file.iter().take(MAX_MAP_FILES) {
        let numbers: Vec<String> = file
            .lines
            .iter()
            .map(|(line_no, _)| line_no.to_string())
            .collect();
        // `lines` stops at the match cap while `matched` counts every hit, so the two disagree on
        // a file whose matches ran past it. Printing coordinates alone there would understate the
        // file and read as complete, which is the one thing a coordinate list must not do.
        if numbers.is_empty() {
            out.push_str(&format!("  {}: {} matches\n", file.display, file.matched));
        } else if numbers.len() < file.matched {
            out.push_str(&format!(
                "  {}: {} (of {} matches)\n",
                file.display,
                numbers.join(", "),
                file.matched
            ));
        } else {
            out.push_str(&format!("  {}: {}\n", file.display, numbers.join(", ")));
        }
    }
    if per_file.len() > MAX_MAP_FILES {
        out.push_str(&format!(
            "  ... and {} more files\n",
            per_file.len() - MAX_MAP_FILES
        ));
    }
    // Coordinates say where every match is; the samples say what one looks like. Neither answers
    // the other's question, and the samples cost five lines, so both forms carry both.
    push_samples(&mut out, per_file);
    out.push_str(match limit {
        GrepLimit::Walk => "\nThe coordinates are partial. Read a line with `read` and its number.\n",
        _ => "\nText omitted to fit. Read a line with `read` and its number, or narrow the pattern.\n",
    });
    out
}

/// The first few matched lines, whole enough to show what a hit looks like.
fn push_samples(out: &mut String, per_file: &[FileMatches]) {
    let samples: Vec<String> = per_file
        .iter()
        .flat_map(|file| {
            file.lines.iter().map(move |(line_no, text)| {
                format!(
                    "  {}:{}: {}\n",
                    file.display,
                    line_no,
                    clip_to_bytes(text, MAP_SAMPLE_LINE_BYTES)
                )
            })
        })
        .take(MAP_SAMPLE_MATCHES)
        .collect();
    if samples.is_empty() {
        return;
    }
    out.push_str(&format!("\nFirst {} matches:\n", samples.len()));
    for sample in samples {
        out.push_str(&sample);
    }
}

/// How many files have to share a matched line before it is written once instead of per file.
///
/// Three, not two: a pair costs about as much either way, and the collapsed form reads worse for
/// it. Measured on a recorded call - `impl2-sufficefork-rep20` searched 88 component documents for
/// a sentence every one of them carries, and got 93 copies of the same 140 characters across
/// 13,138 bytes whose whole information content was one line and a file list.
const MIN_IDENTICAL_FILES: usize = 3;

/// One matched line and every place it occurs.
struct SharedLine<'a> {
    text: &'a str,
    /// `(display, line_no)` for each occurrence, in walk order.
    places: Vec<(&'a str, usize)>,
}

/// Splits matches into lines shared by several files and lines belonging to one.
///
/// Lossless: every `file:line` survives, and so does the text. What goes is the repetition - the
/// text is written once rather than once per file. Grouped on the text alone and not on the line
/// number as well, because the same declaration sits at a different line in each file as often as
/// not, and the repetition is just as expensive either way.
fn group_shared_lines<'a>(
    per_file: &'a [FileMatches],
) -> (Vec<SharedLine<'a>>, Vec<&'a FileMatches>) {
    let mut order: Vec<&'a str> = Vec::new();
    let mut places: HashMap<&'a str, Vec<(&'a str, usize)>> = HashMap::new();
    for file in per_file {
        for (line_no, text) in &file.lines {
            let entry = places.entry(text.as_str());
            if matches!(entry, std::collections::hash_map::Entry::Vacant(_)) {
                order.push(text.as_str());
            }
            entry.or_default().push((file.display.as_str(), *line_no));
        }
    }

    // A line is shared only when distinct *files* carry it. Three hits inside one file is
    // repetition the model asked for by searching that file.
    let mut shared = Vec::new();
    let mut shared_texts: HashSet<&'a str> = HashSet::new();
    for text in order {
        let occurrences = places.remove(text).unwrap_or_default();
        let mut files: Vec<&str> = occurrences.iter().map(|(display, _)| *display).collect();
        files.sort_unstable();
        files.dedup();
        if files.len() >= MIN_IDENTICAL_FILES {
            shared_texts.insert(text);
            shared.push(SharedLine {
                text,
                places: occurrences,
            });
        }
    }

    // Files keep their place in the listing when they still have a line of their own to show.
    let rest: Vec<&'a FileMatches> = per_file
        .iter()
        .filter(|file| {
            file.lines
                .iter()
                .any(|(_, text)| !shared_texts.contains(text.as_str()))
        })
        .collect();
    (shared, rest)
}

/// Entries named in the count map before it starts summarising, so one pathological pattern cannot
/// turn the map into the dump it replaces.
const MAX_MAP_FILES: usize = 200;
/// Sample matches shown under the map: enough to learn the shape of a hit, not to carry the data.
const MAP_SAMPLE_MATCHES: usize = 5;
/// Ceiling for the whole count map.
///
/// Not `MAX_GREP_OUTPUT_BYTES`: that is the ceiling for a result that lists lines, and a map
/// allowed to grow to it would be four times the 11.5 KB partial dump it exists to replace, which
/// is the opposite of the point. With paths written relative to the cwd a 200-file map lands near
/// 9 KB, so this leaves headroom for long relative paths and still stays under the dump.
const MAX_MAP_OUTPUT_BYTES: usize = 12_000;
/// How much of a sampled line the map carries.
///
/// The sentence above was the intent from the start and the code did not enforce it: a sample
/// arrived already clipped to `MAX_MATCH_LINE_BYTES`, so five of them could be 10 KB on their own
/// and the map - the thing written to stop a result exploding - reached 29 KB against the 11.5 KB
/// partial dump it replaces. A sample is there to show what a hit looks like; a line's first 200
/// bytes do that.
const MAP_SAMPLE_LINE_BYTES: usize = 200;

/// What was found, per file, when the lines themselves cannot all be shown.
///
/// Files stay in walk order rather than sorting by count: the question a truncated grep leaves
/// open is "which files am I missing", and walk order is the order the model builds inventories in.
/// The first line every capped form opens with, naming the total and why it was capped.
///
/// Shared so the coordinate tier and the count map cannot drift apart on it, and kept byte for
/// byte what the map alone used to print.
fn capped_header(found: usize, files: usize, limit: GrepLimit) -> String {
    let why = match limit {
        GrepLimit::Matches => "too many to list in full",
        GrepLimit::Bytes => "listing them in full would exceed this tool's size cap",
        // The search stopped before every file was read, so the totals below are a floor.
        GrepLimit::Walk => "the search stopped early, so these counts are partial",
        GrepLimit::None => unreachable!("a capped form is only reached for a capped result"),
    };
    format!("Found {found} matches in {files} files - {why}.\n")
}

fn render_map(found: usize, per_file: &[FileMatches], limit: GrepLimit) -> String {
    let mut out = capped_header(found, per_file.len(), limit);
    out.push_str("Matches per file:\n");
    // `render_grep` only ever checked a byte cap on the branch that lists lines, so this branch
    // could grow without anything noticing. Room for the samples and the closing sentence is held
    // back before the file list starts, so a long list cannot crowd them out.
    let reserve = MAP_SAMPLE_MATCHES.saturating_mul(MAP_SAMPLE_LINE_BYTES + 192) + 256;
    let list_budget = MAX_MAP_OUTPUT_BYTES.saturating_sub(reserve);
    let mut listed = 0usize;
    for file in per_file.iter().take(MAX_MAP_FILES) {
        let row = format!("  {}: {}\n", file.display, file.matched);
        if out.len().saturating_add(row.len()) > list_budget {
            break;
        }
        out.push_str(&row);
        listed += 1;
    }
    if per_file.len() > listed {
        out.push_str(&format!(
            "  ... and {} more files\n",
            per_file.len() - listed
        ));
    }
    push_samples(&mut out, per_file);
    out.push_str("\nNarrow the pattern or the path to see the lines themselves.\n");
    out
}

impl ToolExecutor<ToolInvocation> for GrepHandler {
    fn tool_name(&self) -> ToolName {
        ToolName::plain(GREP_TOOL_NAME)
    }

    fn spec(&self) -> ToolSpec {
        create_grep_tool(self.options)
    }

    fn supports_parallel_tool_calls(&self) -> bool {
        true
    }

    fn handle<'a>(&'a self, invocation: ToolInvocation) -> codex_tools::ToolExecutorFuture<'a>
    where
        ToolInvocation: 'a,
    {
        Box::pin(self.handle_call(invocation))
    }
}

impl CoreToolRuntime for GrepHandler {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn seed(root: &std::path::Path) {
        fs::create_dir_all(root.join("src/sse")).expect("dirs");
        fs::create_dir_all(root.join("target/debug")).expect("dirs");
        fs::create_dir_all(root.join(".git")).expect("dirs");
        fs::write(root.join(".gitignore"), "target/\n").expect("gitignore");
        fs::write(root.join(".git/config"), "[core]\n").expect("git config");
        fs::write(root.join("src/lib.rs"), "pub async fn connect() {}\n").expect("lib");
        fs::write(root.join("src/sse/chat.rs"), "pub fn chat() {}\n").expect("chat");
        fs::write(root.join("src/notes.md"), "pub async fn connect() {}\n").expect("notes");
        fs::write(root.join("target/debug/build.rs"), "pub fn built() {}\n").expect("built");
        fs::write(root.join(".hidden.rs"), "pub fn hidden() {}\n").expect("hidden");
    }

    fn names(walked: &Walked, root: &std::path::Path) -> Vec<String> {
        walked
            .files
            .iter()
            .map(|path| {
                std::path::Path::new(path)
                    .strip_prefix(root)
                    .unwrap_or_else(|_| std::path::Path::new(path))
                    .to_string_lossy()
                    .replace('\\', "/")
            })
            .collect()
    }

    /// The bug this file was rewritten for: `target/` is not hidden, so the first version walked
    /// every build artefact in a Rust repo while OpenCode's `rg` skipped them.
    #[test]
    fn a_gitignored_directory_is_not_walked() {
        let temp = tempfile::tempdir().expect("temp");
        seed(temp.path());
        let walked = walk_local(
            temp.path().to_path_buf(),
            Some("**/*.rs".to_string()),
            /*include_hidden*/ false,
        )
        .expect("walk");
        let found = names(&walked, temp.path());
        assert!(found.contains(&"src/lib.rs".to_string()), "{found:?}");
        assert!(
            !found.iter().any(|name| name.starts_with("target/")),
            "gitignored files must not be walked: {found:?}"
        );
    }

    /// `rg --glob=*.rs` matches a basename at any depth; a `globset` with `literal_separator` does
    /// not. The override installs the caller's pattern the way ripgrep installs it.
    #[test]
    fn a_bare_extension_pattern_matches_at_any_depth() {
        let temp = tempfile::tempdir().expect("temp");
        seed(temp.path());
        let walked = walk_local(
            temp.path().to_path_buf(),
            Some("*.rs".to_string()),
            /*include_hidden*/ false,
        )
        .expect("walk");
        let found = names(&walked, temp.path());
        assert!(found.contains(&"src/sse/chat.rs".to_string()), "{found:?}");
        assert!(!found.contains(&"src/notes.md".to_string()), "{found:?}");
    }

    /// OpenCode's grep passes `--hidden`, its glob never does.
    #[test]
    fn hidden_files_follow_the_tool_that_asked() {
        let temp = tempfile::tempdir().expect("temp");
        seed(temp.path());
        let for_glob = walk_local(
            temp.path().to_path_buf(),
            None,
            /*include_hidden*/ false,
        )
        .expect("walk");
        assert!(!names(&for_glob, temp.path()).contains(&".hidden.rs".to_string()));

        let for_grep = walk_local(
            temp.path().to_path_buf(),
            None,
            /*include_hidden*/ true,
        )
        .expect("walk");
        let found = names(&for_grep, temp.path());
        assert!(found.contains(&".hidden.rs".to_string()), "{found:?}");
        assert!(
            !found.iter().any(|name| name.starts_with(".git/")),
            "`.git` stays excluded even with hidden files on: {found:?}"
        );
    }

    #[test]
    fn glob_renders_one_path_per_line_and_uses_opencodes_truncation_sentence() {
        let paths: Vec<String> = (0..MAX_GLOB_PATHS + 5)
            .map(|index| format!("C:\\repo\\file{index}.rs"))
            .collect();
        let out = render_glob(&paths, /*walk_truncated*/ false);
        assert!(out.starts_with("C:\\repo\\file0.rs\n"), "{out}");
        assert!(
            out.contains("(Results are truncated: showing first 100 results."),
            "{out}"
        );
    }

    #[test]
    fn glob_says_no_files_found_when_nothing_matched() {
        assert_eq!(render_glob(&[], /*walk_truncated*/ false), "No files found");
    }

    /// Including the sentinel: OpenCode's grep says "No files found", not "no matches".
    #[test]
    fn grep_says_no_files_found_when_nothing_matched() {
        assert_eq!(
            render_grep(0, &[], /*stopped_early*/ false),
            "No files found"
        );
    }

    /// An unsaturated result is untouched by the count map, byte for byte.
    #[test]
    fn grep_uses_opencodes_layout() {
        let per_file = vec![FileMatches {
            display: "C:\\repo\\a.rs".to_string(),
            matched: 1,
            lines: vec![(791, "    pub async fn connect(".to_string())],
        }];
        assert_eq!(
            render_grep(1, &per_file, /*stopped_early*/ false),
            "Found 1 matches\nC:\\repo\\a.rs:\n  Line 791:     pub async fn connect(\n\n"
        );
    }

    #[test]
    fn a_saturated_grep_returns_counts_instead_of_a_partial_dump() {
        let per_file = vec![
            FileMatches {
                display: "a.rs".to_string(),
                matched: MAX_GREP_MATCHES,
                lines: (0..MAX_GREP_MATCHES)
                    .map(|n| (n + 1, "x".to_string()))
                    .collect(),
            },
            // The file the old renderer dropped: every match of its own landed past the cap.
            FileMatches {
                display: "b.rs".to_string(),
                matched: 76,
                lines: Vec::new(),
            },
        ];
        let out = render_grep(176, &per_file, /*stopped_early*/ false);
        assert!(
            out.starts_with("Found 176 matches in 2 files - too many to list in full.\n"),
            "{out}"
        );
        // The coordinate tier, because the coordinates fit: every line number of `a.rs`, and the
        // count for the file whose own matches all landed past the cap. Naming that file at all
        // is what the old renderer failed to do.
        assert!(out.contains("\n  a.rs: 1, 2, 3, "), "{out}");
        assert!(out.contains("\n  b.rs: 76 matches\n"), "{out}");
        assert!(out.contains("First 5 matches:\n"), "{out}");
        assert!(
            out.contains("Read a line with `read` and its number"),
            "{out}"
        );
        // Still a fraction of the dump it replaces.
        assert!(out.len() < 4_000, "{}", out.len());
    }

    /// A search that gave up has counted only what it reached, and the result has to say so.
    #[test]
    fn a_partial_search_says_its_counts_are_partial() {
        let per_file = vec![FileMatches {
            display: "a.rs".to_string(),
            matched: 2,
            lines: vec![(1, "x".to_string()), (2, "y".to_string())],
        }];
        let out = render_grep(2, &per_file, /*stopped_early*/ true);
        assert!(out.contains("the search stopped early"), "{out}");
    }

    #[test]
    fn grep_output_stays_bounded_on_pathological_input() {
        let per_file: Vec<FileMatches> = (0..40)
            .map(|file| FileMatches {
                display: format!("C:\\repo\\f{file}.rs"),
                matched: 20,
                lines: (0..20)
                    .map(|line| (line + 1, "x".repeat(MAX_MATCH_LINE_BYTES)))
                    .collect(),
            })
            .collect();
        let out = render_grep(800, &per_file, /*stopped_early*/ false);
        // Against the map's own ceiling, not the line dump's: at `MAX_GREP_OUTPUT_BYTES + 256`
        // this assertion passed while the map was reaching 29 KB, four times the dump it replaces.
        assert!(out.len() <= MAX_MAP_OUTPUT_BYTES, "{}", out.len());
        assert!(out.contains("too many to list in full"), "{out}");
    }

    /// The worst shape the map can be asked for: every slot full and every path long.
    #[test]
    fn the_map_stays_under_its_ceiling_with_long_paths_and_long_samples() {
        let per_file: Vec<FileMatches> = (0..MAX_MAP_FILES + 40)
            .map(|file| FileMatches {
                display: format!("codex-rs/core/src/tools/handlers/deeply/nested/f{file}.rs"),
                matched: 9,
                lines: vec![(1, "x".repeat(MAX_MATCH_LINE_BYTES))],
            })
            .collect();
        let out = render_grep(
            9 * (MAX_MAP_FILES + 40),
            &per_file,
            /*stopped_early*/ false,
        );
        assert!(out.len() <= MAX_MAP_OUTPUT_BYTES, "{}", out.len());
        // The samples and the closing sentence survive a file list long enough to crowd them out.
        assert!(out.contains("First 5 matches:\n"), "{out}");
        assert!(out.contains("Narrow the pattern or the path"), "{out}");
    }

    /// A sample shows the shape of a hit; it is not a way to carry the line.
    #[test]
    fn a_sampled_line_is_clipped_harder_than_a_listed_one() {
        let per_file: Vec<FileMatches> = (0..2)
            .map(|file| FileMatches {
                display: format!("src/f{file}.rs"),
                matched: MAX_GREP_MATCHES,
                lines: vec![(1, "y".repeat(MAX_MATCH_LINE_BYTES))],
            })
            .collect();
        let out = render_grep(
            2 * MAX_GREP_MATCHES,
            &per_file,
            /*stopped_early*/ false,
        );
        assert!(
            !out.contains(&"y".repeat(MAP_SAMPLE_LINE_BYTES + 1)),
            "a sample carried more than {MAP_SAMPLE_LINE_BYTES} bytes of its line"
        );
        assert!(out.contains(&"y".repeat(MAP_SAMPLE_LINE_BYTES)), "{out}");
    }

    #[test]
    fn a_result_path_is_written_relative_to_the_cwd() {
        // Built with `join` rather than a literal so the separators are the host's: a hard-coded
        // `C:\repo\src\lib.rs` is a single component on Unix and would strip to nothing there.
        let cwd = std::path::Path::new("repo");
        let file = cwd.join("src").join("lib.rs");
        let expected = std::path::Path::new("src").join("lib.rs");
        assert_eq!(
            display_path(Some(cwd), &file.to_string_lossy()),
            expected.to_string_lossy().into_owned()
        );
    }

    /// With several environments a relative path is ambiguous, so the tools keep the absolute one.
    #[test]
    fn several_environments_keep_absolute_paths() {
        let cwd = std::path::Path::new("repo");
        let one = SearchToolOptions {
            include_environment_id: false,
        };
        let many = SearchToolOptions {
            include_environment_id: true,
        };
        assert_eq!(display_base(&one, cwd), Some(cwd));
        assert_eq!(display_base(&many, cwd), None);

        let file = cwd.join("src").join("lib.rs");
        let file = file.to_string_lossy().into_owned();
        assert_eq!(display_path(display_base(&many, cwd), &file), file);
    }

    /// A file above the cwd has no relative name worth printing, so it keeps the absolute one.
    #[test]
    fn a_result_path_outside_the_cwd_stays_absolute() {
        let cwd = std::path::Path::new("repo");
        let outside = std::path::Path::new("elsewhere").join("lib.rs");
        let outside = outside.to_string_lossy().into_owned();
        assert_eq!(display_path(Some(cwd), &outside), outside);
        // The cwd itself is not a result, but it must not render as an empty path either.
        assert_eq!(display_path(Some(cwd), "repo"), "repo".to_string());
    }

    /// The bug: a `path` naming a file used to be answered with its whole parent directory.
    #[test]
    fn a_path_that_names_a_file_selects_that_file_alone() {
        let temp = tempfile::tempdir().expect("temp");
        seed(temp.path());

        let file = PathUri::from_host_native_path(temp.path().join("src/lib.rs")).expect("uri");
        assert_eq!(
            single_file_target(&file),
            Some(file.inferred_native_path_string())
        );

        let dir = PathUri::from_host_native_path(temp.path().join("src")).expect("uri");
        assert_eq!(
            single_file_target(&dir),
            None,
            "a directory is still walked"
        );
    }

    /// The fallback engine still answers, and answers with cwd-relative paths.
    #[test]
    fn the_in_process_scan_shortens_its_paths_against_the_cwd() {
        let temp = tempfile::tempdir().expect("temp");
        seed(temp.path());
        let files = vec![
            temp.path()
                .join("src")
                .join("lib.rs")
                .to_string_lossy()
                .into_owned(),
            temp.path()
                .join("src")
                .join("sse")
                .join("chat.rs")
                .to_string_lossy()
                .into_owned(),
        ];
        let regex = Regex::new("pub").expect("regex");

        let (found, per_file) =
            scan_in_process(files, Some(temp.path()), &regex, &ReadGuard::Unrestricted);

        assert_eq!(found, 2);
        let displays: Vec<String> = per_file
            .iter()
            .map(|file| file.display.replace('\\', "/"))
            .collect();
        assert_eq!(
            displays,
            vec!["src/lib.rs".to_string(), "src/sse/chat.rs".to_string()]
        );
    }

    /// The shape that motivated folding: 93 documents carrying one identical sentence.
    ///
    /// Taken from `impl2-sufficefork-rep20`, which searched 88 component documents for a line
    /// every one of them has and got 13,138 bytes of the same 140 characters. Nothing was capped -
    /// 93 matches is under `MAX_GREP_MATCHES` and 13 kB under the byte cap - so neither cap could
    /// have helped. Folding is lossless: the text once, every `file:line` kept.
    #[test]
    fn one_line_shared_by_many_files_is_written_once() {
        let sentence = "code changes its classification and is a contract violation".repeat(3);
        let per_file: Vec<FileMatches> = (0..93)
            .map(|file| FileMatches {
                display: format!("docs/components/c{file}.md"),
                matched: 1,
                lines: vec![(67, sentence.clone())],
            })
            .collect();

        let out = render_grep(93, &per_file, /*stopped_early*/ false);

        assert_eq!(out.matches(&sentence).count(), 1, "the text was repeated");
        assert!(out.starts_with("Found 93 matches\n"), "{out}");
        assert!(out.contains("93 files share this line:"), "{out}");
        // Lossless: every file and its line number is still there.
        for file in 0..93 {
            assert!(
                out.contains(&format!("docs/components/c{file}.md:67")),
                "c{file}.md lost its coordinate:\n{out}"
            );
        }
        // The measured shape was 13,138 bytes for this much text.
        assert!(out.len() < 4_500, "folded to {} bytes", out.len());
    }

    /// Two files sharing a line is not worth the folded form's own framing.
    #[test]
    fn a_line_shared_by_two_files_is_left_alone() {
        let per_file: Vec<FileMatches> = (0..2)
            .map(|file| FileMatches {
                display: format!("src/f{file}.rs"),
                matched: 1,
                lines: vec![(7, "pub fn connect()".to_string())],
            })
            .collect();

        let out = render_grep(2, &per_file, /*stopped_early*/ false);

        assert!(!out.contains("share this line"), "{out}");
        assert_eq!(out.matches("pub fn connect()").count(), 2, "{out}");
    }

    /// A file keeps its own listing for the lines that are not shared.
    #[test]
    fn folding_leaves_a_files_unshared_lines_in_place() {
        let shared = "use std::fmt;".to_string();
        let mut per_file: Vec<FileMatches> = (0..4)
            .map(|file| FileMatches {
                display: format!("src/f{file}.rs"),
                matched: 1,
                lines: vec![(1, shared.clone())],
            })
            .collect();
        per_file[0].matched = 2;
        per_file[0]
            .lines
            .push((42, "pub fn only_here()".to_string()));

        let out = render_grep(5, &per_file, /*stopped_early*/ false);

        assert_eq!(out.matches(&shared).count(), 1, "{out}");
        assert!(out.contains("  Line 42: pub fn only_here()"), "{out}");
        assert!(out.contains("src/f0.rs:"), "{out}");
    }

    /// The coordinate tier: too much text to carry, but every line number fits.
    #[test]
    fn a_capped_result_gives_coordinates_when_they_fit() {
        let per_file: Vec<FileMatches> = (0..30)
            .map(|file| FileMatches {
                display: format!("src/f{file}.rs"),
                matched: 4,
                lines: (0..4).map(|n| (n * 17 + 3, "x".repeat(300))).collect(),
            })
            .collect();

        let out = render_grep(120, &per_file, /*stopped_early*/ false);

        // Coordinates, not counts: the model can write a citation from this without another call.
        assert!(out.contains("Lines per file:\n"), "{out}");
        assert!(out.contains("  src/f0.rs: 3, 20, 37, 54\n"), "{out}");
        assert!(!out.contains("Matches per file:"), "{out}");
        // And a taste of what matched, which coordinates alone cannot give.
        assert!(out.contains("First 5 matches:\n"), "{out}");
        assert!(out.len() <= MAX_MAP_OUTPUT_BYTES, "{}", out.len());
    }

    /// A file whose matches ran past the cap must not read as fully enumerated.
    #[test]
    fn coordinates_say_so_when_they_are_a_subset_of_a_files_matches() {
        let per_file = vec![FileMatches {
            display: "src/big.rs".to_string(),
            matched: 40,
            lines: vec![(5, "x".repeat(400)), (9, "x".repeat(400))],
        }];

        let out = render_grep(
            MAX_GREP_MATCHES + 40,
            &per_file,
            /*stopped_early*/ false,
        );

        assert!(
            out.contains("  src/big.rs: 5, 9 (of 40 matches)\n"),
            "{out}"
        );
    }

    /// The map has its own cap, or one pathological pattern turns it into the dump it replaces.
    #[test]
    fn the_count_map_is_itself_bounded() {
        let per_file: Vec<FileMatches> = (0..MAX_MAP_FILES + 50)
            .map(|file| FileMatches {
                display: format!("C:\\repo\\f{file}.rs"),
                matched: 3,
                lines: Vec::new(),
            })
            .collect();
        let out = render_grep(
            3 * (MAX_MAP_FILES + 50),
            &per_file,
            /*stopped_early*/ false,
        );
        assert!(out.contains("... and 50 more files"), "{out}");
        assert!(!out.contains("f250.rs"), "{out}");
    }

    /// The load-bearing test of the engine swap: on one tree, both engines must render the same
    /// answer, byte for byte. It covers ordering, line numbering, the `\r` trim, `.gitignore`,
    /// hidden files and `.git` exclusion in a single assertion.
    ///
    /// Skipped when ripgrep cannot be run - a source build resolves it to a bare name that may not
    /// be on `PATH`, and a sandbox without it must not fail the suite. To exercise it locally, put
    /// `rg` on `PATH` first.
    #[test]
    fn both_engines_render_the_same_answer() {
        let temp = tempfile::tempdir().expect("temp");
        seed(temp.path());
        let cwd = temp.path().to_path_buf();
        let pattern = "pub fn";

        let request = search_rg::RgRequest {
            program: InstallContext::current().rg_command(),
            cwd: cwd.clone(),
            display_base: Some(cwd.clone()),
            target: cwd.clone(),
            pattern: pattern.to_string(),
            include: None,
            read_guard: ReadGuard::Unrestricted,
        };
        let cancellation = tokio_util::sync::CancellationToken::new();
        let Ok(Some(scan)) = search_rg::run_rg(&request, &cancellation) else {
            return;
        };

        let walked = walk_local(cwd.clone(), None, /*include_hidden*/ true).expect("walk");
        let regex = Regex::new(pattern).expect("regex");
        let (found, per_file) =
            scan_in_process(walked.files, Some(&cwd), &regex, &ReadGuard::Unrestricted);

        assert!(found > 0, "the fixture must match something");
        assert_eq!(
            render_grep(scan.found, &scan.per_file, scan.stopped_early),
            render_grep(found, &per_file, /*stopped_early*/ false),
        );
    }

    #[test]
    fn a_long_matched_line_is_clipped_on_a_character_boundary() {
        let line = "ö".repeat(2_000);
        let clipped = clip_to_bytes(&line, MAX_MATCH_LINE_BYTES);
        assert!(clipped.len() <= MAX_MATCH_LINE_BYTES);
        assert!(line.starts_with(clipped));
    }
}
