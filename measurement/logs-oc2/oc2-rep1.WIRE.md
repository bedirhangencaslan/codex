# WIRE.md

## codex-api/src/

- `codex-api/src/api_bridge.rs` — maps API and transport failures into `CodexErr`, including overload, Cloudflare, cyber-policy, usage-limit, timeout, and connection cases; `codex-api/src/api_bridge.rs:21`
- `codex-api/src/api_bridge_tests.rs` — tests API-error mapping for overload, retry delays, Cloudflare blocks, and cyber-policy failures; `codex-api/src/api_bridge_tests.rs:7`
- `codex-api/src/auth.rs` — defines `AuthProvider` for header-only or request-signing authentication and associated auth telemetry; `codex-api/src/auth.rs:30`
- `codex-api/src/common.rs` — defines canonical inference wire DTOs such as `ResponseEvent`, `Reasoning`, `ResponsesApiRequest`, `ResponsesApiTools`, and WebSocket request types; `codex-api/src/common.rs:274`
- `codex-api/src/error.rs` — defines the crate's API error variants; `codex-api/src/error.rs:9`
- `codex-api/src/files.rs` — uploads hosted OpenAI files through create/upload/finalize and handles size, blob, and readiness errors; `codex-api/src/files.rs:121`
- `codex-api/src/images.rs` — defines image generation and edit request/response DTOs; `codex-api/src/images.rs:4`
- `codex-api/src/lib.rs` — declares modules and re-exports the public crate API; `codex-api/src/lib.rs:1`
- `codex-api/src/provider.rs` — configures provider endpoints and URL/request/WebSocket-URL construction, retry settings, and `WireApi::{Responses,Chat}`; `codex-api/src/provider.rs:56`
- `codex-api/src/rate_limits.rs` — parses rate-limit headers, snapshots, credits, promo/reached-type headers, and `codex.rate_limits` events; `codex-api/src/rate_limits.rs:57`
- `codex-api/src/safety_buffering.rs` — parses safety-buffering treatment headers; `codex-api/src/safety_buffering.rs:8`
- `codex-api/src/search.rs` — defines search endpoint request, response, and command DTOs; `codex-api/src/search.rs:8`
- `codex-api/src/telemetry.rs` — defines SSE/WebSocket telemetry traits and a retry-aware request telemetry wrapper; `codex-api/src/telemetry.rs:68`

## codex-api/src/endpoint/

- `codex-api/src/endpoint/mod.rs` — organizes endpoint modules and exports endpoint types; `codex-api/src/endpoint/mod.rs:1`
- `codex-api/src/endpoint/compact.rs` — posts to `responses/compact` and decodes compacted output; `codex-api/src/endpoint/compact.rs:39`
- `codex-api/src/endpoint/images.rs` — implements image generation/edit requests, including request-ID extraction; `codex-api/src/endpoint/images.rs:35`
- `codex-api/src/endpoint/memories.rs` — posts memory summaries to `memories/trace_summarize`; `codex-api/src/endpoint/memories.rs:36`
- `codex-api/src/endpoint/models.rs` — lists models with the client-version query and parses Etags; `codex-api/src/endpoint/models.rs:46`
- `codex-api/src/endpoint/responses.rs` — implements Responses/Chat-aware HTTP inference clients, session headers, routes, and streaming; `codex-api/src/endpoint/responses.rs:112`
- `codex-api/src/endpoint/search.rs` — posts typed search requests to `alpha/search`; `codex-api/src/endpoint/search.rs:35`
- `codex-api/src/endpoint/session.rs` — implements the generic endpoint session: request construction, authentication, retry telemetry, and unary/streaming execution; `codex-api/src/endpoint/session.rs:26`
- `codex-api/src/endpoint/realtime_call.rs` — creates WebRTC realtime calls from raw SDP, backend JSON, or multipart bodies and extracts call IDs; `codex-api/src/endpoint/realtime_call.rs:90`
- `codex-api/src/endpoint/responses_websocket.rs` — connects and streams Responses WebSocket requests while handling compression, headers, errors, metadata, limits, and safety buffering; `codex-api/src/endpoint/responses_websocket.rs:397`

