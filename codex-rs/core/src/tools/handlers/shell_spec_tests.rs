use super::*;
use pretty_assertions::assert_eq;
use std::collections::BTreeMap;

fn windows_shell_guidance_description() -> String {
    format!("\n\n{}", windows_shell_guidance())
}

fn file_work_guidance_description() -> String {
    format!("\n\n{}", file_work_guidance())
}

fn has_parameter(tool: &ToolSpec, parameter_name: &str) -> bool {
    serde_json::to_value(tool)
        .expect("tool spec should serialize")
        .pointer(&format!("/parameters/properties/{parameter_name}"))
        .is_some()
}

/// Under `approval_policy = never` the model is offered three parameters, not ten.
///
/// This is the shape the measurement moved: replacing the whole spec with OpenCode's
/// three-parameter `bash` took reads from the whole file to 45% of it and the median window from
/// 220 lines to 80, six replays across two batches at 0.39-0.45 against a base at 0.85-1.00
/// (`_sim/probe.py`). Nothing smaller reproduced it, so the parameter list is not decoration and
/// must not quietly grow back.
#[test]
fn a_non_interactive_run_is_offered_three_parameters() {
    let tool = create_exec_command_tool_with_environment_id(
        CommandToolOptions {
            allow_login_shell: true,
            exec_permission_approvals_enabled: true,
            lean_parameters: true,
        },
        /*include_environment_id*/ false,
        /*include_shell_parameter*/ true,
        /*include_windows_shell_guidance*/ true,
    );

    for offered in ["cmd", "workdir", "timeout_ms"] {
        assert!(has_parameter(&tool, offered), "{offered} must be offered");
    }
    // Every one of these still deserializes - `ExecCommandArgs` defaults them - so dropping them
    // from the schema costs no behaviour, only the tokens and the invitation.
    for withheld in [
        "shell",
        "login",
        "tty",
        "yield_time_ms",
        "max_output_tokens",
        "sandbox_permissions",
        "additional_permissions",
        "justification",
        "prefix_rule",
    ] {
        assert!(
            !has_parameter(&tool, withheld),
            "{withheld} must not be offered to a run that cannot act on it"
        );
    }

    // `one_shot_exec_command_spec` rewrites the opening sentence by matching on it.
    let ToolSpec::Function(function) = &tool else {
        panic!("exec_command has a function schema");
    };
    assert!(
        function.description.starts_with(
            "Runs a command in a PTY, returning output or a session ID for ongoing interaction."
        ),
        "the one-shot rewrite anchors on this sentence"
    );
}

#[test]
fn exec_command_tool_matches_expected_spec() {
    let tool = create_exec_command_tool(CommandToolOptions {
        allow_login_shell: true,
        exec_permission_approvals_enabled: false,
        lean_parameters: false,
    });

    let description = if cfg!(windows) {
        format!(
            "Runs a command in a PTY, returning output or a session ID for ongoing interaction.{}{}",
            file_work_guidance_description(),
            windows_shell_guidance_description()
        )
    } else {
        format!(
            "Runs a command in a PTY, returning output or a session ID for ongoing interaction.{}",
            file_work_guidance_description()
        )
    };
    let yield_time_ms_description = if cfg!(windows) {
        "Maximum time to wait before returning a session ID for a still-running command. Commands that finish sooner return immediately. For ordinary commands, omit this parameter to use the 10000 ms default. Effective range on Windows is 10000-30000 ms."
    } else {
        "Wait before yielding output. Defaults to 10000 ms; effective range is 250-30000 ms."
    };

    let mut properties = BTreeMap::from([
        (
            "cmd".to_string(),
            JsonSchema::string(Some("Shell command to execute.".to_string())),
        ),
        (
            "workdir".to_string(),
            JsonSchema::string(Some(
                    "Working directory for the command. Defaults to the turn cwd."
                        .to_string(),
                )),
        ),
        (
            "shell".to_string(),
            JsonSchema::string(Some(
                    "Shell binary to launch. Defaults to the user's default shell.".to_string(),
                )),
        ),
        (
            "tty".to_string(),
            JsonSchema::boolean(Some(
                    "True allocates a PTY for the command; false or omitted uses plain pipes."
                        .to_string(),
                )),
        ),
        (
            "yield_time_ms".to_string(),
            JsonSchema::number(Some(yield_time_ms_description.to_string())),
        ),
        (
            "max_output_tokens".to_string(),
            JsonSchema::number(Some(
                    "Cap on this one command's output, in tokens. Defaults to 10000; larger requests may be capped by policy. It bounds this command only and says nothing about how much conversation context remains.".to_string(),
                )),
        ),
        (
            "login".to_string(),
            JsonSchema::boolean(Some(
                    "True runs the shell with -l/-i semantics; false disables them. Defaults to true.".to_string(),
                )),
        ),
    ]);
    properties.extend(create_approval_parameters(
        /*exec_permission_approvals_enabled*/ false,
    ));

    assert_eq!(
        tool,
        ToolSpec::Function(ResponsesApiTool {
            name: "exec_command".to_string(),
            description,
            strict: false,
            defer_loading: None,
            parameters: JsonSchema::object(
                properties,
                Some(vec!["cmd".to_string()]),
                Some(false.into())
            ),
            output_schema: Some(unified_exec_output_schema()),
        })
    );
}

