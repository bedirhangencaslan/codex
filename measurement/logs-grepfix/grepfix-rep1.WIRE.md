# Wire overview

## codex-api/src

- `codex-api/src/api_bridge.rs` — Translates API/transport failures, overloaded responses, policy violations, and rate-limit signals into `CodexErr` (`codex-api/src/api_bridge.rs:21`).
- `codex-api/src/api_bridge_tests.rs` — Covers API/transport error translation, retry delays, Cloudflare bodies, policy errors, rate-limit headers, and identity auth errors (`codex-api/src/api_bridge_tests.rs:8`).
- `codex-api/src/auth.rs` — Defines the auth provider contract for adding/resolving headers or signing complete outbound requests (`codex-api/src/auth.rs:30`).
- `codex-api/src/common.rs` — Defines shared inference request/response stream types, including Responses requests and translated stream events (`codex-api/src/common.rs:275`).
- `codex-api/src/error.rs` — Defines the crate's API error variants for transport, HTTP, stream, limits, policy, and overload conditions (`codex-api/src/error.rs:9`).
- `codex-api/src/files.rs` — Handles the hosted OpenAI file upload and download-link lifecycle (`codex-api/src/files.rs:121`).
- `codex-api/src/images.rs` — Defines image generation/edit request and response wire payloads (`codex-api/src/images.rs:5`).
- `codex-api/src/lib.rs` — Declares crate modules and exports the public wire/client API (`codex-api/src/lib.rs:16`).
- `codex-api/src/provider.rs` — Configures provider endpoints, URLs, retries, HTTP/WebSocket schemes, and the Responses versus Chat wire (`codex-api/src/provider.rs:38`).
- `codex-api/src/rate_limits.rs` — Parses rate-limit, credits, plan, and limit-reached data from HTTP headers and events (`codex-api/src/rate_limits.rs:23`).
- `codex-api/src/safety_buffering.rs` — Reads safety-buffering treatment settings from response headers (`codex-api/src/safety_buffering.rs:8`).
- `codex-api/src/search.rs` — Defines the search request/response and command, filter, location, and web-access payloads (`codex-api/src/search.rs:9`).
- `codex-api/src/telemetry.rs` — Adds SSE and WebSocket telemetry hooks and wraps retryable HTTP calls with request telemetry (`codex-api/src/telemetry.rs:68`).

## codex-api/src/endpoint

- `codex-api/src/endpoint/compact.rs` — Posts compaction requests to `responses/compact`, captures turn state, and decodes response items (`codex-api/src/endpoint/compact.rs:39`).
- `codex-api/src/endpoint/images.rs` — Sends image generation/edit requests and decodes responses plus image request IDs (`codex-api/src/endpoint/images.rs:58`).
- `codex-api/src/endpoint/memories.rs` — Posts memory trace summarization requests and decodes summarized outputs (`codex-api/src/endpoint/memories.rs:36`).
- `codex-api/src/endpoint/mod.rs` — Organizes endpoint clients and re-exports their public types (`codex-api/src/endpoint/mod.rs:12`).
- `codex-api/src/endpoint/models.rs` — Requests the model catalog, appends client version, and returns models plus the ETag (`codex-api/src/endpoint/models.rs:46`).
- `codex-api/src/endpoint/realtime_call.rs` — Creates WebRTC realtime calls, translates SDP/multipart/session payloads, and joins calls to sideband WebSocket sessions (`codex-api/src/endpoint/realtime_call.rs:29`).
- `codex-api/src/endpoint/responses.rs` — Streams Responses HTTP requests, selects inference routes, translates Chat-only providers, and applies session headers (`codex-api/src/endpoint/responses.rs:112`).
- `codex-api/src/endpoint/responses_websocket.rs` — Manages Responses WebSocket connections, sends requests, and converts wire messages into response events (`codex-api/src/endpoint/responses_websocket.rs:345`).
- `codex-api/src/endpoint/search.rs` — Sends typed search requests to `alpha/search` and decodes search responses (`codex-api/src/endpoint/search.rs:35`).
- `codex-api/src/endpoint/session.rs` — Builds authenticated, retried, telemetry-instrumented HTTP requests shared by endpoint clients (`codex-api/src/endpoint/session.rs:80`).

