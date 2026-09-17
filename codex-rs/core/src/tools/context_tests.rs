use super::*;
use codex_protocol::models::DEFAULT_IMAGE_DETAIL;
use codex_protocol::models::SearchToolCallParams;
use core_test_support::assert_regex_match;
use pretty_assertions::assert_eq;
use serde_json::json;

#[test]
fn custom_tool_calls_should_roundtrip_as_custom_outputs() {
    let payload = ToolPayload::Custom {
        input: "patch".to_string(),
    };
    let response = FunctionToolOutput::from_text("patched".to_string(), Some(true))
        .to_response_item("call-42", &payload);

    match response {
        ResponseInputItem::CustomToolCallOutput {
            call_id, output, ..
        } => {
            assert_eq!(call_id, "call-42");
            assert_eq!(output.content_items(), None);
            assert_eq!(output.body.to_text().as_deref(), Some("patched"));
            assert_eq!(output.success, Some(true));
        }
        other => panic!("expected CustomToolCallOutput, got {other:?}"),
    }
}

#[test]
fn function_payloads_remain_function_outputs() {
    let payload = ToolPayload::Function {
        arguments: "{}".to_string(),
    };
    let response = FunctionToolOutput::from_text("ok".to_string(), Some(true))
        .to_response_item("fn-1", &payload);

    match response {
        ResponseInputItem::FunctionCallOutput { call_id, output } => {
            assert_eq!(call_id, "fn-1");
            assert_eq!(output.content_items(), None);
            assert_eq!(output.body.to_text().as_deref(), Some("ok"));
            assert_eq!(output.success, Some(true));
        }
        other => panic!("expected FunctionCallOutput, got {other:?}"),
    }
}

#[test]
fn mcp_code_mode_result_omits_private_metadata() {
    let output = CallToolResult {
        content: vec![serde_json::json!({
            "type": "text",
            "text": "ignored",
        })],
        structured_content: Some(serde_json::json!({
            "threadId": "thread_123",
            "content": "done",
        })),
        is_error: Some(false),
        meta: Some(serde_json::json!({
            "source": "mcp",
        })),
    };

    let result = output.code_mode_result(&ToolPayload::Function {
        arguments: "{}".to_string(),
    });

    assert_eq!(
        result,
        serde_json::json!({
            "content": [{
                "type": "text",
                "text": "ignored",
            }],
            "structuredContent": {
                "threadId": "thread_123",
                "content": "done",
            },
            "isError": false,
        })
    );
    assert_eq!(output.meta, Some(serde_json::json!({ "source": "mcp" })));
}

#[test]
fn mcp_tool_output_response_item_includes_wall_time() {
    let output = McpToolOutput {
        result: CallToolResult {
            content: vec![serde_json::json!({
                "type": "text",
                "text": "done",
            })],
            structured_content: None,
            is_error: Some(false),
            meta: None,
        },
        tool_input: json!({}),
        wall_time: std::time::Duration::from_millis(1250),
        original_image_detail_supported: false,
        truncation_policy: TruncationPolicy::Bytes(1024),
    };

    let response = output.to_response_item(
        "mcp-call-1",
        &ToolPayload::Function {
            arguments: "{}".to_string(),
        },
    );

    assert_eq!(
        response,
        ResponseInputItem::FunctionCallOutput {
            call_id: "mcp-call-1".to_string(),
            output: FunctionCallOutputPayload {
                body: FunctionCallOutputBody::ContentItems(vec![
                    FunctionCallOutputContentItem::InputText {
                        text: "Wall time: 1.2500 seconds\nOutput:".to_string(),
                    },
                    FunctionCallOutputContentItem::InputText {
                        text: "done".to_string(),
                    },
                ]),
                success: Some(true),
            },
        }
    );
}

