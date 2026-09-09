# WIRE

## codex-api/src

- codex-api/src/api_bridge.rs — maps provider `ApiError` and HTTP transport failures into protocol `CodexErr`, preserving overload, retry, usage-limit, and policy semantics (codex-api/src/api_bridge.rs:21)
- codex-api/src/api_bridge_tests.rs — tests `map_api_error` overload, retry-delay, Cloudflare, cyber-policy, and misalignment translation (codex-api/src/api_bridge_tests.rs:8)
- codex-api/src/auth.rs — defines auth errors and the async `AuthProvider` contract for static/refreshed headers and complete-request signing (codex-api/src/auth.rs:30)
- codex-api/src/common.rs — defines shared wire payload and normalized stream types, including `ResponseEvent` items, deltas, completion, limits, and metadata (codex-api/src/common.rs:98)
- codex-api/src/error.rs — defines provider-facing `ApiError` variants and converts `RateLimitError` into it (codex-api/src/error.rs:8)
- codex-api/src/files.rs — uploads OpenAI files via `/files` and blob PUT, including hosted-upload metadata and upload/finalize errors (codex-api/src/files.rs:121)
- codex-api/src/images.rs — defines serialized image generation/edit request and image response wire types (codex-api/src/images.rs:4)
- codex-api/src/lib.rs — declares crate modules and re-exports clients, payload types, provider config, telemetry, and realtime events (codex-api/src/lib.rs:1)
- codex-api/src/provider.rs — defines retry config, Responses-vs-Chat wire selection, HTTP/WebSocket URL construction, and Azure provider detection (codex-api/src/provider.rs:38)
- codex-api/src/rate_limits.rs — parses HTTP headers and `codex.rate_limits` events into snapshots, credits, promo messages, and reached-type metadata (codex-api/src/rate_limits.rs:22)
- codex-api/src/safety_buffering.rs — derives safety-buffering treatment and faster-model fallback from HTTP headers (codex-api/src/safety_buffering.rs:8)
- codex-api/src/search.rs — defines search request, operation commands, settings, locations, filters, and response wire types (codex-api/src/search.rs:8)
- codex-api/src/telemetry.rs — defines SSE/WebSocket telemetry traits and wraps HTTP retries with per-attempt request telemetry (codex-api/src/telemetry.rs:17)

## codex-api/src/endpoint

- codex-api/src/endpoint/compact.rs — posts compaction input to `responses/compact`, records turn state, and parses compacted response history (codex-api/src/endpoint/compact.rs:39)
- codex-api/src/endpoint/images.rs — posts image generation/edit requests and decodes `ImageResponse` plus the imagegen request id (codex-api/src/endpoint/images.rs:35)
- codex-api/src/endpoint/memories.rs — posts memory summarize input to `memories/trace_summarize` and parses output summaries (codex-api/src/endpoint/memories.rs:36)
- codex-api/src/endpoint/mod.rs — declares endpoint modules and re-exports their public clients and types (codex-api/src/endpoint/mod.rs:1)
- codex-api/src/endpoint/models.rs — GETs `/models` with a client-version query and parses the model list plus ETag (codex-api/src/endpoint/models.rs:46)
- codex-api/src/endpoint/realtime_call.rs — creates WebRTC realtime calls, sending SDP/session JSON or multipart and parsing SDP/call-id responses (codex-api/src/endpoint/realtime_call.rs:90)
- codex-api/src/endpoint/responses.rs — streams Responses or Chat Completions inference, translating bodies/headers and selecting endpoint paths (codex-api/src/endpoint/responses.rs:112)
- codex-api/src/endpoint/responses_websocket.rs — implements Responses WebSocket connection/client request, event-stream, close, and probe handling (codex-api/src/endpoint/responses_websocket.rs:183)
- codex-api/src/endpoint/search.rs — posts a search request to `alpha/search` and decodes the search response (codex-api/src/endpoint/search.rs:35)
- codex-api/src/endpoint/session.rs — centralizes authenticated, retried, telemetry-wrapped unary and streaming endpoint requests (codex-api/src/endpoint/session.rs:63)

