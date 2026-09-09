# Wire Coverage

## seed-rs/codex-api/src

- `seed-rs/codex-api/src/api_bridge.rs` — maps API and transport failures into `CodexErr`, including request IDs, Cloudflare blocks, usage, and rate limits (`api_bridge.rs:14`).
- `seed-rs/codex-api/src/api_bridge_tests.rs` — tests API and transport failure translation (`api_bridge_tests.rs:1`).
- `seed-rs/codex-api/src/auth.rs` — defines the API auth-provider contract and auth telemetry hooks (`auth.rs:21`, `auth.rs:53`).
- `seed-rs/codex-api/src/common.rs` — defines shared Responses request, event, stream, error, and usage wire types (`common.rs:130`).
- `seed-rs/codex-api/src/error.rs` — defines the API transport and protocol error variants (`error.rs:6`).
- `seed-rs/codex-api/src/files.rs` — handles OpenAI file creation, authorized uploads, blob PUTs, finalization, and retry behavior (`files.rs:121`, `files.rs:185`, `files.rs:256`).
- `seed-rs/codex-api/src/images.rs` — defines image generation and edit request/response wire types (`images.rs:3`).
- `seed-rs/codex-api/src/lib.rs` — declares and re-exports the crate's API modules (`lib.rs:1`).
- `seed-rs/codex-api/src/provider.rs` — defines provider configuration, `WireApi`, and endpoint URL construction (`provider.rs:48`, `provider.rs:68`).
- `seed-rs/codex-api/src/rate_limits.rs` — parses rate-limit headers into codex rate-limit structures (`rate_limits.rs:18`).
- `seed-rs/codex-api/src/safety_buffering.rs` — parses safety-buffering treatment headers (`safety_buffering.rs:12`).
- `seed-rs/codex-api/src/search.rs` — defines search request, command, and response wire types (`search.rs:7`).
- `seed-rs/codex-api/src/telemetry.rs` — defines SSE and WebSocket telemetry hooks (`telemetry.rs:14`, `telemetry.rs:22`).

## seed-rs/codex-api/src/endpoint

- `seed-rs/codex-api/src/endpoint/mod.rs` — declares the endpoint modules (`mod.rs:1`).
- `seed-rs/codex-api/src/endpoint/compact.rs` — POSTs Responses compaction requests (`compact.rs:23`).
- `seed-rs/codex-api/src/endpoint/images.rs` — POSTs image generation and edit requests (`images.rs:25`).
- `seed-rs/codex-api/src/endpoint/memories.rs` — POSTs memory trace-summary requests (`memories.rs:23`).
- `seed-rs/codex-api/src/endpoint/models.rs` — GETs the available model list (`models.rs:30`).
- `seed-rs/codex-api/src/endpoint/realtime_call.rs` — POSTs WebRTC SDP offers to establish realtime calls (`realtime_call.rs:57`).
- `seed-rs/codex-api/src/endpoint/responses.rs` — translates Responses and Chat requests and routes them to their endpoint transport (`responses.rs:80`).
- `seed-rs/codex-api/src/endpoint/responses_websocket.rs` — connects and probes Responses WebSockets, serializes requests, and maps wrapped errors into event streams (`responses_websocket.rs:344`, `responses_websocket.rs:397`, `responses_websocket.rs:685`, `responses_websocket.rs:728`, `responses_websocket.rs:920`).
- `seed-rs/codex-api/src/endpoint/search.rs` — POSTs alpha search requests (`search.rs:18`).
- `seed-rs/codex-api/src/endpoint/session.rs` — executes shared authenticated endpoint requests (`session.rs:45`).

## seed-rs/codex-api/src/endpoint/realtime_websocket

- `seed-rs/codex-api/src/endpoint/realtime_websocket/mod.rs` — declares realtime protocol and method modules (`mod.rs:1`).
- `seed-rs/codex-api/src/endpoint/realtime_websocket/methods.rs` — handles realtime WebSocket client and connection lifecycles, URL construction, and the event pump (`methods.rs:390`, `methods.rs:400`, `methods.rs:590`, `methods.rs:650`).
- `seed-rs/codex-api/src/endpoint/realtime_websocket/methods_common.rs` — dispatches realtime method calls by protocol version (`methods_common.rs:26`).
- `seed-rs/codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs` — tests common realtime method dispatch (`methods_common_tests.rs:1`).
- `seed-rs/codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs` — builds frameless bidirectional messages and chunks 500-byte context (`methods_frameless_bidi.rs:20`, `methods_frameless_bidi.rs:65`).
- `seed-rs/codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs` — tests frameless bidirectional message construction (`methods_frameless_bidi_tests.rs:1`).
- `seed-rs/codex-api/src/endpoint/realtime_websocket/methods_v1.rs` — builds realtime v1 outbound messages and `quicksilver` intents (`methods_v1.rs:8`, `methods_v1.rs:50`).
- `seed-rs/codex-api/src/endpoint/realtime_websocket/methods_v2.rs` — builds realtime v2 outbound messages, tool outputs, `background_agent`, and `remain_silent` controls (`methods_v2.rs:10`, `methods_v2.rs:24`).
- `seed-rs/codex-api/src/endpoint/realtime_websocket/protocol.rs` — defines realtime parser, outbound, configuration, and session types (`protocol.rs:12`, `protocol.rs:30`).
- `seed-rs/codex-api/src/endpoint/realtime_websocket/protocol_common.rs` — parses shared realtime payloads, sessions, transcripts, and errors (`protocol_common.rs:6`).
- `seed-rs/codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs` — parses frameless bidirectional inbound events (`protocol_frameless_bidi.rs:15`).
- `seed-rs/codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs` — tests frameless bidirectional event parsing (`protocol_frameless_bidi_tests.rs:1`).
- `seed-rs/codex-api/src/endpoint/realtime_websocket/protocol_v1.rs` — parses realtime v1 inbound events (`protocol_v1.rs:15`).
- `seed-rs/codex-api/src/endpoint/realtime_websocket/protocol_v2.rs` — parses realtime v2 inbound events (`protocol_v2.rs:17`).

## seed-rs/codex-api/src/requests

- `seed-rs/codex-api/src/requests/chat.rs` — translates outbound Responses requests into Chat Completions payloads (`chat.rs:18`, `chat.rs:208`, `chat.rs:260`).
- `seed-rs/codex-api/src/requests/headers.rs` — builds session, thread, and subagent request headers (`headers.rs:5`, `headers.rs:16`).
- `seed-rs/codex-api/src/requests/mod.rs` — declares request translation modules (`mod.rs:1`).
- `seed-rs/codex-api/src/requests/responses.rs` — defines compression encodings accepted by Responses requests (`responses.rs:1`).

## seed-rs/codex-api/src/sse

- `seed-rs/codex-api/src/sse/chat.rs` — translates inbound Chat Completions SSE into Responses stream events (`sse/chat.rs:30`, `sse/chat.rs:77`, `sse/chat.rs:246`).
- `seed-rs/codex-api/src/sse/mod.rs` — declares and re-exports SSE translation modules (`sse/mod.rs:1`).
- `seed-rs/codex-api/src/sse/responses.rs` — parses Responses SSE, verifies metadata and model output, tracks safety buffering, and classifies completion and error events (`sse/responses.rs:36`, `sse/responses.rs:169`, `sse/responses.rs:353`, `sse/responses.rs:557`).