#[test]
fn mcp_tool_output_response_item_truncates_large_structured_content() {
    let output = McpToolOutput {
        result: CallToolResult {
            content: vec![serde_json::json!({
                "type": "text",
                "text": "ignored when structured content is present",
            })],
            structured_content: Some(serde_json::json!({
                "items": "large structured value ".repeat(1_000),
            })),
            is_error: Some(false),
            meta: None,
        },
        tool_input: json!({}),
        wall_time: std::time::Duration::from_millis(1250),
        original_image_detail_supported: false,
        truncation_policy: TruncationPolicy::Bytes(128),
    };

    assert_eq!(
        output.log_output(),
        format!(
            "Wall time: 1.2500 seconds\nOutput:\n{}",
            json!({"items": "large structured value ".repeat(1_000)})
        )
    );
    let response = output.to_response_item(
        "mcp-call-large",
        &ToolPayload::Function {
            arguments: "{}".to_string(),
        },
    );

    match response {
        ResponseInputItem::FunctionCallOutput { call_id, output } => {
            assert_eq!(call_id, "mcp-call-large");
            assert_eq!(output.success, Some(true));
            let text = output
                .body
                .to_text()
                .expect("MCP output should serialize as text");
            assert!(text.starts_with("Wall time: 1.2500 seconds\nOutput:\n"));
            assert!(text.contains("chars truncated"));
            assert!(!text.contains("ignored when structured content is present"));
        }
        other => panic!("expected FunctionCallOutput, got {other:?}"),
    }
}

#[test]
fn mcp_tool_output_response_item_preserves_content_items() {
    let image_url = "data:image/png;base64,AAA";
    let output = McpToolOutput {
        result: CallToolResult {
            content: vec![serde_json::json!({
                "type": "image",
                "mimeType": "image/png",
                "data": "AAA",
            })],
            structured_content: None,
            is_error: Some(false),
            meta: None,
        },
        tool_input: json!({}),
        wall_time: std::time::Duration::from_millis(500),
        original_image_detail_supported: false,
        truncation_policy: TruncationPolicy::Bytes(1024),
    };

    let response = output.to_response_item(
        "mcp-call-2",
        &ToolPayload::Function {
            arguments: "{}".to_string(),
        },
    );

    match response {
        ResponseInputItem::FunctionCallOutput { output, .. } => {
            assert_eq!(
                output.content_items(),
                Some(
                    vec![
                        FunctionCallOutputContentItem::InputText {
                            text: "Wall time: 0.5000 seconds\nOutput:".to_string(),
                        },
                        FunctionCallOutputContentItem::InputImage {
                            image_url: image_url.to_string(),
                            detail: Some(DEFAULT_IMAGE_DETAIL),
                        },
                    ]
                    .as_slice()
                )
            );
            assert_eq!(
                output.body.to_text().as_deref(),
                Some("Wall time: 0.5000 seconds\nOutput:")
            );
        }
        other => panic!("expected FunctionCallOutput, got {other:?}"),
    }
}

#[test_case::test_case(TruncationPolicy::Bytes(64); "byte budget")]
#[test_case::test_case(TruncationPolicy::Tokens(1); "token budget")]
fn mcp_tool_output_code_mode_result_preserves_content_without_private_metadata(
    truncation_policy: TruncationPolicy,
) {
    let large_content = "large structured value ".repeat(1_000);
    let output = McpToolOutput {
        result: CallToolResult {
            content: vec![serde_json::json!({
                "type": "text",
                "text": "ignored",
            })],
            structured_content: Some(serde_json::json!({
                "content": large_content,
            })),
            is_error: Some(false),
            meta: Some(serde_json::json!({
                "hive_dispatch_id": "private-dispatch-id",
            })),
        },
        tool_input: json!({}),
        wall_time: std::time::Duration::from_millis(1250),
        original_image_detail_supported: false,
        truncation_policy,
    };

    let payload = ToolPayload::Function {
        arguments: "{}".to_string(),
    };
    let result = output.code_mode_result(&payload);

    assert_eq!(
        result,
        serde_json::json!({
            "content": [{
                "type": "text",
                "text": "ignored",
            }],
            "structuredContent": {
                "content": "large structured value ".repeat(1_000),
            },
            "isError": false,
        })
    );
    assert_eq!(
        output.result.meta,
        Some(serde_json::json!({ "hive_dispatch_id": "private-dispatch-id" }))
    );
}

