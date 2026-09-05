//! Translation from a Responses-shaped request into an OpenAI-compatible
//! `/chat/completions` body.
//!
//! Suffice speaks the Responses API everywhere above this module. Providers that
//! only expose Chat Completions (Z.ai/GLM, most OSS servers) get their request
//! rewritten here, and their SSE frames rewritten back in `crate::sse::chat`.

use crate::common::ResponsesApiRequest;
use codex_protocol::models::ContentItem;
use codex_protocol::models::FunctionCallOutputContentItem;
use codex_protocol::models::ReasoningItemContent;
use codex_protocol::models::ResponseItem;
use codex_protocol::openai_models::ReasoningEffort;
use serde_json::Value;
use serde_json::json;
use std::collections::HashMap;

/// Builds the `/chat/completions` request body for `request`.
pub(crate) fn chat_body_from_responses_request(request: &ResponsesApiRequest) -> Value {
    let mut messages = Vec::<Value>::new();
    if !request.instructions.is_empty() {
        messages.push(json!({"role": "system", "content": request.instructions}));
    }

    let input = request.input.as_slice();
    let reasoning_by_anchor_index = anchor_reasoning(input);
    let mut last_assistant_text: Option<String> = None;

    for (idx, item) in input.iter().enumerate() {
        match item {
            ResponseItem::Message { role, content, .. } => {
                let mut text = String::new();
                let mut parts: Vec<Value> = Vec::new();
                let mut saw_image = false;

                for c in content {
                    match c {
                        ContentItem::InputText { text: t }
                        | ContentItem::OutputText { text: t } => {
                            text.push_str(t);
                            parts.push(json!({"type": "text", "text": t}));
                        }
                        ContentItem::InputImage { image_url, .. } => {
                            saw_image = true;
                            parts.push(
                                json!({"type": "image_url", "image_url": {"url": image_url}}),
                            );
                        }
                        // Audio has no Chat Completions equivalent.
                        _ => {}
                    }
                }

                // Chat Completions has no notion of duplicate assistant turns;
                // collapsing them keeps the transcript well-formed.
                if role == "assistant" {
                    if last_assistant_text.as_ref() == Some(&text) {
                        continue;
                    }
                    last_assistant_text = Some(text.clone());
                }

                let content_value = if role != "assistant" && saw_image {
                    json!(parts)
                } else {
                    json!(text)
                };

                // Chat Completions has no `developer` role, and GLM rejects the
                // whole request with error 1214 rather than ignoring it. The
                // role was introduced as a rename of `system`, so map it back.
                let chat_role = if role == "developer" {
                    "system"
                } else {
                    role.as_str()
                };
                let mut msg = json!({"role": chat_role, "content": content_value});
                if role == "assistant"
                    && let Some(reasoning) = reasoning_by_anchor_index.get(&idx)
                    && let Some(obj) = msg.as_object_mut()
                {
                    obj.insert("reasoning_content".to_string(), json!(reasoning));
                }
                messages.push(msg);
            }
            ResponseItem::FunctionCall {
                name,
                arguments,
                call_id,
                ..
            } => {
                push_tool_call(
                    &mut messages,
                    json!({
                        "id": call_id,
                        "type": "function",
                        "function": {"name": name, "arguments": arguments},
                    }),
                    reasoning_by_anchor_index.get(&idx).map(String::as_str),
                );
            }
            ResponseItem::FunctionCallOutput {
                call_id, output, ..
            } => {
                let content_value = match output.content_items() {
                    Some(items) => json!(
                        items
                            .iter()
                            .filter_map(|it| match it {
                                FunctionCallOutputContentItem::InputText { text } => {
                                    Some(json!({"type": "text", "text": text}))
                                }
                                FunctionCallOutputContentItem::InputImage { image_url, .. } => {
                                    Some(json!({"type": "image_url", "image_url": {"url": image_url}}))
                                }
                                _ => None,
                            })
                            .collect::<Vec<_>>()
                    ),
                    None => json!(output.text_content().unwrap_or_default()),
                };
                messages.push(json!({
                    "role": "tool",
                    "tool_call_id": call_id,
                    "content": content_value,
                }));
            }
            ResponseItem::CustomToolCall {
                call_id,
                name,
                input,
                ..
            } => {
                // Replay it in the shape the tool was declared with, so the model's own
                // history keeps teaching it the calling convention rather than a second one.
                push_tool_call(
                    &mut messages,
                    json!({
                        "id": call_id,
                        "type": "function",
                        "function": {
                            "name": name,
                            "arguments": json!({"input": input}).to_string(),
                        },
                    }),
                    reasoning_by_anchor_index.get(&idx).map(String::as_str),
                );
            }
            ResponseItem::CustomToolCallOutput {
                call_id, output, ..
            } => {
                messages.push(json!({
                    "role": "tool",
                    "tool_call_id": call_id,
                    "content": output,
                }));
            }
            // Reasoning is folded into its anchor message above; everything else
            // has no Chat Completions representation.
            _ => continue,
        }
    }

    let mut body = json!({
        "model": request.model,
        "messages": messages,
        "stream": true,
        // Without this the provider omits `usage`, and Suffice cannot track cost
        // or drive the auto-compaction trigger.
        "stream_options": {"include_usage": true},
    });

    let Some(obj) = body.as_object_mut() else {
        return body;
    };

    if let Some(tools) = request.tools.as_ref().map(chat_tools_from_responses_tools)
        && !tools.is_empty()
    {
        obj.insert("tools".to_string(), json!(tools));
        obj.insert("tool_choice".to_string(), json!(request.tool_choice));
        obj.insert(
            "parallel_tool_calls".to_string(),
            json!(request.parallel_tool_calls),
        );
    }

    // Z.ai/GLM gate chain-of-thought behind `thinking` rather than `reasoning`.
    // GLM-5.3 rejects `{"type": "disabled"}` outright, so enabled is the only value
    // worth sending; depth is chosen with `reasoning_effort` instead.
    if let Some(reasoning) = request.reasoning.as_ref() {
        obj.insert("thinking".to_string(), json!({"type": "enabled"}));
        if let Some(effort) = reasoning.effort.as_ref() {
            obj.insert(
                "reasoning_effort".to_string(),
                json!(glm_reasoning_effort(effort)),
            );
        }
    }

    if let Some(text) = request.text.as_ref()
        && let Ok(Value::Object(text)) = serde_json::to_value(text)
        && let Some(format) = text.get("format")
    {
        obj.insert("response_format".to_string(), format.clone());
    }

    body
}

