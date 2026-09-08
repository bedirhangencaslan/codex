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
    // An entry is a path or a path with its own window. The union is a superset of the shape the
    // model already emits, so a plain list of strings keeps working, and it is what lets one call
    // mix whole files with windows: without it a window can only be asked for one file at a time,
    // which is how a batching tool ends up being called once per file.
    let entry = JsonSchema::any_of(
        vec![
            JsonSchema::string(/*description*/ None),
            JsonSchema::object(
                BTreeMap::from([
                    ("path".to_string(), JsonSchema::string(/*description*/ None)),
                    (
                        "offset".to_string(),
                        JsonSchema::integer(/*description*/ None),
                    ),
                    (
                        "limit".to_string(),
                        JsonSchema::integer(/*description*/ None),
                    ),
                ]),
                Some(vec!["path".to_string()]),
                Some(false.into()),
            ),
        ],
        // The variants carry no descriptions of their own: each one would be paid for on the fixed
        // prefix of every request, and the tool description below already says what they mean.
        None,
    );

    let mut properties = BTreeMap::from([
        (
            "paths".to_string(),
            JsonSchema::array(
                entry,
                Some("Files to read: a path, or an object with a window.".to_string()),
            ),
        ),
        (
            "offset".to_string(),
            JsonSchema::integer(Some("1-indexed first line.".to_string())),
        ),
        (
            "limit".to_string(),
            JsonSchema::integer(Some(format!(
                "Max lines per file. Defaults to {DEFAULT_LINE_LIMIT}."
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
        description: "Read files. Batch every file you already know you need into one call.

- An entry is a path, or {\"path\": \"...\", \"offset\": 1, \"limit\": 200} for one window of that \
file. Mix both in one call.
- Top-level `offset`/`limit` apply to every entry that sets neither, so put the window on the entry when the files differ in what they need.
- Each section is headed by its path and printed as `N: line`, so a citation can be \
written from the read itself; the numbers are display only, never copy them into `apply_patch`.
- A section that was cut says which lines it showed and the offset to continue from. Files that \
did not fit are named at the end; ask for those next."
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The spec rides the fixed prefix of every request, which is the single biggest line of the
    /// bill, so it is not allowed to grow quietly. The `anyOf` entry shape costs about 1_212 bytes
    /// against the 1_157 of the string-only one it replaced, plus the sentence saying `limit` is
    /// per file - the misreading that made one measured call ask for 33,800 lines.
    #[test]
    fn the_spec_stays_small_because_every_request_pays_for_it() {
        let spec = create_read_tool(ReadToolOptions::default());
        let json = serde_json::to_string(&spec).expect("spec must serialize");
        assert!(
            json.len() <= 1_400,
            "read spec grew to {} bytes; every request carries it",
            json.len()
        );
    }
}
