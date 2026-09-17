//! Condensing of exec output on its way to the model.
//!
//! A coding agent spends most of its context on command output, and most of that output is
//! noise: terminal escape sequences, the dependency tree under `node_modules`, a build's
//! `Compiling ...` lines. What the model still needs is the summary at the end, plus enough
//! of the head to recognize what ran.
//!
//! Four stages, cheapest and safest first:
//!
//! 1. [`normalize`] is lossless and unconditional. Escape sequences and carriage-return
//!    overwrites carry no information a reader could act on, so there is no case in which
//!    keeping them is right.
//! 2. [`strip_line_ending_warnings`] is lossless for the same reason: git's `core.autocrlf`
//!    notice is addressed to whoever configured the repository, not to the command that
//!    tripped it.
//! 3. [`filter_artifact_paths`] drops listing lines that live under a build or dependency
//!    directory. Lossy, so it is gated on the command not having asked for one.
//! 4. [`shrink_text`] keeps the head and tail of output whose middle is a receipt.
//!
//! The first two lose nothing and are what [`condense_exec_output_lossless`] runs on its own,
//! for code mode: its result is handed to a program the model wrote, and a stage that announces
//! what it dropped is enough for a reader but not for a parser.
//!
//! All of it runs before the output is truncated to the model's budget, so noise is never
//! what pushes real content past the limit, and before the harness header is prepended, so
//! the head that is kept is the command's own first lines rather than the harness's metadata.
//! The condensed text is the only version the prompt ever holds, which is why this costs no
//! cache: rewriting history afterwards would re-prefill everything after the rewrite.

use std::sync::LazyLock;

use codex_utils_output_truncation::approx_token_count;
use regex_lite::Regex;

/// Lines kept from the start, so the model can still tell what ran.
const KEEP_HEAD_LINES: usize = 5;
/// Lines kept from the end, where these commands put their summary.
const KEEP_TAIL_LINES: usize = 30;
/// Shrinking below this size would remove almost nothing while still costing information.
const MIN_LINES_TO_SHRINK: usize = KEEP_HEAD_LINES + KEEP_TAIL_LINES + 10;

/// Path components whose contents are generated, vendored, or version-control internals.
///
/// A recursive listing of a JavaScript or Rust workspace is mostly these: one measured
/// `Get-ChildItem -Recurse` returned 500 lines of which 82% were under `node_modules`, and the
/// truncation that followed dropped the project's own files to make room for them.
/// The second half of this list comes from RTK's `NOISE_DIRS`, which names twenty-five where this
/// had twelve. Note what is *not* here, in both lists and for the same reason: `.env` is a file an
/// agent has to be able to see.
const ARTIFACT_DIRECTORIES: &[&str] = &[
    "node_modules",
    ".git",
    "dist",
    "build",
    ".next",
    "target",
    "__pycache__",
    ".venv",
    ".pytest_cache",
    ".mypy_cache",
    ".gradle",
    "vendor",
    "venv",
    "env",
    ".tox",
    ".eggs",
    "coverage",
    ".nyc_output",
    ".cache",
    ".turbo",
    ".vercel",
    ".idea",
    ".vscode",
    ".vs",
];

/// Below this, filtering costs a line of explanation to save less than it spends.
const MIN_ARTIFACT_LINES: usize = 10;

/// Status lines a build tool prints to show it is still alive.
///
/// Cargo right-aligns its verb in a twelve-column field, so every one of these arrives indented,
/// and the indent is load-bearing: it separates cargo's own `   Compiling foo v0.1.0` from a test
/// that prints `Compiling shaders` at column zero. This stage drops lines rather than rewriting
/// them, so a wrong match is a lost line and the narrower rule is the right one.
///
/// `Finished` is deliberately absent. It is one line, it carries the profile and the wall time,
/// and it is what a reader looks for to confirm the build got to the end.
const BUILD_PROGRESS_VERBS: &[&str] = &[
    "Compiling",
    "Checking",
    "Downloading",
    "Downloaded",
    "Fresh",
    "Blocking",
    "Waiting",
];

/// How many diagnostic blocks of each kind survive.
///
/// RTK's numbers and its reasoning: errors are the most actionable so they are shown the most,
/// warnings are the same shape at a lower signal density.
///
/// Note what this is *not*. It does not decide which lines of a diagnostic matter - every line of
/// an `error[E0308]` block is the compiler explaining itself, and guessing among them is how a
/// harness drops the one that mattered. It caps how many blocks are carried, because the
/// twentieth type error tells the model nothing the first three did not, and a workspace with a
/// bad signature emits one per call site. The rest are in the spill file.
const MAX_ERROR_BLOCKS: usize = 20;
const MAX_WARNING_BLOCKS: usize = 10;