## codex-api/src/endpoint/realtime_websocket

- `codex-api/src/endpoint/realtime_websocket/methods.rs` — Implements the realtime WebSocket client, connection lifecycle, reconnects, audio/transcript I/O, and transcript state (`codex-api/src/endpoint/realtime_websocket/methods.rs:765`).
- `codex-api/src/endpoint/realtime_websocket/methods_common.rs` — Routes realtime outbound requests to the V1, Frameless Bidi, or Realtime V2 wire adapter (`codex-api/src/endpoint/realtime_websocket/methods_common.rs:41`).
- `codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs` — Tests serialized realtime session updates, handoffs, function-call outputs, and context channels (`codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs:16`).
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs` — Builds Frameless Bidi context-append, delegation, and session-update messages with chunk limits (`codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs:13`).
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs` — Tests Frameless Bidi text chunking and initial-session JSON encoding (`codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs:11`).
- `codex-api/src/endpoint/realtime_websocket/methods_v1.rs` — Builds legacy realtime V1 conversation, handoff, and session-update messages (`codex-api/src/endpoint/realtime_websocket/methods_v1.rs:18`).
- `codex-api/src/endpoint/realtime_websocket/methods_v2.rs` — Builds Realtime V2 conversation, function-output, and detailed session-update messages (`codex-api/src/endpoint/realtime_websocket/methods_v2.rs:39`).
- `codex-api/src/endpoint/realtime_websocket/mod.rs` — Organizes realtime WebSocket methods/protocol modules and exports public types (`codex-api/src/endpoint/realtime_websocket/mod.rs:12`).
- `codex-api/src/endpoint/realtime_websocket/protocol.rs` — Defines realtime parser/session configuration, outbound message wire shapes, and event dispatch (`codex-api/src/endpoint/realtime_websocket/protocol.rs:52`).
- `codex-api/src/endpoint/realtime_websocket/protocol_common.rs` — Provides shared parsing for JSON payload types, session updates, transcripts, and errors (`codex-api/src/endpoint/realtime_websocket/protocol_common.rs:7`).
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs` — Parses Frameless Bidi audio, transcripts, turn completion, delegation, and error events (`codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs:15`).
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs` — Tests legacy/frameless delegation compatibility and common internal realtime events (`codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs:7`).
- `codex-api/src/endpoint/realtime_websocket/protocol_v1.rs` — Parses legacy realtime V1 audio, transcript, item, handoff, and error events (`codex-api/src/endpoint/realtime_websocket/protocol_v1.rs:12`).
- `codex-api/src/endpoint/realtime_websocket/protocol_v2.rs` — Parses Realtime V2 audio, transcript, item, response lifecycle, tool, handoff, and error events (`codex-api/src/endpoint/realtime_websocket/protocol_v2.rs:24`).

## codex-api/src/requests

- `codex-api/src/requests/chat.rs` — Translates Responses-shaped history, tools, reasoning, images, and controls into Chat Completions request bodies (`codex-api/src/requests/chat.rs:19`).
- `codex-api/src/requests/headers.rs` — Builds session headers and maps sub-agent sources to outbound header values (`codex-api/src/requests/headers.rs:5`).
- `codex-api/src/requests/mod.rs` — Organizes request translation modules and exports the compression option (`codex-api/src/requests/mod.rs:5`).
- `codex-api/src/requests/responses.rs` — Defines the optional request-body compression wire choice (`codex-api/src/requests/responses.rs:2`).

## codex-api/src/sse

- `codex-api/src/sse/chat.rs` — Translates Chat Completions SSE deltas into Responses-shaped `ResponseEvent`s and reconstructs completed items (`codex-api/src/sse/chat.rs:30`).
- `codex-api/src/sse/mod.rs` — Organizes SSE translators and exports the response stream entry point (`codex-api/src/sse/mod.rs:4`).
- `codex-api/src/sse/responses.rs` — Parses Responses SSE into internal response events while extracting usage, rate limits, model metadata, and errors (`codex-api/src/sse/responses.rs:353`).