#[test]
fn custom_tool_calls_can_derive_text_from_content_items() {
    let payload = ToolPayload::Custom {
        input: "patch".to_string(),
    };
    let response = FunctionToolOutput::from_content(
        vec![
            FunctionCallOutputContentItem::InputText {
                text: "line 1".to_string(),
            },
            FunctionCallOutputContentItem::InputImage {
                image_url: "data:image/png;base64,AAA".to_string(),
                detail: Some(DEFAULT_IMAGE_DETAIL),
            },
            FunctionCallOutputContentItem::InputText {
                text: "line 2".to_string(),
            },
        ],
        Some(true),
    )
    .to_response_item("call-99", &payload);

    match response {
        ResponseInputItem::CustomToolCallOutput {
            call_id, output, ..
        } => {
            let expected = vec![
                FunctionCallOutputContentItem::InputText {
                    text: "line 1".to_string(),
                },
                FunctionCallOutputContentItem::InputImage {
                    image_url: "data:image/png;base64,AAA".to_string(),
                    detail: Some(DEFAULT_IMAGE_DETAIL),
                },
                FunctionCallOutputContentItem::InputText {
                    text: "line 2".to_string(),
                },
            ];
            assert_eq!(call_id, "call-99");
            assert_eq!(output.content_items(), Some(expected.as_slice()));
            assert_eq!(output.body.to_text().as_deref(), Some("line 1\nline 2"));
            assert_eq!(output.success, Some(true));
        }
        other => panic!("expected CustomToolCallOutput, got {other:?}"),
    }
}

#[test]
fn tool_search_payloads_roundtrip_as_tool_search_outputs() {
    let payload = ToolPayload::ToolSearch {
        arguments: SearchToolCallParams {
            query: "calendar".to_string(),
            limit: None,
        },
    };
    let response = ToolSearchOutput {
        tools: vec![LoadableToolSpec::Function(codex_tools::ResponsesApiTool {
            name: "create_event".to_string(),
            description: String::new(),
            strict: false,
            defer_loading: Some(true),
            parameters: codex_tools::JsonSchema::object(
                /*properties*/ Default::default(),
                /*required*/ None,
                /*additional_properties*/ None,
            ),
            output_schema: None,
        })],
    }
    .to_response_item("search-1", &payload);

    match response {
        ResponseInputItem::ToolSearchOutput {
            call_id,
            status,
            execution,
            tools,
        } => {
            assert_eq!(call_id, "search-1");
            assert_eq!(status, "completed");
            assert_eq!(execution, "client");
            assert_eq!(
                tools,
                vec![json!({
                    "type": "function",
                    "name": "create_event",
                    "description": "",
                    "strict": false,
                    "defer_loading": true,
                    "parameters": {
                        "type": "object",
                        "properties": {}
                    }
                })]
            );
        }
        other => panic!("expected ToolSearchOutput, got {other:?}"),
    }
}

#[test]
fn log_preview_uses_content_items_when_plain_text_is_missing() {
    let output = FunctionToolOutput::from_content(
        vec![FunctionCallOutputContentItem::InputText {
            text: "preview".to_string(),
        }],
        Some(true),
    );

    assert_eq!(output.log_output(), "preview");
    assert_eq!(
        function_call_output_content_items_to_text(&output.body),
        Some("preview".to_string())
    );
}

#[test]
fn exec_command_tool_output_formats_truncated_response() {
    let payload = ToolPayload::Function {
        arguments: "{}".to_string(),
    };
    let output = ExecCommandToolOutput {
        event_call_id: "call-42".to_string(),
        chunk_id: "abc123".to_string(),
        wall_time: std::time::Duration::from_millis(1250),
        raw_output: b"token one token two token three token four token five".to_vec(),
        truncation_policy: TruncationPolicy::Tokens(10_000),
        max_output_tokens: Some(4),
        process_id: None,
        exit_code: Some(0),
        original_token_count: Some(10),
        output_omitted_bytes: None,
        hook_command: None,
        spill_dir: None,
    };
    assert_eq!(
        output.log_output(),
        "Chunk ID: abc123\nWall time: 1.2500 seconds\nProcess exited with code 0\nOriginal token count: 10\nOutput:\ntoken one token two token three token four token five"
    );
    let response = output.to_response_item("call-42", &payload);

    match response {
        ResponseInputItem::FunctionCallOutput { call_id, output } => {
            assert_eq!(call_id, "call-42");
            assert_eq!(output.success, Some(true));
            let text = output
                .body
                .to_text()
                .expect("exec output should serialize as text");
            assert_regex_match(
                r#"(?sx)
                    ^Chunk\ ID:\ abc123
                    \nWall\ time:\ \d+\.\d{4}\ seconds
                    \nProcess\ exited\ with\ code\ 0
                    \nOriginal\ token\ count:\ 10
                    \nOutput:
                    \n.*tokens\ truncated.*
                    $"#,
                &text,
            );
        }
        other => panic!("expected FunctionCallOutput, got {other:?}"),
    }
}