/// What a lossy stage has to remove before it is worth running at all.
///
/// The stages below are gated on line counts, and a line count says nothing about size: a
/// `git commit` that prints fifty one-word lines is over every line threshold and still
/// smaller than the notice explaining what was cut. The notice is ~40 tokens in the header
/// plus ~16 in the body marker, so anything under roughly double that is a loss.
const MIN_LOSSY_TOKENS: usize = 120;
/// Above this share the listing *is* the artifact tree, so the model went looking for it.
const MAX_ARTIFACT_SHARE: f32 = 0.9;

/// CSI escape sequences, which is what colour and cursor movement are made of.
///
/// `regex-lite` is already a dependency of this crate. The `codex-ansi-escape` crate is not
/// usable here: it renders into `ratatui` types and would pull a TUI dependency into core.
static CSI_ESCAPE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\x1b\[[0-9;?]*[ -/]*[@-~]").unwrap());

/// The prefix every git line-ending warning starts with, and the cheapest way to rule one out.
const LINE_ENDING_WARNING_PREFIX: &str = "warning: in the working copy of '";

/// Recognizes git's notice that it is about to rewrite a file's line endings.
///
/// Matched as the whole sentence rather than on the word `warning`, because a compiler
/// diagnostic is the most valuable thing in a failing build's output and shares only its first
/// token. Both directions are spelled out: `core.autocrlf=true` writes the first, `input` the
/// second, and enumerating them avoids matching `LF will be replaced by LF`, which git never
/// prints. The path is whatever lies between, so an apostrophe in a filename does not end it
/// early.
fn is_line_ending_warning(line: &str) -> bool {
    let Some(rest) = line.strip_prefix(LINE_ENDING_WARNING_PREFIX) else {
        return false;
    };
    let Some(rest) = rest.strip_suffix(" the next time Git touches it") else {
        return false;
    };
    rest.ends_with("', LF will be replaced by CRLF")
        || rest.ends_with("', CRLF will be replaced by LF")
}

/// What condensing removed, so the harness can say so in the response header.
///
/// Counted rather than merely flagged: a model that is told output was altered but not how
/// much has no way to judge whether re-running the command would show it more.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CondenseReport {
    /// Terminal escape sequences removed.
    pub(crate) escape_sequences: usize,
    /// Lines that lost a carriage-return overwrite or a line terminator.
    pub(crate) rewritten_lines: usize,
    /// Git line-ending warnings dropped, one per line.
    pub(crate) line_ending_warnings: usize,
    /// Listing lines dropped for living under a build or dependency directory.
    pub(crate) artifact_lines: usize,
    /// Progress lines dropped from a build or test runner's output.
    pub(crate) progress_lines: usize,
    /// Diagnostic blocks dropped past the per-kind cap.
    pub(crate) dropped_errors: usize,
    pub(crate) dropped_warnings: usize,
    /// Middle lines dropped by head/tail shrinking.
    pub(crate) middle_lines: usize,
    /// Approximate tokens the whole pass removed.
    pub(crate) removed_tokens: usize,
}

impl CondenseReport {
    pub(crate) fn is_empty(&self) -> bool {
        *self == Self::default()
    }

    /// One line for the response header, naming what the model can no longer see.
    ///
    /// **Lossless removals are not announced.** Nothing was lost, so there is nothing the
    /// model could do differently, and the notice would cost more than it saved: a line of
    /// explanation on each affected output outweighs what the lossless stages remove from all
    /// of them. Announcing only the lossy stages keeps the notice on 12-25% of outputs.
    ///
    /// What it does say is short and load-bearing: a model that reads a gap, blames the
    /// command and runs it again spends a whole request, which is worth far more than this
    /// line.
    pub(crate) fn summary(&self) -> Option<String> {
        let mut causes = Vec::new();
        if self.middle_lines > 0 {
            causes.push(format!("{} middle lines", self.middle_lines));
        }
        if self.artifact_lines > 0 {
            causes.push(format!("{} generated-directory lines", self.artifact_lines));
        }
        if self.progress_lines > 0 {
            causes.push(format!("{} progress lines", self.progress_lines));
        }
        // Named by kind rather than counted together: "8 errors" tells the model there is more of
        // the same to fix, where "8 diagnostics" leaves it guessing whether it saw the failure.
        if self.dropped_errors > 0 {
            causes.push(format!("{} further errors", self.dropped_errors));
        }
        if self.dropped_warnings > 0 {
            causes.push(format!("{} further warnings", self.dropped_warnings));
        }
        if causes.is_empty() {
            return None;
        }
        Some(format!(
            "Harness trimmed {} (~{} tokens); re-running is identical.",
            causes.join(", "),
            self.removed_tokens,
        ))
    }
}

