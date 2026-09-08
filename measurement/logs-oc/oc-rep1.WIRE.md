# Wire

## `codex-api/src`

- `codex-api/src/lib.rs` — declares the API crate modules and re-exports its public wire/provider/error/stream types; `lib.rs:1`.
- `codex-api/src/common.rs` — defines shared Responses/Chat wire types, including `ResponseEvent`, `Reasoning`, `TextControls`, `ResponsesApiRequest`, WebSocket requests, and `ResponseStream`; `common.rs:274`.
- `codex-api/src/api_bridge.rs` — translates HTTP `ApiError` values into `CodexErr`, extracting retry, status, headers, and usage details; `api_bridge.rs:21`.
- `codex-api/src/api_bridge_tests.rs` — tests API-error mapping for retries, overload/Cloudflare responses, policy rejections, usage headers, and auth failures; `api_bridge_tests.rs:8`.
- `codex-api/src/auth.rs` — defines auth errors, the `AuthProvider` trait that applies authorization headers, and shared auth-provider state; `auth.rs:30`.
- `codex-api/src/error.rs` — defines `ApiError` variants for transport, API, stream, quota, policy, and retryable failures; `error.rs:9`.
- `codex-api/src/provider.rs` — defines provider base URLs, retry/timeout settings, `WireApi::Responses` or `Chat` selection, and endpoint URL construction; `provider.rs:56`.
- `codex-api/src/telemetry.rs` — defines SSE/WebSocket telemetry hooks and wraps requests with retry and telemetry behavior; `telemetry.rs:18`.
- `codex-api/src/rate_limits.rs` — parses rate-limit headers and models rate-limit reached conditions and `codex.rate_limits` events; `rate_limits.rs:57`.
- `codex-api/src/images.rs` — defines image generation/edit request and response wire types; `images.rs:5`.
- `codex-api/src/files.rs` — handles the hosted file upload flow: create, blob upload, uploaded completion, retries, and errors; `files.rs:121`.
- `codex-api/src/safety_buffering.rs` — maps safety-buffering response headers into treatment decisions; `safety_buffering.rs:8`.
- `codex-api/src/search.rs` — defines search requests, commands, filters, settings, locations, and response wire types; `search.rs:9`.

## `codex-api/src/requests`

- `codex-api/src/requests/mod.rs` — declares the headers/chat/responses request modules and exports `Compression`; `requests/mod.rs:1`.
- `codex-api/src/requests/headers.rs` — builds session, thread/request-id, and subagent request headers; `requests/headers.rs:5`.
- `codex-api/src/requests/responses.rs` — defines the Responses request compression encoding `None|Zstd`; `requests/responses.rs:2`.
- `codex-api/src/requests/chat.rs` — translates Responses requests into Chat Completions bodies, tools, reasoning, and grouped tool calls; `requests/chat.rs:19`.

## `codex-api/src/sse`

- `codex-api/src/sse/mod.rs` — declares and re-exports the Responses and Chat SSE streaming entry points; `sse/mod.rs:4`.
- `codex-api/src/sse/responses.rs` — parses Responses SSE into `ResponseEvent`, extracting headers, metadata, rate limits, errors, completion, and idle timeout; `sse/responses.rs:353`.
- `codex-api/src/sse/chat.rs` — parses Chat Completions SSE into Responses events while accumulating deltas, tool calls, usage, and completion; `sse/chat.rs:30`.

## `codex-api/src/endpoint`

- `codex-api/src/endpoint/mod.rs` — declares endpoint modules and re-exports the public endpoint clients; `endpoint/mod.rs:1`.
- `codex-api/src/endpoint/session.rs` — applies provider URL, auth, retry, telemetry, and request/stream setup for endpoint sessions; `endpoint/session.rs:80`.
- `codex-api/src/endpoint/search.rs` — sends typed search requests to `alpha/search` and decodes the response; `endpoint/search.rs:35`.
- `codex-api/src/endpoint/responses.rs` — provides the HTTP Responses streaming endpoint, chooses Responses or Chat wire, and spawns the matching SSE stream; `endpoint/responses.rs:112`.
- `codex-api/src/endpoint/responses_websocket.rs` — connects, probes, authenticates, and sends Responses WebSocket requests, then parses events, metadata, and errors into response streams; `endpoint/responses_websocket.rs:235`.
- `codex-api/src/endpoint/realtime_call.rs` — creates WebRTC realtime calls and handles SDP, backend JSON, multipart session payloads, query shaping, and call IDs; `endpoint/realtime_call.rs:129`.
- `codex-api/src/endpoint/models.rs` — fetches `models`, appends `client_version`, decodes model data, and returns its ETag; `endpoint/models.rs:46`.
- `codex-api/src/endpoint/memories.rs` — sends memory trace summarization to `memories/trace_summarize` and decodes its output; `endpoint/memories.rs:36`.
- `codex-api/src/endpoint/images.rs` — sends image generation/edit POSTs, decodes image responses, and extracts the generation request ID; `endpoint/images.rs:58`.
- `codex-api/src/endpoint/compact.rs` — posts compaction to `responses/compact`, applies timeout, captures turn state, and decodes output items; `endpoint/compact.rs:39`.

## `codex-api/src/endpoint/realtime_websocket`

- `codex-api/src/endpoint/realtime_websocket/mod.rs` — declares and re-exports realtime protocol and client modules; `mod.rs:1`.
- `codex-api/src/endpoint/realtime_websocket/protocol.rs` — defines realtime parser/session enums, outbound wire messages, session payloads, and parser dispatch; `protocol.rs:263`.
- `codex-api/src/endpoint/realtime_websocket/protocol_v1.rs` — parses realtime v1 JSON events into `RealtimeEvent`; `protocol_v1.rs:12`.
- `codex-api/src/endpoint/realtime_websocket/protocol_v2.rs` — parses realtime v2 audio, transcript, lifecycle, handoff, and silence-tool events; `protocol_v2.rs:24`.
- `codex-api/src/endpoint/realtime_websocket/protocol_common.rs` — provides shared realtime JSON payload, session, transcript, and error parsers; `protocol_common.rs:7`.
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs` — parses frameless bidirectional session, audio, transcript, and delegation events; `protocol_frameless_bidi.rs:15`.
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs` — verifies legacy and frameless delegation, transcript, and audio decoding equivalence; `protocol_frameless_bidi_tests.rs:7`.
- `codex-api/src/endpoint/realtime_websocket/methods.rs` — implements the realtime WebSocket client, connection, writer, event pump, transcript state, URL construction, session initialization, retries, close behavior, and its tests; `methods.rs:765`.
- `codex-api/src/endpoint/realtime_websocket/methods_common.rs` — routes outbound realtime message construction by parser version and exposes shared session JSON; `methods_common.rs:41`.
- `codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs` — tests outbound payload differences for frameless, legacy, context-channel, and completed-handoff cases; `methods_common_tests.rs:16`.
- `codex-api/src/endpoint/realtime_websocket/methods_v1.rs` — builds realtime v1 item, handoff, session, and WebSocket intent payloads; `methods_v1.rs:51`.
- `codex-api/src/endpoint/realtime_websocket/methods_v2.rs` — builds realtime v2 item, function-output, session, background-agent, and silence-tool payloads; `methods_v2.rs:75`.
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs` — builds frameless session/context append messages and chunks text into 500-byte parts; `methods_frameless_bidi.rs:109`.
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs` — tests frameless context chunk preservation and initial-item/session JSON encoding; `methods_frameless_bidi_tests.rs:11`.