/// Projects Suffice's effort ladder onto the three rungs GLM-5.3 accepts.
///
/// `low`, `high` and `max` are the only legal values; anything else is rejected outright,
/// and omitting the field entirely silently selects `max`, the most expensive rung. So a
/// config still carrying an effort written for another provider must be clamped rather than
/// forwarded or dropped. Clamping at the wire boundary leaves the rest of Suffice free to keep
/// its own wider ladder.
fn glm_reasoning_effort(effort: &ReasoningEffort) -> &'static str {
    match effort {
        ReasoningEffort::None | ReasoningEffort::Minimal | ReasoningEffort::Low => "low",
        ReasoningEffort::Medium | ReasoningEffort::High => "high",
        ReasoningEffort::XHigh | ReasoningEffort::Max | ReasoningEffort::Ultra => "max",
        // Rungs with no depth ordering to project. `high` is the model default here, and the
        // cheaper of the two rungs the picker offers without an explicit detour.
        ReasoningEffort::Persistent | ReasoningEffort::Custom(_) => "high",
    }
}

/// Rewrites Responses tool specs (`{type, name, parameters}`) into the nested
/// Chat Completions shape (`{type, function: {name, parameters}}`).
/// Names of the freeform tools in `tools`, which reach the model as functions.
///
/// The SSE side needs these to turn the reply back into a `CustomToolCall`, which is the
/// shape every handler above this module already expects.
pub(crate) fn freeform_tool_names(tools: Option<&crate::common::ResponsesApiTools>) -> Vec<String> {
    let Some(tools) = tools else {
        return Vec::new();
    };
    let Ok(Value::Array(tools)) = serde_json::from_str::<Value>(tools.as_raw_value().get()) else {
        return Vec::new();
    };
    tools
        .iter()
        .filter(|tool| tool.get("type").and_then(Value::as_str) == Some("custom"))
        .filter_map(|tool| tool.get("name").and_then(Value::as_str))
        .map(str::to_string)
        .collect()
}

