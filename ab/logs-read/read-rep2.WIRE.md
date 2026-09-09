# Wire files

## src/

- `codex-api/src/telemetry.rs` — wraps unary/streaming request retries and SSE/WS activity with telemetry hooks — `telemetry.rs:68`
- `codex-api/src/search.rs` — defines search request, command, setting, and response wire shapes — `search.rs:9`
- `codex-api/src/safety_buffering.rs` — reads safety-buffering treatment from HTTP headers — `safety_buffering.rs:8`
- `codex-api/src/rate_limits.rs` — parses rate-limit headers and events into snapshots — `rate_limits.rs:57`
- `codex-api/src/provider.rs` — models provider endpoints, retry policy, wire API, and URL/request helpers — `provider.rs:56`
- `codex-api/src/lib.rs` — crate facade exporting wire types and clients — `lib.rs:1`
- `codex-api/src/images.rs` — image generation/edit wire request and response types — `images.rs:5`
- `codex-api/src/files.rs` — creates, streams, uploads, and finalizes hosted OpenAI files — `files.rs:121`
- `codex-api/src/error.rs` — API error variants for transport, stream, policy, quota, and rate failures — `error.rs:9`
- `codex-api/src/auth.rs` — auth header/request-signing contract for outbound requests — `auth.rs:30`
- `codex-api/src/common.rs` — shared Responses request, event, stream, and WS request wire types — `common.rs:275`
- `codex-api/src/api_bridge.rs` — maps `ApiError` and transport HTTP failures to `CodexErr` — `api_bridge.rs:21`
- `codex-api/src/api_bridge_tests.rs` — tests `map_api_error` behavior — `api_bridge_tests.rs:8`

## src/requests/

- `codex-api/src/requests/chat.rs` — translates Responses-shaped requests into Chat Completions bodies — `requests/chat.rs:19`
- `codex-api/src/requests/headers.rs` — builds session and subagent headers — `requests/headers.rs:5`
- `codex-api/src/requests/responses.rs` — defines request `Compression` — `requests/responses.rs:1`
- `codex-api/src/requests/mod.rs` — request submodule exports — `requests/mod.rs:1`

## src/sse/

- `codex-api/src/sse/chat.rs` — translates Chat Completions SSE into `ResponseEvent`s — `sse/chat.rs:77`
- `codex-api/src/sse/responses.rs` — parses Responses SSE frames into `ResponseEvent`s and API errors — `sse/responses.rs:353`
- `codex-api/src/sse/mod.rs` — SSE submodule exports — `sse/mod.rs:1`

## src/endpoint/

- `codex-api/src/endpoint/mod.rs` — endpoint submodule declarations and public client exports — `endpoint/mod.rs:1`
- `codex-api/src/endpoint/session.rs` — executes authenticated, retried unary and streaming provider requests — `session.rs:78`
- `codex-api/src/endpoint/responses.rs` — routes Responses/Guardian requests, translates Responses to Chat when needed, and dispatches SSE parsers — `responses.rs:106`
- `codex-api/src/endpoint/models.rs` — requests and decodes model catalogs plus ETag metadata — `models.rs:46`
- `codex-api/src/endpoint/memories.rs` — encodes memory trace inputs and decodes summarize outputs — `memories.rs:23`
- `codex-api/src/endpoint/images.rs` — posts image generation/edit requests and decodes image responses and request IDs — `images.rs:54`
- `codex-api/src/endpoint/compact.rs` — posts compaction inputs and decodes compacted Response items and turn state — `compact.rs:29`
- `codex-api/src/endpoint/search.rs` — encodes typed search requests and decodes search responses — `search.rs:25`
- `codex-api/src/endpoint/realtime_call.rs` — creates WebRTC realtime calls, encodes SDP/session bodies, and extracts call IDs from Location — `realtime_call.rs:129`
- `codex-api/src/endpoint/responses_websocket.rs` — connects and probes Responses WebSockets, sends serialized response requests, and parses events/errors into `ResponseEvent`s — `responses_websocket.rs:235`

## src/endpoint/realtime_websocket/

- `codex-api/src/endpoint/realtime_websocket/mod.rs` — declares protocol/method modules and exports realtime client/connection types — `realtime_websocket/mod.rs:1`
- `codex-api/src/endpoint/realtime_websocket/protocol.rs` — defines realtime parser/session modes and outbound WebSocket message/session types — `realtime_websocket/protocol.rs:11`
- `codex-api/src/endpoint/realtime_websocket/protocol_common.rs` — parses realtime JSON payloads, session updates, transcripts, and errors — `realtime_websocket/protocol_common.rs:7`
- `codex-api/src/endpoint/realtime_websocket/protocol_v1.rs` — maps realtime v1 event names to internal events — `realtime_websocket/protocol_v1.rs:11`
- `codex-api/src/endpoint/realtime_websocket/protocol_v2.rs` — maps realtime v2 events, audio, handoff/silence tools, and responses — `realtime_websocket/protocol_v2.rs:16`
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs` — maps Frameless Bidi live events to internal realtime events — `realtime_websocket/protocol_frameless_bidi.rs:16`
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs` — verifies legacy/frameless handoff and transcript/audio decoding — `realtime_websocket/protocol_frameless_bidi_tests.rs:3`
- `codex-api/src/endpoint/realtime_websocket/methods.rs` — manages realtime WebSocket connections, URLs, retries, outbound messages, inbound events, and transcript state — `realtime_websocket/methods.rs:765`
- `codex-api/src/endpoint/realtime_websocket/methods_common.rs` — dispatches outbound message construction across realtime wire adapters — `realtime_websocket/methods_common.rs:33`
- `codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs` — verifies adapter-specific session, handoff, context, and function-output payloads — `realtime_websocket/methods_common_tests.rs:8`
- `codex-api/src/endpoint/realtime_websocket/methods_v1.rs` — builds v1 conversation items, handoffs, Quicksilver sessions, and intent — `realtime_websocket/methods_v1.rs:11`
- `codex-api/src/endpoint/realtime_websocket/methods_v2.rs` — builds v2 conversation/function payloads and realtime/transcription sessions — `realtime_websocket/methods_v2.rs:31`
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs` — builds frameless context/session updates and chunks context appends — `realtime_websocket/methods_frameless_bidi.rs:15`
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs` — verifies frameless chunk limits and initial-item JSON — `realtime_websocket/methods_frameless_bidi_tests.rs:7`
