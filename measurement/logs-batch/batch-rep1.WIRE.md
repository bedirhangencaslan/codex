# codex-api wire map

## `codex-api/src`

- `codex-api/src/api_bridge.rs` — Translates `ApiError` and transport failures into `CodexErr`, preserving retry timing, rate limits, policy messages, and request tracking metadata. Backed by `codex-api/src/api_bridge.rs:21`.
- `codex-api/src/api_bridge_tests.rs` — Tests the API-to-protocol error bridge, including overload, retry-delay, Cloudflare, cyber/misalignment policy, usage-limit, and identity-auth mappings. Backed by `codex-api/src/api_bridge_tests.rs:7`.
- `codex-api/src/auth.rs` — Handles outbound API authentication through a provider trait that can add static headers, asynchronously resolve them, or sign a complete request; also exposes shared auth handles and auth telemetry. Backed by `codex-api/src/auth.rs:30`.
- `codex-api/src/common.rs` — Defines the canonical Responses request, event, stream, reasoning, text, tools, and WebSocket request types shared by inference endpoints. Backed by `codex-api/src/common.rs:275`.
- `codex-api/src/error.rs` — Defines `ApiError`, normalizing transport, HTTP, stream, context-window, quota, rate-limit, policy, invalid-request, and overload failures. Backed by `codex-api/src/error.rs:9`.
- `codex-api/src/files.rs` — Handles hosted OpenAI file upload and metadata retrieval: create, streaming blob upload, finalize, download URL, limits, and `sediment://` file URIs. Backed by `codex-api/src/files.rs:121`.
- `codex-api/src/images.rs` — Defines image generation/edit wire requests and image responses, including URL/background/quality/base64-data fields. Backed by `codex-api/src/images.rs:5`.
- `codex-api/src/lib.rs` — Organizes the crate modules and exports the public API surface for providers, auth, endpoints, requests, SSE, files, search, and telemetry. Backed by `codex-api/src/lib.rs:1`.
- `codex-api/src/provider.rs` — Handles concrete provider endpoint configuration: URLs, headers/query parameters, retries, stream timeouts, Responses versus Chat wire selection, and Azure detection. Backed by `codex-api/src/provider.rs:56`.
- `codex-api/src/rate_limits.rs` — Translates HTTP rate-limit header families, credits snapshots, promo messages, and WebSocket rate-limit events into protocol `RateLimitSnapshot` values. Backed by `codex-api/src/rate_limits.rs:28`.
- `codex-api/src/safety_buffering.rs` — Translates safety-buffering HTTP headers into a treatment value carrying the faster-model setting. Backed by `codex-api/src/safety_buffering.rs:8`.
- `codex-api/src/search.rs` — Defines the search endpoint wire schema, including text/item input, browsing/finance/weather/sports/time operations, filters, settings, and response data. Backed by `codex-api/src/search.rs:9`.
- `codex-api/src/telemetry.rs` — Defines SSE and Responses-WebSocket telemetry callbacks and request telemetry/retry plumbing used around wire calls. Backed by `codex-api/src/telemetry.rs:18`.

## `codex-api/src/endpoint`

- `codex-api/src/endpoint/compact.rs` — Handles the `responses/compact` endpoint: serializes compaction input, executes the POST, decodes compacted history, and captures turn state. Backed by `codex-api/src/endpoint/compact.rs:39`.
- `codex-api/src/endpoint/images.rs` — Handles image generation and edit endpoint calls, serializes their requests, decodes responses, and returns the image-generation request ID. Backed by `codex-api/src/endpoint/images.rs:35`.
- `codex-api/src/endpoint/memories.rs` — Handles the memory trace-summarization endpoint and decodes the resulting memory summaries. Backed by `codex-api/src/endpoint/memories.rs:36`.
- `codex-api/src/endpoint/mod.rs` — Organizes endpoint modules and re-exports their public clients and realtime wire types. Backed by `codex-api/src/endpoint/mod.rs:12`.
- `codex-api/src/endpoint/models.rs` — Handles model listing, including client-version request URLs, response decoding, and ETag extraction. Backed by `codex-api/src/endpoint/models.rs:46`.
- `codex-api/src/endpoint/realtime_call.rs` — Handles WebRTC realtime call creation: builds backend or OpenAI-compatible requests, posts SDP, and extracts the SDP answer and call ID. Backed by `codex-api/src/endpoint/realtime_call.rs:90`.
- `codex-api/src/endpoint/responses.rs` — Handles HTTP Responses inference calls, selecting the Responses/Guardian route and Chat versus Responses wire, applying headers/compression, and starting the matching SSE stream. Backed by `codex-api/src/endpoint/responses.rs:29`.
- `codex-api/src/endpoint/responses_websocket.rs` — Handles Responses WebSocket connections and handshake probes, request serialization/reuse, event pumping into `ResponseEvent`s, close/error mapping, rate limits, safety buffering, and timing telemetry. Backed by `codex-api/src/endpoint/responses_websocket.rs:344`.
- `codex-api/src/endpoint/search.rs` — Handles the `alpha/search` endpoint by serializing `SearchRequest` and decoding `SearchResponse`. Backed by `codex-api/src/endpoint/search.rs:35`.
- `codex-api/src/endpoint/session.rs` — Provides the shared endpoint execution session that combines provider URLs, auth, transport, bodies, headers, telemetry, retries, and streaming. Backed by `codex-api/src/endpoint/session.rs:19`.

