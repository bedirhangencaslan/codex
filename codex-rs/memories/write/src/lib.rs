//! Write-path implementation for Suffice memories.
//!
//! This crate owns the startup memory pipeline, file-backed memory artifact
//! helpers, Phase 1 and Phase 2 prompt rendering, extension pruning, and
//! workspace diffing.

mod control;
mod extensions;
mod guard;
mod metrics;
mod phase1;
mod phase1_output;
mod phase2;
mod prompts;
mod rollout_input;
mod runtime;
mod start;
mod storage;
pub mod workspace;

use codex_utils_absolute_path::AbsolutePathBuf;
use std::path::Path;
use std::path::PathBuf;

pub use control::clear_memory_roots_contents;
pub use extensions::prune_old_extension_resources;
pub use prompts::build_consolidation_prompt;
pub use prompts::build_stage_one_input_message;
pub use start::start_memories_startup_task;
pub use storage::rebuild_raw_memories_file_from_memories;
pub use storage::rollout_summary_file_stem;
pub use storage::sync_rollout_summaries_from_memories;

/// The model a background memory phase should run on.
///
/// The order is: whatever the user configured for this phase, else the provider's own preference,
/// else the model the session is already using.
///
/// That last step is the fix. `ModelProvider::memory_*_preferred_model` has a hard-coded default of
/// `gpt-5.6-terra` / `gpt-5.6-luna`, which is right for providers that serve those slugs and is
/// meant to be overridden by the ones that do not - Bedrock overrides it with its own model ids.
/// A provider configured through `[model_providers.*]` in config.toml has no implementation to
/// override it with, so it inherited the OpenAI slug and asked a third-party endpoint for a model
/// it has never heard of. Observed on 12 of 20 runs against a Z.ai endpoint: one request per run,
/// ~18k tokens of prompt, rejected 400. Free only because it was rejected - against an endpoint
/// that does serve `gpt-5.6-terra`, this silently bills a model the user did not choose.
pub(crate) fn memory_phase_model(
    configured_for_phase: Option<&str>,
    provider_preference: &str,
    provider_default: &str,
    session_model: Option<&str>,
) -> String {
    if let Some(explicit) = configured_for_phase {
        return explicit.to_string();
    }
    if provider_preference != provider_default {
        return provider_preference.to_string();
    }
    // No session model to inherit means nothing better than the old behaviour is available.
    session_model.unwrap_or(provider_preference).to_string()
}

#[cfg(test)]
mod memory_phase_model_tests {
    use super::memory_phase_model;

    const DEFAULT: &str = "gpt-5.6-terra";

    const SESSION: Option<&str> = Some("glm-5.3-flash");

    #[test]
    fn an_explicit_choice_wins() {
        assert_eq!(
            memory_phase_model(Some("o-something"), "bedrock.gpt", DEFAULT, SESSION),
            "o-something"
        );
    }

    #[test]
    fn a_provider_that_overrode_the_default_is_honoured() {
        assert_eq!(
            memory_phase_model(None, "bedrock.gpt", DEFAULT, SESSION),
            "bedrock.gpt"
        );
    }

    #[test]
    fn a_provider_that_did_not_falls_back_to_the_session_model() {
        assert_eq!(memory_phase_model(None, DEFAULT, DEFAULT, SESSION), "glm-5.3-flash");
    }

    #[test]
    fn with_no_session_model_the_old_behaviour_stands() {
        assert_eq!(memory_phase_model(None, DEFAULT, DEFAULT, None), DEFAULT);
    }
}

#[cfg(test)]
mod startup_tests;

mod artifacts {
    pub(super) const EXTENSIONS_SUBDIR: &str = "extensions";
    pub(super) const ROLLOUT_SUMMARIES_SUBDIR: &str = "rollout_summaries";
    pub(super) const RAW_MEMORIES_FILENAME: &str = "raw_memories.md";
}

mod extension_resources {
    pub(super) const FILENAME_TS_FORMAT: &str = "%Y-%m-%dT%H-%M-%S";
    pub(super) const RETENTION_DAYS: i64 = 7;
}

