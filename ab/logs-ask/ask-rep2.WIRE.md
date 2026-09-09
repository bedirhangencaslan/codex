# codex-api wire map

## codex-api/src

- `codex-api/src/api_bridge.rs` — maps provider `ApiError` and transport/HTTP responses into protocol errors, decoding overload, policy, usage-limit, and tracking headers. `codex-api/src/api_bridge.rs:21`
- `codex-api/src/api_bridge_tests.rs` — tests `map_api_error` behavior across overload, retry delays, Cloudflare, policy, and usage-limit responses. `codex-api/src/api_bridge_tests.rs:8`
- `codex-api/src/auth.rs` — defines the `AuthProvider` contract for header-only and request-signing auth plus auth telemetry. `codex-api/src/auth.rs:30`
- `codex-api/src/common.rs` — defines Responses wire payloads, compaction/memory inputs, response tool/text controls, and the unified `ResponseEvent` stream. `codex-api/src/common.rs:97`
- `codex-api/src/error.rs` — defines API error variants such as transport, stream, quota, retry, rate limit, policy, and overload. `codex-api/src/error.rs:9`
- `codex-api/src/files.rs` — implements OpenAI hosted file upload through create, blob upload, and finalize flows. `codex-api/src/files.rs:121`
- `codex-api/src/images.rs` — defines image generation/edit request and response wire types. `codex-api/src/images.rs:5`
- `codex-api/src/lib.rs` — declares crate modules and re-exports the public API clients, wire types, auth, files, search, telemetry, and errors. `codex-api/src/lib.rs:1`
- `codex-api/src/provider.rs` — defines provider endpoint configuration, retry policy, Responses/Chat wire selection, request/WebSocket URL building, and Azure detection. `codex-api/src/provider.rs:56`
- `codex-api/src/rate_limits.rs` — parses rate-limit headers and WebSocket rate-limit events into snapshots, credits, promo messages, and reached types. `codex-api/src/rate_limits.rs:23`
- `codex-api/src/safety_buffering.rs` — derives safety-buffering treatment from response headers. `codex-api/src/safety_buffering.rs:8`
- `codex-api/src/search.rs` — defines typed search requests, commands, operation schemas, settings, and response lengths. `codex-api/src/search.rs:9`
- `codex-api/src/telemetry.rs` — defines SSE/WebSocket telemetry traits and wraps retry execution with per-request telemetry. `codex-api/src/telemetry.rs:18`

## codex-api/src/endpoint

- `codex-api/src/endpoint/compact.rs` — POSTs `responses/compact`, parses compacted history, and captures turn state. `codex-api/src/endpoint/compact.rs:39`
- `codex-api/src/endpoint/images.rs` — POSTs image generation/edit requests and decodes image responses and request IDs. `codex-api/src/endpoint/images.rs:35`
- `codex-api/src/endpoint/memories.rs` — POSTs `memories/trace_summarize` and decodes memory summaries. `codex-api/src/endpoint/memories.rs:36`
- `codex-api/src/endpoint/mod.rs` — exports endpoint clients and related realtime, Responses, and search public types. `codex-api/src/endpoint/mod.rs:12`
- `codex-api/src/endpoint/models.rs` — GETs model listings with client-version URLs and returns models plus ETag. `codex-api/src/endpoint/models.rs:46`
- `codex-api/src/endpoint/realtime_call.rs` — creates WebRTC realtime calls from SDP and optional session configuration, handling backend/legacy paths. `codex-api/src/endpoint/realtime_call.rs:90`
- `codex-api/src/endpoint/responses.rs` — streams Responses-compatible requests over HTTP, translating Responses or Chat bodies and dispatching the matching SSE parser. `codex-api/src/endpoint/responses.rs:112`
- `codex-api/src/endpoint/responses_websocket.rs` — manages Responses WebSocket connections, serialization, streaming, headers, retries/reconnect cues, and telemetry. `codex-api/src/endpoint/responses_websocket.rs:206`
- `codex-api/src/endpoint/search.rs` — POSTs typed search requests to `alpha/search` and decodes search responses. `codex-api/src/endpoint/search.rs:35`
- `codex-api/src/endpoint/session.rs` — shared endpoint transport that builds authenticated, retrying unary or streaming requests with telemetry. `codex-api/src/endpoint/session.rs:80`

