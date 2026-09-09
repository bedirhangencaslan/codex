# Codex API wire coverage

## `codex-api/src/`

- `codex-api/src/api_bridge.rs` — maps `ApiError` and transport errors into `CodexErr`, including overloaded, policy, invalid-image, usage-limit, and header diagnostics (`codex-api/src/api_bridge.rs:21`).
- `codex-api/src/api_bridge_tests.rs` — tests error mapping for overload, retry delay, Cloudflare, cyber policy, misalignment, usage limits, and identity headers (`codex-api/src/api_bridge_tests.rs:8`).
- `codex-api/src/auth.rs` — defines the `AuthProvider` trait for header and full-request auth plus auth telemetry (`codex-api/src/auth.rs:30`).
- `codex-api/src/common.rs` — defines shared Responses wire payloads, events, WebSocket request shapes, and `ResponseStream` (`codex-api/src/common.rs:98`).
- `codex-api/src/error.rs` — defines `ApiError` variants used across HTTP and streaming clients (`codex-api/src/error.rs:9`).
- `codex-api/src/files.rs` — implements OpenAI hosted-file upload via create, blob PUT, and finalize polling (`codex-api/src/files.rs:121`).
- `codex-api/src/images.rs` — defines image generation/edit requests and image responses (`codex-api/src/images.rs:5`).
- `codex-api/src/lib.rs` — declares crate modules and public re-export surface (`codex-api/src/lib.rs:1`).
- `codex-api/src/provider.rs` — defines provider endpoint configuration, `WireApi::Responses` vs `Chat`, URL/request building, and Azure detection (`codex-api/src/provider.rs:56`).
- `codex-api/src/rate_limits.rs` — parses rate-limit headers and `codex.rate_limits` events into snapshots (`codex-api/src/rate_limits.rs:57`).
- `codex-api/src/safety_buffering.rs` — reads safety-buffering treatment headers (`codex-api/src/safety_buffering.rs:8`).
- `codex-api/src/search.rs` — defines search request commands/settings and response payload (`codex-api/src/search.rs:9`).
- `codex-api/src/telemetry.rs` — defines SSE/WebSocket telemetry hooks and retry/request telemetry wrapper (`codex-api/src/telemetry.rs:68`).

## `codex-api/src/endpoint/`

- `codex-api/src/endpoint/mod.rs` — module/export hub for endpoint clients (`codex-api/src/endpoint/mod.rs:1`).
- `codex-api/src/endpoint/compact.rs` — posts to `responses/compact`, parses compacted output, and captures turn state (`codex-api/src/endpoint/compact.rs:39`).
- `codex-api/src/endpoint/images.rs` — posts image generation/edit requests and parses responses plus imagegen request ID (`codex-api/src/endpoint/images.rs:58`).
- `codex-api/src/endpoint/memories.rs` — posts memory trace summaries to `memories/trace_summarize` (`codex-api/src/endpoint/memories.rs:36`).
- `codex-api/src/endpoint/models.rs` — lists models with `client_version` query and extracts `ETag` (`codex-api/src/endpoint/models.rs:46`).
- `codex-api/src/endpoint/realtime_call.rs` — creates WebRTC realtime calls using raw SDP, backend JSON, or multipart session bodies and parses call IDs (`codex-api/src/endpoint/realtime_call.rs:129`).
- `codex-api/src/endpoint/responses.rs` — HTTP Responses streaming client; selects Responses/Guardian routes and translates Chat-only providers (`codex-api/src/endpoint/responses.rs:112`).
- `codex-api/src/endpoint/responses_websocket.rs` — connects/probes Responses WebSocket, serializes requests, and streams/maps metadata, errors, rate limits, safety buffering, and completion events (`codex-api/src/endpoint/responses_websocket.rs:685`).
- `codex-api/src/endpoint/search.rs` — posts typed search requests to `alpha/search` and parses results (`codex-api/src/endpoint/search.rs:35`).
- `codex-api/src/endpoint/session.rs` — generic `EndpointSession` for authenticated, retried, telemetry-aware unary and streaming requests (`codex-api/src/endpoint/session.rs:80`).

