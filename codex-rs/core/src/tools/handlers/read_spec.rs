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
    let mut properties = BTreeMap::from([
        (
            "paths".to_string(),
            JsonSchema::array(
                JsonSchema::string(Some("Path to a file, relative to the working directory or absolute.".to_string())),
                Some("Files to read. Pass every file you already know you need in one call.".to_string()),
            ),
        ),
        (
            "offset".to_string(),
            JsonSchema::integer(Some(
                "1-indexed line to start at. Only used when `paths` has exactly one entry.".to_string(),
            )),
        ),
        (
            "limit".to_string(),
            JsonSchema::integer(Some(format!(
                "Maximum lines to return. Only used when `paths` has exactly one entry. Defaults to {DEFAULT_LINE_LIMIT}."
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
        description: "Read whole files from the filesystem.

- Pass every file you already know you need in a single call. Reading twenty files in one call \
costs one round trip; reading them one at a time costs twenty, and every round trip resends the \
whole conversation.
- Use `offset` and `limit` to read a window of one file when you already know the line you want.
- Output is one section per file, each headed by its path.
- If the call runs out of budget it says so and names the files it did not reach, so ask for those \
in a follow-up call."
            .to_string(),
        strict: false,
        defer_loading: None,
        parameters: JsonSchema::object(
            properties,
            Some(vec!["paths".to_string()]),
            Some(false.into()),
        ),
        output_schema: None,
    })
}
