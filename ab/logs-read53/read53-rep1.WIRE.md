# Wire map

One-line map of every Rust file under `codex-api/src/`, with one implementation citation.

## codex-api/src

- `codex-api/src/api_bridge.rs` — translates API errors into Codex protocol errors through `map_api_error` (`codex-api/src/api_bridge.rs:21`).
- `codex-api/src/api_bridge_tests.rs` — tests protocol error translation for overload, policy, usage-limit, and identity-header cases (`codex-api/src/api_bridge_tests.rs:7`).
- `codex-api/src/auth.rs` — defines authentication attachment and reshaping for outbound API requests (`codex-api/src/auth.rs:30`).
- `codex-api/src/common.rs` — defines shared Responses request and event, including WebSocket request, wire shapes (`codex-api/src/common.rs:275`).
- `codex-api/src/error.rs` — defines the unified API error taxonomy used by transport and decoding paths (`codex-api/src/error.rs:9`).
- `codex-api/src/files.rs` — creates, uploads, and finalizes OpenAI files and parses their upload/download responses (`codex-api/src/files.rs:69`).
- `codex-api/src/images.rs` — defines image generation, editing, and response DTOs (`codex-api/src/images.rs:5`).
- `codex-api/src/lib.rs` — wires the API crate's public module and export surface (`codex-api/src/lib.rs:1`).
- `codex-api/src/provider.rs` — defines provider routes, retry policy, and the Responses-versus-Chat wire selection (`codex-api/src/provider.rs:45`).
- `codex-api/src/rate_limits.rs` — parses rate-limit HTTP headers and serialized rate-limit events into snapshots (`codex-api/src/rate_limits.rs:134`).
- `codex-api/src/safety_buffering.rs` — parses safety-buffering treatment settings from HTTP headers (`codex-api/src/safety_buffering.rs:5`).
- `codex-api/src/search.rs` — defines search requests, commands, settings, and response wire types (`codex-api/src/search.rs:9`).
- `codex-api/src/telemetry.rs` — instruments request, SSE, and WebSocket transport activity (`codex-api/src/telemetry.rs:18`).

## codex-api/src/endpoint

- `codex-api/src/endpoint/compact.rs` — posts compaction requests and parses returned response items (`codex-api/src/endpoint/compact.rs:18`).
- `codex-api/src/endpoint/images.rs` — sends image generation and edit requests and decodes their responses (`codex-api/src/endpoint/images.rs:18`).
- `codex-api/src/endpoint/memories.rs` — posts memory summarization requests and parses summarized output (`codex-api/src/endpoint/memories.rs:15`).
- `codex-api/src/endpoint/mod.rs` — exposes endpoint clients and their wire types (`codex-api/src/endpoint/mod.rs:1`).
- `codex-api/src/endpoint/models.rs` — requests the model list and parses models plus cache metadata (`codex-api/src/endpoint/models.rs:14`).
- `codex-api/src/endpoint/realtime_call.rs` — creates realtime calls and encodes SDP/session payloads in backend or multipart shapes (`codex-api/src/endpoint/realtime_call.rs:29`).
- `codex-api/src/endpoint/responses.rs` — streams Responses and Chat Completions over HTTP, including routing, headers, compression, and SSE dispatch (`codex-api/src/endpoint/responses.rs:52`).
- `codex-api/src/endpoint/responses_websocket.rs` — connects, probes, streams, serializes, and decodes Responses WebSocket traffic (`codex-api/src/endpoint/responses_websocket.rs:345`).
- `codex-api/src/endpoint/search.rs` — posts search requests and parses search responses (`codex-api/src/endpoint/search.rs:14`).
- `codex-api/src/endpoint/session.rs` — supplies shared HTTP request construction, authentication, retry, and streaming plumbing (`codex-api/src/endpoint/session.rs:19`).

## codex-api/src/endpoint/realtime_websocket

- `codex-api/src/endpoint/realtime_websocket/methods.rs` — manages realtime WebSocket connections, writer/event streams, transcripts, and URL construction (`codex-api/src/endpoint/realtime_websocket/methods.rs:765`).
- `codex-api/src/endpoint/realtime_websocket/methods_common.rs` — dispatches outbound realtime messages across wire adapters and encodes session updates (`codex-api/src/endpoint/realtime_websocket/methods_common.rs:140`).
- `codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs` — tests adapter-specific session updates, context channels, and handoff output encoding (`codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs:7`).
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs` — constructs frameless bidirectional context, delegation, and session-update frames (`codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs:11`).
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs` — tests frameless chunk limits and initial-item/session JSON encoding (`codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs:7`).
- `codex-api/src/endpoint/realtime_websocket/methods_v1.rs` — builds legacy realtime v1 outbound frames and connection intent (`codex-api/src/endpoint/realtime_websocket/methods_v1.rs:81`).
- `codex-api/src/endpoint/realtime_websocket/methods_v2.rs` — builds realtime v2 session, tool, output-modality, and intent frames (`codex-api/src/endpoint/realtime_websocket/methods_v2.rs:171`).
- `codex-api/src/endpoint/realtime_websocket/mod.rs` — wires realtime method and protocol modules and exposes their public types (`codex-api/src/endpoint/realtime_websocket/mod.rs:1`).
- `codex-api/src/endpoint/realtime_websocket/protocol.rs` — defines realtime parser modes, session configuration, and outbound message types (`codex-api/src/endpoint/realtime_websocket/protocol.rs:15`).
- `codex-api/src/endpoint/realtime_websocket/protocol_common.rs` — parses common realtime payloads, session updates, transcripts, and errors (`codex-api/src/endpoint/realtime_websocket/protocol_common.rs:7`).
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs` — decodes frameless audio, transcript, turn, and delegation events (`codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs:15`).
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs` — tests frameless compatibility with legacy handoffs and internal transcript/audio events (`codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs:4`).
- `codex-api/src/endpoint/realtime_websocket/protocol_v1.rs` — decodes realtime v1 WebSocket events (`codex-api/src/endpoint/realtime_websocket/protocol_v1.rs:12`).
- `codex-api/src/endpoint/realtime_websocket/protocol_v2.rs` — decodes realtime v2 events, including audio, items, handoffs, and no-ops (`codex-api/src/endpoint/realtime_websocket/protocol_v2.rs:24`).

## codex-api/src/requests

- `codex-api/src/requests/chat.rs` — translates Responses requests into Chat Completions request bodies (`codex-api/src/requests/chat.rs:19`).
- `codex-api/src/requests/headers.rs` — builds session and subagent HTTP request headers (`codex-api/src/requests/headers.rs:5`).
- `codex-api/src/requests/mod.rs` — wires request modules and exports the compression choice (`codex-api/src/requests/mod.rs:1`).
- `codex-api/src/requests/responses.rs` — defines compression options for Responses HTTP requests (`codex-api/src/requests/responses.rs:2`).

## codex-api/src/sse

- `codex-api/src/sse/chat.rs` — translates Chat Completions SSE into Responses-shaped internal events (`codex-api/src/sse/chat.rs:30`).
- `codex-api/src/sse/mod.rs` — wires the Chat and Responses SSE translation modules (`codex-api/src/sse/mod.rs:1`).
- `codex-api/src/sse/responses.rs` — parses Responses SSE frames, stream metadata, errors, completion, and usage (`codex-api/src/sse/responses.rs:36`).
