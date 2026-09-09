# codex-api wire map

## `codex-api/src/`

- `codex-api/src/api_bridge.rs` — Translates `ApiError` and HTTP transport failures into `CodexErr`, including overloaded, rate-limit, cyber-policy, misalignment, Cloudflare, and request-tracking details. (`codex-api/src/api_bridge.rs:21`)
- `codex-api/src/api_bridge_tests.rs` — Tests error mapping across overloaded bodies, retry delays, Cloudflare blocks, cyber and misalignment policies, usage limits, and identity headers. (`codex-api/src/api_bridge_tests.rs:8`)
- `codex-api/src/auth.rs` — Handles the outbound auth provider abstraction, header-only and request-signing authentication futures, shared auth handles, and auth telemetry. (`codex-api/src/auth.rs:30`)
- `codex-api/src/common.rs` — Defines the common Responses API request, WebSocket request, response event/stream, reasoning, text, compaction, memory, and safety-buffering wire types. (`codex-api/src/common.rs:275`)
- `codex-api/src/error.rs` — Defines `ApiError`, the crate-wide error taxonomy for transport, stream, quota, retry, rate-limit, policy, and overload failures. (`codex-api/src/error.rs:9`)
- `codex-api/src/files.rs` — Handles hosted OpenAI file uploads, including create, blob upload, finalize/download polling, limits, and resulting file metadata. (`codex-api/src/files.rs:121`)
- `codex-api/src/images.rs` — Defines image generation/edit requests and image response/data wire types. (`codex-api/src/images.rs:5`)
- `codex-api/src/lib.rs` — Declares the crate modules and public API surface for auth, clients, provider settings, requests, SSE, telemetry, and errors. (`codex-api/src/lib.rs:1`)
- `codex-api/src/provider.rs` — Handles provider endpoint construction, retry configuration, HTTP/WebSocket URL building, and Responses versus Chat wire selection. (`codex-api/src/provider.rs:56`)
- `codex-api/src/rate_limits.rs` — Translates rate-limit HTTP headers and WebSocket rate-limit events into protocol snapshots, including credits, promo messages, and reached type. (`codex-api/src/rate_limits.rs:57`)
- `codex-api/src/safety_buffering.rs` — Reads the safety-buffering enabled/faster-model headers into a wire treatment. (`codex-api/src/safety_buffering.rs:8`)
- `codex-api/src/search.rs` — Defines the search endpoint request, commands, settings, filters, and response wire types. (`codex-api/src/search.rs:9`)
- `codex-api/src/telemetry.rs` — Wraps unary and streaming requests with retry/request telemetry and defines SSE and WebSocket telemetry traits. (`codex-api/src/telemetry.rs:68`)

## `codex-api/src/requests/`

- `codex-api/src/requests/chat.rs` — Translates a Responses API request into an OpenAI-compatible Chat Completions body, including messages, tools, reasoning, roles, and response format. (`codex-api/src/requests/chat.rs:19`)
- `codex-api/src/requests/headers.rs` — Builds session/thread headers and derives subagent and generic header values from protocol session sources. (`codex-api/src/requests/headers.rs:5`)
- `codex-api/src/requests/mod.rs` — Assembles the outbound request modules and re-exports the compression wire option. (`codex-api/src/requests/mod.rs:1`)
- `codex-api/src/requests/responses.rs` — Defines the HTTP compression selection for Responses requests. (`codex-api/src/requests/responses.rs:2`)

## `codex-api/src/sse/`

- `codex-api/src/sse/chat.rs` — Translates OpenAI-compatible Chat Completions SSE deltas back into Responses-shaped items, tool calls, usage, completion, and errors. (`codex-api/src/sse/chat.rs:77`)
- `codex-api/src/sse/mod.rs` — Assembles SSE translation modules and exposes the Chat and Responses stream entry points. (`codex-api/src/sse/mod.rs:1`)
- `codex-api/src/sse/responses.rs` — Parses Responses SSE events, emits common response events, and translates completion, usage, model, turn-state, moderation, and error information. (`codex-api/src/sse/responses.rs:353`)

## `codex-api/src/endpoint/`

