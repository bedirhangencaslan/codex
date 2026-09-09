# codex-api wire inventory

## `codex-api/src`

`codex-api/src/lib.rs` — declares the API crate modules and exposes their public client, provider, request/stream, and error types — `codex-api/src/lib.rs:16`

`codex-api/src/api_bridge.rs` — translates transport and `ApiError` failures into `CodexErr`, preserving policy, usage, overload, and request-id details — `codex-api/src/api_bridge.rs:21`

`codex-api/src/api_bridge_tests.rs` — tests API-error-to-Codex-error mapping, including cyber-policy and misalignment classification and diagnostics — `codex-api/src/api_bridge_tests.rs:139`

`codex-api/src/auth.rs` — defines async authentication providers that produce headers or signed requests, plus shared auth handles and telemetry — `codex-api/src/auth.rs:30`

`codex-api/src/common.rs` — defines the canonical Responses request and normalized `ResponseEvent` stream contract, including websocket requests and text controls — `codex-api/src/common.rs:98`

`codex-api/src/error.rs` — defines the crate's `ApiError` taxonomy for transport, API status, policy, quota, rate-limit, and stream failures — `codex-api/src/error.rs:9`

`codex-api/src/files.rs` — handles OpenAI-compatible multipart-free file upload through create, blob PUT, and finalize endpoints — `codex-api/src/files.rs:121`

`codex-api/src/images.rs` — defines request and response wire structs for image generation and editing — `codex-api/src/images.rs:4`

`codex-api/src/provider.rs` — models provider endpoints, retry settings, Responses versus Chat wire APIs, request URL construction, and Azure behavior — `codex-api/src/provider.rs:16`

`codex-api/src/rate_limits.rs` — parses known provider rate-limit response headers and events into typed rate-limit snapshots — `codex-api/src/rate_limits.rs:23`

`codex-api/src/safety_buffering.rs` — parses safety-buffering HTTP headers into the corresponding stream treatment value — `codex-api/src/safety_buffering.rs:8`

`codex-api/src/search.rs` — defines search requests, filters, settings, commands, callers, and response payload types — `codex-api/src/search.rs:8`

`codex-api/src/telemetry.rs` — defines SSE and request telemetry hooks and wraps retry attempts with per-attempt observability — `codex-api/src/telemetry.rs:18`

## `codex-api/src/endpoint`

`codex-api/src/endpoint/mod.rs` — declares and exports the typed API endpoint clients — `codex-api/src/endpoint/mod.rs:1`

`codex-api/src/endpoint/session.rs` — provides the shared authenticated, retried helper for unary and streaming endpoint HTTP requests — `codex-api/src/endpoint/session.rs:19`

`codex-api/src/endpoint/compact.rs` — sends compaction requests and deserializes compacted `ResponseItem`s plus turn state — `codex-api/src/endpoint/compact.rs:18`

`codex-api/src/endpoint/images.rs` — sends image generation and edit requests and parses image outputs and request IDs — `codex-api/src/endpoint/images.rs:35`

`codex-api/src/endpoint/memories.rs` — sends memory trace-summarization requests and parses memory summaries — `codex-api/src/endpoint/memories.rs:32`

`codex-api/src/endpoint/models.rs` — fetches provider model metadata and its model-list ETag — `codex-api/src/endpoint/models.rs:31`

`codex-api/src/endpoint/responses.rs` — routes Responses requests to the correct endpoint or Chat Completions translation, adds session metadata, and starts SSE parsing — `codex-api/src/endpoint/responses.rs:31`

`codex-api/src/endpoint/realtime_call.rs` — creates WebRTC realtime calls using backend JSON, multipart, or raw SDP forms and parses returned SDP/call IDs — `codex-api/src/endpoint/realtime_call.rs:29`

`codex-api/src/endpoint/responses_websocket.rs` — establishes, authenticates, probes, sends, and parses Responses-over-WebSocket streams, including wrapped metadata and errors — `codex-api/src/endpoint/responses_websocket.rs:183`