mod guard_limits {
    pub(super) const CODEX_LIMIT_ID: &str = "codex";
}

mod prompt_blocks {
    pub(super) const EXTENSIONS_FOLDER_STRUCTURE: &str = r#"
Memory extensions (under {{ memory_extensions_root }}/):

- <extension_name>/instructions.md
  - Source-specific guidance for interpreting additional memory signals. If an
    extension folder exists, you must read its instructions.md to determine how to use this memory
    source.

If the user has any memory extensions, you MUST read the instructions for each extension to
determine how to use the memory source. If the workspace diff shows deleted extension resource files,
remove stale memories derived only from those resources. If it has no extension folders, continue
with the standard memory inputs only.
"#;

    pub(super) const EXTENSIONS_PRIMARY_INPUTS: &str = r#"
Optional source-specific inputs:
Under `{{ memory_extensions_root }}/`:

- `<extension_name>/instructions.md`
  - If extension folders exist, read each instructions.md first and follow it when interpreting
    that extension's memory source.

If the workspace diff shows deleted memory extension resources, use that extension-specific deletion
signal to remove stale memories derived only from those resources.
"#;
}

mod stage_one {
    pub(super) const REASONING_EFFORT: codex_protocol::openai_models::ReasoningEffort =
        codex_protocol::openai_models::ReasoningEffort::Low;
    pub(super) const CONCURRENCY_LIMIT: usize = 8;
    pub(super) const JOB_LEASE_SECONDS: i64 = 3_600;
    pub(super) const JOB_RETRY_DELAY_SECONDS: i64 = 3_600;
    pub(super) const THREAD_SCAN_LIMIT: usize = 5_000;
    pub(super) const PRUNE_BATCH_SIZE: usize = 200;

    /// Prompt used for phase 1 extraction.
    pub(super) const PROMPT: &str = include_str!("../templates/memories/stage_one_system.md");

    /// Fallback stage-1 rollout truncation limit (tokens) when model metadata
    /// does not include a valid context window.
    pub(super) const DEFAULT_ROLLOUT_TOKEN_LIMIT: usize = 150_000;

    /// Portion of the model effective input window reserved for the stage-1
    /// rollout input.
    ///
    /// Keeping this below 100% leaves room for system instructions, prompt framing,
    /// and model output.
    pub(super) const CONTEXT_WINDOW_PERCENT: i64 = 70;
}

mod stage_two {
    pub(super) const REASONING_EFFORT: codex_protocol::openai_models::ReasoningEffort =
        codex_protocol::openai_models::ReasoningEffort::Medium;
    pub(super) const JOB_LEASE_SECONDS: i64 = 3_600;
    pub(super) const JOB_RETRY_DELAY_SECONDS: i64 = 3_600;
    pub(super) const JOB_HEARTBEAT_SECONDS: u64 = 90;
}

mod workspace_diff {
    /// Generated diff file the Phase 2 consolidation agent reads before editing memories.
    pub(super) const FILENAME: &str = "phase2_workspace_diff.md";
    pub(super) const MAX_BYTES: usize = 4 * 1024 * 1024;
}

pub fn memory_root(codex_home: &AbsolutePathBuf) -> AbsolutePathBuf {
    codex_home.join("memories")
}

pub fn rollout_summaries_dir(root: &Path) -> PathBuf {
    root.join(artifacts::ROLLOUT_SUMMARIES_SUBDIR)
}

pub fn memory_extensions_root(root: &Path) -> PathBuf {
    root.join(artifacts::EXTENSIONS_SUBDIR)
}

pub fn raw_memories_file(root: &Path) -> PathBuf {
    root.join(artifacts::RAW_MEMORIES_FILENAME)
}

pub async fn ensure_layout(root: &Path) -> std::io::Result<()> {
    tokio::fs::create_dir_all(root).await?;
    if tokio::fs::symlink_metadata(root)
        .await?
        .file_type()
        .is_symlink()
    {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("memory root cannot be a symbolic link: {}", root.display()),
        ));
    }

    workspace::remove_memory_symlinks(root).await?;
    tokio::fs::create_dir_all(rollout_summaries_dir(root)).await
}
