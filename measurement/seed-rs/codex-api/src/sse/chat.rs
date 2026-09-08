//! Translation of OpenAI-compatible `/chat/completions` SSE frames into the
//! Responses-shaped [`ResponseEvent`] stream the rest of Suffice consumes.
//!
//! See `crate::requests::chat` for the outbound half of the bridge.

use crate::common::ResponseEvent;
use crate::common::ResponseStream;
use crate::error::ApiError;
use crate::telemetry::SseTelemetry;
use codex_client::StreamResponse;
use codex_protocol::models::ContentItem;
use codex_protocol::models::ReasoningItemContent;
use codex_protocol::models::ResponseItem;
use codex_protocol::protocol::TokenUsage;
use eventsource_stream::Eventsource;
use futures::Stream;
use futures::StreamExt;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::Instant;
use tokio::time::timeout;
use tracing::debug;
use tracing::trace;

const REQUEST_ID_HEADER: &str = "x-request-id";

pub(crate) fn spawn_chat_stream(
    stream_response: StreamResponse,
    idle_timeout: Duration,
    telemetry: Option<Arc<dyn SseTelemetry>>,
) -> ResponseStream {
    let upstream_request_id = stream_response
        .headers
        .get(REQUEST_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);

    let (tx_event, rx_event) = mpsc::channel::<Result<ResponseEvent, ApiError>>(1600);
    tokio::spawn(async move {
        process_chat_sse(stream_response.bytes, tx_event, idle_timeout, telemetry).await;
    });

    ResponseStream {
        rx_event,
        upstream_request_id,
    }
}

#[derive(Default, Debug)]
struct ToolCallState {
    id: Option<String>,
    name: Option<String>,
    arguments: String,
}

/// Accumulates the in-flight assistant turn while chunks arrive.
///
/// Chat Completions streams deltas with no item identity, so we rebuild whole
/// `ResponseItem`s here and emit them as `OutputItemDone` at the finish boundary.
#[derive(Default)]
struct ChatTurn {
    assistant: Option<ResponseItem>,
    reasoning: Option<ResponseItem>,
    tool_calls: HashMap<usize, ToolCallState>,
    tool_call_order: Vec<usize>,
    tool_call_index_by_id: HashMap<String, usize>,
    next_tool_call_index: usize,
    last_tool_call_index: Option<usize>,
    response_id: String,
    token_usage: Option<TokenUsage>,
    end_turn: Option<bool>,
}

