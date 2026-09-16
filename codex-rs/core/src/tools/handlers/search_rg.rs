//! `grep`'s fast engine: the bundled ripgrep, read back as JSON Lines.
//!
//! The in-process engine next door reads every candidate file whole, decodes it and runs the
//! pattern over each line, one file at a time. On this repository a root-level search walks 6,881
//! files and 69.6 MB that way, and `std::fs::read` blocks while it does. ripgrep streams, spreads
//! the work over the cores, and prefilters with SIMD: measured on that same tree, `--json` with a
//! pattern as broad as `pub fn ` finishes in **804 ms** and emits 1.3 MB for 3,827 matches, and a
//! realistic search of one crate's `src` takes **49 ms**. Those two numbers are also why this
//! module drains the whole stream rather than killing the child at the match cap - the scan is
//! already paid for, and parsing the rest is what keeps `Found N matches` a total instead of a
//! floor.
//!
//! It was already half of this tool - the walker is ripgrep's own `ignore` crate - and it is
//! already shipped: `InstallContext::rg_command()` resolves `codex-path/rg`, which the packaging,
//! the installers and CI all place.
//!
//! Three things this module is careful about, each a way the swap could have changed what the model
//! sees rather than only how fast it sees it:
//!
//! 1. **Determinism.** ripgrep is multi-threaded, so files arrive in no particular order. Taking
//!    the first hundred matches as they land and sorting afterwards would return a different
//!    hundred on every identical call. `--sort=path` would fix that and ripgrep's own help says it
//!    is "Always single-threaded", which gives back the speed this change is for. So the retained
//!    set is kept as the hundred smallest `(path, line)` keys instead: "the first hundred in sorted
//!    order" and "the hundred smallest keys" are the same hundred, and a `BTreeMap` holds them in
//!    O(cap) memory whatever order they arrive in.
//! 2. **`\r`.** The in-process engine splits with `str::lines()`, which drops a trailing `\r`, so
//!    `foo$` matches on a CRLF file today. ripgrep keeps the `\r` in the line unless told not to,
//!    which would make `$` quietly stop matching on any CRLF checkout. Hence `--crlf`.
//! 3. **The environment.** `RIPGREP_CONFIG_PATH` lets a config file inject flags into every run,
//!    including `-i` and `--pre=<program>`, and `--pre` makes ripgrep execute an arbitrary program
//!    per file. The child is spawned with a cleared environment and `--no-config`.
//!
//! The scan runs synchronously under `spawn_blocking`, the way `walk_local` already does. Streaming
//! it on the async runtime would mean adding tokio's `io-util` and `time` features to this crate,
//! and a blocking read on a blocking pool is what this work actually is.

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::time::Duration;
use std::time::Instant;

use serde::Deserialize;
use tokio_util::sync::CancellationToken;

use crate::function_tool::FunctionCallError;
use crate::tools::handlers::search::FileMatches;
use crate::tools::handlers::search::MAX_GREP_MATCHES;
use crate::tools::handlers::search::MAX_MATCH_LINE_BYTES;
use crate::tools::handlers::search::clip_to_bytes;
use crate::tools::handlers::search::display_path;

/// Backstop only. The deterministic guard is [`MAX_RG_MATCH_EVENTS`]; a wall-clock limit cuts a
/// different result depending on how loaded the machine is, so it is not allowed to be the usual
/// way a search ends.
const RG_TIMEOUT: Duration = Duration::from_secs(60);
/// A single JSON event. One matched line plus its path and offsets; a quarter megabyte is already
/// far past `MAX_MATCH_LINE_BYTES` and the surrounding envelope.
const MAX_RG_LINE_BYTES: usize = 256 * 1024;
/// Match events read before the scan gives up and says its counts are a floor.
const MAX_RG_MATCH_EVENTS: usize = 500_000;
/// Files named in the count map before it stops growing. `render_map` lists 200 of them, and the
/// rest only ever become the "and N more" tail, so carrying millions buys nothing.
const MAX_MATCHED_FILES: usize = 20_000;
/// How much of ripgrep's own diagnostics is kept to put in an error message.
const MAX_RG_STDERR_BYTES: usize = 8 * 1024;

