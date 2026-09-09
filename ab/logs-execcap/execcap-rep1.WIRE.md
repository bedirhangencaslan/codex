# codex-api wire inventory

## `codex-api/src`

- `codex-api/src/api_bridge.rs` — Translates provider `ApiError` statuses, response bodies, and rate-limit headers into the crate-facing `CodexErr`. (`codex-api/src/api_bridge.rs:21`)
- `codex-api/src/api_bridge_tests.rs` — Tests API-error mapping for overloaded servers, cyber-policy and misalignment failures, rate limits, and identity details. (`codex-api/src/api_bridge_tests.rs:8`)
- `codex-api/src/auth.rs` — Defines the outbound-request authentication provider, auth errors, future aliases, shared provider type, and auth telemetry. (`codex-api/src/auth.rs:30`)
- `codex-api/src/common.rs` — Defines shared Responses request/response stream types, events, tracing metadata, compaction and memory payloads, and stream options. (`codex-api/src/common.rs:98`)
- `codex-api/src/error.rs` — Defines the transport, HTTP, streaming, quota, retry, and rate-limit `ApiError` variants. (`codex-api/src/error.rs:9`)
- `codex-api/src/files.rs` — Handles OpenAI-hosted file uploads, including the multipart request, response parsing, and finalization. (`codex-api/src/files.rs:121`)
- `codex-api/src/images.rs` — Defines image-generation and image-edit request and response wire types. (`codex-api/src/images.rs:5`)
- `codex-api/src/lib.rs` — Declares the crate modules and exports its public API. (`codex-api/src/lib.rs:1`)
- `codex-api/src/provider.rs` — Defines provider base URLs, wire API choice, retry configuration, and Azure Responses detection. (`codex-api/src/provider.rs:56`)
- `codex-api/src/rate_limits.rs` — Parses default and per-limit rate-limit snapshots, limit events, promo messages, and reached types. (`codex-api/src/rate_limits.rs:28`)
- `codex-api/src/safety_buffering.rs` — Maps HTTP headers into the safety-buffering treatment. (`codex-api/src/safety_buffering.rs:8`)
- `codex-api/src/search.rs` — Defines search requests, commands, settings, filters, and responses. (`codex-api/src/search.rs:9`)
- `codex-api/src/telemetry.rs` — Provides SSE, WebSocket, and HTTP request telemetry hooks and retry wrapping. (`codex-api/src/telemetry.rs:68`)

## `codex-api/src/endpoint`

- `codex-api/src/endpoint/compact.rs` — Sends compaction requests through the authenticated endpoint session. (`codex-api/src/endpoint/compact.rs:18`)
- `codex-api/src/endpoint/images.rs` — Sends image-generation and image-edit requests and parses image responses. (`codex-api/src/endpoint/images.rs:18`)
- `codex-api/src/endpoint/memories.rs` — Sends memory-summary requests and parses their results. (`codex-api/src/endpoint/memories.rs:15`)
- `codex-api/src/endpoint/mod.rs` — Declares endpoint modules and re-exports their public clients and types. (`codex-api/src/endpoint/mod.rs:1`)
- `codex-api/src/endpoint/models.rs` — Fetches model metadata and caches the response ETag. (`codex-api/src/endpoint/models.rs:14`)
- `codex-api/src/endpoint/realtime_call.rs` — Creates realtime calls, including multipart session setup, SDP decoding, and call-ID handling. (`codex-api/src/endpoint/realtime_call.rs:29`)
- `codex-api/src/endpoint/responses.rs` — Builds Responses requests and chooses HTTP or WebSocket transport, spawning the resulting response stream. (`codex-api/src/endpoint/responses.rs:52`)
- `codex-api/src/endpoint/responses_websocket.rs` — Manages Responses WebSocket connections, request serialization, event parsing, timing, errors, and stream construction. (`codex-api/src/endpoint/responses_websocket.rs:375`)
- `codex-api/src/endpoint/search.rs` — Sends search requests and parses search responses. (`codex-api/src/endpoint/search.rs:14`)
- `codex-api/src/endpoint/session.rs` — Provides the shared authenticated transport session for unary and streaming endpoint calls. (`codex-api/src/endpoint/session.rs:19`)

