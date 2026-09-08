# codex-api wire map

## `codex-api/src`

- `codex-api/src/lib.rs` — declares the crate modules and public API re-exports for clients, auth, provider, requests, SSE, telemetry, and endpoint types (`codex-api/src/lib.rs:1`).
- `codex-api/src/api_bridge.rs` — maps API and transport errors into `CodexErr`, preserving retry delays, rate/usage limits, policy violations, Cloudflare blocks, and request IDs (`codex-api/src/api_bridge.rs:21`).
- `codex-api/src/api_bridge_tests.rs` — tests `map_api_error` handling of overload, retry delays, Cloudflare, policy, usage, and generic HTTP errors (`codex-api/src/api_bridge_tests.rs:7`).
- `codex-api/src/auth.rs` — defines the auth-provider interface for static or async header auth, full request signing, shared handles, and auth telemetry (`codex-api/src/auth.rs:30`).
- `codex-api/src/common.rs` — defines shared request and event wire types, Responses/WebSocket request shapes, reasoning and text controls, tracing metadata, and `ResponseStream` (`codex-api/src/common.rs:97`).
- `codex-api/src/error.rs` — defines the crate-wide `ApiError` taxonomy and conversion to rate-limit errors (`codex-api/src/error.rs:8`).
- `codex-api/src/files.rs` — handles the multi-step OpenAI file upload wire flow: create, authorized blob upload, finalize, retry until ready, and canonical file metadata (`codex-api/src/files.rs:121`).
- `codex-api/src/images.rs` — models image generation/edit request and response payloads (`codex-api/src/images.rs:4`).
- `codex-api/src/rate_limits.rs` — parses rate-limit and credits headers for all known limit IDs, plus WebSocket rate-limit events, promo messages, and reached-type metadata (`codex-api/src/rate_limits.rs:57`).
- `codex-api/src/provider.rs` — configures endpoint base URLs, headers, query parameters, retries, idle timeout, HTTP/WebSocket URL construction, wire protocol, and Azure detection (`codex-api/src/provider.rs:56`).
- `codex-api/src/safety_buffering.rs` — reads safety-buffering treatment headers into the internal treatment type (`codex-api/src/safety_buffering.rs:8`).
- `codex-api/src/search.rs` — models the search request, commands, settings, input, and response wire payloads (`codex-api/src/search.rs:8`).
- `codex-api/src/telemetry.rs` — defines SSE and WebSocket telemetry hooks and wraps retry execution with per-attempt request telemetry (`codex-api/src/telemetry.rs:68`).

## `codex-api/src/endpoint`

- `codex-api/src/endpoint/mod.rs` — declares endpoint modules and re-exports their public client and connection types (`codex-api/src/endpoint/mod.rs:1`).
- `codex-api/src/endpoint/compact.rs` — posts compaction requests to `responses/compact`, captures turn-state headers, and decodes compacted `ResponseItem` output (`codex-api/src/endpoint/compact.rs:39`).
- `codex-api/src/endpoint/images.rs` — posts image generation and edit requests, captures the imagegen request ID, and decodes image responses (`codex-api/src/endpoint/images.rs:35`).
- `codex-api/src/endpoint/memories.rs` — posts memory summaries to `memories/trace_summarize` and decodes summarized output (`codex-api/src/endpoint/memories.rs:36`).
- `codex-api/src/endpoint/models.rs` — requests the model list with a client-version query and decodes models plus the ETag header (`codex-api/src/endpoint/models.rs:46`).
- `codex-api/src/endpoint/responses.rs` — dispatches Responses-compatible inference over HTTP, translating Chat-provider bodies and choosing the corresponding SSE decoder (`codex-api/src/endpoint/responses.rs:112`).
- `codex-api/src/endpoint/responses_websocket.rs` — manages persistent Responses WebSocket connections, serializes response-create requests, and pumps events into the shared response stream (`codex-api/src/endpoint/responses_websocket.rs:235`).
- `codex-api/src/endpoint/realtime_call.rs` — creates WebRTC realtime calls with SDP and optional multipart session config, parses call IDs and SDP responses (`codex-api/src/endpoint/realtime_call.rs:90`).
- `codex-api/src/endpoint/search.rs` — posts `SearchRequest` to `alpha/search` and decodes `SearchResponse` (`codex-api/src/endpoint/search.rs:35`).
- `codex-api/src/endpoint/session.rs` — centralizes endpoint request construction, auth, extra headers, retries, request telemetry, JSON execution, and streaming (`codex-api/src/endpoint/session.rs:80`).

