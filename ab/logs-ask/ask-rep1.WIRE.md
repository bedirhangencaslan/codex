# codex-api wire modules

## codex-api/src/

- `codex-api/src/api_bridge.rs` — maps API/HTTP transport failures into protocol-facing `ApiError`s, including usage limits, policy blocks, overload, and request IDs. `codex-api/src/api_bridge.rs:21`
- `codex-api/src/api_bridge_tests.rs` — covers API error mapping for overload/retry, Cloudflare, cyber-policy, and misalignment responses. `codex-api/src/api_bridge_tests.rs:8`
- `codex-api/src/auth.rs` — defines authentication providers, asynchronous credential/header resolution, and request authentication behavior. `codex-api/src/auth.rs:30`
- `codex-api/src/common.rs` — defines shared request/response wire payloads and the normalized `ResponseEvent` inference stream. `codex-api/src/common.rs:98`
- `codex-api/src/error.rs` — defines the crate's `ApiError` variants, including transport, rate-limit, context-window, and policy errors. `codex-api/src/error.rs:9`
- `codex-api/src/files.rs` — uploads files through the OpenAI file API or Azure blob URL, then validates readiness and reports upload errors. `codex-api/src/files.rs:121`
- `codex-api/src/images.rs` — declares image generation and edit request/response wire DTOs. `codex-api/src/images.rs:5`
- `codex-api/src/lib.rs` — declares the crate's modules and its public API surface. `codex-api/src/lib.rs:1`
- `codex-api/src/provider.rs` — configures provider endpoints, retries, HTTP/WebSocket URLs, Responses-vs-Chat wire API, and Azure behavior. `codex-api/src/provider.rs:44`
- `codex-api/src/rate_limits.rs` — parses rate-limit and credit headers/events into normalized rate-limit snapshots. `codex-api/src/rate_limits.rs:57`
- `codex-api/src/safety_buffering.rs` — extracts safety-buffering treatment and faster-model fallback information from response headers. `codex-api/src/safety_buffering.rs:8`
- `codex-api/src/search.rs` — defines the search API's request, command, settings, filter, location, and response wire types. `codex-api/src/search.rs:9`
- `codex-api/src/telemetry.rs` — defines SSE/WebSocket telemetry hooks and a retrying request wrapper with per-attempt telemetry. `codex-api/src/telemetry.rs:18`

## codex-api/src/endpoint/

- `codex-api/src/endpoint/compact.rs` — posts compact requests, captures turn state, and returns compacted response items. `codex-api/src/endpoint/compact.rs:39`
- `codex-api/src/endpoint/images.rs` — posts image generation/edit requests and extracts the image generation request ID. `codex-api/src/endpoint/images.rs:35`
- `codex-api/src/endpoint/memories.rs` — posts memory traces for summarization and parses the resulting memory summaries. `codex-api/src/endpoint/memories.rs:36`
- `codex-api/src/endpoint/mod.rs` — organizes and re-exports endpoint implementations. `codex-api/src/endpoint/mod.rs:1`
- `codex-api/src/endpoint/models.rs` — fetches the model catalog with client version and returns models plus cache ETag. `codex-api/src/endpoint/models.rs:46`
- `codex-api/src/endpoint/realtime_call.rs` — creates WebRTC realtime calls with multipart/JSON session configuration and extracts call IDs. `codex-api/src/endpoint/realtime_call.rs:90`
- `codex-api/src/endpoint/responses.rs` — executes Responses streaming requests, bridging Chat-only providers and dispatching SSE parsing. `codex-api/src/endpoint/responses.rs:112`
- `codex-api/src/endpoint/responses_websocket.rs` — manages Responses WebSocket connections, request dispatch, headers, turn state, timing, and probing. `codex-api/src/endpoint/responses_websocket.rs:235`
- `codex-api/src/endpoint/search.rs` — posts typed search requests to the alpha search endpoint and parses responses. `codex-api/src/endpoint/search.rs:35`
- `codex-api/src/endpoint/session.rs` — executes authenticated unary and streaming endpoint calls with retry and telemetry. `codex-api/src/endpoint/session.rs:63`

