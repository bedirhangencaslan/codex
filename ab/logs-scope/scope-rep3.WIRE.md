# codex-api wire inventory

## codex-api/src

- `codex-api/src/api_bridge.rs` — Translates provider API and transport failures into `CodexErr`, including policy, overload, usage-limit, and encoded-error handling. (`codex-api/src/api_bridge.rs:21`)
- `codex-api/src/api_bridge_tests.rs` — Tests API error mapping across overload, Cloudflare, policy, usage-limit, and encoded-error payloads. (`codex-api/src/api_bridge_tests.rs:8`)
- `codex-api/src/auth.rs` — Handles auth errors, providers, async header resolution/request signing, shared auth state, and auth telemetry. (`codex-api/src/auth.rs:30`)
- `codex-api/src/common.rs` — Defines shared Responses, compaction, memory, reasoning, tool, metadata, WebSocket request, and response-stream wire types. (`codex-api/src/common.rs:47`)
- `codex-api/src/error.rs` — Defines the provider API error model and rate-limit error conversion. (`codex-api/src/error.rs:9`)
- `codex-api/src/files.rs` — Translates file-upload creation, Azure blob PUT, finalize/retry, and sediment URI diagnostics. (`codex-api/src/files.rs:121`)
- `codex-api/src/images.rs` — Defines image generation and image editing request/response wire DTOs. (`codex-api/src/images.rs:4`)
- `codex-api/src/lib.rs` — Declares the crate modules and public API exports. (`codex-api/src/lib.rs:1`)
- `codex-api/src/provider.rs` — Defines retry configuration, Responses/Chat wire selection, provider URLs, and Azure detection. (`codex-api/src/provider.rs:38`)
- `codex-api/src/rate_limits.rs` — Parses rate-limit headers across default, per-limit, family, event, promo, reached, and credit forms. (`codex-api/src/rate_limits.rs:22`)
- `codex-api/src/safety_buffering.rs` — Parses safety-buffering headers into treatment metadata. (`codex-api/src/safety_buffering.rs:8`)
- `codex-api/src/search.rs` — Defines search requests, responses, operations, and settings wire DTOs. (`codex-api/src/search.rs:8`)
- `codex-api/src/telemetry.rs` — Defines SSE/WebSocket telemetry traits and retry-aware request telemetry. (`codex-api/src/telemetry.rs:17`)

## codex-api/src/endpoint

- `codex-api/src/endpoint/compact.rs` — Posts `responses/compact`, parses compacted output items, and captures turn state. (`codex-api/src/endpoint/compact.rs:39`)
- `codex-api/src/endpoint/images.rs` — Calls image generation/edit endpoints, parses responses, and returns request IDs. (`codex-api/src/endpoint/images.rs:35`)
- `codex-api/src/endpoint/memories.rs` — Posts memory trace summarization and decodes the summarize output. (`codex-api/src/endpoint/memories.rs:36`)
- `codex-api/src/endpoint/mod.rs` — Registers endpoint modules and public client exports. (`codex-api/src/endpoint/mod.rs:1`)
- `codex-api/src/endpoint/models.rs` — Calls the models endpoint with client version and parses models plus ETag. (`codex-api/src/endpoint/models.rs:46`)
- `codex-api/src/endpoint/realtime_call.rs` — Creates WebRTC realtime calls and parses SDP, backend JSON, multipart API responses, and call IDs. (`codex-api/src/endpoint/realtime_call.rs:90`)
- `codex-api/src/endpoint/responses.rs` — Routes Responses-compatible requests over Responses or Chat HTTP APIs with headers, compression, retries, and SSE dispatch. (`codex-api/src/endpoint/responses.rs:112`)
- `codex-api/src/endpoint/responses_websocket.rs` — Connects and pumps Responses WebSocket sessions, serializes requests, and converts wire events into `ResponseEvent`s. (`codex-api/src/endpoint/responses_websocket.rs:685`)
- `codex-api/src/endpoint/search.rs` — Posts alpha search requests and decodes search responses. (`codex-api/src/endpoint/search.rs:35`)
- `codex-api/src/endpoint/session.rs` — Provides the shared endpoint session for provider requests, auth, retries, telemetry, unary calls, and streaming. (`codex-api/src/endpoint/session.rs:63`)