## codex-api/src/endpoint/realtime_websocket

- codex-api/src/endpoint/realtime_websocket/methods.rs — implements realtime websocket client, connection, writer/event handling, and the send/receive pump (codex-api/src/endpoint/realtime_websocket/methods.rs:64)
- codex-api/src/endpoint/realtime_websocket/methods_common.rs — dispatches realtime outbound messages and session JSON across V1, Frameless Bidi, and V2 adapters (codex-api/src/endpoint/realtime_websocket/methods_common.rs:41)
- codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs — tests adapter-specific session, handoff, and function-output message shapes (codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs:15)
- codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs — builds frameless delegation/session context appends, session JSON, and 500-byte chunks (codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs:13)
- codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs — tests context chunk limits and frameless session JSON initial items (codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs:10)
- codex-api/src/endpoint/realtime_websocket/methods_v1.rs — builds V1 realtime item, handoff, session-update, and quicksilver intent messages (codex-api/src/endpoint/realtime_websocket/methods_v1.rs:18)
- codex-api/src/endpoint/realtime_websocket/methods_v2.rs — builds V2 realtime item/function-output and conversational/transcription session updates with tools (codex-api/src/endpoint/realtime_websocket/methods_v2.rs:39)
- codex-api/src/endpoint/realtime_websocket/mod.rs — declares and re-exports realtime websocket method and protocol modules (codex-api/src/endpoint/realtime_websocket/mod.rs:1)
- codex-api/src/endpoint/realtime_websocket/protocol.rs — defines realtime parser adapters, session config, outbound messages, and session wire structures (codex-api/src/endpoint/realtime_websocket/protocol.rs:14)
- codex-api/src/endpoint/realtime_websocket/protocol_common.rs — parses common realtime JSON payloads, session updates, transcripts, and errors (codex-api/src/endpoint/realtime_websocket/protocol_common.rs:7)
- codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs — maps Frameless Bidi events to internal audio, transcript, handoff, and error events (codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs:15)
- codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs — tests legacy/frameless handoff equivalence and frameless event parsing (codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs:6)
- codex-api/src/endpoint/realtime_websocket/protocol_v1.rs — maps realtime V1 events to internal audio, transcript, item, handoff, and error events (codex-api/src/endpoint/realtime_websocket/protocol_v1.rs:12)
- codex-api/src/endpoint/realtime_websocket/protocol_v2.rs — maps realtime V2 events and background-agent/noop function calls to internal events (codex-api/src/endpoint/realtime_websocket/protocol_v2.rs:24)

## codex-api/src/requests

- codex-api/src/requests/chat.rs — rewrites Responses requests into Chat Completions message, tool, reasoning, and text wire format (codex-api/src/requests/chat.rs:18)
- codex-api/src/requests/headers.rs — builds session/thread and subagent headers (codex-api/src/requests/headers.rs:5)
- codex-api/src/requests/mod.rs — declares request modules and re-exports `Compression` (codex-api/src/requests/mod.rs:1)
- codex-api/src/requests/responses.rs — defines `None`/`Zstd` request compression (codex-api/src/requests/responses.rs:1)

## codex-api/src/sse

- codex-api/src/sse/chat.rs — parses Chat Completions SSE deltas, tool calls, errors, and usage into Responses-shaped events (codex-api/src/sse/chat.rs:1)
- codex-api/src/sse/mod.rs — declares and re-exports chat and Responses SSE stream entry points/types (codex-api/src/sse/mod.rs:1)
- codex-api/src/sse/responses.rs — parses Responses SSE headers, metadata, items, deltas, completion, safety buffering, and rate-limit events (codex-api/src/sse/responses.rs:36)
