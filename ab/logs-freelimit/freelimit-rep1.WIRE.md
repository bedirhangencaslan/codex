# codex-api wire map

## codex-api/src

- `codex-api/src/api_bridge.rs` — translates API and transport failures, policy codes, usage limits, and identity metadata into protocol-facing errors (`codex-api/src/api_bridge.rs:21`).
- `codex-api/src/api_bridge_tests.rs` — tests error mapping for overload, retry, policy, usage-limit, and identity-auth responses (`codex-api/src/api_bridge_tests.rs:8`).
- `codex-api/src/auth.rs` — defines the outbound authentication trait and how credentials become request headers (`codex-api/src/auth.rs:30`).
- `codex-api/src/common.rs` — defines shared Responses wire payloads, response events, reasoning/text structures, and tool DTOs (`codex-api/src/common.rs:274`).
- `codex-api/src/error.rs` — defines the API-layer error variants returned across HTTP, SSE, and WebSocket transports (`codex-api/src/error.rs:9`).
- `codex-api/src/files.rs` — handles hosted-file uploads, including the Azure blob PUT upload path (`codex-api/src/files.rs:121`).
- `codex-api/src/images.rs` — defines image generation and edit request/response wire DTOs (`codex-api/src/images.rs:5`).
- `codex-api/src/lib.rs` — declares the crate modules and its public API surface (`codex-api/src/lib.rs:1`).
- `codex-api/src/provider.rs` — models provider base URLs, retry policy, wire protocol selection, and Azure detection (`codex-api/src/provider.rs:56`).
- `codex-api/src/rate_limits.rs` — parses rate-limit headers and `codex.rate_limits` payloads into snapshots (`codex-api/src/rate_limits.rs:57`).
- `codex-api/src/safety_buffering.rs` — parses safety-buffering headers into treatment settings (`codex-api/src/safety_buffering.rs:8`).
- `codex-api/src/search.rs` — defines web search request, response, operation, and settings DTOs (`codex-api/src/search.rs:9`).
- `codex-api/src/telemetry.rs` — defines SSE/WS telemetry hooks and a retry-aware telemetry wrapper (`codex-api/src/telemetry.rs:68`).

## codex-api/src/endpoint

- `codex-api/src/endpoint/compact.rs` — POSTs `responses/compact` and decodes the compacted response (`codex-api/src/endpoint/compact.rs:39`).
- `codex-api/src/endpoint/images.rs` — POSTs image generation and edit routes and captures request IDs (`codex-api/src/endpoint/images.rs:35`).
- `codex-api/src/endpoint/memories.rs` — POSTs the `memories/trace_summarize` endpoint (`codex-api/src/endpoint/memories.rs:36`).
- `codex-api/src/endpoint/mod.rs` — declares and re-exports the endpoint modules (`codex-api/src/endpoint/mod.rs:12`).
- `codex-api/src/endpoint/models.rs` — GETs model listings, constructs version queries, and captures ETags (`codex-api/src/endpoint/models.rs:46`).
- `codex-api/src/endpoint/realtime_call.rs` — creates WebRTC realtime calls from SDP/session data using JSON or multipart bodies (`codex-api/src/endpoint/realtime_call.rs:129`).
- `codex-api/src/endpoint/responses.rs` — performs Responses HTTP/SSE requests, including Chat-provider wire translation (`codex-api/src/endpoint/responses.rs:112`).
- `codex-api/src/endpoint/responses_websocket.rs` — connects/probes Responses WebSocket sessions and translates request, event, and error streams (`codex-api/src/endpoint/responses_websocket.rs:235`).
- `codex-api/src/endpoint/search.rs` — POSTs the `alpha/search` endpoint (`codex-api/src/endpoint/search.rs:35`).
- `codex-api/src/endpoint/session.rs` — provides the shared endpoint request path for auth, retries, and stream handling (`codex-api/src/endpoint/session.rs:80`).

## codex-api/src/endpoint/realtime_websocket

- `codex-api/src/endpoint/realtime_websocket/methods.rs` — manages realtime WebSocket clients, connections, outbound writes, inbound events, and transcript state (`codex-api/src/endpoint/realtime_websocket/methods.rs:791`).
- `codex-api/src/endpoint/realtime_websocket/methods_common.rs` — routes outbound realtime message construction by wire adapter (`codex-api/src/endpoint/realtime_websocket/methods_common.rs:41`).
- `codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs` — tests adapter-specific outbound realtime messages (`codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs:16`).
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs` — builds Frameless Bidi session/context messages and byte-bounded append chunks (`codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs:13`).
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs` — tests Frameless Bidi chunk limits and session payloads (`codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs:11`).
- `codex-api/src/endpoint/realtime_websocket/methods_v1.rs` — builds Realtime v1 outbound messages and session updates (`codex-api/src/endpoint/realtime_websocket/methods_v1.rs:18`).
- `codex-api/src/endpoint/realtime_websocket/methods_v2.rs` — builds Realtime v2 outbound items and session configurations (`codex-api/src/endpoint/realtime_websocket/methods_v2.rs:39`).
- `codex-api/src/endpoint/realtime_websocket/mod.rs` — declares and re-exports the realtime WebSocket modules (`codex-api/src/endpoint/realtime_websocket/mod.rs:12`).
- `codex-api/src/endpoint/realtime_websocket/protocol.rs` — defines realtime parser/session/outbound wire types and event dispatch (`codex-api/src/endpoint/realtime_websocket/protocol.rs:37`).
- `codex-api/src/endpoint/realtime_websocket/protocol_common.rs` — provides shared realtime JSON/event parsing helpers (`codex-api/src/endpoint/realtime_websocket/protocol_common.rs:7`).
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs` — parses Frameless Bidi realtime events (`codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs:15`).
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs` — tests decoding Frameless Bidi delegation, transcript, and audio events (`codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs:7`).
- `codex-api/src/endpoint/realtime_websocket/protocol_v1.rs` — parses Realtime v1 events (`codex-api/src/endpoint/realtime_websocket/protocol_v1.rs:12`).
- `codex-api/src/endpoint/realtime_websocket/protocol_v2.rs` — parses Realtime v2 events (`codex-api/src/endpoint/realtime_websocket/protocol_v2.rs:24`).

## codex-api/src/requests

- `codex-api/src/requests/chat.rs` — translates Responses requests into OpenAI-compatible Chat Completions bodies (`codex-api/src/requests/chat.rs:19`).
- `codex-api/src/requests/headers.rs` — builds session/thread and subagent request headers (`codex-api/src/requests/headers.rs:5`).
- `codex-api/src/requests/mod.rs` — declares request modules and re-exports compression selection (`codex-api/src/requests/mod.rs:1`).
- `codex-api/src/requests/responses.rs` — defines request compression choices (`codex-api/src/requests/responses.rs:1`).

## codex-api/src/sse

- `codex-api/src/sse/chat.rs` — translates Chat Completions SSE frames into Responses-shaped event streams (`codex-api/src/sse/chat.rs:30`).
- `codex-api/src/sse/mod.rs` — declares SSE modules and re-exports chat/response stream handling (`codex-api/src/sse/mod.rs:1`).
- `codex-api/src/sse/responses.rs` — spawns and processes Responses SSE streams, extracting events, metadata, usage, limits, and errors (`codex-api/src/sse/responses.rs:36`).