- `codex-api/src/endpoint/compact.rs` — Calls the compaction endpoint, serializes/decodes compact history, applies timeouts, and records turn state. (`codex-api/src/endpoint/compact.rs:39`)
- `codex-api/src/endpoint/images.rs` — Calls image generation/edit endpoints and decodes image responses plus the image request ID header. (`codex-api/src/endpoint/images.rs:35`)
- `codex-api/src/endpoint/memories.rs` — Calls the memory trace-summary endpoint and translates between typed memory input and summary output. (`codex-api/src/endpoint/memories.rs:36`)
- `codex-api/src/endpoint/models.rs` — Calls the model list endpoint and decodes model info and ETag metadata. (`codex-api/src/endpoint/models.rs:46`)
- `codex-api/src/endpoint/mod.rs` — Assembles endpoint clients and their public exports. (`codex-api/src/endpoint/mod.rs:1`)
- `codex-api/src/endpoint/realtime_call.rs` — Creates Realtime WebRTC calls, sends SDP/session payloads, decodes SDP and call IDs, and adapts backend versus provider paths. (`codex-api/src/endpoint/realtime_call.rs:90`)
- `codex-api/src/endpoint/responses.rs` — Sends streaming Responses or Chat Completions requests, selects endpoint routes, adds session/compression headers, and chooses the matching SSE adapter. (`codex-api/src/endpoint/responses.rs:112`)
- `codex-api/src/endpoint/responses_websocket.rs` — Manages Responses-over-WebSocket connections, request serialization, streams, probes/close frames, telemetry, rate limits, and wrapped errors. (`codex-api/src/endpoint/responses_websocket.rs:345`)
- `codex-api/src/endpoint/search.rs` — Calls the search endpoint, serializes typed search requests, and decodes search responses. (`codex-api/src/endpoint/search.rs:35`)
- `codex-api/src/endpoint/session.rs` — Provides the shared endpoint execution layer for authenticated, retried, telemetry-instrumented unary and streaming HTTP calls. (`codex-api/src/endpoint/session.rs:80`)

## `codex-api/src/endpoint/realtime_websocket/`

- `codex-api/src/endpoint/realtime_websocket/methods.rs` — Handles Realtime WebSocket clients, connections, writers, transcript state, URL/path normalization, event dispatch, and session lifecycle. (`codex-api/src/endpoint/realtime_websocket/methods.rs:765`)
- `codex-api/src/endpoint/realtime_websocket/methods_common.rs` — Routes Realtime outbound operations and session configuration to the V1, V2, or Frameless Bidi wire adapter. (`codex-api/src/endpoint/realtime_websocket/methods_common.rs:114`)
- `codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs` — Tests adapter-specific session updates, context channels, handoff output, and V1-only agent-final prefixes. (`codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs:16`)
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs` — Builds Frameless Bidi session updates, context/delegation appends, and byte-safe context chunks. (`codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs:13`)
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs` — Tests Frameless Bidi chunk limits, omitted initial items, and role-bearing initial-item encoding. (`codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs:11`)
- `codex-api/src/endpoint/realtime_websocket/methods_v1.rs` — Builds legacy Quicksilver Realtime conversation items, handoff appends, session updates, and WebSocket intents. (`codex-api/src/endpoint/realtime_websocket/methods_v1.rs:18`)
- `codex-api/src/endpoint/realtime_websocket/methods_v2.rs` — Builds Realtime V2 conversation/function-output items, session modes and modalities, audio formats, transcription, and agent/silence tools. (`codex-api/src/endpoint/realtime_websocket/methods_v2.rs:75`)
- `codex-api/src/endpoint/realtime_websocket/mod.rs` — Assembles the realtime wire protocol and outbound method modules and exports the realtime client types. (`codex-api/src/endpoint/realtime_websocket/mod.rs:1`)
- `codex-api/src/endpoint/realtime_websocket/protocol.rs` — Defines Realtime parser/config/modality types and outbound message wire payloads, then dispatches inbound events by parser. (`codex-api/src/endpoint/realtime_websocket/protocol.rs:263`)
- `codex-api/src/endpoint/realtime_websocket/protocol_common.rs` — Provides shared inbound JSON parsing for realtime payloads, session updates, transcript deltas/done events, and errors. (`codex-api/src/endpoint/realtime_websocket/protocol_common.rs:7`)
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs` — Parses Frameless Bidi session, audio, transcript, turn-done, delegation, and error events into realtime events. (`codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs:15`)
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs` — Tests compatibility between legacy and frameless delegation events and common transcript/audio decoding. (`codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs:7`)
- `codex-api/src/endpoint/realtime_websocket/protocol_v1.rs` — Parses legacy Realtime V1 session, audio, transcript, item, handoff, and error events. (`codex-api/src/endpoint/realtime_websocket/protocol_v1.rs:12`)
- `codex-api/src/endpoint/realtime_websocket/protocol_v2.rs` — Parses Realtime V2 session, audio, transcript, response lifecycle, conversation item, handoff, silence, and error events. (`codex-api/src/endpoint/realtime_websocket/protocol_v2.rs:24`)
