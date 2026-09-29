use codex_tools::JsonSchema;
use codex_tools::ResponsesApiTool;
use codex_tools::ToolSpec;
use std::collections::BTreeMap;

pub const READ_TOOL_NAME: &str = "read";

/// Default number of lines returned per file when the model does not ask for a window.
pub(crate) const DEFAULT_LINE_LIMIT: usize = 2_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ReadToolOptions {
    pub include_environment_id: bool,
}

pub fn create_read_tool(options: ReadToolOptions) -> ToolSpec {
    // One file per call, because the model reasons about one file at a time.
    //
    // The batched shape asked for a list and answered with one result carrying every file, which
    // sounds cheaper and measured worse: a window is one number applied to a heterogeneous batch,
    // so it is either wrong for most of them or left out, and the model that leaves it out gets
    // the whole corpus in a single tool output. OpenCode's model, given the same task and one file
    // per call, chose a window on 259 of 264 calls and pulled a third of the bytes.
    //
    // Batching does not disappear, it moves to the protocol: several `read` calls in one assistant
    // message run in parallel and come back as separate results, which is what the description
    // below asks for and what `supports_parallel_tool_calls` already advertises.
    let mut properties = BTreeMap::from([
        (
            "filePath".to_string(),
            JsonSchema::string(Some("File or directory; relative to the turn cwd or absolute.".to_string())),
        ),
        (
            "offset".to_string(),
            JsonSchema::integer(Some(
                "First line to return (1-indexed).".to_string(),
            )),
        ),
        (
            "limit".to_string(),
            JsonSchema::integer(Some(format!(
                "Maximum lines to return. Defaults to {DEFAULT_LINE_LIMIT}."
            ))),
        ),
    ]);
    if options.include_environment_id {
        properties.insert(
            "environment_id".to_string(),
            JsonSchema::string(Some(
                "Environment id from <environment_context>. Omit to use the primary environment."
                    .to_string(),
            )),
        );
    }

    ToolSpec::Function(ResponsesApiTool {
        name: READ_TOOL_NAME.to_string(),
        description: "Reads a file with 1-indexed line numbers (`<line>: <content>`), or lists a directory (subdirectories end in `/`).

- Returns up to 2000 lines from `offset`; lines over 2000 characters are cut.
- A file's purpose is usually in its first ~100 lines: read a window that size, not the whole file.
- Everything read stays in context and is paid for on every later request.
- Line numbers are display only; never copy them into `apply_patch`.
- Several reads in one response run in parallel."
            .to_string(),
        strict: false,
        defer_loading: None,
        parameters: JsonSchema::object(
            properties,
            Some(vec!["filePath".to_string()]),
            Some(false.into()),
        ),
        output_schema: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The spec rides the fixed prefix of every request, which is the single biggest line of the
    /// bill, so it is not allowed to grow quietly. The difference over the batched shape it
    /// replaced buys two sentences: that several calls may go out together, and the pair that hand
    /// a large file to `grep` and a missing name to `glob` instead of to another read.
    ///
    /// A third sentence used to be here, naming the byte ceiling so the model would not "discover
    /// it by being cut". Measured, that is the single most expensive sentence in the tool surface:
    /// told a ceiling is enforced for it, this model stops setting `limit` at all and takes the
    /// 2000-line default. Eight simulated runs carrying it read 6,939 bytes a file against 3,184
    /// for thirteen without, five of the eight over 6,000 against none - one-sided Fisher
    /// p = 0.0028. OpenCode does not advertise its own 50 KB cap either, and a read that is cut
    /// still says so in its own output, which is where the model can act on it.
    ///
    /// The ceiling moved 1_900 -> 2_000 to make room for the window sentence, which came out of
    /// `instructions_template` and cost the prompt the same 306 bytes it added here, so the fixed
    /// prefix is unchanged. It had to move: measured, the prompt's copy of that guidance works in
    /// OpenCode's two-message layout and stops working in ours, where it sits ~30 KB and a role
    /// boundary upstream of the task - the fork's prompt and the fork's layout are each harmless
    /// alone and cost 2.71x together, 9,406 bytes a read against 3,465, with no overlap across
    /// three reps a side. A tool description is adjacent to the decision however the messages are
    /// arranged, which is why OpenCode keeps its reading guidance here and none in its prompt.
    #[test]
    fn the_spec_stays_small_because_every_request_pays_for_it() {
        let spec = create_read_tool(ReadToolOptions::default());
        let json = serde_json::to_string(&spec).expect("spec must serialize");
        println!("read spec serializes to {} bytes", json.len());
        assert!(
            json.len() <= 2_000,
            "read spec grew to {} bytes; every request carries it",
            json.len()
        );
    }
}
