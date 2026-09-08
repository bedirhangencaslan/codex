# Codex API wire map

## `codex-api/src`

- `codex-api/src/api_bridge.rs` — maps `ApiError` and transport failures into `CodexErr`, including overload, policy, Cloudflare, and retry-delay cases — `codex-api/src/api_bridge.rs:21`
- `codex-api/src/api_bridge_tests.rs` — tests `map_api_error` for overload, retry delays, Cloudflare, and policy-shaped response bodies — `codex-api/src/api_bridge_tests.rs:7`
- `codex-api/src/auth.rs` — defines `AuthProvider`, auth-header resolution, request signing, auth errors, and auth telemetry — `codex-api/src/auth.rs:30`
- `codex-api/src/common.rs` — defines shared Responses request/websocket payload types, compaction and memory inputs, `ResponseEvent`, trace metadata, and `ResponseStream` — `codex-api/src/common.rs:275`
- `codex-api/src/error.rs` — defines the crate-wide `ApiError` taxonomy — `codex-api/src/error.rs:9`
- `codex-api/src/files.rs` — handles OpenAI file upload creation, size limits, blob upload, finalization, and uploaded-file metadata — `codex-api/src/files.rs:121`
- `codex-api/src/images.rs` — defines image generation/edit wire requests and image response payloads — `codex-api/src/images.rs:5`
- `codex-api/src/lib.rs` — declares the crate modules and re-exports the public API surface — `codex-api/src/lib.rs:1`
- `codex-api/src/provider.rs` — defines provider endpoint configuration, retry policy, `WireApi::Responses` vs `Chat`, URL/request construction, and Azure detection — `codex-api/src/provider.rs:56`
- `codex-api/src/rate_limits.rs` — parses default and per-limit rate-limit headers, credits, plan state, and rate-limit events into snapshots — `codex-api/src/rate_limits.rs:57`
- `codex-api/src/safety_buffering.rs` — reads safety-buffering treatment headers into the common treatment type — `codex-api/src/safety_buffering.rs:8`
- `codex-api/src/search.rs` — defines search requests, commands, filters, locations, and response payloads for the search API — `codex-api/src/search.rs:9`
- `codex-api/src/telemetry.rs` — defines SSE and WebSocket telemetry hooks and wraps retryable HTTP/stream calls with request telemetry — `codex-api/src/telemetry.rs:68`

## `codex-api/src/endpoint`

- `codex-api/src/endpoint/compact.rs` — posts compaction requests to `responses/compact`, captures turn state, and decodes compacted history output — `codex-api/src/endpoint/compact.rs:39`
- `codex-api/src/endpoint/images.rs` — sends image generation and edit requests to `images/generations` and `images/edits`, returning decoded images and request IDs — `codex-api/src/endpoint/images.rs:35`
- `codex-api/src/endpoint/memories.rs` — posts memory inputs to `memories/trace_summarize` and decodes memory summaries — `codex-api/src/endpoint/memories.rs:36`
- `codex-api/src/endpoint/mod.rs` — declares endpoint submodules and exports their clients and realtime types — `codex-api/src/endpoint/mod.rs:12`
- `codex-api/src/endpoint/models.rs` — requests the provider model catalog from `models`, attaches client version, and returns models plus ETag — `codex-api/src/endpoint/models.rs:46`
- `codex-api/src/endpoint/realtime_call.rs` — creates WebRTC realtime calls, sends SDP/session payloads, and parses SDP plus call ID from response headers — `codex-api/src/endpoint/realtime_call.rs:129`
- `codex-api/src/endpoint/responses.rs` — routes Responses inference over HTTP, translating Responses requests for Chat-only providers and spawning the matching SSE stream — `codex-api/src/endpoint/responses.rs:112`
- `codex-api/src/endpoint/responses_websocket.rs` — connects Responses WebSocket sessions, serializes/sends requests, streams events, and probes handshakes — `codex-api/src/endpoint/responses_websocket.rs:235`
- `codex-api/src/endpoint/search.rs` — posts search requests to `alpha/search` and decodes search responses — `codex-api/src/endpoint/search.rs:35`
- `codex-api/src/endpoint/session.rs` — shared endpoint session that builds authenticated requests and runs unary and streaming HTTP calls with retry/telemetry — `codex-api/src/endpoint/session.rs:80`

