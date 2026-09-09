# codex-api wire translation map

## codex-api/src/

- `codex-api/src/lib.rs` — Crate root: declares all submodules and re-exports the public API surface (clients, auth, errors, provider, SSE and file types). `codex-api/src/lib.rs:1`
- `codex-api/src/api_bridge.rs` — Translates crate-internal `ApiError` values into `CodexErr` protocol errors, including rate-limit, overloaded, cyber-policy and misalignment detection from HTTP bodies. `codex-api/src/api_bridge.rs:21`
- `codex-api/src/api_bridge_tests.rs` — Tests for `map_api_error`: server-overloaded, retry delays, Cloudflare blocks and transport-body error mapping. `codex-api/src/api_bridge_tests.rs:8`
- `codex-api/src/auth.rs` — `AuthProvider` trait and future types for applying auth headers or full-request signing to outbound API requests. `codex-api/src/auth.rs:30`
- `codex-api/src/common.rs` — Shared wire types: Responses API request/event/stream structs, compaction and memory-summarize payloads, safety-buffering and WS metadata keys. `codex-api/src/common.rs:46`
- `codex-api/src/error.rs` — The `ApiError` enum covering transport, HTTP status, rate limits, invalid requests, cyber policy and server-overloaded cases. `codex-api/src/error.rs:9`
- `codex-api/src/files.rs` — Hosted file upload to OpenAI (`sediment://`): create session, blob upload, finalize with retries, and size/timeout limits. `codex-api/src/files.rs:17`
- `codex-api/src/images.rs` — Wire structs for image generation and edit requests/responses (prompt, background, quality, size, b64 output). `codex-api/src/images.rs:4`
- `codex-api/src/provider.rs` — `Provider` endpoint configuration (base URL, headers, retry, idle timeout) and `WireApi` enum selecting Responses vs Chat Completions. `codex-api/src/provider.rs:44`
- `codex-api/src/rate_limits.rs` — Parses rate-limit HTTP headers into `RateLimitSnapshot` values for default and per-limit-id header families. `codex-api/src/rate_limits.rs:23`
- `codex-api/src/safety_buffering.rs` — Extracts safety-buffering treatment (enabled, faster model) from `x-codex-safety-buffering-*` response headers. `codex-api/src/safety_buffering.rs:8`
- `codex-api/src/search.rs` — Wire structs for the search endpoint: requests, commands (web/image/finance/weather etc.) and responses. `codex-api/src/search.rs:8`
- `codex-api/src/telemetry.rs` — Telemetry traits for SSE and WebSocket transports plus retry-wrapping helpers that attach per-attempt request telemetry. `codex-api/src/telemetry.rs:18`

## codex-api/src/endpoint/

- `codex-api/src/endpoint/mod.rs` — Module hub re-exporting all endpoint clients (compact, images, memories, models, realtime, responses, search). `codex-api/src/endpoint/mod.rs:12`
- `codex-api/src/endpoint/compact.rs` — `CompactClient` for POST `responses/compact`, returning compacted `ResponseItem`s and capturing turn state. `codex-api/src/endpoint/compact.rs:18`
- `codex-api/src/endpoint/images.rs` — `ImagesClient` for POST `images/generations` and `images/edits`, returning image responses with request IDs. `codex-api/src/endpoint/images.rs:18`
- `codex-api/src/endpoint/memories.rs` — `MemoriesClient` for POST `memories/trace_summarize`, returning memory summarize outputs. `codex-api/src/endpoint/memories.rs:15`
- `codex-api/src/endpoint/models.rs` — `ModelsClient` for GET `models` with client-version query and ETag capture, returning `ModelInfo` list. `codex-api/src/endpoint/models.rs:14`
- `codex-api/src/endpoint/realtime_call.rs` — `RealtimeCallClient` for WebRTC realtime call creation over multipart POST `realtime/calls`, returning SDP and call ID. `codex-api/src/endpoint/realtime_call.rs:29`
- `codex-api/src/endpoint/responses.rs` — `ResponsesClient` for Responses/Guardian inference: builds requests, dispatches SSE streams, handles compression and prompt-cache keep-alive. `codex-api/src/endpoint/responses.rs:52`
- `codex-api/src/endpoint/responses_websocket.rs` — `ResponsesWebsocketClient` and connection for Responses inference over WebSocket, including event dispatch, rate limits and safety buffering. `codex-api/src/endpoint/responses_websocket.rs:50`
- `codex-api/src/endpoint/search.rs` — `SearchClient` for POST `alpha/search`, serializing search requests and deserializing responses. `codex-api/src/endpoint/search.rs:14`
- `codex-api/src/endpoint/session.rs` — `EndpointSession`: shared per-endpoint execution helper combining provider, transport, auth and telemetry for unary HTTP calls. `codex-api/src/endpoint/session.rs:19`