`codex-api/src/endpoint/search.rs` — sends alpha search requests and parses search results — `codex-api/src/endpoint/search.rs:14`

## `codex-api/src/endpoint/realtime_websocket`

`codex-api/src/endpoint/realtime_websocket/mod.rs` — declares realtime WebSocket method and protocol modules and exports their client/connection/types — `codex-api/src/endpoint/realtime_websocket/mod.rs:1`

`codex-api/src/endpoint/realtime_websocket/methods.rs` — owns realtime WebSocket connections, command pumps, outbound method serialization, event parsing, transcript state, and URL construction — `codex-api/src/endpoint/realtime_websocket/methods.rs:211`

`codex-api/src/endpoint/realtime_websocket/methods_common.rs` — routes realtime outbound message construction to the selected V1, V2, or Frameless Bidi wire adapter — `codex-api/src/endpoint/realtime_websocket/methods_common.rs:41`

`codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs` — tests adapter-specific realtime session, item, handoff, and delegation messages — `codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs:16`

`codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs` — builds Frameless Bidi session updates and context-appends and chunks oversized text to the wire limit — `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs:35`

`codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs` — tests Frameless Bidi chunking and initial-session JSON encoding — `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs:11`

`codex-api/src/endpoint/realtime_websocket/methods_v1.rs` — builds legacy realtime V1 conversation item, handoff, session update, and intent wire data — `codex-api/src/endpoint/realtime_websocket/methods_v1.rs:18`

`codex-api/src/endpoint/realtime_websocket/methods_v2.rs` — builds realtime V2 conversation, session, modality, tool, and intent wire data — `codex-api/src/endpoint/realtime_websocket/methods_v2.rs:39`

`codex-api/src/endpoint/realtime_websocket/protocol.rs` — defines realtime parser/config/outbound-message types and dispatches event parsing to the selected protocol — `codex-api/src/endpoint/realtime_websocket/protocol.rs:14`

`codex-api/src/endpoint/realtime_websocket/protocol_common.rs` — supplies common JSON payload extraction and shared realtime session/transcript/error parsers — `codex-api/src/endpoint/realtime_websocket/protocol_common.rs:7`

`codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs` — parses Frameless Bidi session, audio, transcript, turn, delegation, and error events — `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs:15`

`codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs` — tests legacy/frameless delegation equivalence and reuse of normalized realtime events — `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs:7`

`codex-api/src/endpoint/realtime_websocket/protocol_v1.rs` — parses legacy realtime V1 audio, transcript, item, handoff, and error events — `codex-api/src/endpoint/realtime_websocket/protocol_v1.rs:12`

`codex-api/src/endpoint/realtime_websocket/protocol_v2.rs` — parses realtime V2 response, audio, transcript, item, tool, response lifecycle, and error events — `codex-api/src/endpoint/realtime_websocket/protocol_v2.rs:24`

## `codex-api/src/requests`

`codex-api/src/requests/mod.rs` — declares request-building modules and re-exports the compression choice — `codex-api/src/requests/mod.rs:1`

`codex-api/src/requests/headers.rs` — builds session/thread headers and derives subagent headers from the session source — `codex-api/src/requests/headers.rs:5`

`codex-api/src/requests/responses.rs` — defines the optional request compression setting — `codex-api/src/requests/responses.rs:1`

`codex-api/src/requests/chat.rs` — translates a Responses API request into an OpenAI-compatible Chat Completions request body — `codex-api/src/requests/chat.rs:18`

## `codex-api/src/sse`

`codex-api/src/sse/mod.rs` — declares SSE modules and re-exports Chat and Responses stream entry points — `codex-api/src/sse/mod.rs:1`

`codex-api/src/sse/chat.rs` — translates Chat Completions SSE deltas into normalized Responses `ResponseEvent`s, reassembling reasoning, text, tool calls, and usage — `codex-api/src/sse/chat.rs:30`

`codex-api/src/sse/responses.rs` — parses Responses SSE frames, emits normalized events and metadata, and classifies stream errors — `codex-api/src/sse/responses.rs:36`
