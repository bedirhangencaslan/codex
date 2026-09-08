# codex-api wire inventory

## `codex-api/src/`

- codex-api/src/lib.rs — module registry and public crate re-exports (`lib.rs:1`).
- codex-api/src/common.rs — canonical Responses API request and response-stream types (`common.rs:275`).
- codex-api/src/auth.rs — authentication header/request contract (`auth.rs:30`).
- codex-api/src/api_bridge.rs — maps API and transport errors into `CodexErr` (`api_bridge.rs:21`).
- codex-api/src/api_bridge_tests.rs — tests overloaded, retry-delay, and blocked-error mapping (`api_bridge_tests.rs:8`).
- codex-api/src/error.rs — `ApiError` variants (`error.rs:9`).
- codex-api/src/telemetry.rs — SSE/WS telemetry plus retry-aware request telemetry (`telemetry.rs:68`).
- codex-api/src/provider.rs — provider endpoint, retry, and wire configuration (`provider.rs:56`).
- codex-api/src/rate_limits.rs — parses rate-limit headers and events (`rate_limits.rs:28`).
- codex-api/src/search.rs — search request, response, and operation DTOs (`search.rs:9`).
- codex-api/src/safety_buffering.rs — safety-buffering treatment headers (`safety_buffering.rs:8`).
- codex-api/src/files.rs — hosted file upload orchestration (`files.rs:121`).
- codex-api/src/images.rs — image generation and edit DTOs (`images.rs:5`).

## `codex-api/src/sse/`

- codex-api/src/sse/mod.rs — SSE module registry (`mod.rs:1`).
- codex-api/src/sse/responses.rs — translates Responses SSE events and headers into `ResponseEvent`s (`responses.rs:353`).
- codex-api/src/sse/chat.rs — translates Chat Completions SSE into Responses-shaped events (`chat.rs:77`).

## `codex-api/src/requests/`

- codex-api/src/requests/mod.rs — request module registry and `Compression` export (`mod.rs:1`).
- codex-api/src/requests/headers.rs — session, thread, and subagent header construction (`headers.rs:5`).
- codex-api/src/requests/chat.rs — outbound Responses-to-Chat request translation (`chat.rs:19`).
- codex-api/src/requests/responses.rs — defines Responses request compression (`responses.rs:1`).

## `codex-api/src/endpoint/`

- codex-api/src/endpoint/mod.rs — endpoint module registry and public exports (`mod.rs:1`).
- codex-api/src/endpoint/compact.rs — POSTs `responses/compact` (`compact.rs:39`).
- codex-api/src/endpoint/images.rs — POSTs image generation and edit endpoints (`images.rs:35`).
- codex-api/src/endpoint/models.rs — GETs `models` and handles ETags (`models.rs:46`).
- codex-api/src/endpoint/memories.rs — POSTs memory trace summarization (`memories.rs:36`).
- codex-api/src/endpoint/search.rs — POSTs `alpha/search` (`search.rs:35`).
- codex-api/src/endpoint/session.rs — shared authenticated and telemetried HTTP endpoint execution (`session.rs:80`).
- codex-api/src/endpoint/responses.rs — Responses and Chat HTTP streaming client with wire routing (`responses.rs:112`).
- codex-api/src/endpoint/responses_websocket.rs — connects, probes, and streams Responses WebSocket events (`responses_websocket.rs:397`).
- codex-api/src/endpoint/realtime_call.rs — creates WebRTC realtime calls and parses SDP and call IDs (`realtime_call.rs:90`).

## `codex-api/src/endpoint/realtime_websocket/`

- codex-api/src/endpoint/realtime_websocket/mod.rs — realtime WebSocket module registry (`mod.rs:1`).
- codex-api/src/endpoint/realtime_websocket/protocol.rs — realtime parser dispatch, outbound messages, and session payload types (`protocol.rs:263`).
- codex-api/src/endpoint/realtime_websocket/protocol_common.rs — shared realtime JSON, transcript, session, and error parsing (`protocol_common.rs:7`).
- codex-api/src/endpoint/realtime_websocket/protocol_v1.rs — realtime v1 inbound event parser (`protocol_v1.rs:12`).
- codex-api/src/endpoint/realtime_websocket/protocol_v2.rs — realtime v2 inbound event parser (`protocol_v2.rs:24`).
- codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs — frameless bidirectional inbound event parser (`protocol_frameless_bidi.rs:15`).
- codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs — tests frameless and legacy event equivalence (`protocol_frameless_bidi_tests.rs:6`).
- codex-api/src/endpoint/realtime_websocket/methods.rs — realtime WebSocket client, connection, event handling, URLs, and transcripts (`methods.rs:791`).
- codex-api/src/endpoint/realtime_websocket/methods_common.rs — dispatches realtime outbound/session construction across wire adapters (`methods_common.rs:41`).
- codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs — tests shared realtime wire-adapter encodings (`methods_common_tests.rs:16`).
- codex-api/src/endpoint/realtime_websocket/methods_v1.rs — builds realtime v1 outbound messages and sessions (`methods_v1.rs:18`).
- codex-api/src/endpoint/realtime_websocket/methods_v2.rs — builds realtime v2 outbound messages, sessions, and tools (`methods_v2.rs:39`).
- codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs — builds frameless context updates and chunks context append text (`methods_frameless_bidi.rs:13`).
- codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs — tests frameless chunking and session JSON (`methods_frameless_bidi_tests.rs:11`).