## codex-api/src/endpoint/realtime_websocket/

- `codex-api/src/endpoint/realtime_websocket/mod.rs` — Module hub re-exporting realtime WS client, connection, parser and protocol types. `codex-api/src/endpoint/realtime_websocket/mod.rs:1`
- `codex-api/src/endpoint/realtime_websocket/methods.rs` — `RealtimeWebsocketClient`: connects, authenticates, sends outbound realtime messages and dispatches parsed events to subscribers. `codex-api/src/endpoint/realtime_websocket/methods.rs:59`
- `codex-api/src/endpoint/realtime_websocket/methods_common.rs` — Adapter-layer routing of outbound realtime operations (session update, item create, handoff, context append) across V1/Frameless/V2 wire adapters. `codex-api/src/endpoint/realtime_websocket/methods_common.rs:29`
- `codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs` — Tests for cross-adapter outbound message construction (delegation ack filler, handoff channels). `codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs:16`
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs` — Builds frameless-bidi outbound messages (session context/delegation append, session update JSON, byte-limited chunks). `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs:13`
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs` — Tests for chunk size limits and session JSON encoding of initial items and delegation ack filler. `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs:11`
- `codex-api/src/endpoint/realtime_websocket/methods_v1.rs` — Builds V1 realtime outbound messages (item create, handoff append, session update session shape). `codex-api/src/endpoint/realtime_websocket/methods_v1.rs:18`
- `codex-api/src/endpoint/realtime_websocket/methods_v2.rs` — Builds RealtimeV2 outbound messages (item create, function call output, session update with tools, turn detection and transcription). `codex-api/src/endpoint/realtime_websocket/methods_v2.rs:39`
- `codex-api/src/endpoint/realtime_websocket/protocol.rs` — Realtime protocol types: parser selector, session config/mode, context-append channels, and tagged `RealtimeOutboundMessage` union. `codex-api/src/endpoint/realtime_websocket/protocol.rs:15`
- `codex-api/src/endpoint/realtime_websocket/protocol_common.rs` — Shared realtime event-parsing helpers: JSON payload extraction, session-updated, transcript delta/done and error events. `codex-api/src/endpoint/realtime_websocket/protocol_common.rs:7`
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs` — Parses frameless-bidi WebSocket events into `RealtimeEvent` (audio, transcripts, turn done, delegation created, errors). `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs:15`
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs` — Tests that legacy and frameless handoff events decode to the same `RealtimeEvent` and reuse internal event types. `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs:7`
- `codex-api/src/endpoint/realtime_websocket/protocol_v1.rs` — Parses legacy V1 realtime WebSocket events into `RealtimeEvent` (audio, transcripts, item lifecycle, handoffs). `codex-api/src/endpoint/realtime_websocket/protocol_v1.rs:12`
- `codex-api/src/endpoint/realtime_websocket/protocol_v2.rs` — Parses RealtimeV2 WebSocket events into `RealtimeEvent`, including response lifecycle and speech-started notifications. `codex-api/src/endpoint/realtime_websocket/protocol_v2.rs:24`

## codex-api/src/requests/

- `codex-api/src/requests/mod.rs` — Module hub for request translation submodules and re-export of `Compression`. `codex-api/src/requests/mod.rs:1`
- `codex-api/src/requests/chat.rs` — Translates a Responses-shaped request into an OpenAI-compatible `/chat/completions` body (system messages, tool calls, images, reasoning anchors). `codex-api/src/requests/chat.rs:19`
- `codex-api/src/requests/headers.rs` — Builds session/thread headers and subagent source headers for outbound requests. `codex-api/src/requests/headers.rs:5`
- `codex-api/src/requests/responses.rs` — Defines the `Compression` enum (None vs Zstd) for Responses requests. `codex-api/src/requests/responses.rs:1`

## codex-api/src/sse/

- `codex-api/src/sse/mod.rs` — Module hub re-exporting chat SSE spawning and Responses SSE event types. `codex-api/src/sse/mod.rs:1`
- `codex-api/src/sse/chat.rs` — Translates Chat Completions SSE frames into Responses-shaped `ResponseEvent`s, accumulating tool calls and assistant turns. `codex-api/src/sse/chat.rs:30`
- `codex-api/src/sse/responses.rs` — Spawns and processes Responses API SSE streams: emits events, parses rate limits, turn state, safety buffering and completion payloads. `codex-api/src/sse/responses.rs:36`
