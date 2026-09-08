# Wire map

## codex-api/src

- `codex-api/src/api_bridge.rs`: Translates wire-level `ApiError` and HTTP transport failures into `CodexErr` variants, including retry delays, overload, rate-limit, policy, and Cloudflare details. (`codex-api/src/api_bridge.rs:21`)
- `codex-api/src/api_bridge_tests.rs`: Tests error mapping for overloaded responses, retry delays, rate limits, and Cloudflare blocking. (`codex-api/src/api_bridge_tests.rs:7`)
- `codex-api/src/auth.rs`: Defines the outbound auth-provider contract for header-only and request-signing authentication, plus auth telemetry. (`codex-api/src/auth.rs:30`)
- `codex-api/src/common.rs`: Defines shared Responses request, response-event, stream, tool, text-control, and WebSocket request wire types. (`codex-api/src/common.rs:275`)
- `codex-api/src/error.rs`: Defines the crate's API wire/error taxonomy, including transport, rate-limit, policy, and overload cases. (`codex-api/src/error.rs:9`)
- `codex-api/src/files.rs`: Handles OpenAI hosted file upload requests, validation, blob upload, finalization, retries, and errors. (`codex-api/src/files.rs:121`)
- `codex-api/src/images.rs`: Defines image generation/edit request and response wire types. (`codex-api/src/images.rs:5`)
- `codex-api/src/lib.rs`: Declares the crate's modules and selects the public wire API surface. (`codex-api/src/lib.rs:1`)
- `codex-api/src/provider.rs`: Configures provider endpoints, headers, retries, stream timeout, and Responses versus Chat wire protocol. (`codex-api/src/provider.rs:56`)
- `codex-api/src/rate_limits.rs`: Parses default and per-limit rate-limit header families into snapshots, credits, reached types, and plan data. (`codex-api/src/rate_limits.rs:23`)
- `codex-api/src/safety_buffering.rs`: Reads safety-buffering treatment headers into a shared response treatment value. (`codex-api/src/safety_buffering.rs:8`)
- `codex-api/src/search.rs`: Defines the internet/search request, operations, settings, and response wire payloads. (`codex-api/src/search.rs:9`)
- `codex-api/src/telemetry.rs`: Adds request, SSE, and WebSocket telemetry hooks around retries and transport polling. (`codex-api/src/telemetry.rs:68`)

## codex-api/src/endpoint

- `codex-api/src/endpoint/compact.rs`: Implements the `responses/compact` client, turn-state header capture, and history-output decoding. (`codex-api/src/endpoint/compact.rs:39`)
- `codex-api/src/endpoint/images.rs`: Implements image generation and edit HTTP requests and response/request-ID decoding. (`codex-api/src/endpoint/images.rs:35`)
- `codex-api/src/endpoint/memories.rs`: Implements the memory trace-summarization client and output decoding. (`codex-api/src/endpoint/memories.rs:36`)
- `codex-api/src/endpoint/mod.rs`: Exports endpoint clients and realtime types. (`codex-api/src/endpoint/mod.rs:1`)
- `codex-api/src/endpoint/models.rs`: Implements model-list retrieval with client-version URLs and ETag extraction. (`codex-api/src/endpoint/models.rs:46`)
- `codex-api/src/endpoint/realtime_call.rs`: Creates WebRTC realtime calls, sends SDP/session requests, and parses SDP plus call IDs. (`codex-api/src/endpoint/realtime_call.rs:90`)
- `codex-api/src/endpoint/responses.rs`: Implements Responses/Guardian HTTP clients, request assembly, compression, headers, and SSE dispatch. (`codex-api/src/endpoint/responses.rs:52`)
- `codex-api/src/endpoint/responses_websocket.rs`: Implements the Responses WebSocket client, connection lifecycle, request/response flow, and close handling. (`codex-api/src/endpoint/responses_websocket.rs:375`)
- `codex-api/src/endpoint/search.rs`: Implements the alpha search HTTP client and response decoding. (`codex-api/src/endpoint/search.rs:35`)
- `codex-api/src/endpoint/session.rs`: Provides shared authenticated HTTP execution, retry, telemetry, and request customization for endpoint clients. (`codex-api/src/endpoint/session.rs:80`)

