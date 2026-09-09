# `codex-api` wire surface

## `codex-api/src`

- `codex-api/src/api_bridge.rs` — maps provider `ApiError` and HTTP transport failures into `CodexErr`, including policy, usage/rate-limit, request ID, Azure, and Cloudflare handling (`codex-api/src/api_bridge.rs:16`).
- `codex-api/src/api_bridge_tests.rs` — unit tests for API-error bridging, retry delays, provider messages, policy bodies, usage limits, and auth headers (`codex-api/src/api_bridge_tests.rs:4`).
- `codex-api/src/auth.rs` — defines request authentication through `AuthProvider`, header resolution/signing, `SharedAuthProvider`, and auth telemetry (`codex-api/src/auth.rs:49`).
- `codex-api/src/common.rs` — shared Responses/wire DTOs, response events, request wrappers, websocket request mapping, trace metadata, and `ResponseStream` (`codex-api/src/common.rs:275`).
- `codex-api/src/error.rs` — defines `ApiError` variants for transport, streams, quota, usage, retry/rate limits, invalid requests, policy violations, and overload (`codex-api/src/error.rs:8`).
- `codex-api/src/files.rs` — implements hosted OpenAI file upload create, blob upload, diagnostics, finalization/retry, and authenticated requests (`codex-api/src/files.rs:121`).
- `codex-api/src/images.rs` — defines image generation/edit request and response DTOs, including output image payloads (`codex-api/src/images.rs:2`).
- `codex-api/src/lib.rs` — declares the crate module tree and public re-exports for clients, wire types, providers, telemetry, and search (`codex-api/src/lib.rs:1`).
- `codex-api/src/provider.rs` — configures provider retry, base URL, headers, query params, Responses/Chat wire API, and websocket URL selection (`codex-api/src/provider.rs:56`).
- `codex-api/src/rate_limits.rs` — parses rate-limit and credits headers/events into snapshots, including windows, promos, and limit IDs (`codex-api/src/rate_limits.rs:27`).
- `codex-api/src/safety_buffering.rs` — extracts safety-buffering treatment and faster-model selection from custom response headers (`codex-api/src/safety_buffering.rs:8`).
- `codex-api/src/search.rs` — defines search request, command, settings, location/filter, and forward-compatible response DTOs (`codex-api/src/search.rs:2`).
- `codex-api/src/telemetry.rs` — defines SSE/websocket telemetry contracts and wraps retry attempts with request telemetry (`codex-api/src/telemetry.rs:68`).

## `codex-api/src/endpoint`

- `codex-api/src/endpoint/mod.rs` — declares and re-exports endpoint clients (`codex-api/src/endpoint/mod.rs:1`).
- `codex-api/src/endpoint/compact.rs` — posts compact requests, captures turn state, and decodes compacted response items (`codex-api/src/endpoint/compact.rs:56`).
- `codex-api/src/endpoint/images.rs` — posts image generation/edit requests, decodes image responses, and extracts the imagegen request ID (`codex-api/src/endpoint/images.rs:40`).
- `codex-api/src/endpoint/memories.rs` — posts memory/trace summarize requests and decodes memory summarize outputs (`codex-api/src/endpoint/memories.rs:28`).
- `codex-api/src/endpoint/models.rs` — lists models, appends client version, and returns the model list with `ETag` (`codex-api/src/endpoint/models.rs:64`).
- `codex-api/src/endpoint/realtime_call.rs` — creates WebRTC realtime calls and extracts SDP/call IDs across backend and API response shapes (`codex-api/src/endpoint/realtime_call.rs:59`).
- `codex-api/src/endpoint/responses.rs` — routes HTTP Responses/Chat inference requests, translates chat bodies, adds session/subagent headers, and starts SSE streams (`codex-api/src/endpoint/responses.rs:100`).
- `codex-api/src/endpoint/responses_websocket.rs` — builds, connects, probes, authenticates, and streams Responses websocket requests and metadata/errors (`codex-api/src/endpoint/responses_websocket.rs:505`).
- `codex-api/src/endpoint/search.rs` — posts typed search requests to the alpha search endpoint and decodes `SearchResponse` (`codex-api/src/endpoint/search.rs:25`).
- `codex-api/src/endpoint/session.rs` — shared endpoint session for request construction, retries, auth, instrumented JSON calls, and encoded SSE streaming (`codex-api/src/endpoint/session.rs:80`).