## `codex-api/src/endpoint/realtime_websocket`

- `codex-api/src/endpoint/realtime_websocket/methods.rs` — realtime WebSocket client and connection: URL construction, send/receive pumps, writer methods, event parsing, transcripts, and sideband joins — `codex-api/src/endpoint/realtime_websocket/methods.rs:765`
- `codex-api/src/endpoint/realtime_websocket/methods_common.rs` — routes outbound realtime actions across V1, Frameless Bidi, and Realtime V2 adapters — `codex-api/src/endpoint/realtime_websocket/methods_common.rs:41`
- `codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs` — tests adapter-specific session updates, handoff payloads, context channels, and V1 output prefixes — `codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs:16`
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs` — builds Frameless Bidi session, context-append, and delegation payloads, including 500-byte chunking — `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs:52`
- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs` — tests Frameless Bidi chunk limits and initial-item session JSON encoding — `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs:11`
- `codex-api/src/endpoint/realtime_websocket/methods_v1.rs` — builds legacy realtime V1 conversation, handoff, Quicksilver session, and websocket-intent payloads — `codex-api/src/endpoint/realtime_websocket/methods_v1.rs:51`
- `codex-api/src/endpoint/realtime_websocket/methods_v2.rs` — builds Realtime V2 conversation, function-output, conversational/transcription session, tools, and audio-format payloads — `codex-api/src/endpoint/realtime_websocket/methods_v2.rs:75`
- `codex-api/src/endpoint/realtime_websocket/mod.rs` — declares realtime method/protocol modules and exports their public realtime types — `codex-api/src/endpoint/realtime_websocket/mod.rs:12`
- `codex-api/src/endpoint/realtime_websocket/protocol.rs` — defines realtime parser/session types and outbound wire-message shapes, dispatching parsing to protocol adapters — `codex-api/src/endpoint/realtime_websocket/protocol.rs:52`
- `codex-api/src/endpoint/realtime_websocket/protocol_common.rs` — provides shared realtime JSON parsing for session updates, transcript deltas/done events, and errors — `codex-api/src/endpoint/realtime_websocket/protocol_common.rs:7`
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs` — parses Frameless Bidi session, audio, transcript, turn, delegation, and error events — `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs:15`
- `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs` — verifies legacy and Frameless Bidi handoffs decode to the same event and transcript/audio reuse internal events — `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs:7`
- `codex-api/src/endpoint/realtime_websocket/protocol_v1.rs` — parses legacy realtime V1 audio, transcript, item, handoff, and error events — `codex-api/src/endpoint/realtime_websocket/protocol_v1.rs:12`
- `codex-api/src/endpoint/realtime_websocket/protocol_v2.rs` — parses Realtime V2 audio, transcript, response lifecycle, speech, item, handoff, noop, and error events — `codex-api/src/endpoint/realtime_websocket/protocol_v2.rs:24`

## `codex-api/src/requests`

- `codex-api/src/requests/chat.rs` — translates a Responses API request into an OpenAI-compatible `/chat/completions` body, including roles, reasoning, tools, and tool outputs — `codex-api/src/requests/chat.rs:19`
- `codex-api/src/requests/headers.rs` — builds session/thread headers, subagent headers, and safe header insertion helpers — `codex-api/src/requests/headers.rs:5`
- `codex-api/src/requests/mod.rs` — declares request modules and exports `Compression` — `codex-api/src/requests/mod.rs:1`
- `codex-api/src/requests/responses.rs` — defines the request compression choices `None` and `Zstd` — `codex-api/src/requests/responses.rs:2`

## `codex-api/src/sse`

- `codex-api/src/sse/chat.rs` — translates Chat Completions SSE deltas into accumulated Responses-shaped items and events — `codex-api/src/sse/chat.rs:77`
- `codex-api/src/sse/mod.rs` — declares SSE modules and exports the Responses/Chat stream helpers — `codex-api/src/sse/mod.rs:4`
- `codex-api/src/sse/responses.rs` — parses Responses SSE and websocket events into normalized response events, headers, rate limits, usage, errors, and completion state — `codex-api/src/sse/responses.rs:353`