## codex-api/src/endpoint/realtime_websocket

- `codex-api/src/endpoint/realtime_websocket/methods.rs` — implements the realtime WebSocket client lifecycle, connection pump, outbound writes, event queue, and bounded transcript state. `codex-api/src/endpoint/realtime_websocket/methods.rs:64`
- `codex-api/src/endpoint/realtime_websocket/methods_common.rs` — normalizes realtime outbound messages and session JSON across V1, Frameless Bidi, and Realtime V2 adapters. `codex-api/src/endpoint/realtime_websocket/methods_common.rs:41`
- `codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs` — tests adapter-specific serialization of session updates, handoffs, channels, and function outputs. `codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs:16`
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs` — constructs Frameless Bidi session updates, session/context appends, and byte-limited context chunks. `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs:35`
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs` — tests Frameless Bidi context chunk preservation and initial-item JSON encoding. `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs:11`
- `codex-api/src/endpoint/realtime_websocket/methods_v1.rs` — builds V1/Quicksilver conversation item, handoff, session update, and websocket intent messages. `codex-api/src/endpoint/realtime_websocket/methods_v1.rs:18`
- `codex-api/src/endpoint/realtime_websocket/methods_v2.rs` — builds Realtime V2 conversation items, function outputs, transcription/conversational sessions, and tools. `codex-api/src/endpoint/realtime_websocket/methods_v2.rs:39`
- `codex-api/src/endpoint/realtime_websocket/mod.rs` — exports realtime WebSocket modules, client types, adapters, session configuration, and parser utilities. `codex-api/src/endpoint/realtime_websocket/mod.rs:12`
- `codex-api/src/endpoint/realtime_websocket/protocol.rs` — defines realtime adapters, session config, outbound wire messages, and session/audio/tool wire structures. `codex-api/src/endpoint/realtime_websocket/protocol.rs:50`
- `codex-api/src/endpoint/realtime_websocket/protocol_common.rs` — provides common parsing for realtime JSON payloads, session updates, transcript deltas, completions, and errors. `codex-api/src/endpoint/realtime_websocket/protocol_common.rs:7`
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs` — parses Frameless Bidi audio, transcript, turn, delegation, session, and error events. `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs:15`
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs` — tests Frameless Bidi/legacy handoff equivalence and normalized internal events. `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs:7`
- `codex-api/src/endpoint/realtime_websocket/protocol_v1.rs` — parses realtime V1 audio, transcript, item, handoff, and error events. `codex-api/src/endpoint/realtime_websocket/protocol_v1.rs:12`
- `codex-api/src/endpoint/realtime_websocket/protocol_v2.rs` — parses Realtime V2 responses, audio, transcripts, items, background-agent handoffs, silence tool calls, and errors. `codex-api/src/endpoint/realtime_websocket/protocol_v2.rs:24`

## codex-api/src/requests

- `codex-api/src/requests/chat.rs` — translates a Responses request into an OpenAI Chat Completions body, including messages, tools, reasoning, and output format. `codex-api/src/requests/chat.rs:19`
- `codex-api/src/requests/headers.rs` — builds session headers and derives subagent header values from session source. `codex-api/src/requests/headers.rs:5`
- `codex-api/src/requests/mod.rs` — exports request modules and compression type. `codex-api/src/requests/mod.rs:1`
- `codex-api/src/requests/responses.rs` — defines request compression choices. `codex-api/src/requests/responses.rs:2`

## codex-api/src/sse

- `codex-api/src/sse/chat.rs` — converts Chat Completions SSE chunks into unified `ResponseEvent`s while accumulating assistant and tool-call state. `codex-api/src/sse/chat.rs:77`
- `codex-api/src/sse/mod.rs` — exports Chat and Responses SSE stream utilities. `codex-api/src/sse/mod.rs:4`
- `codex-api/src/sse/responses.rs` — converts Responses SSE and header metadata into unified response events, usage, moderation, rate limits, and safety buffering. `codex-api/src/sse/responses.rs:36`
