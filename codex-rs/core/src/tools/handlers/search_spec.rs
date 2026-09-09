//! Specs for the two search tools, copied from OpenCode.
//!
//! The descriptions are OpenCode's own, changed only where a sentence would be false here: it
//! names a `Task` tool we do not expose and a `Bash` tool we call `exec_command`. Everything else
//! is verbatim, because the wording is the thing under test.
//!
//! One exception, and it was measured: OpenCode's grep description ends with "If you need to
//! identify/count the number of matches within files, use the Bash tool with `rg` (ripgrep)
//! directly. Do NOT use `grep`." That is about `rg -c`, but the verb "identify" and the
//! categorical "Do NOT use `grep`" read much wider, and this model takes the wide reading of
//! everything - the trait that made it read whole files to satisfy the word "every". In one
//! recorded run it reasoned "we can use rg with patterns to identify public structs/enums"
//! and left the tool for `exec_command`, whose output then came back "truncated and
//! disordered" and was discarded. The line is narrowed to what it actually means.
//!
//! Why these exist at all: measured on a 44-file task, OpenCode's model read 71 lines per file and
//! never continued a truncated one, then closed the remaining citations with eight `grep` calls.
//! Ours read the same corpus two to six times over. The instruction that separates them is one
//! line in OpenCode's *read* description - "Use the grep tool to find specific content in large
//! files" - which points at a tool we did not have, so the sentence could not be copied without
//! first building the tool.

use codex_tools::JsonSchema;
use codex_tools::ResponsesApiTool;
use codex_tools::ToolSpec;
use std::collections::BTreeMap;

pub const GLOB_TOOL_NAME: &str = "glob";
pub const GREP_TOOL_NAME: &str = "grep";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SearchToolOptions {
    pub include_environment_id: bool,
}

fn environment_id_property() -> (String, JsonSchema) {
    (
        "environment_id".to_string(),
        JsonSchema::string(Some(
            "Environment id from <environment_context>. Omit to use the primary environment."
                .to_string(),
        )),
    )
}

pub fn create_glob_tool(options: SearchToolOptions) -> ToolSpec {
    let mut properties = BTreeMap::from([
        (
            "pattern".to_string(),
            JsonSchema::string(Some("The glob pattern to match files against".to_string())),
        ),
        (
            "path".to_string(),
            JsonSchema::string(Some(
                "The directory to search in. If not specified, the current working directory will \
                 be used."
                    .to_string(),
            )),
        ),
    ]);
    if options.include_environment_id {
        let (key, schema) = environment_id_property();
        properties.insert(key, schema);
    }

    ToolSpec::Function(ResponsesApiTool {
        name: GLOB_TOOL_NAME.to_string(),
        description: "- Fast file pattern matching tool that works with any codebase size
- Supports glob patterns like \"**/*.js\" or \"src/**/*.ts\"
- Returns matching file paths
- Use this tool when you need to find files by name patterns
- You have the capability to call multiple tools in a single response. It is always better to \
speculatively perform multiple searches as a batch that are potentially useful."
            .to_string(),
        strict: false,
        defer_loading: None,
        parameters: JsonSchema::object(
            properties,
            Some(vec!["pattern".to_string()]),
            Some(false.into()),
        ),
        output_schema: None,
    })
}

pub fn create_grep_tool(options: SearchToolOptions) -> ToolSpec {
    let mut properties = BTreeMap::from([
        (
            "pattern".to_string(),
            JsonSchema::string(Some(
                "The regex pattern to search for in file contents".to_string(),
            )),
        ),
        (
            "path".to_string(),
            JsonSchema::string(Some(
                "The directory to search in. Defaults to the current working directory."
                    .to_string(),
            )),
        ),
        (
            "include".to_string(),
            JsonSchema::string(Some(
                "File pattern to include in the search (e.g. \"*.js\", \"*.{ts,tsx}\")".to_string(),
            )),
        ),
    ]);
    if options.include_environment_id {
        let (key, schema) = environment_id_property();
        properties.insert(key, schema);
    }

    ToolSpec::Function(ResponsesApiTool {
        name: GREP_TOOL_NAME.to_string(),
        description: "- Fast content search tool that works with any codebase size
- Searches file contents using regular expressions
- Supports full regex syntax (eg. \"log.*Error\", \"function\\s+\\w+\", etc.)
- Filter files by pattern with the include parameter (eg. \"*.js\", \"*.{ts,tsx}\")
- Returns file paths and line numbers with matching lines
- Use this tool when you need to find files containing specific patterns
- If you need the number of matches rather than the matches themselves, use `exec_command` with \
`rg -c`."
            .to_string(),
        strict: false,
        defer_loading: None,
        parameters: JsonSchema::object(
            properties,
            Some(vec!["pattern".to_string()]),
            Some(false.into()),
        ),
        output_schema: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Both specs ride the fixed prefix of every request, which is the biggest single line of the
    /// bill. OpenCode pays 657 bytes for grep's description and 517 for glob's; the budget here is
    /// the serialized spec including its schema, so it is larger than those two numbers, and it is
    /// asserted so the pair cannot grow quietly once the experiment is over.
    #[test]
    fn the_search_specs_stay_small_because_every_request_pays_for_them() {
        let glob = serde_json::to_string(&create_glob_tool(SearchToolOptions::default()))
            .expect("spec must serialize");
        let grep = serde_json::to_string(&create_grep_tool(SearchToolOptions::default()))
            .expect("spec must serialize");
        assert!(glob.len() <= 900, "glob spec grew to {} bytes", glob.len());
        assert!(
            grep.len() <= 1_300,
            "grep spec grew to {} bytes",
            grep.len()
        );
    }
}
