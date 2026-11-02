use crate::OutputBoundary;
use crate::TruncationPolicy;
use crate::approx_token_count;
use crate::approx_tokens_from_byte_count_i64;
use crate::formatted_output_at_token_boundaries;
use crate::formatted_truncate_function_output_items_with_policy;
use crate::formatted_truncate_text;
use crate::mixed_output_at_token_boundaries;
use crate::never_worse;
use crate::truncate_function_output_items_with_policy;
use crate::truncate_mcp_tool_result;
use crate::truncate_text;
use codex_protocol::mcp::CallToolResult;
use codex_protocol::models::DEFAULT_IMAGE_DETAIL;
use codex_protocol::models::FunctionCallOutputContentItem;
use codex_protocol::models::ImageReference;
use pretty_assertions::assert_eq;

// A budget far under the frame is where the old behaviour was worst: fourteen bytes came back as
// ninety-six, seven times what was asked for, to say that thirteen characters had been saved.
// Both budgets are still exceeded by returning the content - fourteen bytes against a budget of
// one - but by a seventh as much as announcing the cut would have.

#[test]
fn truncate_mcp_tool_result_borrows_small_result() {
    let result = CallToolResult {
        content: vec![serde_json::json!({
            "type": "text",
            "text": "hello",
        })],
        structured_content: Some(serde_json::json!({"x": 1})),
        is_error: Some(false),
        meta: Some(serde_json::json!({"k": "v"})),
    };

    let truncated = truncate_mcp_tool_result(&result, /*max_bytes*/ 1024);

    assert!(matches!(truncated, std::borrow::Cow::Borrowed(_)));
    assert_eq!(truncated.as_ref(), &result);
}

#[test]
fn truncate_mcp_tool_result_compacts_large_result() {
    let max_bytes = 1024;
    let result = CallToolResult {
        content: vec![serde_json::json!({
            "type": "text",
            "text": format!("head\n{}\ntail", "\"\\\n".repeat(5_000)),
        })],
        structured_content: Some(serde_json::json!({"x": "y".repeat(5_000)})),
        is_error: Some(true),
        meta: Some(serde_json::json!({
            "private": format!("meta-head{}meta-tail", "z".repeat(5_000)),
        })),
    };

    let truncated = truncate_mcp_tool_result(&result, max_bytes).into_owned();

    let preview = truncated.content[0]["text"]
        .as_str()
        .expect("truncated result should contain a text preview");
    assert_eq!(
        truncated,
        CallToolResult {
            content: vec![serde_json::json!({
                "type": "text",
                "text": preview,
            })],
            structured_content: None,
            is_error: Some(true),
            meta: None,
        }
    );
    assert!(serde_json::to_string(&truncated).unwrap().len() <= max_bytes);
    assert!(preview.contains("head"));
    assert!(preview.contains("chars truncated"));
    assert!(preview.contains("meta-tail"));

    let truncated_again = truncate_mcp_tool_result(&truncated, max_bytes);
    assert!(matches!(truncated_again, std::borrow::Cow::Borrowed(_)));
    assert_eq!(truncated_again.as_ref(), &truncated);
}

#[test]
fn a_budget_under_the_frame_returns_the_content_rather_than_growing_it() {
    let content = "example output";

    assert_eq!(
        content,
        formatted_truncate_text(content, TruncationPolicy::Bytes(1)),
    );
}

#[test]
fn a_token_budget_under_the_frame_returns_the_content_rather_than_growing_it() {
    let content = "example output";

    assert_eq!(
        content,
        formatted_truncate_text(content, TruncationPolicy::Tokens(1)),
    );
}

#[test]
fn truncate_tokens_under_limit_returns_original() {
    let content = "example output";

    assert_eq!(
        content,
        formatted_truncate_text(content, TruncationPolicy::Tokens(10)),
    );
}

#[test]
fn truncate_bytes_under_limit_returns_original() {
    let content = "example output";

    assert_eq!(
        content,
        formatted_truncate_text(content, TruncationPolicy::Bytes(20)),
    );
}

// The content below is ten times the sentence these tests used to carry. At one sentence the
// frame cost more than the cut saved, so the guard now returns the content and there would be
// nothing left to assert about framing. Ten sentences is where truncating is the right answer,
// which is the case these tests were always about.