## `codex-api/src/endpoint/realtime_websocket`

- `codex-api/src/endpoint/realtime_websocket/methods.rs` — Handles realtime WebSocket client lifecycle and transport: URL construction, connection/reconnection, writer and event streams, audio/transcript handling, handoffs, and wire logging. Backed by `codex-api/src/endpoint/realtime_websocket/methods.rs:765`.
- `codex-api/src/endpoint/realtime_websocket/methods_common.rs` — Translates semantic realtime operations into outbound wire messages by dispatching to the V1, Frameless Bidi, or Realtime V2 adapter. Backed by `codex-api/src/endpoint/realtime_websocket/methods_common.rs:41`.
- `codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs` — Tests adapter-specific outbound realtime session updates, handoff output, function-call output, audio, and intent messages. Backed by `codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs:15`.
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs` — Builds Frameless Bidi outbound session/context messages and chunks text appends to the wire byte limit. Backed by `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs:11`.
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs` — Tests Frameless Bidi chunking limits and session JSON serialization, including model, instructions, roles, and delegation fields. Backed by `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs:10`.
- `codex-api/src/endpoint/realtime_websocket/methods_v1.rs` — Builds realtime V1 outbound conversation, handoff, intent, and session-update messages. Backed by `codex-api/src/endpoint/realtime_websocket/methods_v1.rs:18`.
- `codex-api/src/endpoint/realtime_websocket/methods_v2.rs` — Builds Realtime V2 outbound messages and richer session configuration, including modalities, audio formats, transcription, tools, and background-agent/silence tools. Backed by `codex-api/src/endpoint/realtime_websocket/methods_v2.rs:39`.
- `codex-api/src/endpoint/realtime_websocket/mod.rs` — Organizes realtime method and protocol modules and re-exports realtime client, stream, parser, and session types. Backed by `codex-api/src/endpoint/realtime_websocket/mod.rs:12`.
- `codex-api/src/endpoint/realtime_websocket/protocol.rs` — Defines the realtime adapter selector, session/config types, outbound wire message envelope, conversation/session wire shapes, and event dispatch. Backed by `codex-api/src/endpoint/realtime_websocket/protocol.rs:15`.
- `codex-api/src/endpoint/realtime_websocket/protocol_common.rs` — Provides shared realtime inbound JSON parsing for typed payloads, session updates, transcript deltas/dones, and errors. Backed by `codex-api/src/endpoint/realtime_websocket/protocol_common.rs:7`.
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs` — Translates Frameless Bidi JSON events into internal realtime audio, transcript, delegation/handoff, session, and error events. Backed by `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs:15`.
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs` — Tests Frameless Bidi decoding compatibility with legacy handoffs and reuse of internal transcript/audio events. Backed by `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs:6`.
- `codex-api/src/endpoint/realtime_websocket/protocol_v1.rs` — Translates realtime V1 JSON events into internal audio, transcript, conversation-item, handoff, and error events. Backed by `codex-api/src/endpoint/realtime_websocket/protocol_v1.rs:12`.
- `codex-api/src/endpoint/realtime_websocket/protocol_v2.rs` — Translates Realtime V2 JSON events into audio, transcripts, response lifecycle, tool-call/background-agent, speech-start, and error events. Backed by `codex-api/src/endpoint/realtime_websocket/protocol_v2.rs:24`.

## `codex-api/src/requests`

- `codex-api/src/requests/chat.rs` — Translates Responses-shaped requests into OpenAI Chat Completions request bodies, including message roles, images, tools, reasoning anchoring, and tool-call grouping. Backed by `codex-api/src/requests/chat.rs:19`.
- `codex-api/src/requests/headers.rs` — Builds session/thread headers and translates sub-agent session sources into request headers. Backed by `codex-api/src/requests/headers.rs:5`.
- `codex-api/src/requests/mod.rs` — Organizes request translation modules and exports the request compression setting. Backed by `codex-api/src/requests/mod.rs:5`.
- `codex-api/src/requests/responses.rs` — Defines whether Responses request bodies are sent uncompressed or with Zstandard compression. Backed by `codex-api/src/requests/responses.rs:2`.

## `codex-api/src/sse`

- `codex-api/src/sse/chat.rs` — Translates OpenAI-compatible Chat Completions SSE deltas into Responses-shaped `ResponseEvent`s while accumulating assistant text, reasoning, tool calls, usage, and finish state. Backed by `codex-api/src/sse/chat.rs:30`.
- `codex-api/src/sse/mod.rs` — Organizes SSE modules and exports both Chat and Responses stream processing APIs. Backed by `codex-api/src/sse/mod.rs:1`.
- `codex-api/src/sse/responses.rs` — Translates Responses API SSE events into `ResponseEvent`s, decoding items, usage, rate limits, model/turn-state headers, safety buffering, completion, retries, and protocol errors. Backed by `codex-api/src/sse/responses.rs:36`.