/// Commands whose successful output is progress noise around a final summary line.
///
/// This is an allowlist rather than a list of commands to protect. A command we fail to
/// recognize keeps its full output, which only costs tokens; a command we wrongly shrink
/// loses information the model cannot get back. Only the first failure mode is acceptable.
///
/// An entry with subcommands matches only those subcommands. An entry with an empty
/// subcommand list matches the program whatever its arguments are.
const SHRINKABLE: &[(&str, &[&str])] = &[
    // Rust
    (
        "cargo",
        &[
            "build", "check", "test", "nextest", "clippy", "fmt", "install", "update", "fetch",
            "clean", "bench", "doc",
        ],
    ),
    ("rustfmt", &[]),
    // JavaScript / TypeScript
    (
        "npm",
        &["install", "ci", "i", "add", "run", "test", "build", "prune"],
    ),
    ("pnpm", &["install", "i", "add", "run", "test", "build"]),
    ("yarn", &["install", "add", "run", "test", "build"]),
    ("bun", &["install", "add", "run", "test", "build"]),
    ("tsc", &[]),
    ("jest", &[]),
    ("vitest", &[]),
    ("mocha", &[]),
    ("webpack", &[]),
    ("vite", &["build"]),
    ("next", &["build"]),
    ("eslint", &[]),
    ("prettier", &[]),
    // Python
    ("pip", &["install", "uninstall"]),
    ("pip3", &["install", "uninstall"]),
    ("uv", &["pip", "sync", "install"]),
    ("poetry", &["install", "add", "lock"]),
    ("pytest", &[]),
    ("black", &[]),
    ("ruff", &["format", "check"]),
    // Absent until now, and it is the type checker most likely to print hundreds of lines on a
    // first run over an untyped codebase.
    ("mypy", &[]),
    // Go
    ("go", &["build", "test", "install", "vet", "mod"]),
    ("gofmt", &[]),
    // Native and JVM build systems
    ("make", &[]),
    ("cmake", &[]),
    ("ninja", &[]),
    ("gradle", &[]),
    ("mvn", &[]),
    ("javac", &[]),
    // Containers
    ("docker", &["build", "push", "pull", "compose"]),
    // Git write operations. Read operations such as `log`, `diff`, `show`, and `status` are
    // deliberately absent: their output is the answer, not a receipt.
    (
        "git",
        &[
            "push",
            "pull",
            "fetch",
            "clone",
            "commit",
            "add",
            "checkout",
            "switch",
            "merge",
            "rebase",
            "stash",
            "tag",
            "init",
            "submodule",
        ],
    ),
];

/// Condenses one exec result before it is ever sent to the model.
///
/// Returns `None` when nothing was removed, so the caller can keep the original string rather
/// than pay for a copy of it.
///
/// `arguments` is the tool call's JSON, which carries the command matched against the
/// allowlist and checked for an explicit request for a generated directory. `exit_code` and
/// `process_id` together say which shrinking rule applies:
///
/// - a clean exit of an allowlisted command is a receipt around a summary;
/// - no exit code but a live process is a poll of a still-running session, whose log is
///   unbounded and whose tail is the answer — that is where a server prints the traceback the
///   model is waiting for;
/// - a failure keeps its output whole, because it is the most valuable thing in the context.
pub(crate) fn condense_exec_output(
    arguments: &str,
    exit_code: Option<i32>,
    process_id: Option<i32>,
    text: &str,
) -> Option<(String, CondenseReport)> {
    let mut report = CondenseReport::default();

    let normalized = condense_losslessly(text, &mut report);
    // Measured after the lossless stages, so the reported size is what the model can no longer
    // see rather than escape sequences it was never going to read.
    let visible_tokens = approx_token_count(&normalized);

    // The lossy stages are attempted against a copy, because whether they are worth running
    // is not known until their size is: they are gated on line counts, and ten short lines
    // cost less than the line that has to announce their removal.
    let mut lossy = CondenseReport::default();
    let mut candidate: Option<String> = filter_artifact_paths(arguments, &normalized, &mut lossy);

    let allowlisted = command_is_shrinkable(arguments);
    let dialect = command_dialect(arguments);
    // Gated on the allowlist *or* a known dialect, and not on `shrinkable` below. A receipt is
    // not the answer whether the run passed or failed, and on a failure it is the thing standing
    // between the model and the first error. Either gate is enough to know whose output this is,
    // and the dialect covers what the allowlist misses: `python -m pytest` and `uv run pytest`
    // are how pytest is usually invoked and neither names an allowlisted program first.
    if (allowlisted || dialect.is_some())
        && let Some(filtered) =
            filter_progress_lines(candidate.as_deref().unwrap_or(&normalized), &mut lossy)
    {
        candidate = Some(filtered);
    }

    // Gated on recognizing the dialect rather than on the allowlist, which is the stricter of the
    // two: `make` is allowlisted and prints nothing we can parse. For the same reason it exists at
    // all - one bad signature in a workspace emits an `error[E0308]` per call site, and the
    // failing run is exactly when that happens. What it drops is in the spill file, which is what
    // makes capping defensible rather than a guess.
    if let Some(dialect) = dialect
        && let Some(capped) = cap_diagnostic_blocks(
            candidate.as_deref().unwrap_or(&normalized),
            dialect,
            &mut lossy,
        )
    {
        candidate = Some(capped);
    }

    let shrinkable = match (exit_code, process_id) {
        // A finished command whose output is progress noise around a final summary.
        (Some(0), _) => allowlisted,
        // A poll of a still-running session: unbounded log, and the tail is the answer.
        (None, Some(_)) => true,
        _ => false,
    };
    if shrinkable
        && let Some((shrunk, removed)) = shrink_text(candidate.as_deref().unwrap_or(&normalized))
    {
        lossy.middle_lines = removed;
        candidate = Some(shrunk);
    }

    let mut current = normalized;
    if let Some(condensed) = candidate {
        let removed = visible_tokens.saturating_sub(approx_token_count(&condensed));
        if removed >= MIN_LOSSY_TOKENS {
            report.artifact_lines = lossy.artifact_lines;
            report.progress_lines = lossy.progress_lines;
            report.dropped_errors = lossy.dropped_errors;
            report.dropped_warnings = lossy.dropped_warnings;
            report.middle_lines = lossy.middle_lines;
            report.removed_tokens = removed;
            current = condensed;
        }
        // Otherwise the lossless result stands: losing content the model might want is only
        // justified when it buys more than the notice explaining the loss.
    }

    if report.is_empty() {
        return None;
    }
    Some((current, report))
}

