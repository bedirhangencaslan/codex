# codex-api wire map

## `codex-api/src/`

- `codex-api/src/api_bridge.rs` — Translates crate `ApiError` values into `CodexErr`, decoding overloaded, Cloudflare, cyber-policy, misalignment, and usage-limit bodies and headers. `codex-api/src/api_bridge.rs:21`
- `codex-api/src/api_bridge_tests.rs` — Tests `map_api_error` plus auth and usage-limit header mapping. `codex-api/src/api_bridge_tests.rs:8`
- `codex-api/src/auth.rs` — Defines the `AuthProvider` trait and shared provider for adding, resolving, and applying auth headers. `codex-api/src/auth.rs:30`
- `codex-api/src/common.rs` — Declares Responses request/event wire DTOs, compaction and memory inputs, WebSocket request wrappers, text controls, and `ResponseStream`. `codex-api/src/common.rs:274`
- `codex-api/src/error.rs` — Defines `ApiError` transport, API, stream, quota, rate-limit, and policy variants. `codex-api/src/error.rs:8`
- `codex-api/src/files.rs` — Handles hosted file upload: `POST /files`, blob PUT, finalization/retry, and upload context. `codex-api/src/files.rs:121`
- `codex-api/src/images.rs` — Declares image-generation and image-edit request/response DTOs and quality/background enums. `codex-api/src/images.rs:4`
- `codex-api/src/lib.rs` — Declares crate modules and public re-exports. `codex-api/src/lib.rs:1`
- `codex-api/src/provider.rs` — Defines `Provider` configuration, `WireApi::{Responses, Chat}`, and request/URL builders. `codex-api/src/provider.rs:56`
- `codex-api/src/rate_limits.rs` — Parses `x-codex-*` headers and `codex.rate_limits` events into rate-limit snapshots. `codex-api/src/rate_limits.rs:57`
- `codex-api/src/safety_buffering.rs` — Reads safety-buffering treatment from response headers. `codex-api/src/safety_buffering.rs:8`
- `codex-api/src/search.rs` — Declares search request, input, command, settings, and response wire DTOs. `codex-api/src/search.rs:8`
- `codex-api/src/telemetry.rs` — Defines SSE/WS telemetry traits and the retry wrapper that emits per-request telemetry. `codex-api/src/telemetry.rs:18`

## `codex-api/src/endpoint/`

- `codex-api/src/endpoint/mod.rs` — Declares endpoint submodules and re-exports. `codex-api/src/endpoint/mod.rs:1`
- `codex-api/src/endpoint/compact.rs` — Posts `responses/compact`, captures `x-codex-turn-state`, and parses compact history. `codex-api/src/endpoint/compact.rs:39`
- `codex-api/src/endpoint/images.rs` — Posts image generation/edit requests and captures the imagegen request ID. `codex-api/src/endpoint/images.rs:35`
- `codex-api/src/endpoint/memories.rs` — Posts `memories/trace_summarize`. `codex-api/src/endpoint/memories.rs:36`
- `codex-api/src/endpoint/models.rs` — Gets `models` with the client-version query and ETag handling. `codex-api/src/endpoint/models.rs:46`
- `codex-api/src/endpoint/realtime_call.rs` — Creates realtime calls from SDP/multipart input, parses `Location` call IDs, and handles AVAS/query behavior. `codex-api/src/endpoint/realtime_call.rs:90`
- `codex-api/src/endpoint/responses.rs` — Streams the Responses endpoint over `/responses` or guardian routes and translates to Chat bodies when the provider uses Chat. `codex-api/src/endpoint/responses.rs:112`
- `codex-api/src/endpoint/responses_websocket.rs` — Manages Responses-over-WebSocket connections, handshake, request serialization, and the event/error pump. `codex-api/src/endpoint/responses_websocket.rs:397`
- `codex-api/src/endpoint/search.rs` — Posts `alpha/search`. `codex-api/src/endpoint/search.rs:35`
- `codex-api/src/endpoint/session.rs` — Provides the shared endpoint session that builds requests, applies auth/retry/telemetry, and streams responses. `codex-api/src/endpoint/session.rs:80`