/// Rewrites a freeform (`"type": "custom"`) tool as an ordinary function taking the body
/// as a single string.
///
/// The Responses API carries these with a grammar that constrains decoding; Chat
/// Completions has no equivalent, and Z.ai accepts only `web_search`, `retrieval` and
/// `function`. Dropping the tool instead leaves the model with no way to call it at all,
/// which is how `apply_patch` came to be invoked through the shell.
fn chat_function_from_freeform(obj: &serde_json::Map<String, Value>) -> Option<Value> {
    let name = obj.get("name").and_then(Value::as_str)?;
    // The custom-tool description tells the model not to wrap the body in JSON, which is
    // exactly what this wire has to do. Drop that sentence and state the contract instead.
    let description = obj
        .get("description")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .split_inclusive('.')
        .filter(|sentence| !sentence.contains("FREEFORM"))
        .collect::<String>()
        .trim()
        .to_string();
    Some(json!({
        "type": "function",
        "function": {
            "name": name,
            "description": format!(
                "{description} Send the entire body as the `input` string; it is passed \
                 through verbatim, so do not add any wrapper of your own."
            ),
            "parameters": {
                "type": "object",
                "properties": {
                    "input": {
                        "type": "string",
                        "description": "The complete tool body, exactly as it would be written.",
                    }
                },
                "required": ["input"],
                "additionalProperties": false,
            },
        }
    }))
}

fn chat_tools_from_responses_tools(tools: &crate::common::ResponsesApiTools) -> Vec<Value> {
    let Ok(Value::Array(tools)) = serde_json::from_str::<Value>(tools.as_raw_value().get()) else {
        return Vec::new();
    };

    tools
        .into_iter()
        .filter_map(|tool| {
            let obj = tool.as_object()?;
            match obj.get("type").and_then(Value::as_str) {
                Some("function") => {}
                Some("custom") => return chat_function_from_freeform(obj),
                // Provider-native tools (web_search, local_shell, ...) have no
                // Chat Completions equivalent; dropping them is better than sending
                // a body the provider will reject outright.
                _ => return None,
            }
            // Already nested (some callers hand us Chat-shaped specs).
            if obj.contains_key("function") {
                return Some(tool);
            }

            let mut function = serde_json::Map::new();
            for key in ["name", "description", "parameters", "strict"] {
                if let Some(value) = obj.get(key) {
                    function.insert(key.to_string(), value.clone());
                }
            }
            Some(json!({"type": "function", "function": function}))
        })
        .collect()
}

/// Attaches each `Reasoning` item to the assistant message or tool call it belongs
/// to, so it can ride along as `reasoning_content`.
///
/// Every reasoning item still present in `input` is carried. Which reasoning survives
/// a finished turn is decided in history, where the size of the reasoning can be
/// weighed against the prompt-cache invalidation that removing it costs. Dropping more
/// of it here would rewrite the messages history deliberately left alone, and cost
/// exactly the cache miss that decision was made to avoid.
fn anchor_reasoning(input: &[ResponseItem]) -> HashMap<usize, String> {
    let mut anchored: HashMap<usize, String> = HashMap::new();

    for (idx, item) in input.iter().enumerate() {
        let ResponseItem::Reasoning {
            content: Some(items),
            ..
        } = item
        else {
            continue;
        };

        let mut text = String::new();
        for entry in items {
            match entry {
                ReasoningItemContent::ReasoningText { text: segment }
                | ReasoningItemContent::Text { text: segment } => text.push_str(segment),
            }
        }
        if text.trim().is_empty() {
            continue;
        }

        // Reasoning precedes the item it explains, but a provider may also emit
        // it right after an assistant message. Prefer the following item.
        let anchor = input
            .get(idx + 1)
            .and_then(|next| match next {
                ResponseItem::FunctionCall { .. } | ResponseItem::CustomToolCall { .. } => {
                    Some(idx + 1)
                }
                ResponseItem::Message { role, .. } if role == "assistant" => Some(idx + 1),
                _ => None,
            })
            .or_else(
                || match idx.checked_sub(1).map(|prev| (prev, &input[prev])) {
                    Some((prev, ResponseItem::Message { role, .. })) if role == "assistant" => {
                        Some(prev)
                    }
                    _ => None,
                },
            );

        if let Some(anchor) = anchor {
            anchored
                .entry(anchor)
                .and_modify(|existing| {
                    existing.push('\n');
                    existing.push_str(&text);
                })
                .or_insert(text);
        }
    }

    anchored
}