## `codex-api/src/endpoint/realtime_websocket`

- `codex-api/src/endpoint/realtime_websocket/mod.rs` — declares realtime protocol and method modules and re-exports realtime client/event/config types (`codex-api/src/endpoint/realtime_websocket/mod.rs:1`).
- `codex-api/src/endpoint/realtime_websocket/protocol.rs` — selects realtime parser variants and defines session config plus outbound realtime message variants (`codex-api/src/endpoint/realtime_websocket/protocol.rs:15`).
- `codex-api/src/endpoint/realtime_websocket/protocol_common.rs` — provides shared JSON payload parsing for session updates, transcript deltas/done events, and errors (`codex-api/src/endpoint/realtime_websocket/protocol_common.rs:7`).
- `codex-api/src/endpoint/realtime_websocket/protocol_v1.rs` — translates realtime v1 WebSocket events into `RealtimeEvent` (`codex-api/src/endpoint/realtime_websocket/protocol_v1.rs:12`).
- `codex-api/src/endpoint/realtime_websocket/protocol_v2.rs` — translates realtime v2 WebSocket events, audio, transcripts, response lifecycle, and errors (`codex-api/src/endpoint/realtime_websocket/protocol_v2.rs:24`).
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs` — translates frameless bidirectional WebSocket events into shared realtime events (`codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs:15`).
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs` — verifies frameless delegation, transcript, audio, and handoff decoding compatibility (`codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs:6`).
- `codex-api/src/endpoint/realtime_websocket/methods.rs` — implements the realtime WebSocket client, connection pump, parser selection, sends, receives, and transcript state (`codex-api/src/endpoint/realtime_websocket/methods.rs:64`).
- `codex-api/src/endpoint/realtime_websocket/methods_common.rs` — routes adapter-neutral operations to v1, v2, or frameless-bidi outbound message constructors (`codex-api/src/endpoint/realtime_websocket/methods_common.rs:29`).
- `codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs` — verifies adapter-specific session-update and context/channel serialization (`codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs:15`).
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs` — builds frameless context appends, delegation appends, session updates, and byte-limited chunks (`codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs:13`).
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs` — tests frameless chunk preservation and initial-item/session JSON encoding (`codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs:10`).
- `codex-api/src/endpoint/realtime_websocket/methods_v1.rs` — builds realtime v1 conversation items, handoff appends, quicksilver session updates, and WebSocket intent (`codex-api/src/endpoint/realtime_websocket/methods_v1.rs:18`).
- `codex-api/src/endpoint/realtime_websocket/methods_v2.rs` — builds realtime v2 conversation items, function-call outputs, and detailed session-update payloads (`codex-api/src/endpoint/realtime_websocket/methods_v2.rs:39`).

## `codex-api/src/requests`

- `codex-api/src/requests/mod.rs` — declares request translation modules and re-exports compression (`codex-api/src/requests/mod.rs:1`).
- `codex-api/src/requests/chat.rs` — translates Responses-shaped requests into OpenAI-compatible Chat Completions bodies (`codex-api/src/requests/chat.rs:19`).
- `codex-api/src/requests/headers.rs` — builds session headers and derives subagent and generic HTTP headers (`codex-api/src/requests/headers.rs:5`).
- `codex-api/src/requests/responses.rs` — defines optional zstd request compression (`codex-api/src/requests/responses.rs:1`).

## `codex-api/src/sse`

- `codex-api/src/sse/mod.rs` — declares SSE modules and re-exports Chat and Responses stream helpers (`codex-api/src/sse/mod.rs:1`).
- `codex-api/src/sse/chat.rs` — translates Chat Completions SSE deltas and tool-call chunks into accumulated Responses-shaped events (`codex-api/src/sse/chat.rs:30`).
- `codex-api/src/sse/responses.rs` — parses Responses SSE headers/events into shared response events, metadata, errors, rate limits, and safety-buffering state (`codex-api/src/sse/responses.rs:36`).