/// The stages that cannot lose anything, in order.
///
/// Factored out so [`condense_exec_output`] and [`condense_exec_output_lossless`] can never
/// drift: the difference between them is only what runs *after* this.
///
/// The line-ending warnings go before the size is measured, so their tokens are never
/// attributed to a lossy cause: the header's `Harness trimmed N generated-directory lines
/// (~T tokens)` has to mean those lines and nothing else, and noise must not help a marginal
/// candidate clear `MIN_LOSSY_TOKENS` and take real content with it.
fn condense_losslessly(text: &str, report: &mut CondenseReport) -> String {
    let normalized = normalize(text, report);
    strip_line_ending_warnings(normalized, report)
}

/// Noise removal with nothing that drops content, for callers that must not lose any.
///
/// Code mode hands its result to a program the model wrote, which may count lines or match the
/// text exactly. The lossy stages announce themselves, which is enough for a reader and not
/// enough for a parser, so they are left out here. What goes is only what a terminal would
/// never have shown and what git addressed to whoever configured the repository.
pub(crate) fn condense_exec_output_lossless(text: &str) -> Option<(String, CondenseReport)> {
    let mut report = CondenseReport::default();
    let condensed = condense_losslessly(text, &mut report);
    if report.is_empty() {
        return None;
    }
    Some((condensed, report))
}

/// Removes what a terminal would never have shown a reader, and nothing else.
///
/// Three things, none of which a model can act on:
///
/// - **Escape sequences.** Colour and cursor codes from dev servers and test runners.
/// - **Carriage-return overwrites.** A progress bar writes `10%\r20%\r30%` into one line; a
///   terminal shows only the last. Deleting the `\r` instead would splice the discarded
///   states into one very long line, which is worse than leaving them alone.
/// - **Windows line terminators.** `\r\n` becomes `\n`, one character a line.
///
/// **Column padding is deliberately left in place.** `Format-Table` pads every row out to the
/// widest value, which looks like free tokens and is not: a BPE tokenizer merges a run of
/// spaces into one or two tokens, so 121 trailing spaces cost 2 tokens rather than the 30 a
/// bytes/4 estimate predicts. Measured over one real session, stripping it saved 394 tokens
/// across every read and listing in the session -- 1.3%, about three hundredths of a cent --
/// while rewriting text the model may be about to quote back in a patch. The `len / 4`
/// estimator is what made this look worthwhile; it is accurate in aggregate and wrong by
/// roughly ten times on a transformation that only removes whitespace.
fn normalize(text: &str, report: &mut CondenseReport) -> String {
    let mut lines: Vec<String> = Vec::new();
    for raw_line in text.split('\n') {
        // The line terminator first: on Windows every line ends `\r\n`, and a trailing `\r` is
        // the end of this line rather than the start of an overwrite. Taking the last
        // carriage-return-separated segment without removing it would leave the empty string
        // after that `\r`, which silently deletes every line of Windows output. Measured
        // against real rollouts that bug read as a 75.9% saving.
        let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
        // Then the overwrites: a progress bar writes `10%\r50%\r100%` into one line and a
        // terminal shows only the last state.
        let visible = line.rsplit('\r').next().unwrap_or(line);
        let escapes = CSI_ESCAPE.find_iter(visible).count();
        let stripped = if escapes > 0 {
            CSI_ESCAPE.replace_all(visible, "")
        } else {
            std::borrow::Cow::Borrowed(visible)
        };
        report.escape_sequences += escapes;
        // Counted against the escape-stripped text so a coloured line is not also reported as
        // a rewritten one; escapes have their own counter.
        if raw_line.len() != line.len() || visible.len() != line.len() {
            report.rewritten_lines += 1;
        }
        lines.push(stripped.into_owned());
    }
    lines.join("\n")
}

