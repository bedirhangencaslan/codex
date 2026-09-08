# codex-api/src wire map

## codex-api/src

- `codex-api/src/api_bridge.rs` - translates HTTP and transport errors into `CodexErr` through the shared API error mapping layer (`codex-api/src/api_bridge.rs:21`).
- `codex-api/src/api_bridge_tests.rs` - tests overload, retry, policy, quota, usage, and header handling in API error mapping (`codex-api/src/api_bridge_tests.rs:8`).
- `codex-api/src/auth.rs` - handles authentication errors, auth-provider requests, shared auth handles, and auth-header injection (`codex-api/src/auth.rs:30`).
- `codex-api/src/common.rs` - defines shared Responses request, WebSocket request, response stream, compaction, and memory summarize wire models (`codex-api/src/common.rs:275`).
- `codex-api/src/error.rs` - defines the API-level error variants used across HTTP, SSE, and WebSocket transports (`codex-api/src/error.rs:9`).
- `codex-api/src/files.rs` - handles OpenAI file and blob upload lifecycles, request construction, authorization, and upload errors (`codex-api/src/files.rs:121`).
- `codex-api/src/images.rs` - translates image generation and edit request/response payloads to and from the wire (`codex-api/src/images.rs:4`).
- `codex-api/src/lib.rs` - declares the crate's wire modules and public re-exported API (`codex-api/src/lib.rs:1`).
- `codex-api/src/provider.rs` - handles provider configuration, retries, wire API selection, URL construction, headers, and Azure routing (`codex-api/src/provider.rs:56`).
- `codex-api/src/rate_limits.rs` - parses rate-limit headers, events, and credit metadata into rate-limit snapshots (`codex-api/src/rate_limits.rs:57`).
- `codex-api/src/safety_buffering.rs` - translates safety-buffering response headers into treatment configuration (`codex-api/src/safety_buffering.rs:8`).
- `codex-api/src/search.rs` - defines search requests, commands, settings, and response wire DTOs (`codex-api/src/search.rs:8`).
- `codex-api/src/telemetry.rs` - defines SSE, WebSocket, and request telemetry hooks and wraps requests with retry telemetry (`codex-api/src/telemetry.rs:17`).

## codex-api/src/endpoint

- `codex-api/src/endpoint/compact.rs` - handles the compaction endpoint and translates compact requests and responses (`codex-api/src/endpoint/compact.rs:39`).
- `codex-api/src/endpoint/images.rs` - handles image generation and edit endpoints and their JSON transport (`codex-api/src/endpoint/images.rs:35`).
- `codex-api/src/endpoint/memories.rs` - handles the memory summarization endpoint and input translation (`codex-api/src/endpoint/memories.rs:36`).
- `codex-api/src/endpoint/mod.rs` - declares and re-exports the endpoint clients (`codex-api/src/endpoint/mod.rs:1`).
- `codex-api/src/endpoint/models.rs` - handles model discovery, client-version queries, caching with ETags, and list responses (`codex-api/src/endpoint/models.rs:46`).
- `codex-api/src/endpoint/realtime_call.rs` - translates WebRTC realtime call creation, SDP offers, session payloads, and multipart request bodies (`codex-api/src/endpoint/realtime_call.rs:90`).
- `codex-api/src/endpoint/responses.rs` - handles Responses and Chat Completions HTTP endpoints, request encoding, routing, and SSE stream startup (`codex-api/src/endpoint/responses.rs:112`).
- `codex-api/src/endpoint/responses_websocket.rs` - handles Responses WebSocket connections, request frames, event streams, probes, errors, and timing metadata (`codex-api/src/endpoint/responses_websocket.rs:235`).
- `codex-api/src/endpoint/search.rs` - handles the search endpoint and sends search requests (`codex-api/src/endpoint/search.rs:35`).
- `codex-api/src/endpoint/session.rs` - handles shared endpoint authentication, request execution, retry policy, and JSON streaming (`codex-api/src/endpoint/session.rs:19`).

