use codex_utils_template::Template;
use std::sync::LazyLock;

/// Prompt for the pre-compaction refresh of the project's AGENTS.md memory.
const AGENTS_MD_MEMORY_PROMPT: &str = include_str!("../templates/memory/agents_md.md");

static AGENTS_MD_MEMORY_TEMPLATE: LazyLock<Template> = LazyLock::new(|| {
    Template::parse(AGENTS_MD_MEMORY_PROMPT)
        .unwrap_or_else(|err| panic!("agents md memory prompt must parse: {err}"))
});

/// Renders the memory prompt. `agents_md_path` is the memory file as the model should spell it,
/// relative to the turn's working directory.
pub fn agents_md_memory_prompt(agents_md_path: &str) -> String {
    AGENTS_MD_MEMORY_TEMPLATE
        .render([("agents_md_path", agents_md_path)])
        .unwrap_or_else(|err| panic!("agents md memory prompt must render: {err}"))
}