pub(crate) async fn process_chat_sse<S>(
    stream: S,
    tx_event: mpsc::Sender<Result<ResponseEvent, ApiError>>,
    idle_timeout: Duration,
    telemetry: Option<Arc<dyn SseTelemetry>>,
) where
    S: Stream<Item = Result<bytes::Bytes, codex_client::TransportError>> + Unpin,
{
    let mut stream = stream.eventsource();
    let mut turn = ChatTurn::default();
    let _ = tx_event.send(Ok(ResponseEvent::Created)).await;

    loop {
        let start = Instant::now();
        let response = timeout(idle_timeout, stream.next()).await;
        if let Some(t) = telemetry.as_ref() {
            t.on_sse_poll(&response, start.elapsed());
        }

        let sse = match response {
            Ok(Some(Ok(sse))) => sse,
            Ok(Some(Err(e))) => {
                let _ = tx_event.send(Err(ApiError::Stream(e.to_string()))).await;
                return;
            }
            Ok(None) => {
                complete(&tx_event, &mut turn).await;
                return;
            }
            Err(_) => {
                let _ = tx_event
                    .send(Err(ApiError::Stream("idle timeout waiting for SSE".into())))
                    .await;
                return;
            }
        };

        trace!("chat SSE event: {}", sse.data);
        let data = sse.data.trim();
        if data.is_empty() {
            continue;
        }
        if data == "[DONE]" || data == "DONE" {
            complete(&tx_event, &mut turn).await;
            return;
        }

        let value: Value = match serde_json::from_str(data) {
            Ok(value) => value,
            Err(err) => {
                debug!("failed to parse chat completions SSE event: {err}, data: {data}");
                continue;
            }
        };

        if let Some(error) = value.get("error") {
            let message = error
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("chat completions stream returned an error");
            let _ = tx_event
                .send(Err(ApiError::Stream(message.to_string())))
                .await;
            return;
        }

        if let Some(id) = value.get("id").and_then(Value::as_str)
            && turn.response_id.is_empty()
        {
            turn.response_id = id.to_string();
        }

        // The final `include_usage` chunk carries usage with an empty `choices`.
        if let Some(usage) = value.get("usage").filter(|usage| !usage.is_null()) {
            turn.token_usage = Some(token_usage_from_chat_usage(usage));
        }

        let Some(choices) = value.get("choices").and_then(Value::as_array) else {
            continue;
        };

        for choice in choices {
            if let Some(delta) = choice.get("delta") {
                if let Some(text) = reasoning_text(delta) {
                    append_reasoning(&tx_event, &mut turn.reasoning, text).await;
                }
                if let Some(content) = delta.get("content") {
                    for text in content_texts(content) {
                        append_assistant(&tx_event, &mut turn.assistant, text).await;
                    }
                }
                if let Some(tool_calls) = delta.get("tool_calls").and_then(Value::as_array) {
                    for tool_call in tool_calls {
                        accumulate_tool_call(&mut turn, tool_call);
                    }
                }
            }

            // Non-streaming-shaped replies still land here for some providers.
            if let Some(message) = choice.get("message") {
                if let Some(text) = reasoning_text(message) {
                    append_reasoning(&tx_event, &mut turn.reasoning, text).await;
                }
                if let Some(content) = message.get("content") {
                    for text in content_texts(content) {
                        append_assistant(&tx_event, &mut turn.assistant, text).await;
                    }
                }
            }

            match choice.get("finish_reason").and_then(Value::as_str) {
                Some("length") => {
                    let _ = tx_event.send(Err(ApiError::ContextWindowExceeded)).await;
                    return;
                }
                Some("tool_calls") => {
                    turn.end_turn = Some(false);
                    flush_items(&tx_event, &mut turn).await;
                }
                Some(_) => {
                    turn.end_turn = Some(true);
                    flush_items(&tx_event, &mut turn).await;
                }
                None => {}
            }
        }
    }
}

/// Emits the accumulated reasoning, assistant message, and tool calls in the
/// order the Responses API would have produced them.
async fn flush_items(tx_event: &mpsc::Sender<Result<ResponseEvent, ApiError>>, turn: &mut ChatTurn) {
    if let Some(reasoning) = turn.reasoning.take() {
        let _ = tx_event
            .send(Ok(ResponseEvent::OutputItemDone(reasoning)))
            .await;
    }
    if let Some(assistant) = turn.assistant.take() {
        let _ = tx_event
            .send(Ok(ResponseEvent::OutputItemDone(assistant)))
            .await;
    }

    for index in std::mem::take(&mut turn.tool_call_order) {
        let Some(state) = turn.tool_calls.remove(&index) else {
            continue;
        };
        let Some(name) = state.name else {
            debug!("skipping tool call at index {index}: no function name in stream");
            continue;
        };
        let _ = tx_event
            .send(Ok(ResponseEvent::OutputItemDone(ResponseItem::FunctionCall {
                id: None,
                name,
                namespace: None,
                arguments: state.arguments,
                encrypted_function_args: None,
                call_id: state.id.unwrap_or_else(|| format!("tool-call-{index}")),
                internal_chat_message_metadata_passthrough: None,
            })))
            .await;
    }
}

/// Flushes anything still buffered and closes the turn.
///
/// Deferred until `[DONE]` (or stream end) because the usage chunk arrives after
/// `finish_reason`, and dropping it would leave Suffice blind to token cost.
async fn complete(tx_event: &mpsc::Sender<Result<ResponseEvent, ApiError>>, turn: &mut ChatTurn) {
    flush_items(tx_event, turn).await;
    let _ = tx_event
        .send(Ok(ResponseEvent::Completed {
            response_id: std::mem::take(&mut turn.response_id),
            token_usage: turn.token_usage.take(),
            usage_metadata: None,
            end_turn: turn.end_turn,
        }))
        .await;
}

