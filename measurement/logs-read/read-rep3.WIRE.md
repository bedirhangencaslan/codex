# Wire

## codex-api/src

- `codex-api/src/lib.rs` — declares and re-exports the crate's API-client, auth, endpoint, and transport surface. (`codex-api/src/lib.rs:1`)
- `codex-api/src/api_bridge.rs` — maps `ApiError` and transport HTTP failures into `CodexErr`, including overloaded, policy, usage-limit, retry, timeout, Cloudflare, and tracking-header cases. (`codex-api/src/api_bridge.rs:20`)
- `codex-api/src/api_bridge_tests.rs` — unit tests for error mapping and HTTP-body/header-derived errors. (`codex-api/src/api_bridge_tests.rs:7`)
- `codex-api/src/auth.rs` — defines async `AuthProvider` request-auth/header attachment, shared handles, and auth telemetry. (`codex-api/src/auth.rs:31`)
- `codex-api/src/common.rs` — defines Responses wire payloads, response events, reasoning/text controls, WebSocket request wrappers, and trace metadata helpers. (`codex-api/src/common.rs:30`)
- `codex-api/src/error.rs` — defines the crate's `ApiError` taxonomy. (`codex-api/src/error.rs:9`)
- `codex-api/src/files.rs` — handles hosted/OpenAI file creation, upload limits, blob upload/finalization, and download metadata. (`codex-api/src/files.rs:121`)
- `codex-api/src/images.rs` — defines image generation/edit request and response wire DTOs. (`codex-api/src/images.rs:4`)
- `codex-api/src/provider.rs` — configures provider URLs, headers, retries, timeouts, Responses-vs-Chat wire selection, and WebSocket URL conversion. (`codex-api/src/provider.rs:50`)
- `codex-api/src/rate_limits.rs` — parses rate-limit, credit, promo, and limit-reached headers/events into snapshots. (`codex-api/src/rate_limits.rs:16`)
- `codex-api/src/safety_buffering.rs` — extracts safety-buffering treatment and faster-model fallback from HTTP headers. (`codex-api/src/safety_buffering.rs:12`)
- `codex-api/src/search.rs` — defines search request, commands, filters, external-web settings, and response DTOs. (`codex-api/src/search.rs:6`)
- `codex-api/src/telemetry.rs` — defines SSE/WebSocket telemetry hooks and wraps retry execution with per-request telemetry. (`codex-api/src/telemetry.rs:16`)

## codex-api/src/endpoint

- `codex-api/src/endpoint/mod.rs` — declares endpoint modules and re-exports their public clients/types. (`codex-api/src/endpoint/mod.rs:1`)
- `codex-api/src/endpoint/compact.rs` — POSTs compaction input to `responses/compact`, captures turn state, and decodes output items. (`codex-api/src/endpoint/compact.rs:39`)
- `codex-api/src/endpoint/images.rs` — posts generation/edit requests, decodes image responses, and returns imagegen request IDs. (`codex-api/src/endpoint/images.rs:35`)
- `codex-api/src/endpoint/memories.rs` — POSTs raw memories to `memories/trace_summarize` and decodes summaries. (`codex-api/src/endpoint/memories.rs:36`)
- `codex-api/src/endpoint/models.rs` — lists models with a client-version query and extracts the response ETag. (`codex-api/src/endpoint/models.rs:46`)
- `codex-api/src/endpoint/realtime_call.rs` — creates WebRTC realtime calls with SDP/session payloads, backend or multipart wire shape, and call-ID extraction. (`codex-api/src/endpoint/realtime_call.rs:90`)
- `codex-api/src/endpoint/responses.rs` — routes Responses/Guardian requests, rewrites Responses requests for Chat wire, sends SSE requests, and selects the matching stream translator. (`codex-api/src/endpoint/responses.rs:112`)
- `codex-api/src/endpoint/responses_websocket.rs` — manages Responses WebSocket connections, handshakes/probes, serialized requests, event streams, timing, and close/error mapping. (`codex-api/src/endpoint/responses_websocket.rs:235`)
- `codex-api/src/endpoint/search.rs` — posts `SearchRequest` to `alpha/search` and decodes `SearchResponse`. (`codex-api/src/endpoint/search.rs:35`)
- `codex-api/src/endpoint/session.rs` — shared endpoint transport layer applying provider URLs, auth, retries, JSON encoding, streaming, and telemetry. (`codex-api/src/endpoint/session.rs:17`)