#[test]
fn truncate_tokens_over_limit_returns_truncated() {
    let content = "this is an example of a long output that should be truncated\n".repeat(10);

    let out = formatted_truncate_text(&content, TruncationPolicy::Tokens(5));

    assert!(
        out.starts_with("Warning: truncated output (original token count: "),
        "{out}"
    );
    assert!(out.contains("tokens truncated…"), "{out}");
    assert!(
        out.len() < content.len(),
        "the frame has to earn its bytes: {} against {}",
        out.len(),
        content.len()
    );
}

#[test]
fn truncate_bytes_over_limit_returns_truncated() {
    let content = "this is an example of a long output that should be truncated\n".repeat(10);

    let out = formatted_truncate_text(&content, TruncationPolicy::Bytes(30));

    assert!(
        out.starts_with("Warning: truncated output (original token count: "),
        "{out}"
    );
    assert!(out.contains("chars truncated…"), "{out}");
    assert!(
        out.len() < content.len(),
        "the frame has to earn its bytes: {} against {}",
        out.len(),
        content.len()
    );
}

#[test]
fn truncate_bytes_reports_original_line_count_when_truncated() {
    let content =
        "this is an example of a long output that should be truncated\nalso some other line\n"
            .repeat(10);

    let out = formatted_truncate_text(&content, TruncationPolicy::Bytes(30));

    // Twenty, not two: the count is of the whole input, which is the point of reporting it.
    assert!(out.contains("\nTotal output lines: 20\n\n"), "{out}");
    assert!(out.len() < content.len(), "{out}");
}

#[test]
fn truncate_tokens_reports_original_line_count_when_truncated() {
    let content =
        "this is an example of a long output that should be truncated\nalso some other line\n"
            .repeat(10);

    let out = formatted_truncate_text(&content, TruncationPolicy::Tokens(10));

    assert!(out.contains("\nTotal output lines: 20\n\n"), "{out}");
    assert!(out.len() < content.len(), "{out}");
}

#[test]
fn the_guard_keeps_whichever_rendering_is_smaller() {
    assert_eq!(never_worse("a".repeat(400).as_str(), "ok"), "ok");
    // A tie goes to the filtered text: it is the one the caller went to the trouble of building.
    assert_eq!(never_worse("abcd", "wxyz"), "wxyz");
    assert_eq!(never_worse("{}", "{\n  \"pretty\": true\n}"), "{}");
}

#[test]
fn an_input_a_byte_over_budget_does_not_come_back_bigger() {
    // The band the guard exists for. Without it the frame turns a 1-byte saving into a ~105-byte
    // loss, and every budget in the corpus has such a band immediately above it.
    for over in 1..=8 {
        let budget = 64;
        let content = "x".repeat(budget + over);

        let out = formatted_truncate_text(&content, TruncationPolicy::Bytes(budget));

        assert!(
            out.len() <= content.len(),
            "{} bytes over budget grew to {} from {}",
            over,
            out.len(),
            content.len()
        );
    }
}

#[test]
fn truncate_middle_bytes_handles_utf8_content() {
    let s = "😀😀😀😀😀😀😀😀😀😀\nsecond line with text\n";
    let out = truncate_text(s, TruncationPolicy::Bytes(20));
    assert_eq!(out, "😀😀…21 chars truncated…with text\n");
}

