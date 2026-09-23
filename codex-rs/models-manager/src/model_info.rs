use codex_protocol::config_types::Personality;
use codex_protocol::config_types::ReasoningSummary;
use codex_protocol::openai_models::ApplyPatchToolType;
use codex_protocol::openai_models::ConfigShellToolType;
use codex_protocol::openai_models::ModelInfo;
use codex_protocol::openai_models::ModelMessages;
use codex_protocol::openai_models::ModelVisibility;
use codex_protocol::openai_models::ToolMode;
use codex_protocol::openai_models::TruncationMode;
use codex_protocol::openai_models::TruncationPolicyConfig;
use codex_protocol::openai_models::WebSearchToolType;
use codex_protocol::openai_models::default_input_modalities;

use crate::config::ModelsManagerConfig;
use codex_utils_output_truncation::approx_bytes_for_tokens;
use tracing::warn;

pub const BASE_INSTRUCTIONS: &str = include_str!("../prompt.md");
const PERSONALITY_SECTION_HEADER: &str = "# Personality";

/// The patch format, in words, for models whose provider does not honour the Lark grammar.
///
/// It is held here rather than in each model's `instructions_template` so that the grammar and
/// the prose cannot drift apart, and so that turning a model from `Prose` to `Freeform` is a
/// one-field change in `models.json` instead of a prompt edit.
pub const APPLY_PATCH_PROSE: &str = include_str!("../apply_patch_prose.md");
const APPLY_PATCH_SECTION_HEADER: &str = "## apply_patch";

pub fn with_config_overrides(mut model: ModelInfo, config: &ModelsManagerConfig) -> ModelInfo {
    if let Some(context_window) = config.model_context_window {
        model.context_window = Some(
            model
                .max_context_window
                .map_or(context_window, |max_context_window| {
                    context_window.min(max_context_window)
                }),
        );
    }
    if let Some(auto_compact_token_limit) = config.model_auto_compact_token_limit {
        model.auto_compact_token_limit = Some(auto_compact_token_limit);
    }
    if let Some(token_limit) = config.tool_output_token_limit {
        model.truncation_policy = match model.truncation_policy.mode {
            TruncationMode::Bytes => {
                let byte_limit =
                    i64::try_from(approx_bytes_for_tokens(token_limit)).unwrap_or(i64::MAX);
                TruncationPolicyConfig::bytes(byte_limit)
            }
            TruncationMode::Tokens => {
                let limit = i64::try_from(token_limit).unwrap_or(i64::MAX);
                TruncationPolicyConfig::tokens(limit)
            }
        };
    }

    if let Some(base_instructions) = &config.base_instructions {
        let model_messages = model.model_messages.get_or_insert_default();
        model_messages.instructions_template = Some(base_instructions.clone());
        model_messages.instructions_variables = None;
    } else {
        if config.personality == Some(Personality::None)
            && let Some(instructions_template) = model
                .model_messages
                .as_mut()
                .and_then(|messages| messages.instructions_template.as_mut())
        {
            *instructions_template =
                strip_personality_section(std::mem::take(instructions_template));
        }

        // Last, so it sees the template the other overrides settled on. Skipped entirely when
        // the user supplied `base_instructions`: that prompt is theirs to get right.
        let apply_patch_tool_type = model.apply_patch_tool_type;
        if let Some(instructions_template) = model
            .model_messages
            .as_mut()
            .and_then(|messages| messages.instructions_template.as_mut())
        {
            *instructions_template = align_apply_patch_section(
                std::mem::take(instructions_template),
                apply_patch_tool_type,
            );
        }

        // The mode the model asks for, resolved as core's `requested_tool_mode` does: the
        // catalog entry first, configuration only when the entry leaves it unset. Code mode is
        // only real when its host is installed, so nothing changes without it.
        let tool_mode = model.tool_mode.or(config.config_tool_mode);
        if config.code_mode_host_available
            && let Some(tool_mode) = tool_mode
            && let Some(instructions_template) = model
                .model_messages
                .as_mut()
                .and_then(|messages| messages.instructions_template.as_mut())
        {
            *instructions_template =
                align_code_mode_lines(std::mem::take(instructions_template), tool_mode);
        }
    }

    model
}

/// This fork's own steer toward doing bulk work in a shell script (`1d2c1a5f9`, OpenCode's
/// wording). It was added because nothing else could do that work in one step; `exec` can, so it
/// leaves the instructions whenever `exec` is offered and the prompt reads as it did before it.
const SHELL_SCRIPT_STEER: &str = "- You reach for `exec_command` with a short read-only inline script when the job is local parsing, arithmetic, templating, or a tabular rollup, and you print a short result rather than the data it read.\n";