## codex-api/src/endpoint/realtime_websocket

- `codex-api/src/endpoint/realtime_websocket/mod.rs` — declares and re-exports realtime WebSocket methods and protocol types. (`codex-api/src/endpoint/realtime_websocket/mod.rs:1`)
- `codex-api/src/endpoint/realtime_websocket/methods.rs` — implements realtime WebSocket connections, outbound writer methods, event handling, transcript state, URL construction, and sideband/session setup. (`codex-api/src/endpoint/realtime_websocket/methods.rs:211`)
- `codex-api/src/endpoint/realtime_websocket/methods_common.rs` — dispatches outbound messages and session updates across V1, Frameless Bidi, and V2 realtime wires. (`codex-api/src/endpoint/realtime_websocket/methods_common.rs:29`)
- `codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs` — tests wire-specific session updates, handoff outputs, channels, and V1 prefixing. (`codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs:16`)
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs` — encodes Frameless Bidi session/context/delegation messages and chunks oversized text. (`codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs:13`)
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs` — tests chunk limits and role-bearing initial-session items. (`codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs:11`)
- `codex-api/src/endpoint/realtime_websocket/methods_v1.rs` — encodes legacy V1 conversation items, handoff appends, and Quicksilver session updates. (`codex-api/src/endpoint/realtime_websocket/methods_v1.rs:18`)
- `codex-api/src/endpoint/realtime_websocket/methods_v2.rs` — encodes V2 conversation/function-call messages and conversational/transcription session configs with tools. (`codex-api/src/endpoint/realtime_websocket/methods_v2.rs:39`)
- `codex-api/src/endpoint/realtime_websocket/protocol.rs` — defines realtime parser/session enums, outbound message payloads, and session audio/config DTOs. (`codex-api/src/endpoint/realtime_websocket/protocol.rs:15`)
- `codex-api/src/endpoint/realtime_websocket/protocol_common.rs` — parses common realtime JSON payloads, session updates, transcript deltas/dones, and errors. (`codex-api/src/endpoint/realtime_websocket/protocol_common.rs:7`)
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs` — translates Frameless Bidi audio, transcript, turn-done, delegation, and error events. (`codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs:15`)
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs` — verifies Frameless Bidi events decode to the same internal realtime events as legacy. (`codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs:7`)
- `codex-api/src/endpoint/realtime_websocket/protocol_v1.rs` — translates legacy realtime V1 audio, transcript, item, handoff, and error events. (`codex-api/src/endpoint/realtime_websocket/protocol_v1.rs:12`)
- `codex-api/src/endpoint/realtime_websocket/protocol_v2.rs` — translates realtime V2 audio, transcripts, items, response lifecycle, background-agent handoffs, silence requests, and errors. (`codex-api/src/endpoint/realtime_websocket/protocol_v2.rs:24`)

## codex-api/src/requests

- `codex-api/src/requests/mod.rs` — declares request translation/header modules. (`codex-api/src/requests/mod.rs:1`)
- `codex-api/src/requests/chat.rs` — translates Responses API requests and history into OpenAI-compatible Chat Completions bodies, including tools, reasoning, text format, and GLM effort mapping. (`codex-api/src/requests/chat.rs:1`)
- `codex-api/src/requests/headers.rs` — builds session/thread headers, subagent headers, and safe header insertion. (`codex-api/src/requests/headers.rs:5`)
- `codex-api/src/requests/responses.rs` — defines request compression choices. (`codex-api/src/requests/responses.rs:1`)

## codex-api/src/sse

- `codex-api/src/sse/mod.rs` — declares and re-exports SSE stream translators. (`codex-api/src/sse/mod.rs:1`)
- `codex-api/src/sse/chat.rs` — translates Chat Completions SSE deltas into Responses-shaped events, rebuilding assistant/reasoning/tool items and usage. (`codex-api/src/sse/chat.rs:1`)
- `codex-api/src/sse/responses.rs` — translates native Responses SSE and event headers into `ResponseEvent`s, captures state/rate limits/usage, and maps stream errors. (`codex-api/src/sse/responses.rs:36`)