## `codex-api/src/endpoint/realtime_websocket/`

- `codex-api/src/endpoint/realtime_websocket/mod.rs` — Declares realtime WebSocket submodules and public exports. `codex-api/src/endpoint/realtime_websocket/mod.rs:1`
- `codex-api/src/endpoint/realtime_websocket/methods.rs` — Implements the realtime WebSocket client/connection, `WsStream` pump, writer/event APIs, URL construction, and tests. `codex-api/src/endpoint/realtime_websocket/methods.rs:791`
- `codex-api/src/endpoint/realtime_websocket/methods_common.rs` — Routes outbound message construction by V1, Frameless Bidi, or Realtime V2 parser. `codex-api/src/endpoint/realtime_websocket/methods_common.rs:41`
- `codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs` — Tests wire encoding for session updates, handoffs, channels, and call outputs. `codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs:16`
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs` — Builds frameless outbound session/context messages and performs 500-byte chunking. `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs:13`
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs` — Tests chunk preservation and frameless session JSON. `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs:11`
- `codex-api/src/endpoint/realtime_websocket/methods_v1.rs` — Builds legacy V1 outbound conversation, handoff, and session payloads. `codex-api/src/endpoint/realtime_websocket/methods_v1.rs:18`
- `codex-api/src/endpoint/realtime_websocket/methods_v2.rs` — Builds Realtime V2 session payloads, tools, modalities, and function-call outputs. `codex-api/src/endpoint/realtime_websocket/methods_v2.rs:75`
- `codex-api/src/endpoint/realtime_websocket/protocol.rs` — Declares realtime parser/session enums, outbound wire DTOs, and event dispatch. `codex-api/src/endpoint/realtime_websocket/protocol.rs:263`
- `codex-api/src/endpoint/realtime_websocket/protocol_common.rs` — Provides shared JSON helpers for realtime payloads, transcripts, session updates, and errors. `codex-api/src/endpoint/realtime_websocket/protocol_common.rs:7`
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs` — Decodes frameless bidi audio, transcripts, turn-done, delegation, and error events. `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs:15`
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs` — Tests legacy/frameless handoff and frameless event equivalence. `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs:7`
- `codex-api/src/endpoint/realtime_websocket/protocol_v1.rs` — Decodes legacy realtime V1 events. `codex-api/src/endpoint/realtime_websocket/protocol_v1.rs:12`
- `codex-api/src/endpoint/realtime_websocket/protocol_v2.rs` — Decodes Realtime V2 audio, transcripts, items, responses, handoff/noop tools, and errors. `codex-api/src/endpoint/realtime_websocket/protocol_v2.rs:24`

## `codex-api/src/requests/`

- `codex-api/src/requests/mod.rs` — Declares request submodules and exports `Compression`. `codex-api/src/requests/mod.rs:1`
- `codex-api/src/requests/chat.rs` — Translates Responses requests into Chat Completions bodies, including roles, tools, reasoning, and GLM effort clamping. `codex-api/src/requests/chat.rs:19`
- `codex-api/src/requests/headers.rs` — Builds session/thread headers and subagent header labels. `codex-api/src/requests/headers.rs:5`
- `codex-api/src/requests/responses.rs` — Defines the Responses request compression enum (`None` and `Zstd`). `codex-api/src/requests/responses.rs:1`

## `codex-api/src/sse/`

- `codex-api/src/sse/mod.rs` — Declares SSE submodules and stream/event re-exports. `codex-api/src/sse/mod.rs:1`
- `codex-api/src/sse/chat.rs` — Translates Chat Completions SSE into Responses-shaped `ResponseEvent`s while accumulating reasoning, text, tool calls, and usage. `codex-api/src/sse/chat.rs:30`
- `codex-api/src/sse/responses.rs` — Parses Responses SSE into `ResponseEvent`s, including metadata, safety buffering, usage, and error variants. `codex-api/src/sse/responses.rs:353`
