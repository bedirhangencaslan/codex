# codex-api wire map

### `codex-api/src`

- `codex-api/src/api_bridge.rs` — translates API and transport failures into protocol-facing `ApiError` cases for limits, policy, overload, and identity metadata (`api_bridge.rs:21`).
- `codex-api/src/api_bridge_tests.rs` — tests error mapping for overload, Cloudflare, policy, usage limits, and identity metadata (`api_bridge_tests.rs:8`).
- `codex-api/src/auth.rs` — defines request-auth traits and providers for header-only and request-signing auth (`auth.rs:30`).
- `codex-api/src/common.rs` — defines canonical Responses HTTP/WebSocket request types, `ResponseEvent`, compaction, reasoning/text controls, and trace metadata (`common.rs:275`).
- `codex-api/src/error.rs` — defines shared `ApiError` transport, retry, policy, quota, and stream variants (`error.rs:9`).
- `codex-api/src/files.rs` — implements the create/upload/finalize flow for hosted files (`files.rs:121`).
- `codex-api/src/images.rs` — defines image generation/edit wire requests and parsed responses (`images.rs:5`).
- `codex-api/src/lib.rs` — builds the crate module tree and public re-exports (`lib.rs:16`).
- `codex-api/src/provider.rs` — configures provider URLs, headers, retry, wire API, and request/websocket builders (`provider.rs:56`).
- `codex-api/src/rate_limits.rs` — parses rate-limit headers/events, credits, promotions, and reached-state metadata (`rate_limits.rs:57`).
- `codex-api/src/safety_buffering.rs` — derives safety-buffering treatment and faster-model hints from headers (`safety_buffering.rs:8`).
- `codex-api/src/search.rs` — defines search commands, settings, and response DTOs (`search.rs:9`).
- `codex-api/src/telemetry.rs` — provides SSE/WebSocket telemetry traits and retry-wrapped request telemetry (`telemetry.rs:68`).

### `codex-api/src/endpoint`

- `codex-api/src/endpoint/compact.rs` — posts `responses/compact`, parses compact history, and captures turn state (`compact.rs:39`).
- `codex-api/src/endpoint/images.rs` — posts `images/generations` and `images/edits` and parses results/request IDs (`images.rs:35`).
- `codex-api/src/endpoint/memories.rs` — posts `memories/trace_summarize` and parses summaries (`memories.rs:36`).
- `codex-api/src/endpoint/mod.rs` — registers and re-exports endpoint modules (`mod.rs:12`).
- `codex-api/src/endpoint/models.rs` — gets `models` with client-version query and ETag handling (`models.rs:46`).
- `codex-api/src/endpoint/realtime_call.rs` — creates WebRTC Realtime calls from SDP and parses call IDs (`realtime_call.rs:129`).
- `codex-api/src/endpoint/responses.rs` — streams Responses over HTTP, rewrites requests to Chat where needed, and selects SSE parsers (`responses.rs:127`).
- `codex-api/src/endpoint/responses_websocket.rs` — opens/serializes Responses WebSocket requests, streams JSON frames into `ResponseEvent`, and maps wrapped errors/safety metadata (`responses_websocket.rs:817`).
- `codex-api/src/endpoint/search.rs` — posts `alpha/search` and parses the response (`search.rs:35`).
- `codex-api/src/endpoint/session.rs` — provides shared HTTP execute/stream plumbing with auth, retry, and telemetry (`session.rs:80`).

### `codex-api/src/endpoint/realtime_websocket`

- `codex-api/src/endpoint/realtime_websocket/methods.rs` — manages realtime WS connections, outbound JSON methods, event dispatch, sideband URLs, retries, and bounded transcript state (`methods.rs:791`).
- `codex-api/src/endpoint/realtime_websocket/methods_common.rs` — dispatches outbound session/update/context/handoff calls by realtime wire adapter (`methods_common.rs:41`).
- `codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs` — tests adapter-specific session updates, context channels, and handoff output encoding (`methods_common_tests.rs:15`).
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs` — builds Frameless Bidi session/context messages and chunks long appends (`methods_frameless_bidi.rs:13`).
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs` — tests byte-limited chunks and initial item encoding (`methods_frameless_bidi_tests.rs:10`).
- `codex-api/src/endpoint/realtime_websocket/methods_v1.rs` — builds Realtime V1 conversation, handoff, session, and intent payloads (`methods_v1.rs:18`).
- `codex-api/src/endpoint/realtime_websocket/methods_v2.rs` — builds Realtime V2 conversation/function/session payloads and background/silence tools (`methods_v2.rs:39`).
- `codex-api/src/endpoint/realtime_websocket/mod.rs` — declares and re-exports realtime websocket modules and public types (`mod.rs:1`).
- `codex-api/src/endpoint/realtime_websocket/protocol.rs` — defines realtime outbound/message/session wire types and routes inbound events to protocol parsers (`protocol.rs:14`).
- `codex-api/src/endpoint/realtime_websocket/protocol_common.rs` — shares JSON event parsing and common session/transcript/error extraction (`protocol_common.rs:7`).
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs` — parses Frameless Bidi audio, transcript, turn, delegation, and error events (`protocol_frameless_bidi.rs:15`).
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs` — verifies Frameless events decode to the shared realtime event model (`protocol_frameless_bidi_tests.rs:6`).
- `codex-api/src/endpoint/realtime_websocket/protocol_v1.rs` — parses Realtime V1 audio, transcript, items, handoffs, and errors (`protocol_v1.rs:12`).
- `codex-api/src/endpoint/realtime_websocket/protocol_v2.rs` — parses Realtime V2 responses, audio, transcripts, background handoffs, no-ops, and errors (`protocol_v2.rs:24`).

### `codex-api/src/requests`

- `codex-api/src/requests/chat.rs` — translates Responses requests/history into Chat Completions bodies, tools, reasoning, and text format (`chat.rs:18`).
- `codex-api/src/requests/headers.rs` — builds session/thread headers and derives subagent labels (`headers.rs:5`).
- `codex-api/src/requests/mod.rs` — exports request modules and `Compression` (`mod.rs:1`).
- `codex-api/src/requests/responses.rs` — selects request compression, none or Zstd (`responses.rs:1`).

### `codex-api/src/sse`

- `codex-api/src/sse/chat.rs` — translates Chat Completions SSE deltas into assembled Responses items, reasoning, tool calls, usage, and completion events (`chat.rs:30`).
- `codex-api/src/sse/mod.rs` — registers/re-exports Responses and Chat SSE parsers (`mod.rs:1`).
- `codex-api/src/sse/responses.rs` — parses Responses SSE and headers into `ResponseEvent`, including metadata, errors, usage, items, reasoning, and safety buffering (`responses.rs:36`).