## codex-api/src/endpoint/realtime_websocket

- `codex-api/src/endpoint/realtime_websocket/methods.rs`: Implements realtime WebSocket connections, writers/events, session setup, audio, transcripts, and wire communication. (`codex-api/src/endpoint/realtime_websocket/methods.rs:765`)
- `codex-api/src/endpoint/realtime_websocket/methods_common.rs`: Routes realtime operations to V1, V2, or frameless-bidi wire adapters and normalizes messages. (`codex-api/src/endpoint/realtime_websocket/methods_common.rs:41`)
- `codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs`: Tests realtime outbound message translation across wire adapters. (`codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs:15`)
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs`: Builds frameless-bidi session updates and session/delegation context-append messages. (`codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs:13`)
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs`: Tests frameless chunking and initial-session JSON encoding. (`codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs:10`)
- `codex-api/src/endpoint/realtime_websocket/methods_v1.rs`: Builds V1 realtime conversation, handoff, session, and intent messages. (`codex-api/src/endpoint/realtime_websocket/methods_v1.rs:18`)
- `codex-api/src/endpoint/realtime_websocket/methods_v2.rs`: Builds V2 realtime conversation, tool-output, session, and intent messages. (`codex-api/src/endpoint/realtime_websocket/methods_v2.rs:39`)
- `codex-api/src/endpoint/realtime_websocket/mod.rs`: Organizes realtime protocol/method modules and exports their public surface. (`codex-api/src/endpoint/realtime_websocket/mod.rs:1`)
- `codex-api/src/endpoint/realtime_websocket/protocol.rs`: Defines realtime adapters, session configuration, and outbound wire-message types. (`codex-api/src/endpoint/realtime_websocket/protocol.rs:38`)
- `codex-api/src/endpoint/realtime_websocket/protocol_common.rs`: Provides common realtime payload parsing for session, transcript, and error events. (`codex-api/src/endpoint/realtime_websocket/protocol_common.rs:7`)
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs`: Parses frameless-bidi audio, transcript, turn, delegation, and error events. (`codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs:15`)
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs`: Tests frameless handoff, transcript, and audio event parsing. (`codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs:6`)
- `codex-api/src/endpoint/realtime_websocket/protocol_v1.rs`: Parses V1 realtime audio, transcript, conversation, handoff, and error events. (`codex-api/src/endpoint/realtime_websocket/protocol_v1.rs:12`)
- `codex-api/src/endpoint/realtime_websocket/protocol_v2.rs`: Parses V2 realtime audio, transcript, response, speech, item, and error events. (`codex-api/src/endpoint/realtime_websocket/protocol_v2.rs:24`)

## codex-api/src/requests

- `codex-api/src/requests/chat.rs`: Translates Responses API requests into OpenAI-compatible Chat Completions bodies. (`codex-api/src/requests/chat.rs:19`)
- `codex-api/src/requests/headers.rs`: Builds session/thread headers and derives subagent headers. (`codex-api/src/requests/headers.rs:5`)
- `codex-api/src/requests/mod.rs`: Organizes request modules and re-exports request compression. (`codex-api/src/requests/mod.rs:1`)
- `codex-api/src/requests/responses.rs`: Defines request body compression choices. (`codex-api/src/requests/responses.rs:2`)

## codex-api/src/sse

- `codex-api/src/sse/chat.rs`: Translates Chat Completions SSE deltas into Responses-shaped events while rebuilding assistant/tool turn state. (`codex-api/src/sse/chat.rs:30`)
- `codex-api/src/sse/mod.rs`: Organizes SSE modules and exports stream helpers. (`codex-api/src/sse/mod.rs:1`)
- `codex-api/src/sse/responses.rs`: Parses Responses SSE and response headers into normalized events, usage, limits, turn state, and errors. (`codex-api/src/sse/responses.rs:36`)
