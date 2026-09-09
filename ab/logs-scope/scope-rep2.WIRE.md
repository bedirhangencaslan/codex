# codex-api wire inventory

## `codex-api/src/`

- `codex-api/src/api_bridge.rs` — Maps API and transport failures, including HTTP status bodies, rate-limit/usage errors, cyber/misalignment policies, and tracking headers, into `CodexErr`; `codex-api/src/api_bridge.rs:21`.
- `codex-api/src/api_bridge_tests.rs` — Unit tests for `map_api_error`, covering overload, retry delays, Cloudflare, policy, and HTTP body/error translations; `codex-api/src/api_bridge_tests.rs:8`.
- `codex-api/src/auth.rs` — Defines request-auth translation: header-only auth, async header resolution, complete-request signing, auth errors, and telemetry; `codex-api/src/auth.rs:30`.
- `codex-api/src/common.rs` — Canonical inference wire types: compaction/memory payloads, Responses events, reasoning controls, safety buffering, and shared stream plumbing; `codex-api/src/common.rs:46`.
- `codex-api/src/error.rs` — Defines provider/API error variants used at the wire boundary, including transport, HTTP, quota, policy, retry, and overload errors; `codex-api/src/error.rs:9`.
- `codex-api/src/files.rs` — Implements OpenAI file upload/create, Azure blob streaming, finalize/polling, download URLs, and response parsing; `codex-api/src/files.rs:121`.
- `codex-api/src/images.rs` — Wire models for image generation/edit requests and b64 image responses, including background/quality enums; `codex-api/src/images.rs:4`.
- `codex-api/src/lib.rs` — Declares and publicly exports the crate’s wire modules, clients, providers, auth, streams, search/image/file types, and telemetry; `codex-api/src/lib.rs:1`.
- `codex-api/src/provider.rs` — Configures provider endpoints, headers, query parameters, retry policies, HTTP/WebSocket URL construction, and `Responses` vs `Chat` wire selection; `codex-api/src/provider.rs:38`.
- `codex-api/src/rate_limits.rs` — Parses HTTP and websocket rate-limit headers/events into snapshots, including windows, credits, limit IDs, promo, and reached-type metadata; `codex-api/src/rate_limits.rs:23`.
- `codex-api/src/safety_buffering.rs` — Reads safety-buffering headers and converts them into faster-model treatment metadata; `codex-api/src/safety_buffering.rs:4`.
- `codex-api/src/search.rs` — Defines serialized web-search requests/commands/settings, location, filters, caller, image, finance/weather/sports/time operations, and response types; `codex-api/src/search.rs:8`.
- `codex-api/src/telemetry.rs` — Defines SSE/WebSocket telemetry hooks and wraps retry execution with per-attempt HTTP request telemetry; `codex-api/src/telemetry.rs:18`.

## `codex-api/src/requests/`

- `codex-api/src/requests/chat.rs` — Translates Responses-shaped requests into OpenAI Chat Completions bodies, including messages, tools, reasoning, output format, and GLM effort clamping; `codex-api/src/requests/chat.rs:18`.
- `codex-api/src/requests/headers.rs` — Builds session/thread IDs and maps subagent session sources to outbound headers; `codex-api/src/requests/headers.rs:5`.
- `codex-api/src/requests/mod.rs` — Organizes request modules and re-exports response compression; `codex-api/src/requests/mod.rs:1`.
- `codex-api/src/requests/responses.rs` — Defines outbound Responses request compression options (`None` or Zstd); `codex-api/src/requests/responses.rs:1`.

## `codex-api/src/sse/`

- `codex-api/src/sse/chat.rs` — Translates Chat Completions SSE deltas into Responses-shaped `ResponseEvent`s, rebuilding reasoning, messages, tool calls, usage, and completion; `codex-api/src/sse/chat.rs:30`.
- `codex-api/src/sse/mod.rs` — Organizes SSE modules and exports Chat/Responses stream translation entry points; `codex-api/src/sse/mod.rs:1`.
- `codex-api/src/sse/responses.rs` — Parses Responses SSE frames and response headers into `ResponseEvent`s, including models, limits, turn state, verification, moderation, usage, and safety metadata; `codex-api/src/sse/responses.rs:36`.

## `codex-api/src/endpoint/`

