use crate::TruncationPolicy;
use crate::approx_token_count;
use crate::approx_tokens_from_byte_count_i64;
use crate::formatted_truncate_text;
use crate::formatted_truncate_text_content_items_with_policy;
use crate::never_worse;
use crate::truncate_function_output_items_with_policy;
use crate::truncate_text;
use codex_protocol::models::DEFAULT_IMAGE_DETAIL;
use codex_protocol::models::FunctionCallOutputContentItem;
use pretty_assertions::assert_eq;

// A budget far under the frame is where the old behaviour was worst: fourteen bytes came back as
// ninety-six, seven times what was asked for, to say that thirteen characters had been saved.
// Both budgets are still exceeded by returning the content - fourteen bytes against a budget of
// one - but by a seventh as much as announcing the cut would have.

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
            image_url: "img:mid".to_string(),
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
            image_url: "img:mid".to_string(),
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
            image_url: "img:one".to_string(),
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
fn formatted_truncate_text_content_items_with_policy_returns_original_under_limit() {
    let items = vec![
        FunctionCallOutputContentItem::InputText {
            text: "alpha".to_string(),
        },
        FunctionCallOutputContentItem::InputText {
            text: String::new(),
        },
        FunctionCallOutputContentItem::InputText {
            text: "beta".to_string(),
        },
    ];

    let (output, original_token_count) =
        formatted_truncate_text_content_items_with_policy(&items, TruncationPolicy::Bytes(32));

    assert_eq!(output, items);
    assert_eq!(original_token_count, None);
}

#[test]
fn formatted_truncate_text_content_items_with_policy_preserves_empty_leading_text_behavior() {
    let items = vec![
        FunctionCallOutputContentItem::InputText {
            text: String::new(),
        },
        FunctionCallOutputContentItem::InputText {
            text: "abc".to_string(),
        },
    ];

    let (output, original_token_count) =
        formatted_truncate_text_content_items_with_policy(&items, TruncationPolicy::Bytes(0));

    // The merge is what this test is about, and it is unchanged: the empty leading item
    // contributes no separator. The text is now the merged content rather than a frame around a
    // cut of it, because at three bytes the frame costs thirty times what it announces.
    assert_eq!(
        output,
        vec![FunctionCallOutputContentItem::InputText {
            text: "abc".to_string(),
        }]
    );
    assert_eq!(original_token_count, Some(1));
}

#[test]
fn formatted_truncate_text_content_items_with_policy_merges_text_and_appends_media() {
    let items = vec![
        FunctionCallOutputContentItem::InputText {
            text: "abcd".to_string(),
        },
        FunctionCallOutputContentItem::InputImage {
            image_url: "img:one".to_string(),
            detail: Some(DEFAULT_IMAGE_DETAIL),
        },
        FunctionCallOutputContentItem::InputText {
            text: "efgh".to_string(),
        },
        FunctionCallOutputContentItem::InputAudio {
            audio_url: "audio:one".to_string(),
        },
        FunctionCallOutputContentItem::InputText {
            text: "ijkl".to_string(),
        },
        FunctionCallOutputContentItem::InputImage {
            image_url: "img:two".to_string(),
            detail: Some(DEFAULT_IMAGE_DETAIL),
        },
    ];

    let (output, original_token_count) =
        formatted_truncate_text_content_items_with_policy(&items, TruncationPolicy::Bytes(8));

    assert_eq!(
        output,
        vec![
            // Every text item merged, in order, ahead of the media that follows them.
            FunctionCallOutputContentItem::InputText {
                text: "abcd\nefgh\nijkl".to_string(),
            },
            FunctionCallOutputContentItem::InputImage {
                image_url: "img:one".to_string(),
                detail: Some(DEFAULT_IMAGE_DETAIL),
            },
            FunctionCallOutputContentItem::InputAudio {
                audio_url: "audio:one".to_string(),
            },
            FunctionCallOutputContentItem::InputImage {
                image_url: "img:two".to_string(),
                detail: Some(DEFAULT_IMAGE_DETAIL),
            },
        ]
    );
    assert_eq!(original_token_count, Some(4));
}

#[test]
fn formatted_truncate_text_content_items_with_policy_preserves_encrypted_content() {
    let items = vec![
        FunctionCallOutputContentItem::InputText {
            text: "abcdefgh".to_string(),
        },
        FunctionCallOutputContentItem::EncryptedContent {
            encrypted_content: "enc_opaque".to_string(),
        },
    ];

    let (output, original_token_count) =
        formatted_truncate_text_content_items_with_policy(&items, TruncationPolicy::Bytes(2));

    assert_eq!(
        output,
        vec![
            FunctionCallOutputContentItem::InputText {
                text: "abcdefgh".to_string(),
            },
            // The point of the test: opaque content survives the text path untouched.
            FunctionCallOutputContentItem::EncryptedContent {
                encrypted_content: "enc_opaque".to_string(),
            },
        ]
    );
    assert_eq!(original_token_count, Some(2));
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
fn formatted_truncate_text_content_items_with_policy_merges_all_text_for_token_budget() {
    let items = vec![
        FunctionCallOutputContentItem::InputText {
            text: "abcdefgh".to_string(),
        },
        FunctionCallOutputContentItem::InputText {
            text: "ijklmnop".to_string(),
        },
    ];

    let (output, original_token_count) =
        formatted_truncate_text_content_items_with_policy(&items, TruncationPolicy::Tokens(2));

    assert_eq!(
        output,
        vec![FunctionCallOutputContentItem::InputText {
            text: "abcdefgh\nijklmnop".to_string(),
        }]
    );
    assert_eq!(original_token_count, Some(5));
}

#[test]
fn byte_count_conversion_clamps_non_positive_values() {
    assert_eq!(approx_tokens_from_byte_count_i64(/*bytes*/ -1), 0);
    assert_eq!(approx_tokens_from_byte_count_i64(/*bytes*/ 0), 0);
    assert_eq!(approx_tokens_from_byte_count_i64(/*bytes*/ 5), 2);
}