## `codex-api/src/endpoint/realtime_websocket/`

- `codex-api/src/endpoint/realtime_websocket/mod.rs` — organizes realtime WebSocket modules and exports client, connection, parser, and session types (`codex-api/src/endpoint/realtime_websocket/mod.rs:1`).
- `codex-api/src/endpoint/realtime_websocket/methods.rs` — manages realtime WebSocket connections, transport/retries, outbound writes, inbound events, URLs, and transcript state (`codex-api/src/endpoint/realtime_websocket/methods.rs:791`).
- `codex-api/src/endpoint/realtime_websocket/methods_common.rs` — dispatches outbound session, conversation, handoff, and function-output messages across V1, Frameless Bidi, and Realtime V2 adapters (`codex-api/src/endpoint/realtime_websocket/methods_common.rs:41`).
- `codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs` — tests adapter-specific encoding of session updates, handoffs, and function outputs (`codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs:15`).
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs` — builds Frameless Bidi session/context messages and splits context text into wire-safe chunks (`codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs:25`).
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs` — tests context chunking and Frameless Bidi session JSON (`codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs:10`).
- `codex-api/src/endpoint/realtime_websocket/methods_v1.rs` — builds V1 realtime conversation items, handoff appends, quicksilver sessions, and WebSocket intent (`codex-api/src/endpoint/realtime_websocket/methods_v1.rs:18`).
- `codex-api/src/endpoint/realtime_websocket/methods_v2.rs` — builds Realtime V2 conversation/function-call messages and session configurations, including tools and transcription modes (`codex-api/src/endpoint/realtime_websocket/methods_v2.rs:75`).
- `codex-api/src/endpoint/realtime_websocket/protocol.rs` — defines realtime wire adapters, session modes/config, outbound messages, and parser dispatch (`codex-api/src/endpoint/realtime_websocket/protocol.rs:14`).
- `codex-api/src/endpoint/realtime_websocket/protocol_common.rs` — provides common realtime event decoding for payloads, sessions, transcripts, and errors (`codex-api/src/endpoint/realtime_websocket/protocol_common.rs:7`).
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs` — parses Frameless Bidi session, audio, transcript, turn, delegation, and error events (`codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs:15`).
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs` — tests Frameless Bidi compatibility for handoffs, transcripts, and audio (`codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs:6`).
- `codex-api/src/endpoint/realtime_websocket/protocol_v1.rs` — parses legacy realtime V1 audio, transcript, item, handoff, and error events (`codex-api/src/endpoint/realtime_websocket/protocol_v1.rs:12`).
- `codex-api/src/endpoint/realtime_websocket/protocol_v2.rs` — parses Realtime V2 audio, transcript, response, item, handoff, noop, and error events (`codex-api/src/endpoint/realtime_websocket/protocol_v2.rs:24`).

## `codex-api/src/requests/`

- `codex-api/src/requests/mod.rs` — organizes request modules and re-exports the compression setting (`codex-api/src/requests/mod.rs:1`).
- `codex-api/src/requests/chat.rs` — translates Responses requests, items, tools, reasoning, and output settings into Chat Completions bodies (`codex-api/src/requests/chat.rs:19`).
- `codex-api/src/requests/headers.rs` — builds session/thread headers and maps subagent sources to request headers (`codex-api/src/requests/headers.rs:5`).
- `codex-api/src/requests/responses.rs` — defines the Responses request compression choice (`codex-api/src/requests/responses.rs:1`).

## `codex-api/src/sse/`

- `codex-api/src/sse/mod.rs` — organizes SSE modules and exports Chat/Responses stream entry points (`codex-api/src/sse/mod.rs:1`).
- `codex-api/src/sse/chat.rs` — translates Chat Completions SSE deltas, tool calls, reasoning, errors, and usage into Responses events (`codex-api/src/sse/chat.rs:30`).
- `codex-api/src/sse/responses.rs` — parses Responses SSE headers and events into response events, usage, metadata, errors, and safety/rate-limit state (`codex-api/src/sse/responses.rs:36`).