/// The bullet the `exec` guidance follows. Where it is missing the guidance is not added: a
/// bullet stranded in some other section would read as belonging to it.
const PARALLEL_CALLS_BULLET_PREFIX: &str = "- You issue several tool calls in a single response";

/// gpt-6-astra's two `exec` bullets, with `functions.exec` written as `exec`: that prefix is how
/// OpenAI's own format names tools, and on the Chat Completions wire the tool is called `exec`.
/// Only for `code_mode_only`, where `exec` is the one way to reach any other tool. In hybrid mode
/// the direct tools are still offered and the parallel-calls bullet already covers batching them.
const CODE_MODE_ONLY_EXEC_BULLETS: &str = "- Batch independent searches and reads in one exec using await Promise.allSettled([...]); inspect every result. Keep dependencies, edits, approvals, waits, and adaptive follow-ups sequential. Avoid unnecessary output.\n- When calling `exec`, parallelize independent tool calls by awaiting Promises. Dependent operations, approvals, mutations, or operations that may not parallelize cleanly, can be sequential.\n";

fn align_code_mode_lines(template: String, tool_mode: ToolMode) -> String {
    match tool_mode {
        ToolMode::Direct => template,
        ToolMode::CodeMode => template.replacen(SHELL_SCRIPT_STEER, "", 1),
        ToolMode::CodeModeOnly => insert_after_line(
            template.replacen(SHELL_SCRIPT_STEER, "", 1),
            PARALLEL_CALLS_BULLET_PREFIX,
            CODE_MODE_ONLY_EXEC_BULLETS,
        ),
    }
}

/// Inserts `text` after the first line starting with `prefix`, or returns `template` unchanged.
fn insert_after_line(template: String, prefix: &str, text: &str) -> String {
    let Some(line_start) = template
        .match_indices(prefix)
        .map(|(index, _)| index)
        .find(|&index| index == 0 || template[..index].ends_with('\n'))
    else {
        return template;
    };
    let insert_at = template[line_start..]
        .find('\n')
        .map_or(template.len(), |offset| line_start + offset + 1);
    let mut aligned = template;
    if !aligned[..insert_at].ends_with('\n') {
        aligned.insert(insert_at, '\n');
        aligned.insert_str(insert_at + 1, text.trim_end_matches('\n'));
        return aligned;
    }
    aligned.insert_str(insert_at, text);
    aligned
}

/// Make the instructions carry the patch format exactly when the grammar does not.
///
/// `Freeform` constrains sampling, so the prose is a second copy of the same rules riding every
/// request's fixed prefix; it is removed. `Prose` has nothing enforcing the format, so the
/// section is (re)inserted from `APPLY_PATCH_PROSE`. Stripping first in both cases keeps this
/// idempotent and lets a model's own template stay authoritative about everything else.
fn align_apply_patch_section(
    instructions: String,
    apply_patch_tool_type: Option<ApplyPatchToolType>,
) -> String {
    let Some(apply_patch_tool_type) = apply_patch_tool_type else {
        // No `apply_patch` tool is registered at all, so the model edits through the shell.
        // `APPLY_PATCH_PROSE` describes a tool and would be wrong here, and the shell wording
        // lives in each such model's own template. Leave it alone.
        return instructions;
    };
    let (mut stripped, was_at) = strip_apply_patch_section(instructions);
    match apply_patch_tool_type {
        ApplyPatchToolType::Prose => {
            // Put the section back where the template had it, and normalise line endings.
            // `a71c1231a` appended instead, which moved the block roughly 4,000 characters
            // later in the prompt; and `core.autocrlf` turns every newline in a Windows
            // checkout of `apply_patch_prose.md` into CRLF, which `include_str!` then ships
            // to the model. The prompt is a measured surface, so it should not depend on
            // which platform built the binary, nor reorder itself as a side effect of
            // swapping the grammar for prose.
            let prose = APPLY_PATCH_PROSE.replace("\r\n", "\n");
            match was_at {
                Some(at) => {
                    let mut block = prose.trim_end().to_string();
                    block.push_str("\n\n");
                    stripped.insert_str(at, &block);
                    stripped
                }
                None => {
                    let mut out = stripped.trim_end().to_string();
                    out.push_str("\n\n");
                    out.push_str(prose.trim_end());
                    out.push('\n');
                    out
                }
            }
        }
        ApplyPatchToolType::Freeform => stripped,
    }
}