- `codex-api/src/endpoint/compact.rs` — Sends `POST responses/compact`, captures turn state, and parses compacted Responses items; `codex-api/src/endpoint/compact.rs:39`.
- `codex-api/src/endpoint/images.rs` — Sends image generation/edit requests, parses image responses, and captures image-generation request IDs; `codex-api/src/endpoint/images.rs:35`.
- `codex-api/src/endpoint/memories.rs` — Sends memory/trace summarization requests to `memories/trace_summarize` and parses summary output; `codex-api/src/endpoint/memories.rs:36`.
- `codex-api/src/endpoint/mod.rs` — Organizes endpoint modules and exports HTTP, Realtime, Responses, and Search client APIs; `codex-api/src/endpoint/mod.rs:1`.
- `codex-api/src/endpoint/models.rs` — Builds versioned `GET models` URLs, parses model metadata, and extracts ETags; `codex-api/src/endpoint/models.rs:46`.
- `codex-api/src/endpoint/realtime_call.rs` — Creates Realtime WebRTC calls with SDP/session payloads, selecting backend/multipart and API route shapes and parsing call IDs; `codex-api/src/endpoint/realtime_call.rs:90`.
- `codex-api/src/endpoint/responses.rs` — Routes Responses requests over Responses or Chat wire, adds session/subagent headers, encodes/compresses bodies, and dispatches SSE translation; `codex-api/src/endpoint/responses.rs:112`.
- `codex-api/src/endpoint/responses_websocket.rs` — Manages Responses WebSocket connections, request serialization, exclusive streaming, protocol events, close probes, timing, and reconnect metadata; `codex-api/src/endpoint/responses_websocket.rs:235`.
- `codex-api/src/endpoint/search.rs` — Sends search requests to `alpha/search`, serializes typed requests, and parses `SearchResponse`; `codex-api/src/endpoint/search.rs:35`.
- `codex-api/src/endpoint/session.rs` — Shared endpoint plumbing that builds requests, applies auth, runs transport calls with retry/telemetry, and streams encoded JSON; `codex-api/src/endpoint/session.rs:80`.

## `codex-api/src/endpoint/realtime_websocket/`

- `codex-api/src/endpoint/realtime_websocket/methods.rs` — Implements Realtime WebSocket client/connection, message pump, writer/events, URL/header negotiation, and transcript state; `codex-api/src/endpoint/realtime_websocket/methods.rs:211`.
- `codex-api/src/endpoint/realtime_websocket/methods_common.rs` — Dispatches Realtime outbound messages/session JSON by wire adapter (V1, Frameless Bidi, Realtime V2); `codex-api/src/endpoint/realtime_websocket/methods_common.rs:29`.
- `codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs` — Tests adapter-specific serialization of session updates, handoffs, context channels, and completed-message prefixes; `codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs:16`.
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs` — Builds Frameless Bidi session and delegation/context-append messages and chunks oversized text at the wire limit; `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs:13`.
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs` — Tests Frameless chunking and serialization of session/model/voice/initial-item wire JSON; `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs:11`.
- `codex-api/src/endpoint/realtime_websocket/methods_v1.rs` — Builds Realtime V1 conversation-item, handoff, session-update, and `quicksilver` intent messages; `codex-api/src/endpoint/realtime_websocket/methods_v1.rs:18`.
- `codex-api/src/endpoint/realtime_websocket/methods_v2.rs` — Builds Realtime V2 conversation/function-output/session messages with audio, transcription, turn detection, and builtin tools; `codex-api/src/endpoint/realtime_websocket/methods_v2.rs:39`.
- `codex-api/src/endpoint/realtime_websocket/mod.rs` — Organizes and re-exports Realtime protocol/client modules and wire types; `codex-api/src/endpoint/realtime_websocket/mod.rs:1`.
- `codex-api/src/endpoint/realtime_websocket/protocol.rs` — Defines parser/session enums, outbound message/session/audio/content wire schemas, and routes inbound parsing by adapter; `codex-api/src/endpoint/realtime_websocket/protocol.rs:14`.
- `codex-api/src/endpoint/realtime_websocket/protocol_common.rs` — Shared JSON parsing for Realtime events: payload/type extraction, session updates, transcript deltas/done, and errors; `codex-api/src/endpoint/realtime_websocket/protocol_common.rs:7`.
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs` — Parses Frameless Bidi Realtime events into internal events for sessions, audio, transcripts, turn completion, and client delegations; `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs:15`.
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs` — Tests Frameless Bidi parsing and verifies legacy and Frameless handoffs decode to the same internal event; `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs:7`.
- `codex-api/src/endpoint/realtime_websocket/protocol_v1.rs` — Parses Realtime V1 events into internal audio, transcript, conversation-item, handoff, and error events; `codex-api/src/endpoint/realtime_websocket/protocol_v1.rs:12`.
- `codex-api/src/endpoint/realtime_websocket/protocol_v2.rs` — Parses Realtime V2 events into internal audio, transcript, item, response lifecycle, handoff, silence/no-op, and error events; `codex-api/src/endpoint/realtime_websocket/protocol_v2.rs:24`.
