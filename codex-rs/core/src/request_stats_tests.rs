use super::*;
use pretty_assertions::assert_eq;

const TURN: &str = "turn-1";

fn usage() -> TokenUsage {
    TokenUsage {
        input_tokens: 60_000,
        cached_input_tokens: 56_000,
        output_tokens: 400,
        reasoning_output_tokens: 120,
        total_tokens: 60_400,
        ..TokenUsage::default()
    }
}

#[test]
fn a_request_is_priced_against_what_the_filters_removed() {
    let mut report = ContextFilterReport {
        prefix_break: Some(118),
        prefix_break_tokens: 4_200,
        dropped_reasoning_items: 3,
        dropped_reasoning_tokens: 1_204,
        retained_reasoning_items: 1,
        retained_reasoning_tokens: 300,
        ..ContextFilterReport::default()
    };
    let armed = ArmedRequest {
        started_at: Some(Instant::now()),
        report,
        prompt_items: 214,
        prompt_tokens_estimated: 62_100,
        prompt_tool_specs: 9,
        response_id: Some("resp-1".to_string()),
        tool_calls: 2,
    };

    let line = line(
        /*sequence*/ 41,
        TURN,
        /*invisible*/ false,
        Some(armed),
        /*retention_horizon*/ Some(160),
        &usage(),
    );

    // The whole point of the row: the prompt an unoptimised harness would have sent is the one
    // that was sent plus everything the report says was left out of it.
    assert_eq!(
        line.prompt_tokens_estimated.unwrap_or_default()
            + line.dropped_reasoning_tokens
            + line.dropped_invisible_tokens,
        62_100 + 1_204
    );
    assert_eq!(line.prefix_break, Some(118));
    assert_eq!(line.prefix_break_tokens, 4_200);
    assert_eq!(line.retained_reasoning_tokens, 300);
    assert_eq!(line.retention_horizon, Some(160));
    assert_eq!(line.tool_calls, 2);
    assert_eq!(line.response_id.as_deref(), Some("resp-1"));
    assert_eq!(line.input_tokens, 60_000);
    assert_eq!(line.cached_input_tokens, 56_000);
}

#[test]
fn a_completion_with_no_armed_context_still_records_its_usage() {
    // Compaction builds its own prompt and never arms. Its bill is still part of the session's,
    // so the row has to exist; what it must not do is report a prompt that was never measured.
    let line = line(
        /*sequence*/ 0,
        TURN,
        /*invisible*/ true,
        /*armed*/ None,
        /*retention_horizon*/ None,
        &usage(),
    );

    assert_eq!(line.input_tokens, 60_000);
    assert_eq!(line.total_tokens, 60_400);
    assert_eq!(line.invisible, true);
    assert_eq!(line.prompt_items, None);
    assert_eq!(line.prompt_tokens_estimated, None);
    assert_eq!(line.prompt_tool_specs, None);
    assert_eq!(line.prefix_break, None);
    assert_eq!(line.duration_ms, 0);
}
