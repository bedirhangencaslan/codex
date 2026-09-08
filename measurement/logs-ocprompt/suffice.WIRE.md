# codex-api wire inventory

## codex-api/src

- `codex-api/src/api_bridge.rs`: maps API and transport HTTP failures into `CodexErr` (`codex-api/src/api_bridge.rs:21`).
- `codex-api/src/api_bridge_tests.rs`: tests API error mapping, retry delays, and policy statuses (`codex-api/src/api_bridge_tests.rs:7`).
- `codex-api/src/auth.rs`: handles auth-provider resolution, request signing, and telemetry (`codex-api/src/auth.rs:30`).
- `codex-api/src/common.rs`: defines shared Responses wire payloads and stream events (`codex-api/src/common.rs:46`).
- `codex-api/src/error.rs`: defines API error variants (`codex-api/src/error.rs:9`).
- `codex-api/src/files.rs`: handles hosted-file upload wire constants, context, and errors (`codex-api/src/files.rs:25`).
- `codex-api/src/images.rs`: defines image generation and edit request/response wire structs (`codex-api/src/images.rs:4`).
- `codex-api/src/lib.rs`: exports the crate's modules and public wire API surface (`codex-api/src/lib.rs:1`).
- `codex-api/src/provider.rs`: handles provider URLs, retries, and wire API selection (`codex-api/src/provider.rs:56`).
- `codex-api/src/rate_limits.rs`: parses rate-limit headers into snapshots (`codex-api/src/rate_limits.rs:57`).
- `codex-api/src/safety_buffering.rs`: parses safety-buffering treatment headers (`codex-api/src/safety_buffering.rs:8`).
- `codex-api/src/search.rs`: defines search request and result wire payloads (`codex-api/src/search.rs:8`).
- `codex-api/src/telemetry.rs`: defines SSE/WebSocket and retried-request telemetry traits (`codex-api/src/telemetry.rs:18`).

## codex-api/src/endpoint

- `codex-api/src/endpoint/compact.rs`: sends compaction requests and parses compacted turn state (`codex-api/src/endpoint/compact.rs:18`).
- `codex-api/src/endpoint/images.rs`: sends image generation/edit requests and parses image responses (`codex-api/src/endpoint/images.rs:18`).
- `codex-api/src/endpoint/memories.rs`: sends memory trace-summarization requests and parses responses (`codex-api/src/endpoint/memories.rs:32`).
- `codex-api/src/endpoint/models.rs`: fetches model listings and model ETags (`codex-api/src/endpoint/models.rs:14`).
- `codex-api/src/endpoint/mod.rs`: declares and re-exports endpoint modules (`codex-api/src/endpoint/mod.rs:1`).
- `codex-api/src/endpoint/realtime_call.rs`: creates WebRTC realtime calls and exchanges SDP/call data (`codex-api/src/endpoint/realtime_call.rs:29`).
- `codex-api/src/endpoint/responses.rs`: streams Responses or Chat Completions over HTTP SSE (`codex-api/src/endpoint/responses.rs:30`).
- `codex-api/src/endpoint/responses_websocket.rs`: manages Responses WebSocket connections and wire frames (`codex-api/src/endpoint/responses_websocket.rs:344`).
- `codex-api/src/endpoint/search.rs`: sends alpha search requests and parses results (`codex-api/src/endpoint/search.rs:31`).
- `codex-api/src/endpoint/session.rs`: builds, authenticates, retries, and streams endpoint requests (`codex-api/src/endpoint/session.rs:19`).

## codex-api/src/endpoint/realtime_websocket

- `codex-api/src/endpoint/realtime_websocket/methods.rs`: handles realtime WebSocket clients, connections, frames, and transcript state (`codex-api/src/endpoint/realtime_websocket/methods.rs:765`).
- `codex-api/src/endpoint/realtime_websocket/methods_common.rs`: dispatches outbound realtime message construction across wire adapters (`codex-api/src/endpoint/realtime_websocket/methods_common.rs:41`).
- `codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs`: tests adapter-specific realtime outbound message encoding (`codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs:16`).
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs`: builds Frameless Bidi session and context-append messages (`codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs:35`).
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs`: tests Frameless Bidi chunking and session JSON (`codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs:10`).
- `codex-api/src/endpoint/realtime_websocket/methods_v1.rs`: builds V1 realtime conversation, handoff, and session messages (`codex-api/src/endpoint/realtime_websocket/methods_v1.rs:18`).
- `codex-api/src/endpoint/realtime_websocket/methods_v2.rs`: builds V2 realtime conversation, tool, session, and intent messages (`codex-api/src/endpoint/realtime_websocket/methods_v2.rs:39`).
- `codex-api/src/endpoint/realtime_websocket/mod.rs`: declares and re-exports realtime protocol and method modules (`codex-api/src/endpoint/realtime_websocket/mod.rs:1`).
- `codex-api/src/endpoint/realtime_websocket/protocol.rs`: defines realtime parser/config/outbound wire types and adapter dispatch (`codex-api/src/endpoint/realtime_websocket/protocol.rs:263`).
- `codex-api/src/endpoint/realtime_websocket/protocol_common.rs`: parses shared realtime JSON events and errors (`codex-api/src/endpoint/realtime_websocket/protocol_common.rs:7`).
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs`: parses Frameless Bidi realtime audio, transcript, and delegation events (`codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs:15`).
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs`: tests Frameless Bidi delegation and event reuse (`codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs:6`).
- `codex-api/src/endpoint/realtime_websocket/protocol_v1.rs`: parses Realtime V1 events (`codex-api/src/endpoint/realtime_websocket/protocol_v1.rs:12`).
- `codex-api/src/endpoint/realtime_websocket/protocol_v2.rs`: parses Realtime V2 audio, transcripts, lifecycle, and handoff events (`codex-api/src/endpoint/realtime_websocket/protocol_v2.rs:24`).

## codex-api/src/requests

- `codex-api/src/requests/chat.rs`: translates Responses requests into Chat Completions bodies (`codex-api/src/requests/chat.rs:19`).
- `codex-api/src/requests/headers.rs`: builds session and subagent request headers (`codex-api/src/requests/headers.rs:5`).
- `codex-api/src/requests/mod.rs`: declares request modules and exports compression (`codex-api/src/requests/mod.rs:1`).
- `codex-api/src/requests/responses.rs`: defines response-stream compression options (`codex-api/src/requests/responses.rs:2`).

## codex-api/src/sse

- `codex-api/src/sse/chat.rs`: translates Chat Completions SSE frames into Responses stream events (`codex-api/src/sse/chat.rs:30`).
- `codex-api/src/sse/mod.rs`: declares and re-exports SSE translation modules (`codex-api/src/sse/mod.rs:1`).
- `codex-api/src/sse/responses.rs`: translates Responses SSE events, headers, and metadata into stream events (`codex-api/src/sse/responses.rs:36`).
