# `codex-api` wire map

## `codex-api/src/`

- `codex-api/src/api_bridge.rs` — translates `ApiError` and transport failures into `CodexErr`, preserving retry/rate-limit/policy context; `codex-api/src/api_bridge.rs:21`.
- `codex-api/src/api_bridge_tests.rs` — tests error mapping for overload, retry delays, policy violations, rate limits, and identity auth; `codex-api/src/api_bridge_tests.rs:8`.
- `codex-api/src/auth.rs` — defines request authentication via `AuthProvider`, shared handles, and auth telemetry; `codex-api/src/auth.rs:30`.
- `codex-api/src/common.rs` — defines shared Responses/WebSocket request and stream event payloads; `codex-api/src/common.rs:275`.
- `codex-api/src/error.rs` — defines the crate-wide `ApiError` variants; `codex-api/src/error.rs:9`.
- `codex-api/src/files.rs` — orchestrates OpenAI file creation, blob upload, finalization, and download metadata; `codex-api/src/files.rs:121`.
- `codex-api/src/images.rs` — defines image generation/edit request and response DTOs; `codex-api/src/images.rs:5`.
- `codex-api/src/lib.rs` — declares crate modules and public API re-exports; `codex-api/src/lib.rs:1`.
- `codex-api/src/provider.rs` — configures provider endpoints, retries, `Responses`/`Chat` wire choice, and Azure detection; `codex-api/src/provider.rs:56`.
- `codex-api/src/rate_limits.rs` — parses rate-limit headers and events into snapshots; `codex-api/src/rate_limits.rs:23`.
- `codex-api/src/safety_buffering.rs` — derives safety-buffering treatment from HTTP headers; `codex-api/src/safety_buffering.rs:8`.
- `codex-api/src/search.rs` — defines search request, command, setting, and response DTOs; `codex-api/src/search.rs:9`.
- `codex-api/src/telemetry.rs` — wraps HTTP retries with request telemetry and defines SSE/WebSocket telemetry traits; `codex-api/src/telemetry.rs:18`.

## `codex-api/src/endpoint/`

- `codex-api/src/endpoint/compact.rs` — posts compaction input and parses compacted history; `codex-api/src/endpoint/compact.rs:18`.
- `codex-api/src/endpoint/images.rs` — sends image generation/edit requests and decodes responses; `codex-api/src/endpoint/images.rs:18`.
- `codex-api/src/endpoint/memories.rs` — posts memory trace summaries and parses output; `codex-api/src/endpoint/memories.rs:15`.
- `codex-api/src/endpoint/mod.rs` — wires and re-exports endpoint clients; `codex-api/src/endpoint/mod.rs:1`.
- `codex-api/src/endpoint/models.rs` — lists models, appends client version, and returns ETag; `codex-api/src/endpoint/models.rs:14`.
- `codex-api/src/endpoint/realtime_call.rs` — creates WebRTC Realtime calls with SDP/session payloads; `codex-api/src/endpoint/realtime_call.rs:29`.
- `codex-api/src/endpoint/responses.rs` — streams Responses or Chat-compatible inference requests; `codex-api/src/endpoint/responses.rs:52`.
- `codex-api/src/endpoint/responses_websocket.rs` — manages Responses WebSocket connections, streaming, errors, and probes; `codex-api/src/endpoint/responses_websocket.rs:183`.
- `codex-api/src/endpoint/search.rs` — posts search requests and decodes search responses; `codex-api/src/endpoint/search.rs:14`.
- `codex-api/src/endpoint/session.rs` — centralizes endpoint HTTP execution, auth, retry, and telemetry; `codex-api/src/endpoint/session.rs:19`.

## `codex-api/src/endpoint/realtime_websocket/`

- `codex-api/src/endpoint/realtime_websocket/mod.rs` — wires realtime WebSocket methods and protocol modules; `codex-api/src/endpoint/realtime_websocket/mod.rs:1`.
- `codex-api/src/endpoint/realtime_websocket/methods.rs` — owns realtime WebSocket connection, writer, event, transcript, and client logic; `codex-api/src/endpoint/realtime_websocket/methods.rs:211`.
- `codex-api/src/endpoint/realtime_websocket/methods_common.rs` — builds realtime session update JSON and shared method helpers; `codex-api/src/endpoint/realtime_websocket/methods_common.rs:140`.
- `codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs` — tests frameless/session update and handoff behavior; `codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs:16`.
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs` — builds frameless bidi context/session methods with chunk limits; `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs:11`.
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs` — tests frameless context chunking and session JSON; `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs:11`.
- `codex-api/src/endpoint/realtime_websocket/methods_v1.rs` — builds realtime V1 conversation, handoff, session, and intent payloads; `codex-api/src/endpoint/realtime_websocket/methods_v1.rs:19`.
- `codex-api/src/endpoint/realtime_websocket/methods_v2.rs` — builds realtime V2 session/tool/modality payloads; `codex-api/src/endpoint/realtime_websocket/methods_v2.rs:171`.
- `codex-api/src/endpoint/realtime_websocket/protocol.rs` — defines realtime parser, session config, and outbound message shapes; `codex-api/src/endpoint/realtime_websocket/protocol.rs:15`.
- `codex-api/src/endpoint/realtime_websocket/protocol_common.rs` — parses shared realtime payload, transcript, session, and error events; `codex-api/src/endpoint/realtime_websocket/protocol_common.rs:7`.
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs` — parses frameless bidi audio, transcript, and handoff events; `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs:38`.
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs` — tests frameless handoff/audio/transcript decoding; `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs:7`.
- `codex-api/src/endpoint/realtime_websocket/protocol_v1.rs` — parses realtime V1 events; `codex-api/src/endpoint/realtime_websocket/protocol_v1.rs:12`.
- `codex-api/src/endpoint/realtime_websocket/protocol_v2.rs` — parses realtime V2 events, tools, responses, and audio; `codex-api/src/endpoint/realtime_websocket/protocol_v2.rs:24`.

## `codex-api/src/requests/`

- `codex-api/src/requests/chat.rs` — translates Responses requests into Chat Completions bodies; `codex-api/src/requests/chat.rs:19`.
- `codex-api/src/requests/headers.rs` — builds session and subagent headers; `codex-api/src/requests/headers.rs:5`.
- `codex-api/src/requests/mod.rs` — wires request translation modules; `codex-api/src/requests/mod.rs:1`.
- `codex-api/src/requests/responses.rs` — defines request compression choices; `codex-api/src/requests/responses.rs:2`.

## `codex-api/src/sse/`

- `codex-api/src/sse/chat.rs` — translates Chat Completions SSE into Responses-like events; `codex-api/src/sse/chat.rs:77`.
- `codex-api/src/sse/mod.rs` — wires SSE chat and responses modules; `codex-api/src/sse/mod.rs:1`.
- `codex-api/src/sse/responses.rs` — spawns and parses Responses SSE streams; `codex-api/src/sse/responses.rs:36`.
