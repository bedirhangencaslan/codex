//! Shrinking of tool output that belongs to a finished turn.
//!
//! A coding agent spends most of its context on command output, and most of that output is
//! progress noise: `Compiling ...` lines, per-test `ok` lines, npm's package tree. What the
//! model still needs afterwards is the summary at the end, plus enough of the head to
//! recognize what ran.
//!
//! Shrinking happens on the copy of history built for a request, never on stored history, so
//! the transcript and the UI keep the real output. It applies only to turns that have
//! finished, which is the same boundary where reasoning drops out, so it costs no additional
//! prompt-cache invalidation.

use std::collections::HashMap;
use std::collections::HashSet;

use codex_history::ResponseItemEnvelope;
use codex_protocol::models::FunctionCallOutputBody;
use codex_protocol::models::ResponseItem;

/// Lines kept from the start, so the model can still tell what ran.
const KEEP_HEAD_LINES: usize = 5;
/// Lines kept from the end, where these commands put their summary.
const KEEP_TAIL_LINES: usize = 30;
/// Shrinking below this size would remove almost nothing while still costing information.
const MIN_LINES_TO_SHRINK: usize = KEEP_HEAD_LINES + KEEP_TAIL_LINES + 10;

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

/// Replaces the middle of successful, noisy tool output with an explicit marker.
///
/// `is_completed` reports whether an item's turn has finished. Output from the running turn
/// is left alone, because the model is still acting on it.
pub(crate) fn shrink_completed_outputs(
    items: &mut [ResponseItemEnvelope],
    is_completed: impl Fn(&ResponseItemEnvelope) -> bool,
) {
    let shrinkable = shrinkable_call_ids(items);
    if shrinkable.is_empty() {
        return;
    }
    for envelope in items.iter_mut() {
        if !is_completed(envelope) {
            continue;
        }
        let ResponseItem::FunctionCallOutput {
            call_id, output, ..
        } = &mut envelope.item
        else {
            continue;
        };
        // Only successful commands are shrunk. A failure's output is the most valuable thing
        // in the context and is always kept whole.
        if output.success != Some(true) {
            continue;
        }
        if !call_id
            .as_deref()
            .is_some_and(|call_id| shrinkable.contains(call_id))
        {
            continue;
        }
        let FunctionCallOutputBody::Text(text) = &mut output.body else {
            continue;
        };
        if let Some(shrunk) = shrink_text(text) {
            *text = shrunk;
        }
    }
}

/// Collects the call ids whose command is on the allowlist.
fn shrinkable_call_ids(items: &[ResponseItemEnvelope]) -> HashSet<String> {
    let mut by_call_id: HashMap<&str, &str> = HashMap::new();
    for envelope in items {
        if let ResponseItem::FunctionCall {
            call_id, arguments, ..
        } = &envelope.item
        {
            by_call_id.insert(call_id.as_str(), arguments.as_str());
        }
    }
    by_call_id
        .into_iter()
        .filter(|(_, arguments)| command_is_shrinkable(arguments))
        .map(|(call_id, _)| call_id.to_string())
        .collect()
}

/// Reads the command out of a tool call's JSON arguments and matches it against the allowlist.
fn command_is_shrinkable(arguments: &str) -> bool {
    let Ok(parsed) = serde_json::from_str::<serde_json::Value>(arguments) else {
        return false;
    };
    let Some(command) = parsed.get("command") else {
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
/// Returns `None` when the text is small enough that shrinking would not be worth the lost
/// detail.
fn shrink_text(text: &str) -> Option<String> {
    let lines: Vec<&str> = text.lines().collect();
    if lines.len() < MIN_LINES_TO_SHRINK {
        return None;
    }
    let removed = lines.len() - KEEP_HEAD_LINES - KEEP_TAIL_LINES;
    let head = lines[..KEEP_HEAD_LINES].join("\n");
    let tail = lines[lines.len() - KEEP_TAIL_LINES..].join("\n");
    // The marker names the exit status and the actor. A model that cannot tell whether the
    // command succeeded, or whether the command itself printed nothing, will run it again.
    Some(format!(
        "{head}\n\n[system] exit 0 - the harness trimmed this output, the command did not. \
         {removed} middle lines removed; re-running the command produces the same trimmed \
         result.\n\n{tail}"
    ))
}

#[cfg(test)]
#[path = "tool_output_tests.rs"]
mod tests;