/// Chat Completions requires consecutive tool calls to be grouped into one
/// assistant message carrying `tool_calls: [...]`, followed by the `tool` replies.
fn push_tool_call(messages: &mut Vec<Value>, tool_call: Value, reasoning: Option<&str>) {
    if let Some(Value::Object(obj)) = messages.last_mut()
        && obj.get("role").and_then(Value::as_str) == Some("assistant")
        && obj.get("content").is_some_and(Value::is_null)
        && let Some(tool_calls) = obj.get_mut("tool_calls").and_then(Value::as_array_mut)
    {
        tool_calls.push(tool_call);
        if let Some(reasoning) = reasoning {
            match obj.get_mut("reasoning_content") {
                Some(Value::String(existing)) => {
                    existing.push('\n');
                    existing.push_str(reasoning);
                }
                _ => {
                    obj.insert("reasoning_content".to_string(), json!(reasoning));
                }
            }
        }
        return;
    }

    let mut msg = json!({
        "role": "assistant",
        "content": null,
        "tool_calls": [tool_call],
    });
    if let Some(reasoning) = reasoning
        && let Some(obj) = msg.as_object_mut()
    {
        obj.insert("reasoning_content".to_string(), json!(reasoning));
    }
    messages.push(msg);
}

#[cfg(test)]
mod tests {
    use super::*;
    use codex_protocol::models::FunctionCallOutputPayload;
    use pretty_assertions::assert_eq;

    fn user(text: &str) -> ResponseItem {
        ResponseItem::Message {
            id: None,
            role: "user".to_string(),
            content: vec![ContentItem::InputText {
                text: text.to_string(),
            }],
            phase: None,
            internal_chat_message_metadata_passthrough: None,
        }
    }

    fn call(call_id: &str, name: &str, arguments: &str) -> ResponseItem {
        ResponseItem::FunctionCall {
            id: None,
            name: name.to_string(),
            namespace: None,
            arguments: arguments.to_string(),
            encrypted_function_args: None,
            call_id: call_id.to_string(),
            internal_chat_message_metadata_passthrough: None,
        }
    }

    fn request(input: Vec<ResponseItem>, tools: Option<&str>) -> ResponsesApiRequest {
        ResponsesApiRequest {
            model: "glm-5.3-flash".to_string(),
            instructions: "be terse".to_string(),
            input,
            tools: tools.map(|tools| {
                std::sync::Arc::<serde_json::value::RawValue>::from(
                    serde_json::value::RawValue::from_string(tools.to_string()).expect("raw value"),
                )
                .into()
            }),
            tool_choice: "auto".to_string(),
            parallel_tool_calls: true,
            reasoning: None,
            store: false,
            stream: true,
            stream_options: None,
            include: Vec::new(),
            service_tier: None,
            prompt_cache_key: None,
            text: None,
            client_metadata: None,
            access_programs: None,
        }
    }

    #[test]
    fn rewrites_developer_messages_as_system_messages() {
        let body = chat_body_from_responses_request(&request(
            vec![
                ResponseItem::Message {
                    id: None,
                    role: "developer".to_string(),
                    content: vec![ContentItem::InputText {
                        text: "environment context".to_string(),
                    }],
                    phase: None,
                    internal_chat_message_metadata_passthrough: None,
                },
                user("hello"),
            ],
            None,
        ));

        let messages = body["messages"].as_array().expect("messages");
        assert_eq!(messages[1]["role"], "system");
        assert_eq!(messages[1]["content"], "environment context");
    }

