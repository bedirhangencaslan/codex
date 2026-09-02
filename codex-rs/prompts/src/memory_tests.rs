use super::*;

#[test]
fn agents_md_memory_prompt_substitutes_the_path() {
    let rendered = agents_md_memory_prompt("docs/CONTEXT.md");
    assert!(rendered.contains("`docs/CONTEXT.md`"));
    assert!(!rendered.contains("{{agents_md_path}}"));
}

#[test]
fn agents_md_memory_prompt_opens_as_a_system_procedure() {
    assert!(
        agents_md_memory_prompt("AGENTS.md")
            .starts_with("You are performing a PROJECT MEMORY CHECKPOINT.")
    );
}