#[test]
fn truncates_across_multiple_under_limit_texts_and_reports_omitted() {
    let chunk = "alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu nu xi omicron pi rho sigma tau upsilon phi chi psi omega.\n";
    let chunk_tokens = approx_token_count(chunk);
    assert!(chunk_tokens > 0, "chunk must consume tokens");
    let limit = chunk_tokens * 3;
    let t1 = chunk.to_string();
    let t2 = chunk.to_string();
    let t3 = chunk.repeat(10);
    let t4 = chunk.to_string();
    let t5 = chunk.to_string();

    let items = vec![
        FunctionCallOutputContentItem::InputText { text: t1.clone() },
        FunctionCallOutputContentItem::InputText { text: t2.clone() },
        FunctionCallOutputContentItem::InputImage {
            image: ImageReference::Inline {
                image_url: "img:mid".to_string(),
            },
            detail: Some(DEFAULT_IMAGE_DETAIL),
        },
        FunctionCallOutputContentItem::InputText { text: t3 },
        FunctionCallOutputContentItem::InputText { text: t4 },
        FunctionCallOutputContentItem::InputText { text: t5 },
    ];

    let output =
        truncate_function_output_items_with_policy(&items, TruncationPolicy::Tokens(limit), |_| 0);

    assert_eq!(output.len(), 5);

    let first_text = match &output[0] {
        FunctionCallOutputContentItem::InputText { text } => text,
        other => panic!("unexpected first item: {other:?}"),
    };
    assert_eq!(first_text, &t1);

    let second_text = match &output[1] {
        FunctionCallOutputContentItem::InputText { text } => text,
        other => panic!("unexpected second item: {other:?}"),
    };
    assert_eq!(second_text, &t2);

    assert_eq!(
        output[2],
        FunctionCallOutputContentItem::InputImage {
            image: ImageReference::Inline {
                image_url: "img:mid".to_string()
            },
            detail: Some(DEFAULT_IMAGE_DETAIL),
        }
    );

    let fourth_text = match &output[3] {
        FunctionCallOutputContentItem::InputText { text } => text,
        other => panic!("unexpected fourth item: {other:?}"),
    };
    assert!(
        fourth_text.contains("tokens truncated"),
        "expected marker in truncated snippet: {fourth_text}"
    );

    let summary_text = match &output[4] {
        FunctionCallOutputContentItem::InputText { text } => text,
        other => panic!("unexpected summary item: {other:?}"),
    };
    assert!(summary_text.contains("omitted 2 text items"));
}

#[test]
fn truncate_function_output_items_with_policy_discards_empty_text() {
    let mut items = vec![
        FunctionCallOutputContentItem::InputText {
            text: String::new(),
        };
        16_384
    ];
    for policy in [TruncationPolicy::Bytes(0), TruncationPolicy::Tokens(1)] {
        assert_eq!(
            truncate_function_output_items_with_policy(&items, policy, |_| 0),
            Vec::new()
        );
    }

    let content = vec![
        FunctionCallOutputContentItem::InputText {
            text: "caption".to_string(),
        },
        FunctionCallOutputContentItem::InputImage {
            image: ImageReference::Inline {
                image_url: "img:one".to_string(),
            },
            detail: Some(DEFAULT_IMAGE_DETAIL),
        },
        FunctionCallOutputContentItem::InputImage {
            image: ImageReference::File {
                file_id: "file_123".to_string(),
            },
            detail: Some(DEFAULT_IMAGE_DETAIL),
        },
        FunctionCallOutputContentItem::InputAudio {
            audio_url: "audio:one".to_string(),
        },
        FunctionCallOutputContentItem::EncryptedContent {
            encrypted_content: "enc_opaque".to_string(),
        },
    ];
    items.extend(content.clone());
    for policy in [TruncationPolicy::Bytes(16), TruncationPolicy::Tokens(4)] {
        assert_eq!(
            truncate_function_output_items_with_policy(&items, policy, |_| 1),
            content
        );
    }
}

#[test]
fn formatted_text_output_reuses_owned_items_under_limit() {
    let items = vec![text("alpha"), text(""), text("beta")];
    let policy = TruncationPolicy::Bytes(32);

    let borrowed = formatted_truncate_function_output_items_with_policy(&items, policy, |_| 0);
    assert_eq!(borrowed, items);

    let allocation = items.as_ptr();
    let owned = formatted_truncate_function_output_items_with_policy(items, policy, |_| 0);
    assert_eq!(owned, borrowed);
    assert_eq!(owned.as_ptr(), allocation);
}

#[test]
fn formatted_text_output_preserves_empty_leading_text_behavior() {
    let items = vec![text(""), text("abc")];

    let output = formatted_truncate_function_output_items_with_policy(
        items,
        TruncationPolicy::Bytes(0),
        |_| 0,
    );

    // The merge is what this test is about, and it is unchanged: the empty leading item
    // contributes no separator. The text is now the merged content rather than a frame around a
    // cut of it, because at three bytes the frame costs thirty times what it announces.
    assert_eq!(
        output,
        vec![FunctionCallOutputContentItem::InputText {
            text: "abc".to_string(),
        }]
    );
}

