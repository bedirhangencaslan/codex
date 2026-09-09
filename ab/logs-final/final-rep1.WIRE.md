# Wire Map

## codex-api/src/

- `codex-api/src/api_bridge.rs` — maps `ApiError` and transport HTTP failures into `CodexErr`, preserving retry delays, overload/policy/usage/rate-limit details (`codex-api/src/api_bridge.rs:21`).
- `codex-api/src/api_bridge_tests.rs` — verifies API-error mapping for statuses, headers, overloaded responses, cyber/misalignment policies, and identity/rate-limit details (`codex-api/src/api_bridge_tests.rs:8`).
- `codex-api/src/auth.rs` — defines request authentication providers for static headers, async credential refresh, full-request signing, and auth telemetry (`codex-api/src/auth.rs:30`).
- `codex-api/src/common.rs` — defines the canonical Responses request/stream vocabulary, including requests, tools, text/reasoning controls, WS metadata, and `ResponseEvent`s (`codex-api/src/common.rs:275`).
- `codex-api/src/error.rs` — defines API failure variants such as transport, quota, rate limit, policy violation, invalid request, and overload (`codex-api/src/error.rs:9`).
- `codex-api/src/files.rs` — handles streaming OpenAI hosted-file upload, size limits, chunk transfer, finalization, and download metadata (`codex-api/src/files.rs:121`).
- `codex-api/src/images.rs` — defines image generation/edit request and response wire DTOs (`codex-api/src/images.rs:5`).
- `codex-api/src/lib.rs` — declares crate modules and the public API re-export surface (`codex-api/src/lib.rs:1`).
- `codex-api/src/provider.rs` — configures provider deployment URLs, headers, retries, stream timeout, wire protocol, and request construction (`codex-api/src/provider.rs:56`).
- `codex-api/src/rate_limits.rs` — parses rate-limit header families, rate-limit events, promo messages, reached type, and credit snapshots (`codex-api/src/rate_limits.rs:28`).
- `codex-api/src/safety_buffering.rs` — parses safety-buffering treatment headers into a treatment value (`codex-api/src/safety_buffering.rs:8`).
- `codex-api/src/search.rs` — defines search requests, commands, settings, external web access, locations, filters, and response DTOs (`codex-api/src/search.rs:9`).
- `codex-api/src/telemetry.rs` — defines SSE/WebSocket telemetry hooks and wraps HTTP retry execution with request telemetry (`codex-api/src/telemetry.rs:68`).

## codex-api/src/endpoint/

- `codex-api/src/endpoint/compact.rs` — posts to `responses/compact`, captures turn state, and decodes compacted `ResponseItem`s (`codex-api/src/endpoint/compact.rs:34`).
- `codex-api/src/endpoint/images.rs` — posts image generation/edit requests, decodes image responses, and extracts the image request ID header (`codex-api/src/endpoint/images.rs:24`).
- `codex-api/src/endpoint/memories.rs` — posts `memories/trace_summarize` and decodes summarized memory outputs (`codex-api/src/endpoint/memories.rs:27`).
- `codex-api/src/endpoint/models.rs` — lists models, adds `client_version`, decodes `ModelsResponse`, and returns the `ETag` (`codex-api/src/endpoint/models.rs:33`).
- `codex-api/src/endpoint/mod.rs` — declares endpoint modules and re-exports their public client and wire types (`codex-api/src/endpoint/mod.rs:1`).
- `codex-api/src/endpoint/realtime_call.rs` — creates Realtime WebRTC calls from SDP, selects backend/OpenAI routes, serializes session config, and parses call ID from `Location` (`codex-api/src/endpoint/realtime_call.rs:55`).
- `codex-api/src/endpoint/responses.rs` — exposes Responses/Guardian inference routes, adds session/subagent headers and compression, and translates Responses-native or Chat requests into streams (`codex-api/src/endpoint/responses.rs:84`).
- `codex-api/src/endpoint/responses_websocket.rs` — connects, probes, serializes requests over, and consumes Responses WebSocket frames as Responses events and metadata (`codex-api/src/endpoint/responses_websocket.rs:685`).
- `codex-api/src/endpoint/search.rs` — encodes `SearchRequest`, posts it to `alpha/search`, and decodes `SearchResponse` (`codex-api/src/endpoint/search.rs:20`).
- `codex-api/src/endpoint/session.rs` — provides shared provider/auth endpoint request construction, execution, streaming, retries, and telemetry (`codex-api/src/endpoint/session.rs:85`).

