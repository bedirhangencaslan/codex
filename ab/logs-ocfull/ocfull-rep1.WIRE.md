# `codex-api` wire map

## `codex-api/src/`

- `codex-api/src/api_bridge.rs` — Translates `ApiError` and transport failures into `CodexErr` values for the rest of the crate. (`codex-api/src/api_bridge.rs:21`)
- `codex-api/src/api_bridge_tests.rs` — Tests error translation across overloaded, retry-delay, cyber-policy, misalignment, usage-limit, and header cases. (`codex-api/src/api_bridge_tests.rs:7`)
- `codex-api/src/auth.rs` — Defines outbound request authentication as either header-only or explicit request signing. (`codex-api/src/auth.rs:30`)
- `codex-api/src/common.rs` — Declares shared API request, response-event, stream, and compaction wire types. (`codex-api/src/common.rs:46`)
- `codex-api/src/error.rs` — Defines the API-layer error model, including transport, rate-limit, policy, quota, and stream failures. (`codex-api/src/error.rs:8`)
- `codex-api/src/files.rs` — Handles hosted file upload as create, blob PUT, and finalize polling. (`codex-api/src/files.rs:121`)
- `codex-api/src/images.rs` — Declares image generation and editing request/response wire DTOs. (`codex-api/src/images.rs:4`)
- `codex-api/src/lib.rs` — Declares the crate modules and public API surface. (`codex-api/src/lib.rs:1`)
- `codex-api/src/provider.rs` — Holds provider endpoint configuration, wire-API selection, headers, query parameters, and retry policy. (`codex-api/src/provider.rs:55`)
- `codex-api/src/rate_limits.rs` — Parses primary, secondary, and event-based rate-limit snapshots from response headers or JSON. (`codex-api/src/rate_limits.rs:23`)
- `codex-api/src/safety_buffering.rs` — Parses safety-buffering treatment headers, including enablement and faster-model fallback. (`codex-api/src/safety_buffering.rs:8`)
- `codex-api/src/search.rs` — Declares web-search request, response, result, and operation DTOs. (`codex-api/src/search.rs:8`)
- `codex-api/src/telemetry.rs` — Defines SSE/Websocket telemetry hooks and the request-scoped telemetry runner. (`codex-api/src/telemetry.rs:18`)

## `codex-api/src/endpoint/`

- `codex-api/src/endpoint/compact.rs` — POSTs compact requests, captures turn-state headers, and parses compact output. (`codex-api/src/endpoint/compact.rs:39`)
- `codex-api/src/endpoint/images.rs` — Sends image generation and edit requests and extracts the image-generation request ID. (`codex-api/src/endpoint/images.rs:35`)
- `codex-api/src/endpoint/memories.rs` — Sends memory trace summarization requests and parses summaries. (`codex-api/src/endpoint/memories.rs:36`)
- `codex-api/src/endpoint/mod.rs` — Declares endpoint modules and re-exports their clients. (`codex-api/src/endpoint/mod.rs:1`)
- `codex-api/src/endpoint/models.rs` — Lists models with the client version query and extracts the models ETag. (`codex-api/src/endpoint/models.rs:46`)
- `codex-api/src/endpoint/realtime_call.rs` — Creates realtime calls from SDP offers, supporting multipart and JSON bodies and extracting the call ID. (`codex-api/src/endpoint/realtime_call.rs:129`)
- `codex-api/src/endpoint/responses.rs` — Builds Responses or Chat Completions requests and dispatches them to the corresponding SSE stream. (`codex-api/src/endpoint/responses.rs:112`)
- `codex-api/src/endpoint/responses_websocket.rs` — Manages Responses websocket connections, request envelopes, handshake probing, event streams, timing logs, and websocket errors. (`codex-api/src/endpoint/responses_websocket.rs:375`)
- `codex-api/src/endpoint/search.rs` — Sends web-search requests to the search endpoint and parses responses. (`codex-api/src/endpoint/search.rs:35`)
- `codex-api/src/endpoint/session.rs` — Executes endpoint requests with provider headers, authentication, retry, compression, and telemetry. (`codex-api/src/endpoint/session.rs:63`)

## `codex-api/src/endpoint/realtime_websocket/`