/// Drops git's line-ending warnings, which say nothing about what the command did.
///
/// Lossless in the sense this module means: under `core.autocrlf` git prints one per file it is
/// about to rewrite, on success, naming no error and repeating verbatim. It is addressed to
/// whoever configured the repository. A model that does want the line-ending settings asks for
/// them directly - `git config core.autocrlf`, `git ls-files --eol` - and those answers are a
/// different shape that this never matches.
///
/// Runs after [`normalize`] rather than before it: on the platform that produces these warnings
/// they arrive as `...touches it\r\n`, and the match is anchored to the end of the line.
///
/// Takes the text by value so the common case costs no copy - this is a no-op on most output,
/// unlike [`normalize`], which always rebuilds.
fn strip_line_ending_warnings(text: String, report: &mut CondenseReport) -> String {
    if !text.contains(LINE_ENDING_WARNING_PREFIX) {
        return text;
    }
    // Joined inside the block so the borrow of `text` ends before the early return below.
    let (kept, dropped) = {
        let lines: Vec<&str> = text.split('\n').collect();
        let kept: Vec<&str> = lines
            .iter()
            .copied()
            .filter(|line| !is_line_ending_warning(line))
            .collect();
        let dropped = lines.len() - kept.len();
        (kept.join("\n"), dropped)
    };
    if dropped == 0 {
        return text;
    }
    report.line_ending_warnings = dropped;
    kept
}

/// Drops listing lines that live inside a build or dependency directory.
///
/// Unlike [`normalize`] this loses information, so three things have to hold: the command did
/// not name one of these directories itself, enough lines match to be worth explaining, and
/// they are not almost the whole output. The last two are what separate "a project listing
/// buried under its dependencies" from "the model is reading `node_modules` on purpose".
///
/// The match is on path *components*, not on the text of the line, so a `.gitignore` listing
/// `dist` or a log mentioning a build does not lose lines.
fn filter_artifact_paths(
    arguments: &str,
    text: &str,
    report: &mut CondenseReport,
) -> Option<String> {
    if arguments_name_artifact_directory(arguments) {
        return None;
    }
    let lines: Vec<&str> = text.split('\n').collect();
    let kept: Vec<&str> = lines
        .iter()
        .copied()
        .filter(|line| !is_artifact_path(line))
        .collect();
    let dropped = lines.len() - kept.len();
    if dropped < MIN_ARTIFACT_LINES || dropped as f32 > lines.len() as f32 * MAX_ARTIFACT_SHARE {
        return None;
    }
    report.artifact_lines = dropped;
    Some(kept.join("\n"))
}

/// A `libtest` line that says a test ran and nothing more.
///
/// `... ok` and `... ignored` only. `... FAILED` stays, and so does the `test result:` line that
/// carries the counts - dropping those would be dropping the answer. This is the shape both
/// `cargo test` and `cargo nextest` print.
fn is_passing_test_line(line: &str) -> bool {
    line.starts_with("test ") && (line.ends_with(" ... ok") || line.ends_with(" ... ignored"))
}

/// What `pytest` prints about the machine it is running on, and how far along it is.
///
/// `collected N items` is deliberately not here: it is the denominator for everything below it.
/// The dot line (`test_probe.py ..FFF   [100%]`) goes because the `short test summary info`
/// section names every failure again, with its reason, a few lines further down.
fn is_pytest_session_noise(line: &str) -> bool {
    if line.starts_with("rootdir: ")
        || line.starts_with("plugins: ")
        || line.starts_with("cachedir: ")
        || line.starts_with("configfile: ")
    {
        return true;
    }
    if line.starts_with("platform ") && line.contains(" -- Python ") {
        return true;
    }
    line.ends_with("%]") && line.contains(".py ")
}

fn is_progress_line(line: &str) -> bool {
    if is_passing_test_line(line) || is_pytest_session_noise(line) {
        return true;
    }
    if line.starts_with("running ") && (line.ends_with(" test") || line.ends_with(" tests")) {
        return true;
    }
    // The leading space is the discriminator; see `BUILD_PROGRESS_VERBS`.
    let Some(rest) = line.strip_prefix(' ') else {
        return false;
    };
    let trimmed = rest.trim_start();
    BUILD_PROGRESS_VERBS.iter().any(|verb| {
        trimmed
            .strip_prefix(verb)
            .is_some_and(|tail| tail.starts_with(' '))
    })
}