#[test]
fn exec_command_tool_output_reserves_metadata_budget_and_preserves_policy_units() {
    let payload = ToolPayload::Function {
        arguments: "{}".to_string(),
    };
    let raw_output = (1..=150)
        .map(|line| format!("{line}\n"))
        .collect::<String>()
        .into_bytes();

    for (policy, marker) in [
        (TruncationPolicy::Bytes(200), "chars truncated"),
        (TruncationPolicy::Tokens(50), "tokens truncated"),
    ] {
        let response = ExecCommandToolOutput {
            event_call_id: "call-42".to_string(),
            chunk_id: "abc123".to_string(),
            wall_time: std::time::Duration::from_millis(/*millis*/ 1250),
            raw_output: raw_output.clone(),
            truncation_policy: policy,
            max_output_tokens: None,
            process_id: None,
            exit_code: Some(0),
            original_token_count: Some(123),
            output_omitted_bytes: None,
            hook_command: None,
            spill_dir: None,
        }
        .to_response_item("call-42", &payload);

        let ResponseInputItem::FunctionCallOutput { output, .. } = response else {
            panic!("expected FunctionCallOutput");
        };
        let text = output
            .body
            .to_text()
            .expect("exec output should serialize as text");

        assert!(text.len() <= (policy * 1.2).byte_budget());
        assert_eq!(text.matches(marker).count(), 1);
        assert!(text.contains("Original token count: 123"));
        assert!(text.contains("Total output lines: 150"));
        assert!(text.contains("\n1\n2\n3\n"));
        assert!(text.ends_with("149\n150\n"));
    }
}

#[test]
fn exec_command_tool_output_preserves_omission_metadata_when_truncated() {
    let payload = ToolPayload::Function {
        arguments: "{}".to_string(),
    };
    let marker = format_output_omission_marker(/*omitted_bytes*/ 123_456);
    let raw_output = format!(
        "HEAD-{}\n{marker}\nTAIL-{}",
        "a".repeat(/*n*/ 100),
        "z".repeat(/*n*/ 100)
    )
    .into_bytes();
    let mut output = ExecCommandToolOutput {
        event_call_id: "call-omitted".to_string(),
        chunk_id: "abc123".to_string(),
        wall_time: std::time::Duration::from_millis(/*millis*/ 1250),
        raw_output,
        truncation_policy: TruncationPolicy::Tokens(10_000),
        max_output_tokens: Some(4),
        process_id: None,
        exit_code: Some(0),
        original_token_count: Some(42_000),
        output_omitted_bytes: NonZeroUsize::new(/*n*/ 123_456),
        hook_command: None,
        spill_dir: None,
    };
    let expected_header = "Chunk ID: abc123\nWall time: 1.2500 seconds\nProcess exited with code 0\nOriginal token count: 42000\nOutput:\n";
    assert_eq!(
        output.log_output(),
        format!(
            "{expected_header}{}",
            String::from_utf8_lossy(&output.raw_output)
        )
    );
    let response = output.to_response_item("call-omitted", &payload);

    // Collection may report omitted bytes without including the marker in its text.
    output.raw_output = b"remaining output".to_vec();
    assert_eq!(
        output.log_output(),
        format!("{expected_header}{marker}\nremaining output")
    );

    let ResponseInputItem::FunctionCallOutput { output, .. } = response else {
        panic!("expected FunctionCallOutput");
    };
    let text = output
        .body
        .to_text()
        .expect("exec output should serialize as text");
    assert!(text.contains("Original token count: 42000"));
    assert!(text.contains("Warning: truncated output (original token count: 42000)"));
    assert_eq!(text.matches(&marker).count(), 1);
}

