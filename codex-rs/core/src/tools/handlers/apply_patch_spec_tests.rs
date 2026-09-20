use super::*;
use pretty_assertions::assert_eq;

#[test]
fn create_apply_patch_freeform_tool_matches_expected_spec() {
    assert_eq!(
        create_apply_patch_freeform_tool(
            /*include_environment_id*/ false,
            ApplyPatchToolType::Freeform
        ),
        ToolSpec::Freeform(FreeformTool {
            name: "apply_patch".to_string(),
            description:
                "The `apply_patch` tool can be used to edit files. This is a FREEFORM tool, so do not wrap the patch in JSON."
                    .to_string(),
            defer_loading: None,
            format: FreeformToolFormat::lark(APPLY_PATCH_LARK_GRAMMAR.to_string()),
        })
    );
}

#[test]
fn create_apply_patch_freeform_tool_includes_environment_id_when_requested() {
    let ToolSpec::Freeform(tool) = create_apply_patch_freeform_tool(
        /*include_environment_id*/ true,
        ApplyPatchToolType::Freeform,
    ) else {
        panic!("expected freeform tool");
    };

    let definition = tool.format.definition.expect("grammar carries a definition");
    assert!(definition.contains("environment_id?"));
    assert!(definition.contains("\"*** Environment ID: \" filename LF"));
}

#[test]
fn prose_models_get_a_text_tool_with_no_grammar() {
    let ToolSpec::Freeform(tool) =
        create_apply_patch_freeform_tool(/*include_environment_id*/ false, ApplyPatchToolType::Prose)
    else {
        panic!("expected freeform tool");
    };

    assert_eq!(tool.format, FreeformToolFormat::text());
    assert_eq!(tool.format.r#type, "text");
    assert_eq!(tool.format.syntax, None);
    assert_eq!(tool.format.definition, None);
}

/// A provider that does not understand `syntax`/`definition` must not receive them at all,
/// rather than receiving them empty.
#[test]
fn a_text_tool_serializes_without_the_grammar_fields() {
    let spec = create_apply_patch_freeform_tool(
        /*include_environment_id*/ false,
        ApplyPatchToolType::Prose,
    );

    let json = serde_json::to_value(&spec).expect("serialize tool spec");
    let format = &json["format"];

    assert_eq!(format["type"], "text");
    assert!(format.get("syntax").is_none(), "{json}");
    assert!(format.get("definition").is_none(), "{json}");
    assert_eq!(json["type"], "custom");
}

/// The environment-id rewrite only applies to the grammar, so it must not resurrect one.
#[test]
fn prose_stays_grammarless_even_in_multi_environment_mode() {
    let ToolSpec::Freeform(tool) =
        create_apply_patch_freeform_tool(/*include_environment_id*/ true, ApplyPatchToolType::Prose)
    else {
        panic!("expected freeform tool");
    };

    assert_eq!(tool.format.definition, None);
}