## codex-api/src/endpoint/realtime_websocket/

- `codex-api/src/endpoint/realtime_websocket/methods.rs` — manages Realtime WebSocket connections, writer/events, transcript state, URLs, session initialization, sidebands, retries, and close handling (`codex-api/src/endpoint/realtime_websocket/methods.rs:765`).
- `codex-api/src/endpoint/realtime_websocket/methods_common.rs` — dispatches outbound message construction across V1, Frameless Bidi, and V2 adapters and normalizes session mode (`codex-api/src/endpoint/realtime_websocket/methods_common.rs:140`).
- `codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs` — tests adapter-specific session updates, handoff output, context channels, and V1 prefixes (`codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs:16`).
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs` — builds Frameless Bidi delegation/session context and session-update messages, including 500-byte chunking (`codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs:99`).
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs` — tests Frameless Bidi wire-limit chunking and session JSON encoding (`codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs:11`).
- `codex-api/src/endpoint/realtime_websocket/methods_v1.rs` — builds legacy Realtime V1 conversation items, handoff appends, Quicksilver session updates, and websocket intent (`codex-api/src/endpoint/realtime_websocket/methods_v1.rs:12`).
- `codex-api/src/endpoint/realtime_websocket/methods_v2.rs` — builds Realtime V2 conversation/session/function-output messages, including modalities, tools, transcription, and audio settings (`codex-api/src/endpoint/realtime_websocket/methods_v2.rs:52`).
- `codex-api/src/endpoint/realtime_websocket/mod.rs` — declares realtime protocol/method modules and re-exports realtime client types (`codex-api/src/endpoint/realtime_websocket/mod.rs:1`).
- `codex-api/src/endpoint/realtime_websocket/protocol.rs` — selects realtime parser/session mode/context channel and defines outbound audio, handoff, context, close, response, and session-update messages (`codex-api/src/endpoint/realtime_websocket/protocol.rs:15`).
- `codex-api/src/endpoint/realtime_websocket/protocol_common.rs` — parses common realtime envelopes, session updates, transcript deltas/completions, and errors (`codex-api/src/endpoint/realtime_websocket/protocol_common.rs:17`).
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs` — decodes Frameless Bidi events into internal realtime events (`codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs:17`).
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs` — verifies legacy/frameless delegation equivalence and Frameless transcript/audio event decoding (`codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs:7`).
- `codex-api/src/endpoint/realtime_websocket/protocol_v1.rs` — decodes legacy Realtime V1 audio, transcript, item, handoff, and error events (`codex-api/src/endpoint/realtime_websocket/protocol_v1.rs:13`).
- `codex-api/src/endpoint/realtime_websocket/protocol_v2.rs` — decodes Realtime V2 audio, transcripts, items, response lifecycle, speech/handoff/no-op, and error events (`codex-api/src/endpoint/realtime_websocket/protocol_v2.rs:22`).

## codex-api/src/requests/

- `codex-api/src/requests/chat.rs` — translates Responses requests into OpenAI-compatible Chat Completions messages, tools, reasoning, images, and output format (`codex-api/src/requests/chat.rs:19`).
- `codex-api/src/requests/headers.rs` — builds session/thread headers and derives subagent headers from session source (`codex-api/src/requests/headers.rs:5`).
- `codex-api/src/requests/mod.rs` — declares request-translation modules and exports compression selection (`codex-api/src/requests/mod.rs:1`).
- `codex-api/src/requests/responses.rs` — defines request body compression choices (`codex-api/src/requests/responses.rs:2`).

## codex-api/src/sse/

- `codex-api/src/sse/chat.rs` — translates Chat Completions SSE deltas into Responses-shaped `ResponseEvent`s, buffering items and usage (`codex-api/src/sse/chat.rs:77`).
- `codex-api/src/sse/mod.rs` — declares SSE modules and re-exports their stream processors (`codex-api/src/sse/mod.rs:1`).
- `codex-api/src/sse/responses.rs` — spawns and parses Responses SSE, emits events/metadata, applies idle timeout/safety treatment, and maps stream errors (`codex-api/src/sse/responses.rs:36`).