/// One exec result with `count` lines of output, ready to render.
fn exec_output_of_lines(count: usize, spill_dir: Option<PathBuf>) -> ExecCommandToolOutput {
    let raw = (1..=count)
        .map(|line| line.to_string())
        .collect::<Vec<_>>()
        .join("\n");
    ExecCommandToolOutput {
        event_call_id: "call-spill".to_string(),
        chunk_id: "sp1ll".to_string(),
        wall_time: std::time::Duration::from_millis(/*millis*/ 500),
        raw_output: raw.into_bytes(),
        truncation_policy: TruncationPolicy::Tokens(10_000),
        max_output_tokens: None,
        process_id: None,
        // Not on the shrink allowlist, so condensing leaves the text alone and the only thing
        // that can remove a line here is the budget - which is what these tests are about.
        exit_code: Some(0),
        original_token_count: None,
        output_omitted_bytes: None,
        hook_command: None,
        spill_dir,
    }
}

fn rendered(output: &ExecCommandToolOutput) -> String {
    output.response_text(&ToolPayload::Function {
        arguments: json!({ "cmd": "printf" }).to_string(),
    })
}

#[test]
fn output_the_budget_cuts_is_written_whole_and_the_header_names_it() {
    let dir = tempfile::tempdir().expect("tempdir");
    // 40k lines is far past the 12,000-byte budget a 10k-token policy allows.
    let output = exec_output_of_lines(/*count*/ 40_000, Some(dir.path().to_path_buf()));
    let text = rendered(&output);

    let notice = text
        .lines()
        .find_map(|line| line.strip_prefix("Full output saved to: "))
        .expect("the header names the file");
    let spilled = std::fs::read_to_string(notice).expect("the named file exists");

    // Byte for byte what the model would have been shown had nothing been cut. Not the raw
    // bytes: condensing runs first, and the file has to match what the truncated body is a
    // window into, or a `read` of it would not line up with what the model already has.
    assert_eq!(spilled, String::from_utf8_lossy(&output.raw_output));
    assert!(text.contains("Warning: truncated output"), "{text}");
    // The notice goes between the harness's own facts and the output it introduces.
    let notice_line = text.find("Full output saved to: ").expect("notice present");
    assert!(notice_line < text.find("Output:\n").expect("output marker"));
}

#[test]
fn output_that_fits_writes_no_file_and_reads_exactly_as_before() {
    let dir = tempfile::tempdir().expect("tempdir");
    let with_dir = exec_output_of_lines(/*count*/ 10, Some(dir.path().to_path_buf()));
    let without_dir = exec_output_of_lines(/*count*/ 10, None);

    assert_eq!(rendered(&with_dir), rendered(&without_dir));
    assert!(!rendered(&with_dir).contains("Full output saved to:"));
    assert_eq!(
        std::fs::read_dir(dir.path())
            .expect("tempdir readable")
            .count(),
        0,
        "nothing was cut, so nothing should have been written"
    );
}

#[test]
fn naming_the_file_does_not_push_the_response_past_its_budget() {
    let dir = tempfile::tempdir().expect("tempdir");
    let spilled = rendered(&exec_output_of_lines(
        /*count*/ 40_000,
        Some(dir.path().to_path_buf()),
    ));
    let bare = rendered(&exec_output_of_lines(/*count*/ 40_000, None));

    // The notice's bytes come out of the body, not out of thin air. Without the reservation in
    // `response_text` this response would be longer than the one that carries no notice, and
    // history would cut it a second time - the exact thing the budget exists to prevent.
    assert!(spilled.contains("Full output saved to: "), "{spilled}");
    assert!(
        spilled.len() <= bare.len(),
        "with notice {} > without {}",
        spilled.len(),
        bare.len()
    );
    assert!(spilled.len() <= MAX_EXEC_OUTPUT_BYTES);
}

