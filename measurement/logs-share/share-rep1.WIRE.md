# Wire Map

## codex-api/src/

- `codex-api/src/api_bridge.rs`: maps API errors and transport failures into `CodexErr`, including overload, policy, usage-limit, timeout, request IDs, and identity errors. `codex-api/src/api_bridge.rs:19`
- `codex-api/src/api_bridge_tests.rs`: tests API-to-internal-error translation across overload, Cloudflare, cyber policy, misalignment, usage limits, and identity headers. `codex-api/src/api_bridge_tests.rs:7`
- `codex-api/src/auth.rs`: defines the outbound request `AuthProvider` contract and auth telemetry. `codex-api/src/auth.rs:28`
- `codex-api/src/common.rs`: defines shared Responses/WebSocket request payloads, `ResponseEvent`, reasoning/text controls, trace metadata, and `ResponseStream`. `codex-api/src/common.rs:274`
- `codex-api/src/error.rs`: defines the crate's `ApiError` variants. `codex-api/src/error.rs:6`
- `codex-api/src/files.rs`: performs OpenAI hosted-file upload, including create, blob PUT, finalize/retry, and diagnostics. `codex-api/src/files.rs:113`
- `codex-api/src/images.rs`: defines image generation/edit wire request and response DTOs. `codex-api/src/images.rs:5`
- `codex-api/src/lib.rs`: declares crate modules and public API surface. `codex-api/src/lib.rs:1`
- `codex-api/src/provider.rs`: models provider URLs, headers, retries, WebSocket URL conversion, wire protocol, and Azure detection. `codex-api/src/provider.rs:46`
- `codex-api/src/rate_limits.rs`: parses rate-limit HTTP headers and `codex.rate_limits` events into snapshots. `codex-api/src/rate_limits.rs:19`
- `codex-api/src/safety_buffering.rs`: parses safety-buffering treatment headers. `codex-api/src/safety_buffering.rs:9`
- `codex-api/src/search.rs`: defines search request commands, settings, location filters, and response DTOs. `codex-api/src/search.rs:5`
- `codex-api/src/telemetry.rs`: defines SSE/WebSocket telemetry hooks and wraps retries with request telemetry. `codex-api/src/telemetry.rs:50`

## codex-api/src/endpoint/

- `codex-api/src/endpoint/compact.rs`: POSTs `responses/compact`, decodes compacted output, and captures turn state. `codex-api/src/endpoint/compact.rs:44`
- `codex-api/src/endpoint/images.rs`: POSTs image generation/edit requests and decodes responses/request IDs. `codex-api/src/endpoint/images.rs:31`
- `codex-api/src/endpoint/memories.rs`: POSTs `memories/trace_summarize` and decodes summaries. `codex-api/src/endpoint/memories.rs:31`
- `codex-api/src/endpoint/mod.rs`: declares and exports endpoint modules. `codex-api/src/endpoint/mod.rs:1`
- `codex-api/src/endpoint/models.rs`: GETs models with `client_version`, parses model metadata, and returns ETag. `codex-api/src/endpoint/models.rs:41`
- `codex-api/src/endpoint/realtime_call.rs`: creates WebRTC realtime calls, translating SDP/session configuration into raw SDP, backend JSON, or multipart bodies and extracting call IDs. `codex-api/src/endpoint/realtime_call.rs:89`
- `codex-api/src/endpoint/responses.rs`: streams Responses requests over HTTP, translating Responses-native or Chat Completions bodies and selecting the matching SSE parser. `codex-api/src/endpoint/responses.rs:35`
- `codex-api/src/endpoint/responses_websocket.rs`: connects/probes the Responses WebSocket, wraps streams, sends `response.create`, and translates messages, errors, metadata, rate limits, safety buffering, and timing. `codex-api/src/endpoint/responses_websocket.rs:685`
- `codex-api/src/endpoint/search.rs`: POSTs `alpha/search` and decodes `SearchResponse`. `codex-api/src/endpoint/search.rs:25`
- `codex-api/src/endpoint/session.rs`: provides shared authenticated HTTP execute/stream helpers with provider retries and telemetry. `codex-api/src/endpoint/session.rs:46`

