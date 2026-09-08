# codex-api wire surface

## codex-api/src

- `codex-api/src/lib.rs` — declares the crate modules and exposes the public wire-client API. (`codex-api/src/lib.rs:1`)
- `codex-api/src/error.rs` — defines the API error variants used across wire clients and converts rate-limit errors. (`codex-api/src/error.rs:9`)
- `codex-api/src/auth.rs` — defines auth-provider behavior, shared auth handles, and outbound request authentication. (`codex-api/src/auth.rs:30`)
- `codex-api/src/provider.rs` — defines provider retry settings, `WireApi` selection, and request/WebSocket URL construction. (`codex-api/src/provider.rs:45`)
- `codex-api/src/common.rs` — defines canonical Responses request, WebSocket request, event, and stream wire types. (`codex-api/src/common.rs:98`)
- `codex-api/src/telemetry.rs` — defines SSE and WebSocket telemetry hooks plus request telemetry wrapping. (`codex-api/src/telemetry.rs:18`)
- `codex-api/src/images.rs` — defines image generation and edit request/response DTOs. (`codex-api/src/images.rs:5`)
- `codex-api/src/safety_buffering.rs` — parses safety-buffering HTTP headers into treatment state. (`codex-api/src/safety_buffering.rs:8`)
- `codex-api/src/rate_limits.rs` — parses rate-limit and credits headers/events into normalized snapshots. (`codex-api/src/rate_limits.rs:23`)
- `codex-api/src/search.rs` — defines search commands, settings, and request/response DTOs. (`codex-api/src/search.rs:9`)
- `codex-api/src/files.rs` — handles hosted file upload creation, blob transfer, finalization, and canonical file URIs. (`codex-api/src/files.rs:121`)
- `codex-api/src/api_bridge.rs` — translates provider HTTP and transport failures into `CodexErr`. (`codex-api/src/api_bridge.rs:21`)
- `codex-api/src/api_bridge_tests.rs` — tests HTTP failure translation, including overload, policy, and usage-limit cases. (`codex-api/src/api_bridge_tests.rs:8`)

## codex-api/src/endpoint

- `codex-api/src/endpoint/mod.rs` — declares endpoint modules and re-exports their clients. (`codex-api/src/endpoint/mod.rs:1`)
- `codex-api/src/endpoint/session.rs` — provides the shared endpoint executor for auth, retries, headers, and telemetry. (`codex-api/src/endpoint/session.rs:19`)
- `codex-api/src/endpoint/responses.rs` — implements Responses request routing, compression, headers, and stream selection. (`codex-api/src/endpoint/responses.rs:52`)
- `codex-api/src/endpoint/compact.rs` — implements the conversation compaction endpoint. (`codex-api/src/endpoint/compact.rs:18`)
- `codex-api/src/endpoint/images.rs` — implements image generation and edit POST requests. (`codex-api/src/endpoint/images.rs:18`)
- `codex-api/src/endpoint/search.rs` — implements the search endpoint client. (`codex-api/src/endpoint/search.rs:14`)
- `codex-api/src/endpoint/memories.rs` — implements the memory trace summary endpoint. (`codex-api/src/endpoint/memories.rs:15`)
- `codex-api/src/endpoint/models.rs` — implements model listing and ETag extraction. (`codex-api/src/endpoint/models.rs:14`)
- `codex-api/src/endpoint/realtime_call.rs` — creates WebRTC realtime calls and encodes SDP/session payloads. (`codex-api/src/endpoint/realtime_call.rs:90`)
- `codex-api/src/endpoint/responses_websocket.rs` — manages Responses WebSocket connections and bridges their messages into response streams. (`codex-api/src/endpoint/responses_websocket.rs:235`)

## codex-api/src/endpoint/realtime_websocket

- `codex-api/src/endpoint/realtime_websocket/mod.rs` — declares realtime WebSocket modules and public re-exports. (`codex-api/src/endpoint/realtime_websocket/mod.rs:1`)
- `codex-api/src/endpoint/realtime_websocket/methods.rs` — connects realtime sockets and orchestrates outbound sends, inbound events, retries, URLs, and transcript state. (`codex-api/src/endpoint/realtime_websocket/methods.rs:777`)
- `codex-api/src/endpoint/realtime_websocket/methods_common.rs` — routes realtime session/item/handoff requests to adapter-specific outbound messages. (`codex-api/src/endpoint/realtime_websocket/methods_common.rs:41`)
- `codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs` — tests adapter-specific realtime outbound message encoding. (`codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs:15`)
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs` — encodes Frameless Bidi session and context-append messages. (`codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs:13`)
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs` — tests Frameless Bidi context chunking and session JSON. (`codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs:10`)
- `codex-api/src/endpoint/realtime_websocket/methods_v1.rs` — encodes realtime v1 conversation, handoff, and session-update messages. (`codex-api/src/endpoint/realtime_websocket/methods_v1.rs:18`)
- `codex-api/src/endpoint/realtime_websocket/methods_v2.rs` — encodes realtime v2 conversation, function-output, mode, tool, and session-update messages. (`codex-api/src/endpoint/realtime_websocket/methods_v2.rs:75`)
- `codex-api/src/endpoint/realtime_websocket/protocol.rs` — defines shared realtime wire types, session config, outbound messages, and parser dispatch. (`codex-api/src/endpoint/realtime_websocket/protocol.rs:50`)
- `codex-api/src/endpoint/realtime_websocket/protocol_common.rs` — provides common realtime payload and event parsing helpers. (`codex-api/src/endpoint/realtime_websocket/protocol_common.rs:7`)
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs` — decodes Frameless Bidi audio, transcript, turn, delegation, session, and error events. (`codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs:15`)
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs` — tests Frameless Bidi event decoding and compatibility with legacy handoffs. (`codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs:6`)
- `codex-api/src/endpoint/realtime_websocket/protocol_v1.rs` — decodes realtime v1 audio, transcript, item, handoff, and error events. (`codex-api/src/endpoint/realtime_websocket/protocol_v1.rs:12`)
- `codex-api/src/endpoint/realtime_websocket/protocol_v2.rs` — decodes realtime v2 audio, transcript, response, item, handoff, noop, and error events. (`codex-api/src/endpoint/realtime_websocket/protocol_v2.rs:24`)

## codex-api/src/requests

- `codex-api/src/requests/mod.rs` — declares request modules and re-exports compression handling. (`codex-api/src/requests/mod.rs:1`)
- `codex-api/src/requests/responses.rs` — defines outbound Responses request compression modes. (`codex-api/src/requests/responses.rs:2`)
- `codex-api/src/requests/headers.rs` — builds session, subagent, and generic outbound request headers. (`codex-api/src/requests/headers.rs:5`)
- `codex-api/src/requests/chat.rs` — translates Responses requests into Chat Completions request bodies and tool payloads. (`codex-api/src/requests/chat.rs:19`)

## codex-api/src/sse

- `codex-api/src/sse/mod.rs` — declares SSE modules and re-exports stream handlers. (`codex-api/src/sse/mod.rs:1`)
- `codex-api/src/sse/chat.rs` — translates inbound Chat Completions SSE chunks into Responses events. (`codex-api/src/sse/chat.rs:77`)
- `codex-api/src/sse/responses.rs` — translates inbound Responses SSE events, headers, errors, safety state, and usage into response streams. (`codex-api/src/sse/responses.rs:353`)