#[test]
fn formatted_mixed_output_budgets_text_and_audio_in_order() {
    let audio = FunctionCallOutputContentItem::InputAudio {
        audio_url: "audio:one".to_owned(),
    };
    let file_image = FunctionCallOutputContentItem::InputImage {
        image: ImageReference::File {
            file_id: "file_123".to_owned(),
        },
        detail: Some(DEFAULT_IMAGE_DETAIL),
    };
    let items = vec![
        text("abcd"),
        inline_image("img:one".to_owned()),
        text("efgh"),
        audio.clone(),
        file_image.clone(),
        text("ijkl"),
        inline_image("img:two".to_owned()),
    ];

    let output = formatted_truncate_function_output_items_with_policy(
        items,
        TruncationPolicy::Bytes(12),
        |_| 1,
    );

    assert_eq!(
        output,
        vec![
            text("abcd"),
            inline_image("img:one".to_owned()),
            text("efgh"),
            audio,
            file_image,
            inline_image("img:two".to_owned()),
            text("[omitted 1 text items ...]"),
        ]
    );
}

#[test]
fn formatted_mixed_output_preserves_encrypted_content() {
    let encrypted = FunctionCallOutputContentItem::EncryptedContent {
        encrypted_content: "enc_opaque".to_owned(),
    };
    let items = vec![text("abcdefgh"), encrypted.clone()];

    let output = formatted_truncate_function_output_items_with_policy(
        items,
        TruncationPolicy::Bytes(2),
        |_| 0,
    );

    assert_eq!(output, vec![text("a…6 chars truncated…h"), encrypted]);
}

#[test]
fn truncate_function_output_items_with_policy_omits_audio_over_budget() {
    let items = vec![
        FunctionCallOutputContentItem::InputText {
            text: "abcdefgh".to_string(),
        },
        FunctionCallOutputContentItem::EncryptedContent {
            encrypted_content: "enc_opaque".to_string(),
        },
        FunctionCallOutputContentItem::InputAudio {
            audio_url: "audio:one".to_string(),
        },
    ];

    let output =
        truncate_function_output_items_with_policy(&items, TruncationPolicy::Bytes(2), |_| 1);

    assert_eq!(
        output,
        vec![
            FunctionCallOutputContentItem::InputText {
                text: "a…6 chars truncated…h".to_string(),
            },
            FunctionCallOutputContentItem::EncryptedContent {
                encrypted_content: "enc_opaque".to_string(),
            },
            FunctionCallOutputContentItem::InputText {
                text: "[omitted 1 audio items ...]".to_string(),
            },
        ]
    );
}

#[test]
fn truncate_function_output_items_with_policy_charges_audio_against_byte_budget() {
    let audio = FunctionCallOutputContentItem::InputAudio {
        audio_url: "audio:one".to_string(),
    };
    let items = vec![
        audio.clone(),
        FunctionCallOutputContentItem::InputText {
            text: "abcdefgh".to_string(),
        },
    ];

    let output =
        truncate_function_output_items_with_policy(&items, TruncationPolicy::Bytes(5), |_| 1);

    assert_eq!(
        output,
        vec![
            audio,
            FunctionCallOutputContentItem::InputText {
                text: truncate_text("abcdefgh", TruncationPolicy::Bytes(1)),
            },
        ]
    );
}

#[test]
fn formatted_text_output_merges_all_text_for_token_budget() {
    let items = vec![text("abcdefgh"), text("ijklmnop")];

    let output = formatted_truncate_function_output_items_with_policy(
        items,
        TruncationPolicy::Tokens(2),
        |_| 0,
    );

    assert_eq!(
        output,
        vec![FunctionCallOutputContentItem::InputText {
            text: "abcdefgh\nijklmnop".to_string(),
        }]
    );
}

#[test]
fn byte_count_conversion_clamps_non_positive_values() {
    assert_eq!(approx_tokens_from_byte_count_i64(/*bytes*/ -1), 0);
    assert_eq!(approx_tokens_from_byte_count_i64(/*bytes*/ 0), 0);
    assert_eq!(approx_tokens_from_byte_count_i64(/*bytes*/ 5), 2);
}

fn boundaries<const N: usize>(positions: [(usize, usize); N]) -> [OutputBoundary; N] {
    positions.map(|(item_index, text_offset)| OutputBoundary {
        item_index,
        text_offset,
    })
}

