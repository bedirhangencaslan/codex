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
            JsonSchema::string(Some(
                "The absolute path to the file or directory to read".to_string(),
            )),
        ),
        (
            "offset".to_string(),
            JsonSchema::integer(Some(
                "The line number to start reading from (1-indexed)".to_string(),
            )),
        ),
        (
            "limit".to_string(),
            JsonSchema::integer(Some(format!(
                "The maximum number of lines to read (defaults to {DEFAULT_LINE_LIMIT})"
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
        description: "Read one file or directory from the local filesystem. If the path does not exist, an error is returned.

Usage:
- The filePath parameter should be an absolute path.
- By default, this tool returns up to 2000 lines from the start of the file.
- The offset parameter is the line number to start from (1-indexed).
- To read later sections, call this tool again with a larger offset.
- Use the grep tool to find specific content in large files or files with long lines.
- If you are unsure of the correct file path, use the glob tool to look up filenames by glob pattern.
- Contents are returned with each line prefixed by its line number as `<line>: <content>`. For example, if a file has contents \"foo\\n\", you will receive \"1: foo\\n\". For directories, entries are returned one per line (without line numbers) with a trailing `/` for subdirectories.
- Any line longer than 2000 characters is truncated.
- Call this tool in parallel when you know there are multiple files you want to read: several `read` calls in one response run together and each answers on its own.
- Avoid tiny repeated slices (30 line chunks). If you need more context, read a larger window.
- Line numbers are display only; never copy them into `apply_patch`."
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
    /// A window sentence was tried here too - "the first hundred or so lines" - moved in from
    /// `instructions_template`. It produced the best-behaved run measured (median 180, 44 of 44
    /// bounded) and still cost twice OpenCode, because the prompt's own reading bullets argued the
    /// other way: a round costs more than a call, ask for a window wide enough, a cut read tells you
    /// where to resume. Against three sentences pushing "take it all in one call", one line here
    /// does not win. Both sides are gone now - the prompt's bullets and this sentence - which is
    /// what OpenCode does: no reading advice in the prompt at all, and none here beyond "avoid tiny
    /// repeated slices".
    #[test]
    fn the_spec_stays_small_because_every_request_pays_for_it() {
        let spec = create_read_tool(ReadToolOptions::default());
        let json = serde_json::to_string(&spec).expect("spec must serialize");
        println!("read spec serializes to {} bytes", json.len());
        assert!(
            json.len() <= 1_900,
            "read spec grew to {} bytes; every request carries it",
            json.len()
        );
    }
}