## `codex-api/src/endpoint/realtime_websocket`

- `codex-api/src/endpoint/realtime_websocket/methods.rs` — Implements the realtime WebSocket client, connection/writer/events API, outbound methods, transcript state, and URL setup. (`codex-api/src/endpoint/realtime_websocket/methods.rs:211`)
- `codex-api/src/endpoint/realtime_websocket/methods_common.rs` — Builds common realtime outbound messages and session JSON shared by wire adapters. (`codex-api/src/endpoint/realtime_websocket/methods_common.rs:41`)
- `codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs` — Tests common realtime outbound message construction across adapters. (`codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs:16`)
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs` — Builds frameless bidirectional session, context-append, and delegation messages and chunks context text. (`codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs:13`)
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs` — Tests frameless context chunking and session JSON encoding. (`codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs:11`)
- `codex-api/src/endpoint/realtime_websocket/methods_v1.rs` — Builds realtime v1 outbound session, conversation, and handoff messages. (`codex-api/src/endpoint/realtime_websocket/methods_v1.rs:18`)
- `codex-api/src/endpoint/realtime_websocket/methods_v2.rs` — Builds realtime v2 outbound session, conversation-item, function-output, and handoff messages. (`codex-api/src/endpoint/realtime_websocket/methods_v2.rs:39`)
- `codex-api/src/endpoint/realtime_websocket/mod.rs` — Declares realtime WebSocket modules and re-exports clients, methods, protocol, and parser types. (`codex-api/src/endpoint/realtime_websocket/mod.rs:1`)
- `codex-api/src/endpoint/realtime_websocket/protocol.rs` — Defines realtime wire adapters, session/config types, outbound payload types, and event-parser dispatch. (`codex-api/src/endpoint/realtime_websocket/protocol.rs:15`)
- `codex-api/src/endpoint/realtime_websocket/protocol_common.rs` — Parses realtime JSON payloads plus common session, transcript, and error events. (`codex-api/src/endpoint/realtime_websocket/protocol_common.rs:7`)
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs` — Parses frameless bidirectional audio, transcript, turn-done, and delegation events. (`codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs:15`)
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs` — Tests frameless and legacy handoff compatibility and transcript/audio decoding. (`codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs:7`)
- `codex-api/src/endpoint/realtime_websocket/protocol_v1.rs` — Parses realtime v1 audio, transcript, handoff, and item-done events. (`codex-api/src/endpoint/realtime_websocket/protocol_v1.rs:12`)
- `codex-api/src/endpoint/realtime_websocket/protocol_v2.rs` — Parses realtime v2 response, audio, transcript, handoff, noop, cancellation, and error events. (`codex-api/src/endpoint/realtime_websocket/protocol_v2.rs:24`)

## `codex-api/src/requests`

- `codex-api/src/requests/chat.rs` — Translates Responses API requests into Chat Completions request bodies, tools, reasoning, and tool-call messages. (`codex-api/src/requests/chat.rs:19`)
- `codex-api/src/requests/headers.rs` — Builds session and sub-agent HTTP headers and safely inserts string values. (`codex-api/src/requests/headers.rs:5`)
- `codex-api/src/requests/mod.rs` — Declares request modules and exports the response compression enum. (`codex-api/src/requests/mod.rs:1`)
- `codex-api/src/requests/responses.rs` — Defines the response-body compression choice. (`codex-api/src/requests/responses.rs:2`)

## `codex-api/src/sse`

- `codex-api/src/sse/chat.rs` — Translates Chat Completions SSE deltas into Responses-shaped `ResponseEvent`s, rebuilding items and token usage. (`codex-api/src/sse/chat.rs:77`)
- `codex-api/src/sse/mod.rs` — Declares SSE modules and exports their shared stream helpers. (`codex-api/src/sse/mod.rs:1`)
- `codex-api/src/sse/responses.rs` — Processes Responses SSE events, extracting model, rate-limit, turn-state, usage, moderation, reasoning, and error information. (`codex-api/src/sse/responses.rs:36`)