/// Remove a `## apply_patch` section, up to the next heading of the same or higher level.
///
/// Returns where the section started, so a caller that means to replace it can put the new
/// one in the same place rather than at the end.
fn strip_apply_patch_section(mut instructions: String) -> (String, Option<usize>) {
    let mut section_start = None;
    let mut section_end = None;
    let mut offset = 0;

    for line_with_ending in instructions.split_inclusive('\n') {
        let line = match line_with_ending.strip_suffix('\n') {
            Some(line) => line.strip_suffix('\r').unwrap_or(line),
            None => line_with_ending,
        };
        if section_start.is_some() {
            if is_h1_heading(line) || is_h2_heading(line) {
                section_end = Some(offset);
                break;
            }
        } else if line.trim_end() == APPLY_PATCH_SECTION_HEADER {
            section_start = Some(offset);
        }
        offset += line_with_ending.len();
    }

    if let Some(section_start) = section_start {
        let section_end = section_end.unwrap_or(instructions.len());
        instructions.replace_range(section_start..section_end, "");
    }

    (instructions, section_start)
}

fn is_h2_heading(line: &str) -> bool {
    let Some(rest) = line.strip_prefix("##") else {
        return false;
    };
    !rest.starts_with('#') && (rest.is_empty() || rest.starts_with(' ') || rest.starts_with('\t'))
}

fn strip_personality_section(mut instructions: String) -> String {
    let mut section_start = None;
    let mut section_end = None;
    let mut offset = 0;

    for line_with_ending in instructions.split_inclusive('\n') {
        let line = match line_with_ending.strip_suffix('\n') {
            Some(line) => line.strip_suffix('\r').unwrap_or(line),
            None => line_with_ending,
        };
        if section_start.is_some() {
            if is_h1_heading(line) {
                section_end = Some(offset);
                break;
            }
        } else if line == PERSONALITY_SECTION_HEADER {
            section_start = Some(offset);
        }
        offset += line_with_ending.len();
    }

    if let Some(section_start) = section_start {
        let section_end = section_end.unwrap_or(instructions.len());
        instructions.replace_range(section_start..section_end, "");
    }

    instructions
}

fn is_h1_heading(line: &str) -> bool {
    let Some(rest) = line.strip_prefix('#') else {
        return false;
    };
    rest.is_empty() || rest.starts_with(' ') || rest.starts_with('\t')
}

/// Build a minimal fallback model descriptor for missing/unknown slugs.
pub fn model_info_from_slug(slug: &str) -> ModelInfo {
    warn!("Unknown model {slug} is used. This will use fallback model metadata.");
    ModelInfo {
        slug: slug.to_string(),
        display_name: slug.to_string(),
        description: None,
        default_reasoning_level: None,
        supported_reasoning_levels: Vec::new(),
        shell_type: ConfigShellToolType::UnifiedExec,
        visibility: ModelVisibility::None,
        supported_in_api: true,
        priority: 99,
        additional_speed_tiers: Vec::new(),
        service_tiers: Vec::new(),
        default_service_tier: None,
        available_access_programs: None,
        availability_nux: None,
        upgrade: None,
        model_messages: Some(local_model_messages()),
        include_skills_usage_instructions: false,
        include_plugin_usage_instructions: false,
        include_apps_usage_instructions: false,
        supports_reasoning_summary_parameter: true,
        supports_encrypted_reasoning: true,
        default_reasoning_summary: ReasoningSummary::Auto,
        support_verbosity: false,
        default_verbosity: None,
        apply_patch_tool_type: None,
        web_search_tool_type: WebSearchToolType::Text,
        truncation_policy: TruncationPolicyConfig::bytes(/*limit*/ 10_000),
        supports_image_detail_original: false,
        context_window: Some(272_000),
        max_context_window: Some(272_000),
        auto_compact_token_limit: None,
        comp_hash: None,
        effective_context_window_percent: 95,
        experimental_supported_tools: Vec::new(),
        input_modalities: default_input_modalities(),
        used_fallback_model_metadata: true, // this is the fallback model metadata
        supports_search_tool: false,
        supports_experimental_context: false,
        use_responses_lite: false,
        supports_reasoning_effort_updates: false,
        guardian: None,
        node_repl_auto_review_required: false,
        node_repl_disabled: false,
        auto_review_model_override: None,
        model_specialty: None,
        tool_mode: None,
        multi_agent_version: None,
        multi_agent_reasoning_effort: None,
    }
}

fn local_model_messages() -> ModelMessages {
    ModelMessages {
        instructions_template: Some(BASE_INSTRUCTIONS.to_string()),
        ..Default::default()
    }
}

#[cfg(test)]
#[path = "model_info_tests.rs"]
mod tests;
