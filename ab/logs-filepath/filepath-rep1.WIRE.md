# Wire map: `codex-api/src`

## `codex-api/src`

- `codex-api/src/api_bridge.rs` — maps API/transport failures into `CodexErr`, including HTTP policy, overload, rate/usage limits, Cloudflare, headers, and identity errors (`codex-api/src/api_bridge.rs:21`).
- `codex-api/src/api_bridge_tests.rs` — covers API error mapping for overload, retry delays, Cloudflare, cyber/misalignment policy, rate/usage headers, and identity extraction (`codex-api/src/api_bridge_tests.rs:7`).
- `codex-api/src/auth.rs` — defines auth errors and the async `AuthProvider` contract for applying auth headers/requests, plus auth telemetry (`codex-api/src/auth.rs:30`).
- `codex-api/src/common.rs` — defines shared Responses/WebSocket DTOs, events, safety buffering, request bodies, and the response stream (`codex-api/src/common.rs:274`).
- `codex-api/src/error.rs` — defines the API-layer wire/stream error taxonomy (`codex-api/src/error.rs:9`).
- `codex-api/src/files.rs` — handles the OpenAI file upload lifecycle: size checks, create, blob upload, finalize, and retry diagnostics (`codex-api/src/files.rs:121`).
- `codex-api/src/images.rs` — defines image generation/edit request and response wire types (`codex-api/src/images.rs:4`).
- `codex-api/src/lib.rs` — declares crate modules and public re-exports for API clients, providers, payloads, search, telemetry, and realtime (`codex-api/src/lib.rs:1`).
- `codex-api/src/provider.rs` — stores provider endpoint/retry configuration, constructs request and WebSocket URLs, and selects `Responses` or `Chat` wire API (`codex-api/src/provider.rs:56`).
- `codex-api/src/rate_limits.rs` — parses rate-limit headers/events into snapshots, including per-limit, promo, reached-type, and credits data (`codex-api/src/rate_limits.rs:57`).
- `codex-api/src/safety_buffering.rs` — converts safety-buffering headers into treatment settings and faster-model fallback (`codex-api/src/safety_buffering.rs:8`).
- `codex-api/src/search.rs` — defines search requests, operations, settings, and responses (`codex-api/src/search.rs:8`).
- `codex-api/src/telemetry.rs` — defines SSE/WebSocket telemetry hooks and the request telemetry wrapper (`codex-api/src/telemetry.rs:68`).

## `codex-api/src/endpoint`

- `codex-api/src/endpoint/compact.rs` — posts compaction requests to `responses/compact`, captures turn state, and parses compacted items (`codex-api/src/endpoint/compact.rs:39`).
- `codex-api/src/endpoint/images.rs` — sends image generation/edit requests and extracts imagegen request IDs and responses (`codex-api/src/endpoint/images.rs:18`).
- `codex-api/src/endpoint/memories.rs` — posts memory trace summarization requests and decodes memory summaries (`codex-api/src/endpoint/memories.rs:36`).
- `codex-api/src/endpoint/mod.rs` — declares and exports endpoint submodules and clients (`codex-api/src/endpoint/mod.rs:1`).
- `codex-api/src/endpoint/models.rs` — lists models from `/models`, appends client version, and returns models with ETag (`codex-api/src/endpoint/models.rs:46`).
- `codex-api/src/endpoint/realtime_call.rs` — creates WebRTC realtime calls with backend/API body variants, SDP/session encoding, AVAS queries, and call-ID extraction (`codex-api/src/endpoint/realtime_call.rs:129`).
- `codex-api/src/endpoint/responses.rs` — dispatches HTTP Responses or Chat streams, selects the endpoint, translates Responses requests to Chat, and sets session/subagent headers (`codex-api/src/endpoint/responses.rs:112`).
- `codex-api/src/endpoint/responses_websocket.rs` — runs Responses over WebSocket: connection pump, probes, request wrapping, stream decoding, errors, timing, and safety buffering (`codex-api/src/endpoint/responses_websocket.rs:345`).
- `codex-api/src/endpoint/search.rs` — encodes search requests, posts them to `alpha/search`, and decodes responses (`codex-api/src/endpoint/search.rs:35`).
- `codex-api/src/endpoint/session.rs` — provides the shared HTTP request foundation: URL/body construction, headers, retry telemetry, auth, and unary/stream dispatch (`codex-api/src/endpoint/session.rs:80`).