/// One `grep` call, as ripgrep needs it.
pub(super) struct RgRequest {
    /// From `InstallContext::rg_command()`; may be a bare name resolved through `PATH`.
    pub(super) program: PathBuf,
    /// The child's working directory.
    pub(super) cwd: PathBuf,
    /// What result paths are shortened against, or `None` to keep them absolute. Separate from
    /// `cwd` because the two answer different questions: where ripgrep runs, and what the model
    /// should be shown. See `search::display_base`.
    pub(super) display_base: Option<PathBuf>,
    /// A file or a directory. A file is searched as itself - the whole point of the fix next door.
    pub(super) target: PathBuf,
    pub(super) pattern: String,
    /// `None` when `target` is a file: a glob filters a directory walk and has nothing to filter.
    pub(super) include: Option<String>,
}

/// What either engine produces, before rendering.
pub(super) struct GrepScan {
    pub(super) found: usize,
    pub(super) per_file: Vec<FileMatches>,
    /// The counts are a floor rather than a total, so the result has to say so.
    pub(super) stopped_early: bool,
}

/// ripgrep's argument list.
///
/// Every token is its own argument and the pattern travels under `--regexp`, never as a positional:
/// a pattern that begins with `-` would otherwise be read as a flag, and `--` alone does not help
/// because the target has to follow it. Kept separate from [`run_rg`] so the mapping is testable
/// without a process.
///
/// Deliberately absent, each for a reason:
///
/// - `--case-sensitive`: ripgrep is already case-sensitive by default (smart case is `-S`, opt-in),
///   so this matches the in-process engine without asking.
/// - `--max-filesize`: the in-process engine's 1 MiB limit exists because it reads files whole.
///   ripgrep streams, so the limit has no purpose here and large files are searched.
/// - `--sort=path`: single-threaded, see the module header.
/// - `--max-depth`: the in-process walker caps depth, but a depth cap drops files with nothing to
///   report it; unlimited is the honest default.
/// - an exclusion for `.git`: ripgrep skips it even under `--hidden`.
pub(super) fn rg_args(request: &RgRequest) -> Vec<OsString> {
    let mut args: Vec<OsString> = vec![
        "--json".into(),
        // OpenCode's grep passes `--hidden` unconditionally, and so does the engine next door.
        "--hidden".into(),
        "--crlf".into(),
        "--no-config".into(),
    ];
    if let Some(include) = &request.include {
        args.push("--glob".into());
        args.push(include.into());
    }
    args.push("--regexp".into());
    args.push((&request.pattern).into());
    args.push("--".into());
    args.push(request.target.clone().into_os_string());
    args
}

/// Runs ripgrep and collects its output.
///
/// `Ok(None)` means ripgrep could not be run at all and the caller should fall back to the
/// in-process engine - the shape `rollout::search::ripgrep_rollout_paths` uses for the same reason.
/// A search that ran and found nothing is `Ok(Some(..))` with `found == 0`.
pub(super) fn run_rg(
    request: &RgRequest,
    cancellation: &CancellationToken,
) -> Result<Option<GrepScan>, FunctionCallError> {
    let mut child = match Command::new(&request.program)
        .args(rg_args(request))
        .current_dir(&request.cwd)
        // `RIPGREP_CONFIG_PATH` can inject `--pre=<program>`, which runs an arbitrary program per
        // file. Nothing in the environment is needed to search a directory.
        .env_clear()
        // ripgrep reads stdin when it thinks it has one, and then waits forever.
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(error) => {
            // Including `NotFound`: a source build resolves `rg` to a bare name that may not be on
            // `PATH`. The tool must not fail because the binary was missing or odd.
            tracing::debug!("grep falling back to the in-process engine: {error}");
            return Ok(None);
        }
    };

    // Drained on its own thread: ripgrep reports unreadable files here, and a full stderr pipe
    // that nobody reads blocks the child until the timeout.
    let stderr = child.stderr.take();
    let stderr_drain = std::thread::spawn(move || {
        let mut kept = Vec::new();
        if let Some(mut stderr) = stderr {
            let mut buffer = [0u8; 4096];
            while let Ok(read) = stderr.read(&mut buffer) {
                if read == 0 {
                    break;
                }
                let room = MAX_RG_STDERR_BYTES.saturating_sub(kept.len());
                if room > 0 {
                    kept.extend_from_slice(&buffer[..read.min(room)]);
                }
            }
        }
        String::from_utf8_lossy(&kept).into_owned()
    });

    let mut collector = Collector::new(request.display_base.as_deref());
    let deadline = Instant::now() + RG_TIMEOUT;
    let mut killed = false;
    if let Some(stdout) = child.stdout.take() {
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        loop {
            line.clear();
            let read = (&mut reader)
                .take(MAX_RG_LINE_BYTES as u64 + 1)
                .read_line(&mut line);
            match read {
                Ok(0) | Err(_) => break,
                Ok(_) => {}
            }
            if line.len() > MAX_RG_LINE_BYTES {
                // The match this event carried cannot be counted, so the totals are a floor.
                collector.stopped_early = true;
                continue;
            }
            collector.accept(trim_line_ending(&line));
            if collector.found >= MAX_RG_MATCH_EVENTS || Instant::now() > deadline {
                collector.stopped_early = true;
                killed = true;
                break;
            }
            if cancellation.is_cancelled() {
                let _ = child.kill();
                let _ = child.wait();
                let _ = stderr_drain.join();
                return Err(FunctionCallError::RespondToModel(
                    "grep was cancelled".to_string(),
                ));
            }
        }
    }
    if killed {
        let _ = child.kill();
    }
    let status = child.wait();
    let stderr = stderr_drain.join().unwrap_or_default();

    let scan = collector.finish();
    // Trust the data over the exit code: ripgrep exits 1 when it simply found nothing and 2 when a
    // single unreadable file turned up in an otherwise fine search, so a completed scan is rendered
    // whatever it returned.
    if scan.found > 0 || scan.completed_cleanly {
        if !stderr.is_empty() {
            tracing::debug!("grep: ripgrep reported {stderr}");
        }
        return Ok(Some(scan.into_result()));
    }
    let failed = status
        .map(|status| !matches!(status.code(), Some(0 | 1)))
        .unwrap_or(true);
    if failed {
        let detail = if stderr.is_empty() {
            "ripgrep exited without searching".to_string()
        } else {
            clip_to_bytes(stderr.trim(), 1_000).to_string()
        };
        return Err(FunctionCallError::RespondToModel(format!(
            "grep failed: {detail}"
        )));
    }
    Ok(Some(scan.into_result()))
}

