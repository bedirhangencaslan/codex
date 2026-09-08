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
//! installed as an `Override` exactly as `rg --glob=` installs it. No process is spawned, so the
//! sandbox story is unchanged.
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
use codex_utils_path_uri::PathUri;
use ignore::WalkBuilder;
use ignore::overrides::OverrideBuilder;
use regex_lite::Regex;
use serde_json::Value;
use std::path::PathBuf;

use crate::function_tool::FunctionCallError;
use crate::tools::context::FunctionToolOutput;
use crate::tools::context::ToolInvocation;
use crate::tools::context::ToolOutput;
use crate::tools::context::ToolPayload;
use crate::tools::context::boxed_tool_output;
use crate::tools::handlers::resolve_tool_environment;
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
const MAX_GREP_MATCHES: usize = 100;
/// OpenCode caps a matched line at 2000 characters and appends `...`.
const MAX_MATCH_LINE_BYTES: usize = 2_000;
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

fn clip_to_bytes(text: &str, max: usize) -> &str {
    if text.len() <= max {
        return text;
    }
    let mut end = max;
    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }
    &text[..end]
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
            payload,
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
        let walked = walk(
            filesystem,
            sandbox,
            &root,
            Some(pattern),
            /*include_hidden*/ false,
        )
        .await?;

        Ok(boxed_tool_output(FunctionToolOutput::from_text(
            render_glob(&walked.files, walked.truncated),
            Some(true),
        )))
    }
}

/// OpenCode's layout: absolute paths, one per line, then its own truncation sentence.
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

struct FileMatches {
    display: String,
    lines: Vec<(usize, String)>,
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
            payload,
            ..
        } = invocation;
        let arguments = function_arguments(payload)?;
        let pattern = string_argument(&arguments, "pattern")?.ok_or_else(|| {
            FunctionCallError::RespondToModel("`pattern` is required".to_string())
        })?;
        let path = string_argument(&arguments, "path")?;
        let include = string_argument(&arguments, "include")?;
        let environment_id = string_argument(&arguments, "environment_id")?;

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
        let sandbox = turn_environment.sandbox_context(/*additional_permissions*/ None);
        let sandbox = Some(&sandbox);
        let filesystem = turn_environment.environment.get_filesystem();
        let filesystem = filesystem.as_ref();

        // OpenCode runs ripgrep with `cwd` = the path when it is a directory, its parent otherwise.
        let requested = search_root(turn_environment.cwd(), path.as_deref())?;
        let root = if requested.to_path_buf().is_file() {
            requested.parent().unwrap_or_else(|| requested.clone())
        } else {
            requested
        };
        // `--hidden` is unconditional for OpenCode's grep.
        let walked = walk(
            filesystem, sandbox, &root, include, /*include_hidden*/ true,
        )
        .await?;
        let walk_truncated = walked.truncated;

        let mut found = 0usize;
        let mut per_file: Vec<FileMatches> = Vec::new();
        for display in walked.files {
            let Ok(bytes) = std::fs::read(&display) else {
                continue;
            };
            if bytes.len() > MAX_SEARCHED_FILE_BYTES || bytes.contains(&0) {
                continue;
            }
            let text = String::from_utf8_lossy(&bytes);
            let mut lines = Vec::new();
            for (index, line) in text.lines().enumerate() {
                if !regex.is_match(line) {
                    continue;
                }
                found += 1;
                if found <= MAX_GREP_MATCHES {
                    let clipped = if line.len() > MAX_MATCH_LINE_BYTES {
                        format!("{}...", clip_to_bytes(line, MAX_MATCH_LINE_BYTES))
                    } else {
                        line.to_string()
                    };
                    lines.push((index + 1, clipped));
                }
            }
            if !lines.is_empty() {
                per_file.push(FileMatches { display, lines });
            }
        }

        Ok(boxed_tool_output(FunctionToolOutput::from_text(
            render_grep(found, &per_file, walk_truncated),
            Some(true),
        )))
    }
}

/// OpenCode's own layout, down to the blank line after each match and the header that says
/// "matches" whatever the count is.
fn render_grep(found: usize, per_file: &[FileMatches], walk_truncated: bool) -> String {
    if found == 0 {
        // OpenCode's empty-result sentinel says "files", not "matches". Copied as it is.
        return "No files found".to_string();
    }
    let saturated = found >= MAX_GREP_MATCHES;
    let mut out = format!(
        "Found {found} matches{}\n",
        if saturated {
            " (more matches available)"
        } else {
            ""
        }
    );
    let mut stopped = false;
    for file in per_file {
        let mut block = format!("{}:\n", file.display);
        for (line_no, text) in &file.lines {
            block.push_str(&format!("  Line {line_no}: {text}\n\n"));
        }
        if out.len() + block.len() > MAX_GREP_OUTPUT_BYTES {
            stopped = true;
            break;
        }
        out.push_str(&block);
    }
    if saturated || stopped || walk_truncated {
        out.push_str("\n(Results truncated. Consider using a more specific path or pattern.)\n");
    }
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
            render_grep(0, &[], /*walk_truncated*/ false),
            "No files found"
        );
    }

    #[test]
    fn grep_uses_opencodes_layout() {
        let per_file = vec![FileMatches {
            display: "C:\\repo\\a.rs".to_string(),
            lines: vec![(791, "    pub async fn connect(".to_string())],
        }];
        assert_eq!(
            render_grep(1, &per_file, /*walk_truncated*/ false),
            "Found 1 matches\nC:\\repo\\a.rs:\n  Line 791:     pub async fn connect(\n\n"
        );
    }

    #[test]
    fn grep_announces_saturation_the_way_opencode_does() {
        let per_file = vec![FileMatches {
            display: "a.rs".to_string(),
            lines: (0..MAX_GREP_MATCHES)
                .map(|n| (n + 1, "x".to_string()))
                .collect(),
        }];
        let out = render_grep(MAX_GREP_MATCHES, &per_file, /*walk_truncated*/ false);
        assert!(
            out.starts_with("Found 100 matches (more matches available)\n"),
            "{out}"
        );
        assert!(
            out.contains("(Results truncated. Consider using a more specific path or pattern.)"),
            "{out}"
        );
    }

    #[test]
    fn grep_output_stays_bounded_on_pathological_input() {
        let per_file: Vec<FileMatches> = (0..40)
            .map(|file| FileMatches {
                display: format!("C:\\repo\\f{file}.rs"),
                lines: (0..20)
                    .map(|line| (line + 1, "x".repeat(MAX_MATCH_LINE_BYTES)))
                    .collect(),
            })
            .collect();
        let out = render_grep(800, &per_file, /*walk_truncated*/ false);
        assert!(out.len() <= MAX_GREP_OUTPUT_BYTES + 256, "{}", out.len());
        assert!(out.contains("Results truncated"), "{out}");
    }

    #[test]
    fn a_long_matched_line_is_clipped_on_a_character_boundary() {
        let line = "ö".repeat(2_000);
        let clipped = clip_to_bytes(&line, MAX_MATCH_LINE_BYTES);
        assert!(clipped.len() <= MAX_MATCH_LINE_BYTES);
        assert!(line.starts_with(clipped));
    }
}