## `codex-api/src/endpoint/realtime_websocket`

- `codex-api/src/endpoint/realtime_websocket/mod.rs` — declares protocol/method modules and re-exports the realtime websocket API (`codex-api/src/endpoint/realtime_websocket/mod.rs:1`).
- `codex-api/src/endpoint/realtime_websocket/protocol.rs` — defines realtime wire parsers, session/config types, and outbound realtime message payloads (`codex-api/src/endpoint/realtime_websocket/protocol.rs:14`).
- `codex-api/src/endpoint/realtime_websocket/protocol_common.rs` — common JSON parsing helpers for realtime payloads, session updates, transcripts, and errors (`codex-api/src/endpoint/realtime_websocket/protocol_common.rs:7`).
- `codex-api/src/endpoint/realtime_websocket/protocol_v1.rs` — parses realtime v1 events into internal audio, transcript, item, handoff, and error events (`codex-api/src/endpoint/realtime_websocket/protocol_v1.rs:12`).
- `codex-api/src/endpoint/realtime_websocket/protocol_v2.rs` — parses realtime v2 events, including audio, transcripts, response lifecycle, handoff, and silence/no-op calls (`codex-api/src/endpoint/realtime_websocket/protocol_v2.rs:18`).
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs` — parses frameless bidirectional events into audio, transcripts, delegations, and errors (`codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs:14`).
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs` — verifies frameless legacy-equivalent handoffs, transcript decoding, and audio frames (`codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs:4`).
- `codex-api/src/endpoint/realtime_websocket/methods.rs` — implements realtime websocket connections, writers, event reading, transcript state, URL construction, TLS, and sideband joins/retries (`codex-api/src/endpoint/realtime_websocket/methods.rs:815`).
- `codex-api/src/endpoint/realtime_websocket/methods_common.rs` — dispatches outbound realtime payloads by protocol adapter, including session updates and websocket intents (`codex-api/src/endpoint/realtime_websocket/methods_common.rs:140`).
- `codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs` — tests adapter-specific session, context, handoff, and function-output wire encoding (`codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs:83`).
- `codex-api/src/endpoint/realtime_websocket/methods_v1.rs` — builds realtime v1 conversation items, handoff appends, Quicksilver sessions, and intent (`codex-api/src/endpoint/realtime_websocket/methods_v1.rs:51`).
- `codex-api/src/endpoint/realtime_websocket/methods_v2.rs` — builds realtime v2 items, function outputs, conversational/transcription sessions, tools, and modalities (`codex-api/src/endpoint/realtime_websocket/methods_v2.rs:62`).
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs` — builds frameless delegation/session context appends and session JSON with 500-byte chunking (`codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs:52`).
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs` — tests byte-safe context chunking and role-bearing initial session items (`codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs:51`).

## `codex-api/src/requests`

- `codex-api/src/requests/mod.rs` — declares request translation modules and re-exports compression (`codex-api/src/requests/mod.rs:1`).
- `codex-api/src/requests/headers.rs` — builds session/thread headers, maps subagent sources, and inserts headers safely (`codex-api/src/requests/headers.rs:3`).
- `codex-api/src/requests/chat.rs` — translates outbound Responses requests into Chat Completions bodies, tools, reasoning, stream options, and tool-call grouping (`codex-api/src/requests/chat.rs:18`).
- `codex-api/src/requests/responses.rs` — defines `Compression::{None,Zstd}` for Responses requests (`codex-api/src/requests/responses.rs:1`).

## `codex-api/src/sse`

- `codex-api/src/sse/mod.rs` — declares and re-exports SSE translation modules (`codex-api/src/sse/mod.rs:1`).
- `codex-api/src/sse/chat.rs` — translates inbound Chat Completions SSE into Responses events, including deltas, tools, reasoning, completion, and usage (`codex-api/src/sse/chat.rs:32`).
- `codex-api/src/sse/responses.rs` — parses Responses SSE and websocket-shaped events into response events, metadata, rate/error classification, usage, and safety buffering (`codex-api/src/sse/responses.rs:353`).
