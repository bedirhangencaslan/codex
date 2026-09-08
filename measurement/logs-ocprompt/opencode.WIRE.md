# WIRE.md — codex-api/src/ wire map

One line per `.rs` file under `codex-api/src/`: what it translates or handles on the wire, with a `file:line` citation.

## codex-api/src/

- `codex-api/src/lib.rs` — Crate root: declares all submodules and re-exports the wire-facing public API (clients, request/response types, error, provider, telemetry). Cite: `codex-api/src/lib.rs:1-14`
- `codex-api/src/api_bridge.rs` — Translates wire-level `ApiError`s into internal `CodexErr`s, decoding HTTP bodies for overloaded/cyber-policy/misalignment/usage-limit responses and rate-limit headers. Cite: `codex-api/src/api_bridge.rs:21-54`
- `codex-api/src/api_bridge_tests.rs` — Tests for `map_api_error`: server-overloaded mapping, retry-delay preservation, and 503-body decoding. Cite: `codex-api/src/api_bridge_tests.rs:8-52`
- `codex-api/src/auth.rs` — Handles auth on outbound requests: `AuthProvider` trait for adding/resolving auth headers or signing whole requests, plus auth-telemetry helpers. Cite: `codex-api/src/auth.rs:30-71`
- `codex-api/src/common.rs` — Shared wire types: `ResponsesApiRequest`-adjacent payloads (`CompactionInput`, `MemorySummarizeInput`), access-program wire values, the `ResponseEvent` stream model, and trace-metadata keys. Cite: `codex-api/src/common.rs:98-146`
- `codex-api/src/error.rs` — Defines `ApiError`, the crate-wide error vocabulary for API/stream/rate-limit/policy failures before bridging. Cite: `codex-api/src/error.rs:8-45`
- `codex-api/src/files.rs` — Handles hosted file upload wire flow (`sediment://` URIs): create-file, blob upload, finalize, and download-link requests against OpenAI file endpoints. Cite: `codex-api/src/files.rs:17-118`
- `codex-api/src/images.rs` — Request/response wire types for image generation and edit (`images/generations`, `images/edits`): prompt, size/quality/background params, base64 image data. Cite: `codex-api/src/images.rs:4-70`
- `codex-api/src/provider.rs` — Configures HTTP endpoints per provider: base URL/query params/retry policy, `WireApi` (Responses vs Chat), URL building, and Azure Responses detection. Cite: `codex-api/src/provider.rs:44-114`
- `codex-api/src/rate_limits.rs` — Parses rate-limit headers (`x-<limit>-primary/secondary-used-percent`, reset-at, limit-name, credits) into `RateLimitSnapshot`s. Cite: `codex-api/src/rate_limits.rs:23-101`
- `codex-api/src/safety_buffering.rs` — Reads the `x-codex-safety-buffering-*` response headers into a `SafetyBufferingTreatment`. Cite: `codex-api/src/safety_buffering.rs:4-20`
- `codex-api/src/search.rs` — Wire types for the `alpha/search` endpoint: `SearchRequest`/`SearchResponse`, search commands (web/image queries, open/click/find, finance/weather/sports), filters and settings. Cite: `codex-api/src/search.rs:8-78`
- `codex-api/src/telemetry.rs` — Telemetry hooks for the wire: `SseTelemetry` (SSE polls) and `WebsocketTelemetry` (WS requests/events), plus retry-wrapping with per-attempt request telemetry. Cite: `codex-api/src/telemetry.rs:17-43`

## codex-api/src/endpoint/