## codex-api/src/endpoint/realtime_websocket/

- `codex-api/src/endpoint/realtime_websocket/mod.rs` — organizes realtime WebSocket modules and exports their API; `codex-api/src/endpoint/realtime_websocket/mod.rs:12`
- `codex-api/src/endpoint/realtime_websocket/protocol.rs` — defines parser variants, session configuration, outbound messages, and realtime session/audio wire DTOs; `codex-api/src/endpoint/realtime_websocket/protocol.rs:38`
- `codex-api/src/endpoint/realtime_websocket/protocol_common.rs` — provides shared realtime JSON helpers for payloads, sessions, transcripts, and errors; `codex-api/src/endpoint/realtime_websocket/protocol_common.rs:7`
- `codex-api/src/endpoint/realtime_websocket/protocol_v1.rs` — maps realtime v1 event names to protocol events; `codex-api/src/endpoint/realtime_websocket/protocol_v1.rs:12`
- `codex-api/src/endpoint/realtime_websocket/protocol_v2.rs` — maps v2 audio, transcript, item, response-lifecycle, and error events; `codex-api/src/endpoint/realtime_websocket/protocol_v2.rs:24`
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs` — maps frameless bidirectional events to realtime protocol events; `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs:15`
- `codex-api/src/endpoint/realtime_websocket/methods_common.rs` — dispatches outbound realtime messages by wire adapter; `codex-api/src/endpoint/realtime_websocket/methods_common.rs:41`
- `codex-api/src/endpoint/realtime_websocket/methods_v1.rs` — builds v1 conversation, handoff, session-update, and intent messages; `codex-api/src/endpoint/realtime_websocket/methods_v1.rs:18`
- `codex-api/src/endpoint/realtime_websocket/methods_v2.rs` — builds v2 conversation items, function outputs, and session updates with tools and modalities; `codex-api/src/endpoint/realtime_websocket/methods_v2.rs:39`
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs` — builds frameless context appends, session JSON, and byte-safe chunks; `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs:13`
- `codex-api/src/endpoint/realtime_websocket/methods.rs` — implements the realtime WebSocket client/transport, URL building, writer/event loops, transcript state, and related tests; `codex-api/src/endpoint/realtime_websocket/methods.rs:791`
- `codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs` — tests adapter-specific encoding of session updates, handoffs, channels, and function-call outputs; `codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs:15`
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs` — tests frameless context-append chunking and role-bearing initial-session JSON; `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs:10`
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs` — tests frameless delegation, transcript, and audio event decoding; `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs:6`

## codex-api/src/requests/

- `codex-api/src/requests/chat.rs` — translates `ResponsesApiRequest` into Chat Completions messages, tools, reasoning, and request fields; `codex-api/src/requests/chat.rs:19`
- `codex-api/src/requests/headers.rs` — builds session/thread headers and subagent header values; `codex-api/src/requests/headers.rs:5`
- `codex-api/src/requests/mod.rs` — declares request translator modules and exports `Compression`; `codex-api/src/requests/mod.rs:1`
- `codex-api/src/requests/responses.rs` — defines `Compression::{None,Zstd}`; `codex-api/src/requests/responses.rs:1`

## codex-api/src/sse/

- `codex-api/src/sse/chat.rs` — translates Chat Completions SSE deltas into Responses-shaped `ResponseEvent`s, including tool, reasoning, and usage handling; `codex-api/src/sse/chat.rs:30`
- `codex-api/src/sse/mod.rs` — declares SSE modules and exports stream processors; `codex-api/src/sse/mod.rs:1`
- `codex-api/src/sse/responses.rs` — parses Responses SSE and header metadata into `ResponseEvent`s and error classifications; `codex-api/src/sse/responses.rs:36`