## `codex-api/src/endpoint/realtime_websocket`

- `codex-api/src/endpoint/realtime_websocket/methods.rs` — implements realtime WebSocket connections, writer/event sides, URL construction, retries, transcript state, and integration tests (`codex-api/src/endpoint/realtime_websocket/methods.rs:777`).
- `codex-api/src/endpoint/realtime_websocket/methods_common.rs` — routes outbound realtime requests to V1, Frameless Bidi, or Realtime V2 message shapes (`codex-api/src/endpoint/realtime_websocket/methods_common.rs:114`).
- `codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs` — tests adapter-specific encoding of session updates, handoffs, and function-call outputs (`codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs:16`).
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs` — builds Frameless Bidi session/context messages and chunks context appends (`codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs:35`).
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs` — tests context chunk limits and Frameless Bidi session JSON (`codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs:11`).
- `codex-api/src/endpoint/realtime_websocket/methods_v1.rs` — builds V1 realtime conversation, handoff, session, and quicksilver-intent wire shapes (`codex-api/src/endpoint/realtime_websocket/methods_v1.rs:18`).
- `codex-api/src/endpoint/realtime_websocket/methods_v2.rs` — builds Realtime V2 items and session updates, including audio settings and background-agent/silence tools (`codex-api/src/endpoint/realtime_websocket/methods_v2.rs:75`).
- `codex-api/src/endpoint/realtime_websocket/mod.rs` — declares realtime WebSocket modules and exports their public connection, protocol, and config types (`codex-api/src/endpoint/realtime_websocket/mod.rs:1`).
- `codex-api/src/endpoint/realtime_websocket/protocol.rs` — defines realtime protocol enums/config/outbound messages and dispatches event parsing (`codex-api/src/endpoint/realtime_websocket/protocol.rs:50`).
- `codex-api/src/endpoint/realtime_websocket/protocol_common.rs` — provides shared realtime JSON/event parsing helpers (`codex-api/src/endpoint/realtime_websocket/protocol_common.rs:7`).
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs` — parses Frameless Bidi audio, transcript, turn, delegation, session, and error events (`codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs:15`).
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs` — tests frameless/legacy handoff equivalence and transcript/audio event mapping (`codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs:7`).
- `codex-api/src/endpoint/realtime_websocket/protocol_v1.rs` — parses realtime V1 wire events into internal realtime events (`codex-api/src/endpoint/realtime_websocket/protocol_v1.rs:12`).
- `codex-api/src/endpoint/realtime_websocket/protocol_v2.rs` — parses Realtime V2 events, including background-agent handoffs and silence tool calls (`codex-api/src/endpoint/realtime_websocket/protocol_v2.rs:24`).

## `codex-api/src/requests`

- `codex-api/src/requests/chat.rs` — translates a Responses request into an OpenAI-compatible Chat Completions body (`codex-api/src/requests/chat.rs:19`).
- `codex-api/src/requests/headers.rs` — builds session/thread headers and derives subagent headers (`codex-api/src/requests/headers.rs:5`).
- `codex-api/src/requests/mod.rs` — declares request modules and exports `Compression` (`codex-api/src/requests/mod.rs:1`).
- `codex-api/src/requests/responses.rs` — defines request compression modes (`codex-api/src/requests/responses.rs:1`).

## `codex-api/src/sse`

- `codex-api/src/sse/chat.rs` — translates Chat Completions SSE deltas, tool calls, reasoning, usage, and errors into `ResponseEvent`s (`codex-api/src/sse/chat.rs:30`).
- `codex-api/src/sse/mod.rs` — declares SSE modules and re-exports stream/event helpers (`codex-api/src/sse/mod.rs:1`).
- `codex-api/src/sse/responses.rs` — parses Responses SSE and headers into response events, metadata, rate limits, safety buffering, usage, and classified errors (`codex-api/src/sse/responses.rs:36`).