- `codex-api/src/endpoint/realtime_websocket/mod.rs` — Declares realtime websocket modules and exports their public client, connection, protocol, and event types. (`codex-api/src/endpoint/realtime_websocket/mod.rs:1`)
- `codex-api/src/endpoint/realtime_websocket/methods.rs` — Implements realtime websocket clients, connection transport, URL construction, outbound writers, event pumps, and transcript state. (`codex-api/src/endpoint/realtime_websocket/methods.rs:791`)
- `codex-api/src/endpoint/realtime_websocket/methods_common.rs` — Routes realtime outbound message construction across V1, Frameless Bidi, and Realtime V2 adapters. (`codex-api/src/endpoint/realtime_websocket/methods_common.rs:41`)
- `codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs` — Tests adapter-specific outbound session, context-append, handoff, and function-output messages. (`codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs:15`)
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs` — Builds Frameless Bidi session JSON and context-append messages and chunks them for the wire. (`codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs:35`)
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs` — Tests Frameless Bidi context chunking and initial-session serialization. (`codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs:10`)
- `codex-api/src/endpoint/realtime_websocket/methods_v1.rs` — Builds V1 realtime conversation-item, handoff, session-update, and intent payloads. (`codex-api/src/endpoint/realtime_websocket/methods_v1.rs:18`)
- `codex-api/src/endpoint/realtime_websocket/methods_v2.rs` — Builds Realtime V2 conversation and session-update payloads, including audio tools and transcription settings. (`codex-api/src/endpoint/realtime_websocket/methods_v2.rs:75`)
- `codex-api/src/endpoint/realtime_websocket/protocol.rs` — Defines realtime adapter/session types and outbound wire shapes, then dispatches inbound parsing by adapter. (`codex-api/src/endpoint/realtime_websocket/protocol.rs:263`)
- `codex-api/src/endpoint/realtime_websocket/protocol_common.rs` — Provides shared realtime JSON parsing for payload/type extraction, session updates, transcripts, and errors. (`codex-api/src/endpoint/realtime_websocket/protocol_common.rs:7`)
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs` — Parses Frameless Bidi audio, transcript, turn, delegation, session, and error events. (`codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs:15`)
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs` — Tests Frameless Bidi compatibility with legacy handoffs and internal transcript/audio events. (`codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs:6`)
- `codex-api/src/endpoint/realtime_websocket/protocol_v1.rs` — Parses realtime V1 audio, transcript, item, handoff, and error events. (`codex-api/src/endpoint/realtime_websocket/protocol_v1.rs:12`)
- `codex-api/src/endpoint/realtime_websocket/protocol_v2.rs` — Parses Realtime V2 audio, transcript, response lifecycle, background-agent handoff, silence, item, and error events. (`codex-api/src/endpoint/realtime_websocket/protocol_v2.rs:24`)

## `codex-api/src/requests/`

- `codex-api/src/requests/mod.rs` — Declares request-building modules and exports the compression setting. (`codex-api/src/requests/mod.rs:1`)
- `codex-api/src/requests/chat.rs` — Rewrites Responses-shaped requests into OpenAI-compatible Chat Completions bodies, tools, reasoning, and response formats. (`codex-api/src/requests/chat.rs:19`)
- `codex-api/src/requests/headers.rs` — Builds session/thread headers and derives subagent source headers. (`codex-api/src/requests/headers.rs:5`)
- `codex-api/src/requests/responses.rs` — Defines optional zstd compression for Responses requests. (`codex-api/src/requests/responses.rs:2`)

## `codex-api/src/sse/`

- `codex-api/src/sse/mod.rs` — Declares SSE modules and re-exports chat and Responses stream processing. (`codex-api/src/sse/mod.rs:1`)
- `codex-api/src/sse/chat.rs` — Translates Chat Completions SSE deltas, tool calls, reasoning, usage, and errors into `ResponseEvent`s. (`codex-api/src/sse/chat.rs:30`)
- `codex-api/src/sse/responses.rs` — Decodes Responses SSE events into `ResponseEvent`s while extracting headers, rate limits, metadata, safety buffering, and API failures. (`codex-api/src/sse/responses.rs:36`)