    #[test]
    fn groups_consecutive_tool_calls_and_keeps_outputs_in_order() {
        let body = chat_body_from_responses_request(&request(
            vec![
                user("read these"),
                call("call-a", "read_file", r#"{"path":"a"}"#),
                call("call-b", "read_file", r#"{"path":"b"}"#),
                ResponseItem::FunctionCallOutput {
                    id: None,
                    name: None,
                    namespace: None,
                    call_id: Some("call-a".to_string()),
                    output: FunctionCallOutputPayload::from_text("A".to_string()),
                    internal_chat_message_metadata_passthrough: None,
                },
                ResponseItem::FunctionCallOutput {
                    id: None,
                    name: None,
                    namespace: None,
                    call_id: Some("call-b".to_string()),
                    output: FunctionCallOutputPayload::from_text("B".to_string()),
                    internal_chat_message_metadata_passthrough: None,
                },
            ],
            None,
        ));

        let messages = body["messages"].as_array().expect("messages");
        assert_eq!(messages.len(), 5, "system + user + 1 assistant + 2 tool");
        assert_eq!(messages[0]["role"], "system");
        assert_eq!(messages[1]["role"], "user");
        assert_eq!(messages[2]["role"], "assistant");
        assert_eq!(
            messages[2]["tool_calls"].as_array().map(Vec::len),
            Some(2),
            "consecutive calls must collapse into one assistant message"
        );
        assert_eq!(messages[3]["tool_call_id"], "call-a");
        assert_eq!(messages[4]["tool_call_id"], "call-b");
        assert_eq!(body["stream_options"]["include_usage"], true);
    }

    #[test]
    fn nests_responses_tool_specs_and_drops_provider_native_tools() {
        let body = chat_body_from_responses_request(&request(
            vec![user("hi")],
            Some(
                r#"[{"type":"function","name":"shell","description":"run","parameters":{"type":"object"}},
                    {"type":"web_search"}]"#,
            ),
        ));

        let tools = body["tools"].as_array().expect("tools");
        assert_eq!(tools.len(), 1, "web_search has no chat equivalent");
        assert_eq!(tools[0]["type"], "function");
        assert_eq!(tools[0]["function"]["name"], "shell");
        assert_eq!(tools[0]["function"]["parameters"]["type"], "object");
        assert!(
            tools[0].get("name").is_none(),
            "flat responses shape must not leak through"
        );
        assert_eq!(body["tool_choice"], "auto");
    }

    #[test]
    fn sends_thinking_and_clamps_the_effort_to_a_rung_glm_accepts() {
        let mut request = request(vec![user("hi")], None);
        request.reasoning = Some(crate::common::Reasoning {
            effort: Some(ReasoningEffort::Medium),
            summary: None,
            context: None,
        });

        let body = chat_body_from_responses_request(&request);

        assert_eq!(body["thinking"]["type"], "enabled");
        assert_eq!(
            body["reasoning_effort"], "high",
            "medium is not a GLM rung and would be rejected on the wire"
        );
    }

    #[test]
    fn omits_the_effort_when_reasoning_carries_none() {
        let mut request = request(vec![user("hi")], None);
        request.reasoning = Some(crate::common::Reasoning {
            effort: None,
            summary: None,
            context: None,
        });

        let body = chat_body_from_responses_request(&request);

        assert_eq!(body["thinking"]["type"], "enabled");
        assert!(body.get("reasoning_effort").is_none());
    }

    #[test]
    fn carries_reasoning_on_the_tool_call_it_explains() {
        let body = chat_body_from_responses_request(&request(
            vec![
                user("go"),
                ResponseItem::Reasoning {
                    id: None,
                    summary: Vec::new(),
                    content: Some(vec![ReasoningItemContent::ReasoningText {
                        text: "check the file".to_string(),
                    }]),
                    encrypted_content: None,
                    internal_chat_message_metadata_passthrough: None,
                },
                call("call-a", "read_file", "{}"),
            ],
            None,
        ));

        let messages = body["messages"].as_array().expect("messages");
        let assistant = messages.last().expect("assistant message");
        assert_eq!(assistant["role"], "assistant");
        assert_eq!(assistant["reasoning_content"], "check the file");
    }

    #[test]
    fn carries_reasoning_that_history_kept_from_an_earlier_turn() {
        let thinking = |text: &str| ResponseItem::Reasoning {
            id: None,
            summary: Vec::new(),
            content: Some(vec![ReasoningItemContent::ReasoningText {
                text: text.to_string(),
            }]),
            encrypted_content: None,
            internal_chat_message_metadata_passthrough: None,
        };

        let body = chat_body_from_responses_request(&request(
            vec![
                user("go"),
                thinking("check the file"),
                call("call-a", "read_file", "{}"),
                user("now do the next thing"),
                thinking("write the file"),
                call("call-b", "write_file", "{}"),
            ],
            None,
        ));

        // History is the only place that decides what a finished turn keeps. Stripping the
        // earlier reasoning here would rewrite a message the prefix cache still matches on.
        let messages = body["messages"].as_array().expect("messages");
        let earlier = messages
            .iter()
            .find(|message| message["tool_calls"][0]["id"] == "call-a")
            .expect("earlier tool call");
        assert_eq!(earlier["reasoning_content"], "check the file");
    }
}