fn positions<const N: usize>(boundaries: [OutputBoundary; N]) -> [(usize, usize); N] {
    boundaries.map(|b| (b.item_index, b.text_offset))
}

fn text(s: &str) -> FunctionCallOutputContentItem {
    FunctionCallOutputContentItem::InputText { text: s.to_owned() }
}

fn inline_image(image_url: String) -> FunctionCallOutputContentItem {
    FunctionCallOutputContentItem::InputImage {
        image: ImageReference::Inline { image_url },
        detail: Some(DEFAULT_IMAGE_DETAIL),
    }
}

#[test]
fn boundary_formatters_normalize_input_and_remap_truncated_text() {
    let audio_cost: fn(&str) -> usize = |_| 1;
    // Long enough that the frame is cheaper than the text, or `never_worse` keeps the text whole.
    let left = format!("left{}", "-head".repeat(40));
    let right = format!("{}tail", "right-".repeat(40));
    let items = [text(&left), text(&right)];
    let max_tokens = 2;
    let mut boundary = boundaries([(0, 0), (0, 2), (1, 0), (1, right.len() - 2), (2, 0)]);
    let output =
        formatted_output_at_token_boundaries(&items, max_tokens, audio_cost, &mut boundary);
    let [FunctionCallOutputContentItem::InputText { text: rendered }] = output.as_slice() else {
        panic!("expected one merged text item")
    };
    let head = rendered.find("left").expect("expected retained prefix");
    let tail = rendered
        .strip_suffix("tail")
        .expect("expected retained suffix")
        .len();
    assert_eq!(
        positions(boundary),
        [
            (0, 0),
            (0, head + 2),
            (0, tail),
            (0, tail + 2),
            (0, rendered.len())
        ]
    );

    for (middle, middle_end) in [(text("after"), 5), (inline_image("img:one".to_owned()), 0)] {
        let items = [text("🙂é boundary"), middle, text("last")];
        for formatter in [
            formatted_output_at_token_boundaries,
            mixed_output_at_token_boundaries,
        ] {
            for tokens in [0, 1, 2, 256] {
                let mut b = boundaries([(0, 1), (1, usize::MAX), (0, 8), (usize::MAX, usize::MAX)]);
                let output = formatter(&items, tokens, audio_cost, &mut b);
                assert_eq!(output, formatter(&items, tokens, audio_cost, &mut []));
                let result = positions(b);
                assert!(result.is_sorted());
                assert!(
                    result
                        .iter()
                        .all(|&(index, offset)| match output.get(index) {
                            Some(FunctionCallOutputContentItem::InputText { text }) =>
                                text.is_char_boundary(offset),
                            _ => index <= output.len() && offset == 0,
                        })
                );
                if tokens == 256 {
                    assert_eq!(result, [(0, 0), (1, middle_end), (1, middle_end), (3, 0)]);
                }
            }
        }
    }
}

#[test]
fn owned_mixed_output_moves_a_preserved_image_across_text_truncation() {
    let image_url = "data:image/png;base64,".to_owned() + &"a".repeat(4096);
    let url_ptr = image_url.as_ptr();
    let items = vec![
        text("long prefix"),
        inline_image(image_url),
        text("omitted"),
    ];
    let mut borrowed = boundaries([(0, 5), (2, 3)]);
    let mut owned = boundaries([(0, 5), (2, 3)]);
    let max_tokens = 1;
    let expected = formatted_output_at_token_boundaries(&items, max_tokens, |_| 0, &mut borrowed);
    let actual = formatted_output_at_token_boundaries(items, max_tokens, |_| 0, &mut owned);
    assert_eq!(actual, expected);
    let projected = positions(owned);
    assert_eq!(projected, positions(borrowed));
    let [(index, offset), omitted] = projected;
    let FunctionCallOutputContentItem::InputText { text: snippet } = &actual[index] else {
        panic!("expected boundary inside truncated text");
    };
    assert_eq!((index, &snippet[offset..], omitted), (0, "ix", (2, 0)));
    let FunctionCallOutputContentItem::InputImage {
        image: ImageReference::Inline { image_url },
        ..
    } = &actual[1]
    else {
        panic!("expected preserved image")
    };
    assert_eq!(image_url.as_ptr(), url_ptr);
}