- `codex-api/src/endpoint/mod.rs` — Endpoint module root: declares endpoint clients and re-exports them (Compact, Images, Memories, Models, Realtime, Responses, Search). Cite: `codex-api/src/endpoint/mod.rs:1-36`
- `codex-api/src/endpoint/compact.rs` — `CompactClient`: POSTs the compaction payload to `responses/compact` and decodes the compacted `ResponseItem` output, capturing turn-state headers. Cite: `codex-api/src/endpoint/compact.rs:35-88`
- `codex-api/src/endpoint/images.rs` — `ImagesClient`: POSTs generation/edit requests to `images/generations`/`images/edits` and decodes `ImageResponse` plus the imagegen request-id header. Cite: `codex-api/src/endpoint/images.rs:35-80`
- `codex-api/src/endpoint/memories.rs` — `MemoriesClient`: POSTs memory summarize input to `memories/trace_summarize` and decodes `MemorySummarizeOutput` records. Cite: `codex-api/src/endpoint/memories.rs:32-65`
- `codex-api/src/endpoint/models.rs` — `ModelsClient`: GETs `models` with a `client_version` query param and decodes `ModelsResponse` along with the `ETag` header. Cite: `codex-api/src/endpoint/models.rs:31-79`
- `codex-api/src/endpoint/realtime_call.rs` — `RealtimeCallClient`: creates WebRTC realtime calls via POST `realtime/calls` with SDP (or multipart session+SDP), parses `Location` for `call_id`, and builds sideband WebSocket join URLs. Cite: `codex-api/src/endpoint/realtime_call.rs:33-120`
- `codex-api/src/endpoint/responses.rs` — `ResponsesClient`: the main inference route; encodes Responses-API requests (or rewrites them for Chat-wire providers via `chat_body_from_responses_request`) and dispatches SSE streams. Cite: `codex-api/src/endpoint/responses.rs:112-150`
- `codex-api/src/endpoint/responses_websocket.rs` — `ResponsesWebsocketClient`: Responses inference over WebSocket — connects (with permessage-deflate), sends `response.create`/`response.append`, translates incoming WS messages through the Responses SSE event processor into `ResponseEvent`s. Cite: `codex-api/src/endpoint/responses_websocket.rs:1-48`
- `codex-api/src/endpoint/search.rs` — `SearchClient`: POSTs `SearchRequest` to `alpha/search` and decodes `SearchResponse`. Cite: `codex-api/src/endpoint/search.rs:31-48`
- `codex-api/src/endpoint/session.rs` — `EndpointSession`: shared request plumbing for endpoint clients — builds requests from `Provider`, applies auth, and executes/streaming with retry + request telemetry. Cite: `codex-api/src/endpoint/session.rs:63-114`

### codex-api/src/endpoint/realtime_websocket/

