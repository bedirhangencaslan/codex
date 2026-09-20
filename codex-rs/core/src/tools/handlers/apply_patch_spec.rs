use codex_protocol::openai_models::ApplyPatchToolType;
use codex_tools::FreeformTool;
use codex_tools::FreeformToolFormat;
use codex_tools::ToolSpec;

const APPLY_PATCH_LARK_GRAMMAR: &str = include_str!("../../../assets/tools/apply_patch.lark");

/// Returns a custom tool that can be used to edit files.
/// https://platform.openai.com/docs/guides/function-calling#custom-tools
///
/// `tool_type` decides whether the Lark grammar rides along. Providers that honour
/// grammar-constrained decoding get `ApplyPatchToolType::Freeform` and cannot emit a malformed
/// patch; the rest get `Prose`, where the format is taught in the model's instructions instead
/// and `models-manager` keeps the matching section in the prompt.
pub fn create_apply_patch_freeform_tool(
    include_environment_id: bool,
    tool_type: ApplyPatchToolType,
) -> ToolSpec {
    let format = match tool_type {
        ApplyPatchToolType::Freeform => {
            let definition = if include_environment_id {
                APPLY_PATCH_LARK_GRAMMAR.replace(
                    "start: begin_patch hunk+ end_patch",
                    "start: begin_patch environment_id? hunk+ end_patch\nenvironment_id: \"*** Environment ID: \" filename LF",
                )
            } else {
                APPLY_PATCH_LARK_GRAMMAR.to_string()
            };
            FreeformToolFormat::lark(definition)
        }
        ApplyPatchToolType::Prose => FreeformToolFormat::text(),
    };
    ToolSpec::Freeform(FreeformTool {
        name: "apply_patch".to_string(),
        description: "The `apply_patch` tool can be used to edit files. This is a FREEFORM tool, so do not wrap the patch in JSON.".to_string(),
        defer_loading: None,
        format,
    })
}

#[cfg(test)]
#[path = "apply_patch_spec_tests.rs"]
mod tests;