/// One `\n`, then one `\r`, and nothing else.
///
/// `trim_end` would also eat trailing whitespace that the in-process engine keeps, and trailing
/// whitespace is sometimes the thing being searched for.
fn trim_line_ending(line: &str) -> &str {
    let line = line.strip_suffix('\n').unwrap_or(line);
    line.strip_suffix('\r').unwrap_or(line)
}

/// Accumulates ripgrep's events. No process and no filesystem, so it tests on its own.
struct Collector<'a> {
    cwd: Option<&'a Path>,
    found: usize,
    /// ripgrep's own path text to that file's match count. Its keys iterate in the same order the
    /// in-process engine's `files.sort()` produces: every path shares the target prefix, so
    /// removing that prefix later cannot reorder them.
    files: BTreeMap<String, usize>,
    /// The `MAX_GREP_MATCHES` smallest `(path, line)` matches - see the module header.
    retained: BTreeMap<(String, usize), String>,
    completed: bool,
    stopped_early: bool,
}

/// A finished collection, before it is turned into the renderer's shape.
struct Collected {
    found: usize,
    per_file: Vec<FileMatches>,
    stopped_early: bool,
    completed_cleanly: bool,
}

impl Collected {
    fn into_result(self) -> GrepScan {
        GrepScan {
            found: self.found,
            per_file: self.per_file,
            stopped_early: self.stopped_early,
        }
    }
}

impl<'a> Collector<'a> {
    fn new(cwd: Option<&'a Path>) -> Self {
        Self {
            cwd,
            found: 0,
            files: BTreeMap::new(),
            retained: BTreeMap::new(),
            completed: false,
            stopped_early: false,
        }
    }

    /// Reads one JSON Lines event. A blank line, an unknown event type and a malformed line are all
    /// skipped and the stream continues, which is what the JSON-RPC reader in `exec-server` does:
    /// one bad line is not a reason to throw away a search.
    fn accept(&mut self, line: &str) {
        if line.is_empty() {
            return;
        }
        let event = match serde_json::from_str::<RgEvent>(line) {
            Ok(event) => event,
            Err(_) => {
                self.stopped_early = true;
                return;
            }
        };
        let data = match event {
            RgEvent::Summary => {
                self.completed = true;
                return;
            }
            RgEvent::Match { data } => data,
            RgEvent::Other => return,
        };
        // A path ripgrep could not render as text arrives as `bytes` instead. Inventing a name for
        // it would be worse than admitting the result is partial.
        let (Some(path), Some(line_number)) = (data.path.text, data.line_number) else {
            self.stopped_early = true;
            return;
        };
        self.found += 1;
        // The ceiling only bites on a file the map has not seen yet, so an already-counted file
        // keeps counting however many files came before it.
        if !self.files.contains_key(&path) && self.files.len() >= MAX_MATCHED_FILES {
            self.stopped_early = true;
            return;
        }
        *self.files.entry(path.clone()).or_insert(0) += 1;
        let text = data.lines.text.unwrap_or_default();
        let text = trim_line_ending(&text);
        let text = if text.len() > MAX_MATCH_LINE_BYTES {
            format!("{}...", clip_to_bytes(text, MAX_MATCH_LINE_BYTES))
        } else {
            text.to_string()
        };
        self.retained.insert((path, line_number), text);
        if self.retained.len() > MAX_GREP_MATCHES {
            self.retained.pop_last();
        }
    }

