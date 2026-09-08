//! Read many files in one round trip.
//!
//! Reading through the shell works, but the model has to express a batch as one chained command
//! (`Get-Content a; '---'; Get-Content b; ...`) whose *combined* output is capped, so batches stop
//! at a handful of files and the rest of the corpus costs a round trip each. Measured on a task that
//! reads 44 files: eight read turns here against three for an agent whose read tool takes a list.
//! Every extra turn resends the whole window, and enough of them reach the auto-compaction limit,
//! which destroys the tool output that was just paid for.
//!
//! So this tool takes a list of paths and enforces its own limits per file, rather than letting one
//! large file consume the budget for the batch. When the budget runs out it stops on a file
//! boundary and names what it did not reach, because a model told "read 6 of 22, remaining: ..."
//! asks again, while a model handed a middle-truncated blob does not know it should.

use codex_exec_server::GetMetadataOptions;
use codex_exec_server::ReadFileOptions;
use serde::Deserialize;

use crate::function_tool::FunctionCallError;
use crate::tools::context::FunctionToolOutput;
use crate::tools::context::ToolInvocation;
use crate::tools::context::ToolOutput;
use crate::tools::context::ToolPayload;
use crate::tools::context::boxed_tool_output;
use crate::tools::handlers::parse_arguments;
use crate::tools::handlers::read_spec::DEFAULT_LINE_LIMIT;
use crate::tools::handlers::read_spec::READ_TOOL_NAME;
use crate::tools::handlers::read_spec::ReadToolOptions;
use crate::tools::handlers::read_spec::create_read_tool;
use crate::tools::handlers::resolve_tool_environment;
use crate::tools::registry::CoreToolRuntime;
use crate::tools::registry::ToolExecutor;
use codex_tools::ToolName;
use codex_tools::ToolSpec;

/// Budget for one call, in bytes. The harness truncates a `function_call_output` at
/// `truncation_policy * 1.2` middle-out, which would cut through the middle of a file; staying under
/// it means this tool decides where to stop instead.
const TOTAL_BUDGET_BYTES: usize = 96_000;
/// Ceiling for a single file, so one large file cannot turn a twenty-file batch into a one-file one.
const PER_FILE_BUDGET_BYTES: usize = 32_000;
/// A single line longer than this is almost always minified or generated.
const MAX_LINE_CHARS: usize = 2_000;

#[derive(Default)]
pub struct ReadHandler {
    options: ReadToolOptions,
}

impl ReadHandler {
    pub(crate) fn new(options: ReadToolOptions) -> Self {
        Self { options }
    }
}

#[derive(Deserialize)]
struct ReadArgs {
    paths: Vec<String>,
    #[serde(default)]
    offset: Option<usize>,
    #[serde(default)]
    limit: Option<usize>,
    #[serde(default)]
    environment_id: Option<String>,
}

/// One file's text plus whatever the caller needs to know about how it was cut.
struct FileSection {
    body: String,
    note: Option<String>,
}

fn take_window(contents: &str, offset: Option<usize>, limit: Option<usize>) -> FileSection {
    let start = offset.unwrap_or(1).max(1) - 1;
    let limit = limit.unwrap_or(DEFAULT_LINE_LIMIT);
    let total_lines = contents.lines().count();

    let mut out = String::new();
    let mut kept = 0usize;
    let mut clipped_lines = 0usize;
    for line in contents.lines().skip(start).take(limit) {
        if line.chars().count() > MAX_LINE_CHARS {
            let cut: String = line.chars().take(MAX_LINE_CHARS).collect();
            out.push_str(&cut);
            out.push_str(" ... (line truncated)");
            clipped_lines += 1;
        } else {
            out.push_str(line);
        }
        out.push('\n');
        kept += 1;
        if out.len() > PER_FILE_BUDGET_BYTES {
            break;
        }
    }

    let shown_to = start + kept;
    let mut notes = Vec::new();
    if start > 0 {
        notes.push(format!("from line {}", start + 1));
    }
    if shown_to < total_lines {
        notes.push(format!(
            "showing {kept} of {total_lines} lines; continue with offset {}",
            shown_to + 1
        ));
    }
    if clipped_lines > 0 {
        notes.push(format!("{clipped_lines} long line(s) clipped"));
    }

    FileSection {
        body: out,
        note: (!notes.is_empty()).then(|| notes.join(", ")),
    }
}

