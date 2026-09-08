## `codex-api/src`

- `telemetry.rs` — telemetry interfaces for SSE/WebSocket and retry telemetry wrapping (`codex-api/src/telemetry.rs:18`)
- `files.rs` — hosted/OpenAI file upload create/upload/finalize flow (`codex-api/src/files.rs:121`)
- `auth.rs` — auth errors, async auth-header provider abstraction, shared provider and telemetry hooks (`codex-api/src/auth.rs:30`)
- `api_bridge_tests.rs` — tests for API error, retry-delay, policy, usage, and identity mappings (`codex-api/src/api_bridge_tests.rs:8`)
- `error.rs` — wire/transport `ApiError` variants and rate-limit conversion (`codex-api/src/error.rs:9`)
- `api_bridge.rs` — maps API/transport failures, headers, policies, and usage errors into Codex errors (`codex-api/src/api_bridge.rs:21`)
- `common.rs` — shared request/response DTOs, Responses request shape, WebSocket conversion, and stream type (`codex-api/src/common.rs:274`)
- `rate_limits.rs` — parses rate-limit headers and events into snapshots (`codex-api/src/rate_limits.rs:23`)
- `provider.rs` — provider config, `WireApi`, retry policy, URL/request builders (`codex-api/src/provider.rs:56`)
- `lib.rs` — crate facade/module declarations and selected public re-exports (`codex-api/src/lib.rs:1`)
- `images.rs` — image generation/edit request and response DTOs (`codex-api/src/images.rs:4`)
- `safety_buffering.rs` — parses safety-buffering treatment headers (`codex-api/src/safety_buffering.rs:4`)
- `search.rs` — search request/response wire DTOs and settings (`codex-api/src/search.rs:8`)

## `codex-api/src/sse`

- `responses.rs` — translates Responses SSE plus metadata/rate-limit/error events into `ResponseEvent`s (`codex-api/src/sse/responses.rs:353`)
- `mod.rs` — module exports for SSE stream adapters (`codex-api/src/sse/mod.rs:1`)
- `chat.rs` — translates Chat Completions SSE into Responses-shaped events (`codex-api/src/sse/chat.rs:1`)

## `codex-api/src/requests`

- `responses.rs` — tiny request compression enum (`None`/`Zstd`) (`codex-api/src/requests/responses.rs:1`)
- `mod.rs` — module exports for outbound request translation (`codex-api/src/requests/mod.rs:1`)
- `headers.rs` — session/thread/subagent headers and safe insertion (`codex-api/src/requests/headers.rs:5`)
- `chat.rs` — rewrites Responses requests into Chat Completions bodies, tools, reasoning, and formats (`codex-api/src/requests/chat.rs:18`)

## `codex-api/src/endpoint`

- `mod.rs` — endpoint module registry and public client re-exports (`codex-api/src/endpoint/mod.rs:1`)
- `responses.rs` — HTTP Responses/Chat streaming client, route selection, encoding, headers, SSE spawn (`codex-api/src/endpoint/responses.rs:112`)
- `responses_websocket.rs` — Responses WebSocket connect/probe/request streaming, metadata, errors, and safety buffering (`codex-api/src/endpoint/responses_websocket.rs:235`)
- `compact.rs` — POSTs `responses/compact` and decodes compacted `ResponseItem`s (`codex-api/src/endpoint/compact.rs:39`)
- `session.rs` — shared endpoint execution/streaming with auth, retry, telemetry, and request building (`codex-api/src/endpoint/session.rs:80`)
- `search.rs` — POSTs typed requests to `alpha/search` and parses responses (`codex-api/src/endpoint/search.rs:35`)
- `models.rs` — GETs models with client version query and returns models plus ETag (`codex-api/src/endpoint/models.rs:46`)
- `images.rs` — POSTs image generation/edit requests and captures image request ID (`codex-api/src/endpoint/images.rs:35`)
- `memories.rs` — POSTs memory trace summaries to `memories/trace_summarize` (`codex-api/src/endpoint/memories.rs:36`)
- `realtime_call.rs` — creates WebRTC realtime calls with SDP, backend/API body variants, query params, and call-ID parsing (`codex-api/src/endpoint/realtime_call.rs:90`)

## `codex-api/src/endpoint/realtime_websocket`

- `mod.rs` — realtime submodule exports and parser/config re-exports (`codex-api/src/endpoint/realtime_websocket/mod.rs:1`)
- `protocol.rs` — realtime parser/config enums and outbound wire message shapes; dispatches inbound parsing (`codex-api/src/endpoint/realtime_websocket/protocol.rs:263`)
- `protocol_common.rs` — shared JSON parsing helpers for session, transcript, and error events (`codex-api/src/endpoint/realtime_websocket/protocol_common.rs:7`)
- `protocol_v1.rs` — parses realtime v1 audio, transcript, item, handoff, and error events (`codex-api/src/endpoint/realtime_websocket/protocol_v1.rs:12`)
- `protocol_v2.rs` — parses realtime v2 events, including audio, response lifecycle, handoff, and silence (`codex-api/src/endpoint/realtime_websocket/protocol_v2.rs:24`)
- `protocol_frameless_bidi.rs` — parses Frameless Bidi session/audio/transcript/delegation events (`codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs:15`)
- `protocol_frameless_bidi_tests.rs` — tests Frameless Bidi event compatibility and internal event mapping (`codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs:6`)
- `methods.rs` — realtime WebSocket connection/client, writer/event halves, transcript state, URL/session initialization, and related tests (`codex-api/src/endpoint/realtime_websocket/methods.rs:765`)
- `methods_common.rs` — chooses normalized outbound message construction across v1, v2, and Frameless Bidi (`codex-api/src/endpoint/realtime_websocket/methods_common.rs:29`)
- `methods_common_tests.rs` — tests adapter-specific outbound serialization for session updates, handoffs, context appends, and call outputs (`codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs:15`)
- `methods_v1.rs` — builds realtime v1 outbound conversation items, handoff appends, Quicksilver session updates, and intent (`codex-api/src/endpoint/realtime_websocket/methods_v1.rs:18`)
- `methods_v2.rs` — builds realtime v2 conversation/function-output items and conversational/transcription session updates, tools, audio settings, and intent (`codex-api/src/endpoint/realtime_websocket/methods_v2.rs:39`)
- `methods_frameless_bidi.rs` — builds Frameless session/delegation context appends and session updates, with bounded Unicode-safe chunking (`codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs:13`)
- `methods_frameless_bidi_tests.rs` — tests Frameless context-append chunk limits and session-update JSON shape (`codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs:10`)