## codex-api/src/endpoint/realtime_websocket

- `codex-api/src/endpoint/realtime_websocket/methods.rs` — Implements realtime WebSocket clients, connections, I/O, serialization, session setup, transcripts, and URL construction. (`codex-api/src/endpoint/realtime_websocket/methods.rs:937`)
- `codex-api/src/endpoint/realtime_websocket/methods_common.rs` — Builds outbound realtime messages shared by V1, Frameless Bidi, and Realtime V2 adapters. (`codex-api/src/endpoint/realtime_websocket/methods_common.rs:29`)
- `codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs` — Tests shared outbound message encoding and adapter behavior. (`codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs:15`)
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs` — Builds Frameless Bidi delegation, session-context, JSON, and chunked-context messages. (`codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs:13`)
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs` — Tests Frameless Bidi context chunking and session JSON serialization. (`codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs:10`)
- `codex-api/src/endpoint/realtime_websocket/methods_v1.rs` — Builds legacy realtime V1 outbound item, handoff, session-update, and intent messages. (`codex-api/src/endpoint/realtime_websocket/methods_v1.rs:18`)
- `codex-api/src/endpoint/realtime_websocket/methods_v2.rs` — Builds Realtime V2 outbound outputs, conversation/transcription sessions, tools, and intents. (`codex-api/src/endpoint/realtime_websocket/methods_v2.rs:39`)
- `codex-api/src/endpoint/realtime_websocket/mod.rs` — Registers realtime WebSocket modules and exports. (`codex-api/src/endpoint/realtime_websocket/mod.rs:1`)
- `codex-api/src/endpoint/realtime_websocket/protocol.rs` — Defines the realtime protocol adapter enum, configuration, outbound JSON shapes, and parser dispatch. (`codex-api/src/endpoint/realtime_websocket/protocol.rs:14`)
- `codex-api/src/endpoint/realtime_websocket/protocol_common.rs` — Parses shared realtime JSON for session updates, transcript deltas/completion, and errors. (`codex-api/src/endpoint/realtime_websocket/protocol_common.rs:7`)
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs` — Parses Frameless Bidi session, audio, transcript, turn, and delegation events. (`codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs:15`)
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs` — Tests Frameless Bidi event parsing and legacy parity. (`codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs:6`)
- `codex-api/src/endpoint/realtime_websocket/protocol_v1.rs` — Parses legacy realtime V1 audio, transcript, item, handoff, and error events. (`codex-api/src/endpoint/realtime_websocket/protocol_v1.rs:12`)
- `codex-api/src/endpoint/realtime_websocket/protocol_v2.rs` — Parses Realtime V2 audio, transcript, item, lifecycle, handoff, and noop events. (`codex-api/src/endpoint/realtime_websocket/protocol_v2.rs:24`)

## codex-api/src/requests

- `codex-api/src/requests/chat.rs` — Translates Responses-shaped requests into Chat Completions messages, tools, reasoning, and provider-specific limits. (`codex-api/src/requests/chat.rs:19`)
- `codex-api/src/requests/headers.rs` — Builds session, thread, and subagent request headers. (`codex-api/src/requests/headers.rs:5`)
- `codex-api/src/requests/mod.rs` — Registers request modules and exports `Compression`. (`codex-api/src/requests/mod.rs:1`)
- `codex-api/src/requests/responses.rs` — Defines request compression options for Responses calls. (`codex-api/src/requests/responses.rs:1`)

## codex-api/src/sse

- `codex-api/src/sse/chat.rs` — Translates Chat Completions SSE into Responses-shaped events while accumulating reasoning, content, tools, and usage. (`codex-api/src/sse/chat.rs:30`)
- `codex-api/src/sse/mod.rs` — Registers and exports SSE translation modules. (`codex-api/src/sse/mod.rs:1`)
- `codex-api/src/sse/responses.rs` — Translates Responses SSE into response events and handles metadata, usage, errors, retries, turn state, and safety buffering. (`codex-api/src/sse/responses.rs:353`)
