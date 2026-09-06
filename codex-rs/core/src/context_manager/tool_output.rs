//! Condensing of exec output on its way to the model.
//!
//! A coding agent spends most of its context on command output, and most of that output is
//! noise: terminal escape sequences, column padding, the dependency tree under
//! `node_modules`, a build's `Compiling ...` lines. What the model still needs is the
//! summary at the end, plus enough of the head to recognize what ran.
//!
//! Three stages, cheapest and safest first:
//!
//! 1. [`normalize`] is lossless and unconditional. Escape sequences, carriage-return
//!    overwrites and trailing padding carry no information a reader could act on, so there is
//!    no case in which keeping them is right.
//! 2. [`filter_artifact_paths`] drops listing lines that live under a build or dependency
//!    directory. Lossy, so it is gated on the command not having asked for one.
//! 3. [`shrink_text`] keeps the head and tail of output whose middle is a receipt.
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
];

/// Below this, filtering costs a line of explanation to save less than it spends.
const MIN_ARTIFACT_LINES: usize = 10;

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

/// What condensing removed, so the harness can say so in the response header.
///
/// Counted rather than merely flagged: a model that is told output was altered but not how
/// much has no way to judge whether re-running the command would show it more.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CondenseReport {
    /// Terminal escape sequences removed.
    pub(crate) escape_sequences: usize,
    /// Lines that lost a carriage-return overwrite or trailing padding.
    pub(crate) rewritten_lines: usize,
    /// Listing lines dropped for living under a build or dependency directory.
    pub(crate) artifact_lines: usize,
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
    /// model could do differently, and the notice would cost more than it saved: 59% of
    /// measured outputs carry some padding or a stray escape sequence, and a line of
    /// explanation on each of them outweighs the padding removed from all of them. Announcing
    /// only the lossy stages moves the notice from 59% of outputs to 12-25% of them.
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
            causes.push(format!(
                "{} generated-directory lines",
                self.artifact_lines
            ));
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

    let normalized = normalize(text, &mut report);
    // Measured after normalizing, so the reported size is what the model can no longer see
    // rather than padding it was never going to read.
    let visible_tokens = approx_token_count(&normalized);

    // The lossy stages are attempted against a copy, because whether they are worth running
    // is not known until their size is: they are gated on line counts, and ten short lines
    // cost less than the line that has to announce their removal.
    let mut lossy = CondenseReport::default();
    let mut candidate: Option<String> = filter_artifact_paths(arguments, &normalized, &mut lossy);

    let shrinkable = match (exit_code, process_id) {
        // A finished command whose output is progress noise around a final summary.
        (Some(0), _) => command_is_shrinkable(arguments),
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

/// Removes what a terminal would never have shown a reader, and nothing else.
///
/// Three things, none of which a model can act on:
///
/// - **Escape sequences.** Colour and cursor codes from dev servers and test runners.
/// - **Carriage-return overwrites.** A progress bar writes `10%\r20%\r30%` into one line; a
///   terminal shows only the last. Deleting the `\r` instead would splice the discarded
///   states into one very long line, which is worse than leaving them alone.
/// - **Windows line terminators.** `\r\n` becomes `\n`, one character a line.
/// - **Trailing padding.** `Format-Table` and `Select-String` pad every row to a fixed width.
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
        let trimmed = stripped.trim_end();
        report.escape_sequences += escapes;
        // Counted against the escape-stripped text so a coloured line is not also reported as
        // a rewritten one; escapes have their own counter.
        if raw_line.len() != line.len()
            || visible.len() != line.len()
            || trimmed.len() != stripped.len()
        {
            report.rewritten_lines += 1;
        }
        lines.push(trimmed.to_string());
    }
    lines.join("\n")
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
    if ARTIFACT_DIRECTORIES
        .iter()
        .any(|directory| arguments.contains(directory))
    {
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
    let Ok(parsed) = serde_json::from_str::<serde_json::Value>(arguments) else {
        return false;
    };
    let Some(command) = parsed.get("command").or_else(|| parsed.get("cmd")) else {
        return false;
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
        _ => return false,
    };
    program_is_shrinkable(&words)
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