    fn finish(self) -> Collected {
        let Self {
            cwd,
            found,
            files,
            retained,
            completed,
            stopped_early,
        } = self;
        // `retained` iterates in `(path, line)` order, so each file's lines come out already
        // sorted by line number - the order the in-process engine produces by reading top to
        // bottom.
        let mut grouped: BTreeMap<String, Vec<(usize, String)>> = BTreeMap::new();
        for ((path, line_number), text) in retained {
            grouped.entry(path).or_default().push((line_number, text));
        }
        let per_file = files
            .into_iter()
            .map(|(path, matched)| FileMatches {
                lines: grouped.remove(&path).unwrap_or_default(),
                display: display_path(cwd, &path),
                matched,
            })
            .collect();
        Collected {
            found,
            per_file,
            stopped_early,
            completed_cleanly: completed,
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum RgEvent {
    Match {
        data: RgMatch,
    },
    Summary,
    /// `begin`, `end` and `context`: nothing here needs them. Per-file counts are taken from the
    /// match events rather than from `end.stats`, so a scan cut short still reports numbers that
    /// agree with the lines it kept.
    #[serde(other)]
    Other,
}

#[derive(Deserialize)]
struct RgMatch {
    path: RgText,
    lines: RgText,
    line_number: Option<usize>,
}

#[derive(Deserialize)]
struct RgText {
    text: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn match_event(path: &str, line: usize, text: &str) -> String {
        serde_json::json!({
            "type": "match",
            "data": {
                "path": {"text": path},
                "lines": {"text": text},
                "line_number": line,
            }
        })
        .to_string()
    }

    fn collect(cwd: &Path, lines: &[String]) -> Collected {
        let mut collector = Collector::new(Some(cwd));
        for line in lines {
            collector.accept(line);
        }
        collector.finish()
    }

    #[test]
    fn files_come_back_sorted_however_they_arrived() {
        let cwd = Path::new("repo");
        let out = collect(
            cwd,
            &[
                match_event("repo/z.rs", 2, "hit z"),
                match_event("repo/a.rs", 9, "hit a2"),
                match_event("repo/a.rs", 1, "hit a1"),
            ],
        );
        assert_eq!(out.found, 3);
        let names: Vec<&str> = out.per_file.iter().map(|f| f.display.as_str()).collect();
        assert_eq!(names, vec!["a.rs", "z.rs"]);
        assert_eq!(
            out.per_file[0].lines,
            vec![(1, "hit a1".to_string()), (9, "hit a2".to_string())]
        );
    }

    /// ripgrep keeps the `\r` that `str::lines()` drops, and `--crlf` only fixes the matching.
    #[test]
    fn a_crlf_line_loses_exactly_its_line_ending() {
        let out = collect(
            Path::new("repo"),
            &[match_event("repo/a.rs", 1, "trailing  \r\n")],
        );
        assert_eq!(out.per_file[0].lines[0].1, "trailing  ".to_string());
    }

    /// The determinism guarantee: the retained set is the smallest keys, not the first arrivals.
    #[test]
    fn the_retained_lines_are_the_smallest_keys_whatever_the_arrival_order() {
        let mut events = Vec::new();
        // Arrive worst-first, so "first hundred seen" and "hundred smallest" cannot coincide.
        for line in (1..=(MAX_GREP_MATCHES + 50)).rev() {
            events.push(match_event("repo/a.rs", line, "x"));
        }
        let out = collect(Path::new("repo"), &events);
        assert_eq!(out.found, MAX_GREP_MATCHES + 50);
        assert_eq!(out.per_file[0].matched, MAX_GREP_MATCHES + 50);
        let kept: Vec<usize> = out.per_file[0].lines.iter().map(|(no, _)| *no).collect();
        assert_eq!(kept.len(), MAX_GREP_MATCHES);
        assert_eq!(kept.first(), Some(&1));
        assert_eq!(kept.last(), Some(&MAX_GREP_MATCHES));
    }

    #[test]
    fn a_malformed_or_unknown_event_is_skipped_and_collection_continues() {
        let out = collect(
            Path::new("repo"),
            &[
                "{not json".to_string(),
                serde_json::json!({"type": "begin", "data": {"path": {"text": "repo/a.rs"}}})
                    .to_string(),
                String::new(),
                match_event("repo/a.rs", 1, "hit"),
            ],
        );
        assert_eq!(out.found, 1);
        assert!(
            out.stopped_early,
            "a malformed line makes the counts a floor"
        );
    }

    #[test]
    fn a_summary_event_is_what_says_the_scan_finished() {
        let with = collect(
            Path::new("repo"),
            &[
                match_event("repo/a.rs", 1, "hit"),
                serde_json::json!({"type": "summary", "data": {}}).to_string(),
            ],
        );
        assert!(with.completed_cleanly);
        let without = collect(Path::new("repo"), &[match_event("repo/a.rs", 1, "hit")]);
        assert!(!without.completed_cleanly);
    }

    /// A path ripgrep delivered as bytes has no name to print, so the file is dropped and the
    /// result admits it is partial.
    #[test]
    fn a_path_without_text_is_skipped_and_flagged() {
        let event = serde_json::json!({
            "type": "match",
            "data": {
                "path": {"bytes": "3q2+7w=="},
                "lines": {"text": "hit"},
                "line_number": 1,
            }
        })
        .to_string();
        let out = collect(Path::new("repo"), &[event]);
        assert_eq!(out.found, 0);
        assert!(out.per_file.is_empty());
        assert!(out.stopped_early);
    }

    #[test]
    fn a_long_matched_line_is_clipped_the_way_the_other_engine_clips_it() {
        let long = "z".repeat(MAX_MATCH_LINE_BYTES + 500);
        let out = collect(Path::new("repo"), &[match_event("repo/a.rs", 1, &long)]);
        let kept = &out.per_file[0].lines[0].1;
        assert!(kept.ends_with("..."), "{kept:.40}");
        assert_eq!(kept.len(), MAX_MATCH_LINE_BYTES + 3);
    }

    #[test]
    fn a_result_path_is_shortened_against_the_cwd_and_kept_when_it_cannot_be() {
        let cwd = Path::new("repo");
        let inside = cwd.join("src").join("lib.rs");
        let out = collect(
            cwd,
            &[
                match_event(&inside.to_string_lossy(), 1, "hit"),
                match_event("elsewhere/lib.rs", 1, "hit"),
            ],
        );
        let names: Vec<&str> = out.per_file.iter().map(|f| f.display.as_str()).collect();
        assert!(
            names.contains(&"elsewhere/lib.rs"),
            "a path outside the cwd keeps its own name: {names:?}"
        );
        let expected = Path::new("src")
            .join("lib.rs")
            .to_string_lossy()
            .into_owned();
        assert!(names.contains(&expected.as_str()), "{names:?}");
    }

    #[test]
    fn the_glob_is_passed_only_when_one_was_asked_for() {
        let mut request = RgRequest {
            program: PathBuf::from("rg"),
            cwd: PathBuf::from("repo"),
            display_base: Some(PathBuf::from("repo")),
            target: PathBuf::from("repo/src"),
            pattern: "-dash-leading".to_string(),
            include: None,
        };
        let without = rg_args(&request);
        assert!(!without.iter().any(|arg| arg == "--glob"));
        // A pattern that starts with `-` must not be read as a flag.
        let regexp = without.iter().position(|arg| arg == "--regexp");
        assert_eq!(
            without
                .get(regexp.expect("--regexp") + 1)
                .map(OsString::as_os_str),
            Some(std::ffi::OsStr::new("-dash-leading"))
        );
        assert_eq!(
            without.last().map(OsString::as_os_str),
            Some(request.target.as_os_str())
        );

        request.include = Some("*.rs".to_string());
        let with = rg_args(&request);
        let glob = with.iter().position(|arg| arg == "--glob").expect("--glob");
        assert_eq!(
            with.get(glob + 1).map(OsString::as_os_str),
            Some(std::ffi::OsStr::new("*.rs"))
        );
    }
}