## codex-api/src/endpoint/realtime_websocket/

- `codex-api/src/endpoint/realtime_websocket/mod.rs` — organizes realtime WebSocket modules and re-exports their public items. `codex-api/src/endpoint/realtime_websocket/mod.rs:1`
- `codex-api/src/endpoint/realtime_websocket/protocol.rs` — defines realtime protocol state, outbound messages, audio/session structures, and event parsing dispatch. `codex-api/src/endpoint/realtime_websocket/protocol.rs:52`
- `codex-api/src/endpoint/realtime_websocket/methods.rs` — implements realtime WebSocket client/connection/writer behavior, event pumping, and lifecycle. `codex-api/src/endpoint/realtime_websocket/methods.rs:211`
- `codex-api/src/endpoint/realtime_websocket/methods_common.rs` — routes outbound realtime message construction to the V1, Frameless Bidi, or V2 adapter. `codex-api/src/endpoint/realtime_websocket/methods_common.rs:41`
- `codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs` — tests adapter-specific session updates, handoffs, context channels, and function outputs. `codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs:16`
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs` — builds Frameless Bidi context/session messages and chunks long text for transmission. `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs:13`
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs` — tests Frameless Bidi chunk boundaries, session JSON, and initial items. `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs:11`
- `codex-api/src/endpoint/realtime_websocket/methods_v1.rs` — constructs legacy realtime V1 outbound messages and session configuration. `codex-api/src/endpoint/realtime_websocket/methods_v1.rs:18`
- `codex-api/src/endpoint/realtime_websocket/methods_v2.rs` — constructs Realtime V2 conversation, tool, session, and control messages. `codex-api/src/endpoint/realtime_websocket/methods_v2.rs:39`
- `codex-api/src/endpoint/realtime_websocket/protocol_common.rs` — provides common JSON parsing helpers for realtime sessions, transcripts, and errors. `codex-api/src/endpoint/realtime_websocket/protocol_common.rs:7`
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs` — parses Frameless Bidi audio, transcript, turn, delegation, and error events. `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs:15`
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs` — tests Frameless Bidi handoff event normalization against legacy behavior. `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs:7`
- `codex-api/src/endpoint/realtime_websocket/protocol_v1.rs` — parses legacy realtime V1 audio, transcript, item, handoff, and error events. `codex-api/src/endpoint/realtime_websocket/protocol_v1.rs:12`
- `codex-api/src/endpoint/realtime_websocket/protocol_v2.rs` — parses Realtime V2 response, audio, transcript, tool, handoff, and noop events. `codex-api/src/endpoint/realtime_websocket/protocol_v2.rs:24`

## codex-api/src/requests/

- `codex-api/src/requests/chat.rs` — translates Responses-shaped history, tools, reasoning, output settings, and calls into Chat Completions bodies. `codex-api/src/requests/chat.rs:19`
- `codex-api/src/requests/headers.rs` — builds session/thread headers and derives the subagent telemetry header. `codex-api/src/requests/headers.rs:5`
- `codex-api/src/requests/mod.rs` — organizes request modules and exposes the compression option. `codex-api/src/requests/mod.rs:1`
- `codex-api/src/requests/responses.rs` — defines the request compression mode used by Responses transport. `codex-api/src/requests/responses.rs:1`

## codex-api/src/sse/

- `codex-api/src/sse/chat.rs` — parses Chat Completions SSE deltas and rebuilds normalized Responses stream events. `codex-api/src/sse/chat.rs:30`
- `codex-api/src/sse/mod.rs` — organizes SSE parsers and exports response stream entry points. `codex-api/src/sse/mod.rs:1`
- `codex-api/src/sse/responses.rs` — parses Responses SSE and WebSocket metadata events into normalized response events, errors, usage, and turn metadata. `codex-api/src/sse/responses.rs:36`