/// Drops the lines a build or test runner prints to show progress.
///
/// Lossy in the sense this module means: a crate name that scrolled past is information, even if
/// it is not information anyone asked for.
///
/// No line threshold of its own, unlike [`filter_artifact_paths`]. That one needs a count to tell
/// "a project listing buried under its dependencies" from "the model is reading `node_modules` on
/// purpose"; here there is no such question, because a receipt is never the answer. Whether the
/// removal was worth announcing is left to `MIN_LOSSY_TOKENS` at the end, which measures the
/// saving instead of guessing at it from a line count - and has to, since pytest's session noise
/// is four lines however large the suite is, where cargo's is one per crate.
///
/// Runs before [`shrink_text`] so that what head/tail shrinking then drops is real content rather
/// than the receipt of a build. That ordering is the whole point: on a successful `cargo build` of
/// a large workspace the receipt *is* most of the output, and shrinking alone keeps five lines of
/// it while dropping the warnings underneath.
fn filter_progress_lines(text: &str, report: &mut CondenseReport) -> Option<String> {
    let lines: Vec<&str> = text.split('\n').collect();
    let kept: Vec<&str> = lines
        .iter()
        .copied()
        .filter(|line| !is_progress_line(line))
        .collect();
    let dropped = lines.len() - kept.len();
    if dropped == 0 {
        return None;
    }
    report.progress_lines = dropped;
    Some(kept.join("\n"))
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum DiagnosticKind {
    Error,
    Warning,
}

/// Whose diagnostics we are reading.
///
/// One per grammar, not one per command: `cargo build`, `cargo clippy` and `cargo nextest` all
/// print `rustc`'s, and a dialect we do not recognize means the block capper does not run at all.
#[derive(Clone, Copy, PartialEq, Eq)]
enum DiagnosticDialect {
    /// `error[E0308]:` at column zero, continuation lines indented.
    Rustc,
    /// `____ test_name ____` headers under a `=== FAILURES ===` banner.
    Pytest,
    /// One `path:line: error:` per diagnostic, with `path:line: note:` attached.
    Mypy,
}

/// The dialect a command's output will be in, if we know it.
fn command_dialect(arguments: &str) -> Option<DiagnosticDialect> {
    let words = command_words(arguments);
    let mut words = words.iter().map(String::as_str).peekable();
    if words.peek() == Some(&"env") {
        words.next();
    }
    while words
        .peek()
        .is_some_and(|word| word.split_once('=').is_some_and(|(key, _)| !key.is_empty()))
    {
        words.next();
    }
    let program = words.next().map(program_name)?;
    match program {
        "cargo" | "rustc" | "cross" => Some(DiagnosticDialect::Rustc),
        "pytest" | "py.test" => Some(DiagnosticDialect::Pytest),
        "mypy" => Some(DiagnosticDialect::Mypy),
        // `python -m pytest` and `python -m mypy` are how both are usually run.
        "python" | "python3" | "py" | "uv" | "poetry" => words.find_map(|word| match word {
            "pytest" => Some(DiagnosticDialect::Pytest),
            "mypy" => Some(DiagnosticDialect::Mypy),
            _ => None,
        }),
        _ => None,
    }
}

/// The kind of diagnostic a line opens, if it opens one.
///
/// Anchored at column zero because that is where `rustc` starts a diagnostic and nowhere else:
/// every continuation line of a block is indented (` --> `, `  |`, `14 | ...`). Verified against
/// real `cargo build` output rather than assumed.
///
/// `error: could not compile` and `error: aborting due to N previous errors` open nothing. They
/// are the tail rustc prints after the diagnostics, they are one line each, and they are the line
/// a reader looks at first to learn how bad it is - so they are never a block and never capped.
fn diagnostic_kind(line: &str) -> Option<DiagnosticKind> {
    if line.starts_with("error[") {
        return Some(DiagnosticKind::Error);
    }
    if let Some(rest) = line.strip_prefix("error:") {
        if rest.contains("could not compile") || rest.contains("aborting due to") {
            return None;
        }
        return Some(DiagnosticKind::Error);
    }
    if line.starts_with("warning[") {
        return Some(DiagnosticKind::Warning);
    }
    if let Some(rest) = line.strip_prefix("warning:") {
        // `warning: N warnings emitted` is the same kind of tail as `could not compile`.
        if rest.contains("generated") || rest.contains("emitted") {
            return None;
        }
        return Some(DiagnosticKind::Warning);
    }
    None
}

/// A `pytest` failure header: `____ test_name ____`.
///
/// Three contiguous underscores, which is what separates the header from the `_ _ _ _ _` lines
/// pytest uses *inside* a failure to divide its frames. Matching those would make every frame its
/// own block and cap a single traceback to pieces. Read off real output, not assumed.
fn is_pytest_failure_header(line: &str) -> bool {
    line.starts_with("___") && line.ends_with("___") && line.contains(' ')
}

/// The line a dialect opens a diagnostic with, if this is one.
fn block_start(line: &str, dialect: DiagnosticDialect) -> Option<DiagnosticKind> {
    match dialect {
        DiagnosticDialect::Rustc => diagnostic_kind(line),
        // Every failure is an error; pytest has no warning shape in this section.
        DiagnosticDialect::Pytest => {
            is_pytest_failure_header(line).then_some(DiagnosticKind::Error)
        }
        DiagnosticDialect::Mypy => {
            if line.contains(": error:") {
                Some(DiagnosticKind::Error)
            } else if line.contains(": warning:") {
                Some(DiagnosticKind::Warning)
            } else {
                None
            }
        }
    }
}

/// Whether `line` belongs to the block above it.
fn continues_block(line: &str, dialect: DiagnosticDialect) -> bool {
    match dialect {
        // Indented or blank; anything else at column zero has ended it.
        DiagnosticDialect::Rustc => line.is_empty() || line.starts_with(char::is_whitespace),
        // Everything up to the next header or the next `===` banner, which is how pytest closes
        // the failures section and opens the summary.
        DiagnosticDialect::Pytest => !line.starts_with("==="),
        // mypy attaches context to a diagnostic with `note:` and nothing else.
        DiagnosticDialect::Mypy => line.contains(": note:"),
    }
}

/// Carries the first [`MAX_ERROR_BLOCKS`] errors and [`MAX_WARNING_BLOCKS`] warnings whole, and
/// drops the rest.
///
/// Blocks are kept *whole* or not at all. A diagnostic cut in half is worse than one that is
/// absent and counted, because the model cannot tell which it is looking at.
///
/// Everything that is not part of a block passes through untouched. That is what keeps rustc's
/// `could not compile` count, pytest's `short test summary info` - one line per failure, naming
/// it and why - and mypy's `Found N errors` in place. For pytest in particular that summary is
/// what makes capping cheap: a dropped block is still named and explained a few lines further on.
fn cap_diagnostic_blocks(
    text: &str,
    dialect: DiagnosticDialect,
    report: &mut CondenseReport,
) -> Option<String> {
    let mut kept: Vec<&str> = Vec::new();
    let mut errors = 0usize;
    let mut warnings = 0usize;
    let mut dropped_errors = 0usize;
    let mut dropped_warnings = 0usize;
    // `None` outside a block; `Some(false)` inside one being dropped.
    let mut keeping: Option<bool> = None;

    for line in text.split('\n') {
        if let Some(kind) = block_start(line, dialect) {
            let keep = match kind {
                DiagnosticKind::Error => {
                    errors += 1;
                    errors <= MAX_ERROR_BLOCKS
                }
                DiagnosticKind::Warning => {
                    warnings += 1;
                    warnings <= MAX_WARNING_BLOCKS
                }
            };
            if !keep {
                match kind {
                    DiagnosticKind::Error => dropped_errors += 1,
                    DiagnosticKind::Warning => dropped_warnings += 1,
                }
            }
            keeping = Some(keep);
            if keep {
                kept.push(line);
            }
            continue;
        }
        let continues = continues_block(line, dialect);
        match keeping {
            Some(inside) if continues => {
                if inside {
                    kept.push(line);
                }
            }
            _ => {
                keeping = None;
                kept.push(line);
            }
        }
    }

    if dropped_errors == 0 && dropped_warnings == 0 {
        return None;
    }
    report.dropped_errors = dropped_errors;
    report.dropped_warnings = dropped_warnings;
    Some(kept.join("\n"))
}

/// Whether the command itself named one of [`ARTIFACT_DIRECTORIES`], so its listing is the answer.
///
/// Compared token by token, not as a substring. `arguments.contains("env")` is true of
/// `python -m venv`, of `printenv`, and of every `--environment` flag, and each of those silently
/// turned this stage off - the shorter the name in the list, the more often. Splitting on the
/// characters a path component cannot contain keeps `venv` and `env` apart, and `.vscode` from
/// matching `.vs`.
///
/// Still imperfect, and knowingly: `build` and `target` are subcommand and flag names as well as
/// directory names, so `cargo build` reads as naming `build`. That predates the token split and
/// errs towards leaving output alone, which is the safe direction.
fn arguments_name_artifact_directory(arguments: &str) -> bool {
    arguments
        .split(|c: char| !(c.is_alphanumeric() || c == '.' || c == '_' || c == '-'))
        .any(|token| ARTIFACT_DIRECTORIES.contains(&token))
}

fn is_artifact_path(line: &str) -> bool {
    let trimmed = line.trim();
    if !trimmed.contains(['/', '\\']) {
        return false;
    }
    trimmed
        .split(['/', '\\'])
        .any(|component| ARTIFACT_DIRECTORIES.contains(&component))
}

/// Reads the command out of a tool call's JSON arguments and matches it against the allowlist.
///
/// The two shell tools name the field differently: `shell` sends `command`, `exec_command` sends
/// `cmd`. Reading only one of them silently matches nothing on a session that used the other.
fn command_is_shrinkable(arguments: &str) -> bool {
    program_is_shrinkable(&command_words(arguments))
}

/// The command line a tool call is asking for, split into words.
///
/// Empty when the arguments do not carry one, or when the line chains or redirects - see
/// [`shell_words`], which refuses those, because the output that matters then comes from
/// something other than the first word.
fn command_words(arguments: &str) -> Vec<String> {
    let Ok(parsed) = serde_json::from_str::<serde_json::Value>(arguments) else {
        return Vec::new();
    };
    let Some(command) = parsed.get("command").or_else(|| parsed.get("cmd")) else {
        return Vec::new();
    };
    let words = match command {
        serde_json::Value::String(line) => shell_words(line),
        serde_json::Value::Array(argv) => {
            let argv: Vec<&str> = argv.iter().filter_map(serde_json::Value::as_str).collect();
            // `bash -lc "<script>"` hides the real command in the trailing argument.
            match argv.split_last() {
                Some((script, head))
                    if head.iter().any(|word| {
                        matches!(*word, "-c" | "-lc" | "-lic" | "/c" | "/C" | "-Command")
                    }) =>
                {
                    shell_words(script)
                }
                _ => argv.into_iter().map(str::to_string).collect(),
            }
        }
        _ => Vec::new(),
    };
    words
}

/// Splits a shell line into words, refusing any line that chains or redirects.
///
/// A line such as `cargo build && ./run-server` must not be shrunk on the strength of its
/// first word, because the output that matters comes from what follows.
fn shell_words(line: &str) -> Vec<String> {
    let mut words = Vec::new();
    for word in line.split_whitespace() {
        if word.contains(['|', ';', '>', '<', '`', '$', '&']) {
            return Vec::new();
        }
        words.push(word.to_string());
    }
    words
}

fn program_is_shrinkable(words: &[String]) -> bool {
    let mut words = words.iter().map(String::as_str).peekable();
    // Skip `env` and any leading `VAR=value` assignments.
    if words.peek() == Some(&"env") {
        words.next();
    }
    while words
        .peek()
        .is_some_and(|word| word.split_once('=').is_some_and(|(key, _)| !key.is_empty()))
    {
        words.next();
    }
    let Some(program) = words.next().map(program_name) else {
        return false;
    };
    let Some((_, subcommands)) = SHRINKABLE
        .iter()
        .find(|(candidate, _)| *candidate == program)
    else {
        return false;
    };
    if subcommands.is_empty() {
        return true;
    }
    // The first argument that is not a flag is the subcommand.
    words
        .find(|word| !word.starts_with('-'))
        .is_some_and(|subcommand| subcommands.contains(&subcommand))
}

/// Reduces a path such as `/usr/bin/cargo` or `cargo.exe` to `cargo`.
fn program_name(program: &str) -> &str {
    let name = program
        .rsplit_once(['/', '\\'])
        .map_or(program, |(_, name)| name);
    name.strip_suffix(".exe").unwrap_or(name)
}

/// Keeps the head and tail of `text`, replacing the middle with an explicit marker.
///
/// Returns the shrunk text and the number of lines it dropped, or `None` when the text is small
/// enough that shrinking would not be worth the lost detail.
fn shrink_text(text: &str) -> Option<(String, usize)> {
    let lines: Vec<&str> = text.lines().collect();
    if lines.len() < MIN_LINES_TO_SHRINK {
        return None;
    }
    let removed = lines.len() - KEEP_HEAD_LINES - KEEP_TAIL_LINES;
    let head = lines[..KEEP_HEAD_LINES].join("\n");
    let tail = lines[lines.len() - KEEP_TAIL_LINES..].join("\n");
    // The marker's whole job is to put the gap where the gap is, so the model reads a cut
    // rather than the end of the output. Everything else it could say is already one screen
    // away in the header -- who cut it, how many tokens, that re-running is identical -- and
    // the header is not repeated here because both are paid on every request that carries
    // this output. Shrinking only ever runs on a clean exit, so the model can also see
    // `Process exited with code 0` right above.
    Some((
        format!("{head}\n\n[system] {removed} lines trimmed\n\n{tail}"),
        removed,
    ))
}

#[cfg(test)]
#[path = "tool_output_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "tool_output_real_tests.rs"]
mod real_tests;