## codex-api/src/endpoint/realtime_websocket/

- `codex-api/src/endpoint/realtime_websocket/mod.rs`: declares and re-exports the realtime WebSocket client surface. `codex-api/src/endpoint/realtime_websocket/mod.rs:1`
- `codex-api/src/endpoint/realtime_websocket/protocol.rs`: defines realtime adapters, session config/modes, outbound wire messages, and parser dispatch. `codex-api/src/endpoint/realtime_websocket/protocol.rs:263`
- `codex-api/src/endpoint/realtime_websocket/protocol_common.rs`: parses JSON payloads and common session, transcript, and error events. `codex-api/src/endpoint/realtime_websocket/protocol_common.rs:5`
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs`: parses Frameless Bidi session/audio/transcript/turn/delegation/error events. `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs:16`
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs`: verifies Frameless Bidi delegation, transcript, and audio decoding against internal events. `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs:4`
- `codex-api/src/endpoint/realtime_websocket/protocol_v1.rs`: parses Realtime v1 audio, transcript, item, handoff, and error events. `codex-api/src/endpoint/realtime_websocket/protocol_v1.rs:12`
- `codex-api/src/endpoint/realtime_websocket/protocol_v2.rs`: parses Realtime v2 audio, transcript, response lifecycle, speech, item, handoff, silence, and error events. `codex-api/src/endpoint/realtime_websocket/protocol_v2.rs:18`
- `codex-api/src/endpoint/realtime_websocket/methods.rs`: implements realtime WebSocket connection, framing, sends, event parsing, transcript state, sideband connections, URL construction, and reconnect/retry behavior. `codex-api/src/endpoint/realtime_websocket/methods.rs:791`
- `codex-api/src/endpoint/realtime_websocket/methods_common.rs`: routes outgoing realtime operations by wire adapter and normalizes session configuration. `codex-api/src/endpoint/realtime_websocket/methods_common.rs:114`
- `codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs`: tests adapter-specific session updates, handoff messages, context channels, and v1 output prefixes. `codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs:4`
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs`: builds Frameless Bidi session/context/delegation messages and chunks text at wire limits. `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs:13`
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs`: tests context chunk boundaries and Frameless session/initial-item JSON encoding. `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs:4`
- `codex-api/src/endpoint/realtime_websocket/methods_v1.rs`: builds Realtime v1 conversation items, handoff appends, Quicksilver session updates, and intent. `codex-api/src/endpoint/realtime_websocket/methods_v1.rs:12`
- `codex-api/src/endpoint/realtime_websocket/methods_v2.rs`: builds Realtime v2 conversation/function-output messages and conversational/transcription session updates with tools. `codex-api/src/endpoint/realtime_websocket/methods_v2.rs:46`

## codex-api/src/requests/

- `codex-api/src/requests/chat.rs`: translates Responses requests into OpenAI-compatible `/chat/completions` bodies, including messages, tool calls, reasoning, GLM effort clamping, tools, and response format. `codex-api/src/requests/chat.rs:10`
- `codex-api/src/requests/headers.rs`: builds session/thread and sub-agent request headers. `codex-api/src/requests/headers.rs:5`
- `codex-api/src/requests/mod.rs`: declares request modules and re-exports `Compression`. `codex-api/src/requests/mod.rs:1`
- `codex-api/src/requests/responses.rs`: defines Responses request compression variants. `codex-api/src/requests/responses.rs:3`

## codex-api/src/sse/

- `codex-api/src/sse/mod.rs`: declares SSE modules and re-exports chat/Responses stream entry points. `codex-api/src/sse/mod.rs:1`
- `codex-api/src/sse/chat.rs`: translates OpenAI-compatible Chat Completions SSE into Responses-shaped `ResponseEvent`s, including reasoning, text, tool calls, usage, and completion. `codex-api/src/sse/chat.rs:1`
- `codex-api/src/sse/responses.rs`: parses Responses SSE into `ResponseEvent`s and emits metadata, rate limits, safety buffering, errors, usage, and completion state. `codex-api/src/sse/responses.rs:36`