impl ReadHandler {
    async fn handle_call(
        &self,
        invocation: ToolInvocation,
    ) -> Result<Box<dyn ToolOutput>, FunctionCallError> {
        let ToolInvocation {
            step_context,
            payload,
            ..
        } = invocation;

        let arguments = match payload {
            ToolPayload::Function { arguments } => arguments,
            _ => {
                return Err(FunctionCallError::RespondToModel(
                    "read handler received unsupported payload".to_string(),
                ));
            }
        };

        let ReadArgs {
            paths,
            offset,
            limit,
            environment_id,
        } = parse_arguments(&arguments)?;

        if paths.is_empty() {
            return Err(FunctionCallError::RespondToModel(
                "read requires at least one path".to_string(),
            ));
        }
        // A window only means something for a single file; silently applying it to twenty would
        // return the same line range of each, which is never what was meant.
        let (offset, limit) = if paths.len() == 1 {
            (offset, limit)
        } else {
            (None, None)
        };

        let Some(turn_environment) =
            resolve_tool_environment(&step_context.environments, environment_id.as_deref())?
        else {
            return Err(FunctionCallError::RespondToModel(
                "read is unavailable in this session".to_string(),
            ));
        };
        let sandbox = turn_environment.sandbox_context(/*additional_permissions*/ None);
        let fs = turn_environment.environment.get_filesystem();

        let mut out = String::new();
        let mut used = 0usize;
        let mut unread: Vec<String> = Vec::new();

        for (index, path) in paths.iter().enumerate() {
            if used >= TOTAL_BUDGET_BYTES {
                unread.extend(paths[index..].iter().cloned());
                break;
            }

            let path_uri = match turn_environment.cwd().join(path) {
                Ok(uri) => uri,
                Err(err) => {
                    out.push_str(&format!("===== {path}\nunable to resolve path: {err}\n\n"));
                    continue;
                }
            };
            let shown = path_uri.inferred_native_path_string();

            match fs
                .get_metadata(&path_uri, GetMetadataOptions::default(), Some(&sandbox))
                .await
            {
                Ok(metadata) if !metadata.is_file => {
                    out.push_str(&format!("===== {shown}\nnot a file\n\n"));
                    continue;
                }
                Err(error) => {
                    out.push_str(&format!("===== {shown}\nunable to read: {error}\n\n"));
                    continue;
                }
                Ok(_) => {}
            }

            let bytes = match fs
                .read_file(&path_uri, ReadFileOptions::default(), Some(&sandbox))
                .await
            {
                Ok(bytes) => bytes,
                Err(error) => {
                    out.push_str(&format!("===== {shown}\nunable to read: {error}\n\n"));
                    continue;
                }
            };
            let contents = String::from_utf8_lossy(&bytes);
            let section = take_window(&contents, offset, limit);

            let header = match section.note {
                Some(note) => format!("===== {shown} ({note})\n"),
                None => format!("===== {shown}\n"),
            };
            used += header.len() + section.body.len();
            out.push_str(&header);
            out.push_str(&section.body);
            out.push('\n');
        }

        if !unread.is_empty() {
            out.push_str(&format!(
                "\n[read stopped after the output budget was reached. {} file(s) not read: {}. \
                 Ask for them in another call.]\n",
                unread.len(),
                unread.join(", ")
            ));
        }

        Ok(boxed_tool_output(FunctionToolOutput::from_text(
            out,
            Some(true),
        )))
    }
}

impl ToolExecutor<ToolInvocation> for ReadHandler {
    fn tool_name(&self) -> ToolName {
        ToolName::plain(READ_TOOL_NAME)
    }

    fn spec(&self) -> ToolSpec {
        create_read_tool(self.options)
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

impl CoreToolRuntime for ReadHandler {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_defaults_to_the_whole_file() {
        let section = take_window("a\nb\nc\n", None, None);
        assert_eq!(section.body, "a\nb\nc\n");
        assert!(section.note.is_none());
    }

    #[test]
    fn window_honours_offset_and_limit_and_says_where_to_continue() {
        let contents = (1..=10)
            .map(|n| n.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        let section = take_window(&contents, Some(3), Some(2));
        assert_eq!(section.body, "3\n4\n");
        let note = section.note.expect("a partial read must say so");
        assert!(note.contains("from line 3"), "{note}");
        assert!(note.contains("continue with offset 5"), "{note}");
    }

    #[test]
    fn long_lines_are_clipped_rather_than_dropped() {
        let contents = format!("{}\nshort\n", "x".repeat(MAX_LINE_CHARS + 50));
        let section = take_window(&contents, None, None);
        assert!(section.body.contains("... (line truncated)"));
        assert!(section.body.contains("short"));
        let note = section.note.expect("clipping must be reported");
        assert!(note.contains("1 long line(s) clipped"), "{note}");
    }

    #[test]
    fn a_single_file_cannot_exceed_its_own_budget() {
        let contents = "y".repeat(PER_FILE_BUDGET_BYTES * 2) + "\n";
        let contents = contents.replace("yyyyyyyyyy", "yyyyyyyyy\n");
        let section = take_window(&contents, None, None);
        assert!(
            section.body.len() <= PER_FILE_BUDGET_BYTES + MAX_LINE_CHARS + 64,
            "one file must not spend the whole call budget, got {}",
            section.body.len()
        );
    }
}