#[test]
fn exec_command_tool_can_hide_shell_parameter() {
    let tool = create_exec_command_tool_with_environment_id(
        CommandToolOptions {
            allow_login_shell: true,
            exec_permission_approvals_enabled: false,
            lean_parameters: false,
        },
        /*include_environment_id*/ false,
        /*include_shell_parameter*/ false,
        /*include_windows_shell_guidance*/ cfg!(windows),
    );

    assert!(!has_parameter(&tool, "shell"));
    assert!(has_parameter(&tool, "cmd"));
}

#[test]
fn write_stdin_tool_matches_expected_spec() {
    let tool = create_write_stdin_tool();

    let properties = BTreeMap::from([
        (
            "session_id".to_string(),
            JsonSchema::number(Some(
                "Identifier of the running unified exec session.".to_string(),
            )),
        ),
        (
            "chars".to_string(),
            JsonSchema::string(Some(
                "Bytes to write to stdin. Defaults to empty, which polls without writing.".to_string(),
            )),
        ),
        (
            "yield_time_ms".to_string(),
            JsonSchema::number(Some(
                "Wait before yielding output. Non-empty writes default to 250 ms and cap at 30000 ms; empty polls wait 5000-300000 ms by default.".to_string(),
            )),
        ),
        (
            "max_output_tokens".to_string(),
            JsonSchema::number(Some(
                "Cap on this one command's output, in tokens. Defaults to 10000; larger requests may be capped by policy. It bounds this command only and says nothing about how much conversation context remains.".to_string(),
            )),
        ),
    ]);

    assert_eq!(
        tool,
        ToolSpec::Function(ResponsesApiTool {
            name: "write_stdin".to_string(),
            description:
                "Writes characters to an existing unified exec session and returns recent output."
                    .to_string(),
            strict: false,
            defer_loading: None,
            parameters: JsonSchema::object(
                properties,
                Some(vec!["session_id".to_string()]),
                Some(false.into())
            ),
            output_schema: Some(unified_exec_output_schema()),
        })
    );
}

#[test]
fn request_permissions_tool_includes_full_permission_schema() {
    let tool =
        create_request_permissions_tool("Request extra permissions for this turn.".to_string());

    let properties = BTreeMap::from([
        (
            "reason".to_string(),
            JsonSchema::string(Some(
                "Optional short explanation for why additional permissions are needed.".to_string(),
            )),
        ),
        (
            "environment_id".to_string(),
            JsonSchema::string(Some(
                "Environment id from <environment_context>. Omit to use the primary environment."
                    .to_string(),
            )),
        ),
        ("permissions".to_string(), permission_profile_schema()),
    ]);

    assert_eq!(
        tool,
        ToolSpec::Function(ResponsesApiTool {
            name: "request_permissions".to_string(),
            description: "Request extra permissions for this turn.".to_string(),
            strict: false,
            defer_loading: None,
            parameters: JsonSchema::object(
                properties,
                Some(vec!["permissions".to_string()]),
                Some(false.into())
            ),
            output_schema: None,
        })
    );
}