fn accumulate_tool_call(turn: &mut ChatTurn, tool_call: &Value) {
    let call_id = tool_call.get("id").and_then(Value::as_str);

    // Prefer an index we already associated with this id; providers are
    // inconsistent about repeating `index` on continuation frames.
    let index = call_id
        .and_then(|id| turn.tool_call_index_by_id.get(id).copied())
        .or_else(|| {
            tool_call
                .get("index")
                .and_then(Value::as_u64)
                .map(|i| i as usize)
        })
        .or(if call_id.is_none() {
            turn.last_tool_call_index
        } else {
            None
        })
        .unwrap_or_else(|| {
            while turn.tool_calls.contains_key(&turn.next_tool_call_index) {
                turn.next_tool_call_index += 1;
            }
            let index = turn.next_tool_call_index;
            turn.next_tool_call_index += 1;
            index
        });

    if !turn.tool_calls.contains_key(&index) {
        turn.tool_call_order.push(index);
    }
    let state = turn.tool_calls.entry(index).or_default();

    if let Some(id) = call_id {
        state.id.get_or_insert_with(|| id.to_string());
        turn.tool_call_index_by_id
            .entry(id.to_string())
            .or_insert(index);
    }

    if let Some(function) = tool_call.get("function") {
        if let Some(name) = function.get("name").and_then(Value::as_str)
            && !name.is_empty()
        {
            state.name.get_or_insert_with(|| name.to_string());
        }
        if let Some(arguments) = function.get("arguments").and_then(Value::as_str) {
            state.arguments.push_str(arguments);
        }
    }

    turn.last_tool_call_index = Some(index);
}

/// Reads chain-of-thought text from a delta or message.
///
/// `reasoning_content` is the Z.ai/GLM spelling; `reasoning` is what other
/// OpenAI-compatible servers use, as a string or as `{text}`/`{content}`.
fn reasoning_text(node: &Value) -> Option<String> {
    if let Some(text) = node.get("reasoning_content").and_then(Value::as_str)
        && !text.is_empty()
    {
        return Some(text.to_string());
    }

    let reasoning = node.get("reasoning")?;
    let text = reasoning
        .as_str()
        .or_else(|| reasoning.get("text").and_then(Value::as_str))
        .or_else(|| reasoning.get("content").and_then(Value::as_str))?;
    (!text.is_empty()).then(|| text.to_string())
}

fn content_texts(content: &Value) -> Vec<String> {
    match content {
        Value::String(text) if !text.is_empty() => vec![text.clone()],
        Value::Array(items) => items
            .iter()
            .filter_map(|item| item.get("text").and_then(Value::as_str))
            .filter(|text| !text.is_empty())
            .map(str::to_string)
            .collect(),
        _ => Vec::new(),
    }
}

async fn append_assistant(
    tx_event: &mpsc::Sender<Result<ResponseEvent, ApiError>>,
    assistant: &mut Option<ResponseItem>,
    text: String,
) {
    if assistant.is_none() {
        let item = ResponseItem::Message {
            id: None,
            role: "assistant".to_string(),
            content: Vec::new(),
            phase: None,
            internal_chat_message_metadata_passthrough: None,
        };
        *assistant = Some(item.clone());
        let _ = tx_event
            .send(Ok(ResponseEvent::OutputItemAdded(item)))
            .await;
    }

    if let Some(ResponseItem::Message { content, .. }) = assistant {
        content.push(ContentItem::OutputText { text: text.clone() });
        let _ = tx_event
            .send(Ok(ResponseEvent::OutputTextDelta(text)))
            .await;
    }
}

async fn append_reasoning(
    tx_event: &mpsc::Sender<Result<ResponseEvent, ApiError>>,
    reasoning: &mut Option<ResponseItem>,
    text: String,
) {
    if reasoning.is_none() {
        let item = ResponseItem::Reasoning {
            id: None,
            summary: Vec::new(),
            content: Some(Vec::new()),
            encrypted_content: None,
            internal_chat_message_metadata_passthrough: None,
        };
        *reasoning = Some(item.clone());
        let _ = tx_event
            .send(Ok(ResponseEvent::OutputItemAdded(item)))
            .await;
    }

    if let Some(ResponseItem::Reasoning {
        content: Some(content),
        ..
    }) = reasoning
    {
        let content_index = content.len() as i64;
        content.push(ReasoningItemContent::ReasoningText { text: text.clone() });
        let _ = tx_event
            .send(Ok(ResponseEvent::ReasoningContentDelta {
                delta: text,
                content_index,
            }))
            .await;
    }
}