#[test]
fn an_output_that_only_just_fits_is_not_cut_to_make_room_for_the_notice() {
    let dir = tempfile::tempdir().expect("tempdir");
    // The band that only just fits. `n` lines of ascending integers is `5n - 1108` bytes here,
    // and the budget is 40,000, so 8,221 lines is the last length that arrives whole and the
    // ~115 bytes below it are where reserving room for a notice would start cutting output that
    // needed no file - handing the model a path to its own tail.
    for lines in 8_190..=8_221 {
        let with_dir = exec_output_of_lines(lines, Some(dir.path().to_path_buf()));
        let without_dir = exec_output_of_lines(lines, None);
        assert_eq!(
            rendered(&with_dir),
            rendered(&without_dir),
            "a spill directory changed a response that was never going to be truncated \
             ({lines} lines)"
        );
    }
}

#[test]
fn a_directory_that_cannot_be_written_costs_the_notice_not_the_output() {
    let dir = tempfile::tempdir().expect("tempdir");
    // A file where the directory should be, so `create_dir_all` fails.
    let blocked = dir.path().join("blocked");
    std::fs::write(&blocked, b"not a directory").expect("write blocker");

    let text = rendered(&exec_output_of_lines(/*count*/ 40_000, Some(blocked)));

    assert!(!text.contains("Full output saved to:"), "{text}");
    assert!(text.contains("Warning: truncated output"), "{text}");
}

#[test]
fn a_budget_too_small_to_carry_the_notice_does_not_carry_it() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut output = exec_output_of_lines(/*count*/ 40_000, Some(dir.path().to_path_buf()));
    // ~200 bytes for the body. A ~90-byte path would be nearly half of it, which buys the model
    // a file by taking away the output the file was meant to make recoverable.
    output.max_output_tokens = Some(/*tokens*/ 50);

    let text = rendered(&output);

    assert!(!text.contains("Full output saved to:"), "{text}");
    assert_eq!(
        std::fs::read_dir(dir.path())
            .expect("tempdir readable")
            .count(),
        0,
        "no notice means no file: an unreferenced file is pure disk cost"
    );
}

#[test]
fn rendering_the_same_result_twice_leaves_one_file() {
    let dir = tempfile::tempdir().expect("tempdir");
    let output = exec_output_of_lines(/*count*/ 40_000, Some(dir.path().to_path_buf()));

    let first = rendered(&output);
    let second = rendered(&output);

    assert_eq!(first, second);
    assert_eq!(
        std::fs::read_dir(dir.path())
            .expect("tempdir readable")
            .count(),
        1,
        "the name comes from the call, not the clock"
    );
}

#[test]
fn code_mode_result_removes_noise_but_never_drops_content() {
    // This path used to serialize the raw bytes, so git's line-ending warnings reached the model
    // here after being filtered out everywhere else. They go now -- but only they: code mode
    // hands its result to a program the model wrote, which may count lines or match the text
    // exactly, and a stage that announces what it dropped is enough for a reader and not for a
    // parser. So the lossy stages stay off here even though the text path runs them.
    let payload = ToolPayload::Function {
        arguments: json!({ "cmd": "cargo build" }).to_string(),
    };
    let mut lines = vec![
        "warning: in the working copy of 'a.py', LF will be replaced by CRLF the next time Git touches it"
            .to_string(),
    ];
    lines.extend((0..200).map(|index| format!("   Compiling crate-{index}")));
    lines.push("Finished dev profile".to_string());
    let raw = lines.join("\n");

    let output = ExecCommandToolOutput {
        event_call_id: "call-7".to_string(),
        chunk_id: String::new(),
        wall_time: std::time::Duration::from_millis(500),
        raw_output: raw.into_bytes(),
        truncation_policy: TruncationPolicy::Tokens(100_000),
        max_output_tokens: None,
        process_id: None,
        exit_code: Some(0),
        original_token_count: None,
        output_omitted_bytes: None,
        hook_command: None,
        spill_dir: None,
    };

    let result = output.code_mode_result(&payload);
    let text = result["output"].as_str().expect("output serializes as a string");

    // The noise is gone.
    assert!(!text.contains("will be replaced by"), "{text}");
    // And nothing else is: `cargo build` is on the shrink allowlist, it exited 0, and 201 lines
    // is far over the threshold, so the text path would have cut the middle out of this.
    assert_eq!(text.lines().count(), 201);
    assert!(text.contains("   Compiling crate-100"), "{text}");
    assert!(!text.contains("lines trimmed"), "{text}");
    // Telemetry still records what the command actually printed.
    assert!(output.log_output().contains("will be replaced by"));
}