## codex-api/src/endpoint/realtime_websocket

- `codex-api/src/endpoint/realtime_websocket/mod.rs` - declares realtime WebSocket modules and public client, protocol, and transcript exports (`codex-api/src/endpoint/realtime_websocket/mod.rs:1`).
- `codex-api/src/endpoint/realtime_websocket/methods.rs` - handles realtime WebSocket connection setup, outbound frames, inbound events, transcript state, and wire adapters (`codex-api/src/endpoint/realtime_websocket/methods.rs:211`).
- `codex-api/src/endpoint/realtime_websocket/methods_common.rs` - routes realtime wire-adapter requests to version-specific outbound message builders (`codex-api/src/endpoint/realtime_websocket/methods_common.rs:41`).
- `codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs` - tests adapter-specific serialization of session updates, handoffs, channels, and call outputs (`codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs:13`).
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs` - builds Frameless Bidi session, context-append, delegation, and chunked messages (`codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs:13`).
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs` - tests Frameless Bidi chunk boundaries and initial session item serialization (`codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs:7`).
- `codex-api/src/endpoint/realtime_websocket/methods_v1.rs` - builds realtime V1 conversation, handoff, session-update, audio, and intent payloads (`codex-api/src/endpoint/realtime_websocket/methods_v1.rs:18`).
- `codex-api/src/endpoint/realtime_websocket/methods_v2.rs` - builds realtime V2 conversation items, call outputs, sessions, tools, modalities, and transcription settings (`codex-api/src/endpoint/realtime_websocket/methods_v2.rs:39`).
- `codex-api/src/endpoint/realtime_websocket/protocol.rs` - defines realtime wire enums, outbound messages, session configuration, audio/session DTOs, and parser dispatch (`codex-api/src/endpoint/realtime_websocket/protocol.rs:52`).
- `codex-api/src/endpoint/realtime_websocket/protocol_common.rs` - handles shared realtime JSON parsing for payload types, session updates, transcripts, and errors (`codex-api/src/endpoint/realtime_websocket/protocol_common.rs:7`).
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs` - parses Frameless Bidi audio, transcripts, turns, delegations, session events, and errors (`codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs:15`).
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs` - tests Frameless Bidi compatibility with legacy handoffs and internal transcript/audio events (`codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs:7`).
- `codex-api/src/endpoint/realtime_websocket/protocol_v1.rs` - parses realtime V1 audio, transcript, conversation-item, handoff, and error events (`codex-api/src/endpoint/realtime_websocket/protocol_v1.rs:12`).
- `codex-api/src/endpoint/realtime_websocket/protocol_v2.rs` - parses realtime V2 audio, transcript, response, tool-call, handoff, noop, and error events (`codex-api/src/endpoint/realtime_websocket/protocol_v2.rs:24`).

## codex-api/src/requests

- `codex-api/src/requests/chat.rs` - translates Responses-shaped requests into OpenAI-compatible Chat Completions bodies and tools (`codex-api/src/requests/chat.rs:19`).
- `codex-api/src/requests/headers.rs` - handles session, thread, and subagent header construction for requests (`codex-api/src/requests/headers.rs:5`).
- `codex-api/src/requests/mod.rs` - declares request translation modules and re-exports compression selection (`codex-api/src/requests/mod.rs:1`).
- `codex-api/src/requests/responses.rs` - defines supported request wire compression modes (`codex-api/src/requests/responses.rs:2`).

## codex-api/src/sse

- `codex-api/src/sse/chat.rs` - translates Chat Completions SSE deltas into Responses-shaped events, tool calls, reasoning, and usage (`codex-api/src/sse/chat.rs:30`).
- `codex-api/src/sse/mod.rs` - declares and re-exports Chat and Responses SSE stream processors (`codex-api/src/sse/mod.rs:1`).
- `codex-api/src/sse/responses.rs` - parses Responses SSE headers and events into response items, completion, errors, metadata, and stream events (`codex-api/src/sse/responses.rs:36`).