fn token_usage_from_chat_usage(usage: &Value) -> TokenUsage {
    let field = |name: &str| usage.get(name).and_then(Value::as_i64).unwrap_or(0);
    let nested = |parent: &str, name: &str| {
        usage
            .get(parent)
            .and_then(|details| details.get(name))
            .and_then(Value::as_i64)
            .unwrap_or(0)
    };

    let input_tokens = field("prompt_tokens");
    let output_tokens = field("completion_tokens");
    let total_tokens = match field("total_tokens") {
        0 => input_tokens + output_tokens,
        total => total,
    };

    TokenUsage {
        input_tokens,
        cached_input_tokens: nested("prompt_tokens_details", "cached_tokens"),
        cache_write_input_tokens: 0,
        output_tokens,
        reasoning_output_tokens: nested("completion_tokens_details", "reasoning_tokens"),
        total_tokens,
        codex_rollout_budget_units: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use codex_client::TransportError;
    use pretty_assertions::assert_eq;

    fn stream_of(frames: &[&str]) -> impl Stream<Item = Result<bytes::Bytes, TransportError>> + Unpin
    {
        let body: String = frames
            .iter()
            .map(|frame| format!("data: {frame}\n\n"))
            .collect();
        futures::stream::iter(vec![Ok(bytes::Bytes::from(body))])
    }

    async fn collect(frames: &[&str]) -> Vec<ResponseEvent> {
        let (tx, mut rx) = mpsc::channel(64);
        process_chat_sse(stream_of(frames), tx, Duration::from_secs(5), None).await;

        let mut events = Vec::new();
        while let Some(event) = rx.recv().await {
            events.push(event.expect("stream error"));
        }
        events
    }

    #[tokio::test]
    async fn assembles_glm_reasoning_text_and_usage_into_one_turn() {
        let events = collect(&[
            r#"{"id":"chatcmpl-1","choices":[{"delta":{"reasoning_content":"thinking"}}]}"#,
            r#"{"id":"chatcmpl-1","choices":[{"delta":{"content":"Hel"}}]}"#,
            r#"{"id":"chatcmpl-1","choices":[{"delta":{"content":"lo"},"finish_reason":"stop"}]}"#,
            r#"{"id":"chatcmpl-1","choices":[],"usage":{"prompt_tokens":10,"completion_tokens":4,"total_tokens":14,"prompt_tokens_details":{"cached_tokens":6}}}"#,
            "[DONE]",
        ])
        .await;

        let text: String = events
            .iter()
            .filter_map(|event| match event {
                ResponseEvent::OutputTextDelta(delta) => Some(delta.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(text, "Hello");

        assert!(
            events.iter().any(|event| matches!(
                event,
                ResponseEvent::ReasoningContentDelta { delta, .. } if delta == "thinking"
            )),
            "reasoning_content must surface as a reasoning delta"
        );

        let Some(ResponseEvent::Completed {
            response_id,
            token_usage,
            end_turn,
            ..
        }) = events.last()
        else {
            panic!("expected Completed last, got {:?}", events.last());
        };
        assert_eq!(response_id, "chatcmpl-1");
        assert_eq!(end_turn, &Some(true));
        let usage = token_usage.as_ref().expect("usage must survive finish_reason");
        assert_eq!(usage.input_tokens, 10);
        assert_eq!(usage.cached_input_tokens, 6);
        assert_eq!(usage.output_tokens, 4);
    }

    #[tokio::test]
    async fn reassembles_tool_calls_split_across_frames() {
        let events = collect(&[
            r#"{"id":"c","choices":[{"delta":{"tool_calls":[{"index":0,"id":"call-a","function":{"name":"shell","arguments":"{\"cmd\""}}]}}]}"#,
            r#"{"id":"c","choices":[{"delta":{"tool_calls":[{"index":0,"function":{"arguments":":\"ls\"}"}}]}}]}"#,
            r#"{"id":"c","choices":[{"delta":{},"finish_reason":"tool_calls"}]}"#,
            "[DONE]",
        ])
        .await;

        let calls: Vec<_> = events
            .iter()
            .filter_map(|event| match event {
                ResponseEvent::OutputItemDone(ResponseItem::FunctionCall {
                    name,
                    arguments,
                    call_id,
                    ..
                }) => Some((name.as_str(), arguments.as_str(), call_id.as_str())),
                _ => None,
            })
            .collect();

        assert_eq!(calls, vec![("shell", r#"{"cmd":"ls"}"#, "call-a")]);
        assert!(
            matches!(
                events.last(),
                Some(ResponseEvent::Completed { end_turn: Some(false), .. })
            ),
            "a tool_calls finish must not end the turn"
        );
    }
}
