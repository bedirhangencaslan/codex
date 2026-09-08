# codex-api wire catalog

## `codex-api/src`

- `codex-api/src/api_bridge.rs` — translates API and transport failures into protocol errors across overloaded, usage-limit, cyber-policy, timeout, and connection cases — `api_bridge.rs:25`
- `codex-api/src/api_bridge_tests.rs` — tests API and transport error mapping for overload, Cloudflare, cyber-policy, misalignment, retry delay, and limit cases — `api_bridge_tests.rs:7`
- `codex-api/src/auth.rs` — defines authentication providers, request-auth application, telemetry metadata, and the shared auth handle — `auth.rs:36`
- `codex-api/src/common.rs` — models shared Responses wire payloads and the canonical response event and stream types — `common.rs:84`
- `codex-api/src/error.rs` — defines the API error taxonomy used across HTTP and streaming operations — `error.rs:8`
- `codex-api/src/files.rs` — handles OpenAI hosted file creation, blob upload, finalize-and-retry behavior, and metadata parsing — `files.rs:114`
- `codex-api/src/images.rs` — defines image-generation and image-edit request and response payloads — `images.rs:3`
- `codex-api/src/lib.rs` — organizes the crate modules and declares the public `codex-api` exports — `lib.rs:1`
- `codex-api/src/provider.rs` — configures provider endpoints, URLs, queries, headers, retries, and Responses-versus-Chat wire selection — `provider.rs:79`
- `codex-api/src/rate_limits.rs` — parses rate-limit headers and websocket events into rate-limit snapshots — `rate_limits.rs:50`
- `codex-api/src/safety_buffering.rs` — extracts safety-buffering treatment information from response headers — `safety_buffering.rs:6`
- `codex-api/src/search.rs` — defines search requests, commands, filters, settings, and response wire types — `search.rs:5`
- `codex-api/src/telemetry.rs` — wraps requests, SSE streams, and websocket streams with telemetry and retry instrumentation — `telemetry.rs:60`

## `codex-api/src/endpoint`

- `codex-api/src/endpoint/mod.rs` — declares endpoint modules and re-exports their clients and types — `endpoint/mod.rs:1`
- `codex-api/src/endpoint/compact.rs` — posts `responses/compact`, stores turn-state headers, and parses compacted output — `endpoint/compact.rs:32`
- `codex-api/src/endpoint/images.rs` — posts image-generation and image-edit requests and parses images and request IDs — `endpoint/images.rs:40`
- `codex-api/src/endpoint/memories.rs` — posts memory trace summarization requests and parses memory summaries — `endpoint/memories.rs:23`
- `codex-api/src/endpoint/models.rs` — gets versioned model listings and parses models and ETag metadata — `endpoint/models.rs:23`
- `codex-api/src/endpoint/realtime_call.rs` — creates WebRTC realtime calls and handles SDP, session, backend JSON/multipart, and call-ID payloads — `endpoint/realtime_call.rs:81`
- `codex-api/src/endpoint/responses.rs` — streams Responses-compatible inference and translates Chat-only providers at request and SSE boundaries — `endpoint/responses.rs:88`
- `codex-api/src/endpoint/responses_websocket.rs` — connects and probes Responses websockets, pumps streams, and translates websocket requests into response events — `endpoint/responses_websocket.rs:235`
- `codex-api/src/endpoint/search.rs` — posts alpha search requests and parses their responses — `endpoint/search.rs:21`
- `codex-api/src/endpoint/session.rs` — executes endpoint operations with provider URLs, auth, retries, telemetry, unary transport, and streaming transport — `endpoint/session.rs:40`

## `codex-api/src/endpoint/realtime_websocket`

- `codex-api/src/endpoint/realtime_websocket/mod.rs` — declares realtime method and protocol modules and re-exports public types — `endpoint/realtime_websocket/mod.rs:1`
- `codex-api/src/endpoint/realtime_websocket/protocol.rs` — defines realtime parser/session enums, outbound messages, session-update payloads, and event dispatch — `endpoint/realtime_websocket/protocol.rs:206`
- `codex-api/src/endpoint/realtime_websocket/protocol_common.rs` — provides shared realtime JSON parsing for sessions, transcripts, and errors — `endpoint/realtime_websocket/protocol_common.rs:6`
- `codex-api/src/endpoint/realtime_websocket/protocol_v1.rs` — parses Realtime v1 websocket events into realtime events — `endpoint/realtime_websocket/protocol_v1.rs:8`
- `codex-api/src/endpoint/realtime_websocket/protocol_v2.rs` — parses Realtime v2 audio, transcript, lifecycle, handoff, and tool-call events — `endpoint/realtime_websocket/protocol_v2.rs:20`
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs` — parses frameless bidirectional events into realtime events — `endpoint/realtime_websocket/protocol_frameless_bidi.rs:13`
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs` — tests frameless bidirectional event parsing — `endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs:1`
- `codex-api/src/endpoint/realtime_websocket/methods.rs` — manages realtime websocket connections, writers, event pumps, transcript state, sends, and closes — `endpoint/realtime_websocket/methods.rs:190`
- `codex-api/src/endpoint/realtime_websocket/methods_common.rs` — dispatches outbound builders by realtime wire adapter and builds session JSON — `endpoint/realtime_websocket/methods_common.rs:15`
- `codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs` — tests wire-adapter outbound builders and session JSON construction — `endpoint/realtime_websocket/methods_common_tests.rs:1`
- `codex-api/src/endpoint/realtime_websocket/methods_v1.rs` — builds Realtime v1 outbound items, handoffs, session updates, and intents — `endpoint/realtime_websocket/methods_v1.rs:6`
- `codex-api/src/endpoint/realtime_websocket/methods_v2.rs` — builds Realtime v2 outbound items, function outputs, session updates, and tool configuration — `endpoint/realtime_websocket/methods_v2.rs:16`
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs` — builds frameless context/session updates and chunks context appends — `endpoint/realtime_websocket/methods_frameless_bidi.rs:6`
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs` — tests frameless outbound messages and context chunking — `endpoint/realtime_websocket/methods_frameless_bidi_tests.rs:1`

## `codex-api/src/requests`

- `codex-api/src/requests/mod.rs` — declares request modules and exports the compression option — `requests/mod.rs:1`
- `codex-api/src/requests/chat.rs` — translates Responses-shaped requests into OpenAI-compatible Chat Completions bodies — `requests/chat.rs:9`
- `codex-api/src/requests/headers.rs` — builds session/thread and subagent request headers — `requests/headers.rs:3`
- `codex-api/src/requests/responses.rs` — defines request compression choices for Responses transport — `requests/responses.rs:1`

## `codex-api/src/sse`

- `codex-api/src/sse/mod.rs` — declares SSE modules and re-exports stream functions and types — `sse/mod.rs:1`
- `codex-api/src/sse/chat.rs` — translates Chat Completions SSE deltas into Responses events while buffering reasoning, text, tools, and usage — `sse/chat.rs:26`
- `codex-api/src/sse/responses.rs` — parses native Responses SSE metadata, headers, limits, safety buffering, errors, and response events — `sse/responses.rs:25`