- `codex-api/src/endpoint/realtime_websocket/mod.rs` — Realtime WebSocket module root: declares protocol/method layers per wire adapter (v1, v2, frameless-bidi) and re-exports client types. Cite: `codex-api/src/endpoint/realtime_websocket/mod.rs:1-22`
- `codex-api/src/endpoint/realtime_websocket/methods.rs` — `RealtimeWebsocketClient`/`Connection`/`Writer`: opens realtime WS connections, routes messages through the selected event parser, and manages session lifecycle (connect, audio in/out, transcripts, reconnect/close). Cite: `codex-api/src/endpoint/realtime_websocket/methods.rs:64-120`
- `codex-api/src/endpoint/realtime_websocket/methods_common.rs` — Dispatch layer that routes outbound-message construction to the correct wire adapter (v1, v2, frameless-bidi) for item creates, handoffs, function outputs, and session updates. Cite: `codex-api/src/endpoint/realtime_websocket/methods_common.rs:41-112`
- `codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs` — Tests that per-adapter outbound messages (session update, handoff, function-call output) serialize to the expected wire JSON. Cite: `codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs:15-60`
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs` — Builds frameless-bidi outbound messages: `session.context.append` / `delegation.context.append` with 500-byte chunking, and the frameless `session.update` JSON. Cite: `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs:11-58`
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs` — Tests frameless-bidi chunking limits and `session_json` shape (model/instructions/voice/delegation). Cite: `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs:10-48`
- `codex-api/src/endpoint/realtime_websocket/methods_v1.rs` — Builds Realtime API v1 outbound messages: `conversation.item.create`, `conversation.handoff.append`, and the Quicksilver `session.update` session object with PCM audio formats. Cite: `codex-api/src/endpoint/realtime_websocket/methods_v1.rs:18-79`
- `codex-api/src/endpoint/realtime_websocket/methods_v2.rs` — Builds Realtime v2 outbound messages: conversation items, function-call outputs, and rich `session.update` sessions (modalities, turn detection, noise reduction, background-agent/remain-silent tools). Cite: `codex-api/src/endpoint/realtime_websocket/methods_v2.rs:30-80`
- `codex-api/src/endpoint/realtime_websocket/protocol.rs` — Realtime wire vocabulary: `RealtimeEventParser`/`WireAdapter` selection, session config/mode, and the serde-tagged `RealtimeOutboundMessage` union for all supported wire event types. Cite: `codex-api/src/endpoint/realtime_websocket/protocol.rs:14-85`
- `codex-api/src/endpoint/realtime_websocket/protocol_common.rs` — Shared realtime payload parsing: JSON envelope extraction (`type` field) and common event parsers (session.updated, transcript deltas/done, error). Cite: `codex-api/src/endpoint/realtime_websocket/protocol_common.rs:7-83`
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs` — Parses frameless-bidi wire events (`session.started`, `output_audio.delta`, `input/output_transcript.added`, `turn.done`, `delegation.created`, `error`) into `RealtimeEvent`s. Cite: `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs:15-80`
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs` — Tests that legacy v1 handoffs and frameless `delegation.created` decode to the same internal event, plus transcript/audio parity. Cite: `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs:6-50`
- `codex-api/src/endpoint/realtime_websocket/protocol_v1.rs` — Parses Realtime API v1 SSE/WS events (audio deltas, transcript deltas/done, item added/done, handoff requested) into `RealtimeEvent`s. Cite: `codex-api/src/endpoint/realtime_websocket/protocol_v1.rs:12-80`
- `codex-api/src/endpoint/realtime_websocket/protocol_v2.rs` — Parses Realtime v2 wire events (audio/text deltas, transcription, response created/cancelled/done, speech started, errors) into `RealtimeEvent`s. Cite: `codex-api/src/endpoint/realtime_websocket/protocol_v2.rs:24-79`

## codex-api/src/requests/

- `codex-api/src/requests/mod.rs` — Requests module root: declares chat/responses/headers and re-exports `Compression`. Cite: `codex-api/src/requests/mod.rs:1-5`
- `codex-api/src/requests/chat.rs` — Translates a Responses-shaped request into an OpenAI-compatible `/chat/completions` body (system messages, tool calls, reasoning anchoring, developer→system mapping). Cite: `codex-api/src/requests/chat.rs:18-150`
- `codex-api/src/requests/headers.rs` — Builds wire headers: session-id/thread-id session headers, `x-openai-subagent` from session source, and safe header insertion. Cite: `codex-api/src/requests/headers.rs:5-31`
- `codex-api/src/requests/responses.rs` — Defines `Compression` (None/Zstd) for Responses request body compression on the wire. Cite: `codex-api/src/requests/responses.rs:1-6`

## codex-api/src/sse/

- `codex-api/src/sse/mod.rs` — SSE module root: re-exports the Responses SSE processor and the Chat stream translator. Cite: `codex-api/src/sse/mod.rs:1-7`
- `codex-api/src/sse/chat.rs` — Translates OpenAI-compatible `/chat/completions` SSE frames back into the Responses-shaped `ResponseEvent` stream (delta accumulation, tool-call reconstruction, usage/finish handling). Cite: `codex-api/src/sse/chat.rs:1-75`
- `codex-api/src/sse/responses.rs` — Translates Responses-API SSE streams into `ResponseEvent`s: spawns the stream, extracts rate-limit/model/turn-state/reasoning headers, and processes each SSE event (created, output items, deltas, completed with token usage). Cite: `codex-api/src/sse/responses.rs:36-102`
