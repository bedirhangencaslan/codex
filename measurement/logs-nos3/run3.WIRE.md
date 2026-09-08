# codex-api wire map

## codex-api/src

- `api_bridge.rs` — Maps API and transport errors into protocol-level `CodexErr` variants, preserving retry delays and provider status details. (`codex-api/src/api_bridge.rs:21`)
- `api_bridge_tests.rs` — Tests error mapping for overload, retry delay, Cloudflare blocks, and cyber-policy responses. (`codex-api/src/api_bridge_tests.rs:8`)
- `auth.rs` — Defines auth providers that add headers, asynchronously resolve credentials, or sign complete outbound requests. (`codex-api/src/auth.rs:30`)
- `telemetry.rs` — Defines SSE/WebSocket telemetry hooks and wraps retries with per-attempt request telemetry. (`codex-api/src/telemetry.rs:68`)
- `search.rs` — Defines the search request wire schema and command, filter, location, and caller types. (`codex-api/src/search.rs:9`)
- `safety_buffering.rs` — Reads safety-buffering treatment headers from upstream HTTP responses. (`codex-api/src/safety_buffering.rs:8`)
- `rate_limits.rs` — Parses one or more provider rate-limit header families into snapshots. (`codex-api/src/rate_limits.rs:23`)
- `provider.rs` — Configures provider URLs, headers, retries, stream timeout, WebSocket scheme, and whether it speaks Responses or Chat. (`codex-api/src/provider.rs:56`)
- `lib.rs` — Declares crate modules and exports the public API/client surface. (`codex-api/src/lib.rs:1`)
- `images.rs` — Defines image generation/edit request and response wire types. (`codex-api/src/images.rs:4`)
- `files.rs` — Implements OpenAI hosted-file upload/finalize/download flows and related errors. (`codex-api/src/files.rs:17`)
- `error.rs` — Defines API error variants for transport, streams, quotas, rate limits, policy, and overload. (`codex-api/src/error.rs:9`)
- `common.rs` — Defines shared request/response event and stream types for compaction, memories, reasoning, tools, text controls, tracing, and safety buffering. (`codex-api/src/common.rs:28`)

## codex-api/src/requests

- `mod.rs` — Exposes request encoding modules and compression enum. (`codex-api/src/requests/mod.rs:1`)
- `headers.rs` — Builds session, thread, subagent, and client request headers. (`codex-api/src/requests/headers.rs:5`)
- `responses.rs` — Declares Responses request compression choices. (`codex-api/src/requests/responses.rs:1`)
- `chat.rs` — Translates Responses-shaped requests into OpenAI-compatible Chat Completions bodies. (`codex-api/src/requests/chat.rs:1`)

## codex-api/src/sse

- `mod.rs` — Exposes Responses and Chat SSE stream handlers. (`codex-api/src/sse/mod.rs:1`)
- `responses.rs` — Parses Responses SSE events and HTTP headers into internal `ResponseEvent`s, including errors and usage. (`codex-api/src/sse/responses.rs:36`)
- `chat.rs` — Translates Chat Completions SSE deltas into Responses-shaped `ResponseEvent`s, rebuilding assistant, reasoning, and tool-call items. (`codex-api/src/sse/chat.rs:1`)

## codex-api/src/endpoint

- `mod.rs` — Declares endpoint clients and re-exports their public types. (`codex-api/src/endpoint/mod.rs:1`)
- `responses.rs` — Sends Responses or Chat inference requests, rewrites Chat bodies, and selects the corresponding SSE parser. (`codex-api/src/endpoint/responses.rs:112`)
- `responses_websocket.rs` — Connects and probes Responses WebSockets, serializes requests, runs event streams, and maps WS errors. (`codex-api/src/endpoint/responses_websocket.rs:344`)
- `session.rs` — Builds authenticated, retried HTTP endpoint requests for unary and streaming calls. (`codex-api/src/endpoint/session.rs:19`)
- `search.rs` — Posts typed search requests and decodes search responses. (`codex-api/src/endpoint/search.rs:35`)
- `compact.rs` — Posts compaction input and decodes compacted `ResponseItem`s. (`codex-api/src/endpoint/compact.rs:39`)
- `images.rs` — Posts image generation/edit requests and decodes image responses plus request IDs. (`codex-api/src/endpoint/images.rs:35`)
- `models.rs` — Lists models with client-version query and decodes models and ETag. (`codex-api/src/endpoint/models.rs:46`)
- `memories.rs` — Posts memory summarize input and decodes memory summaries. (`codex-api/src/endpoint/memories.rs:36`)
- `realtime_call.rs` — Creates WebRTC realtime calls from SDP and optional session configs, handling backend and multipart wire shapes. (`codex-api/src/endpoint/realtime_call.rs:29`)

## codex-api/src/endpoint/realtime_websocket

- `mod.rs` — Declares realtime protocol and method modules, re-exporting the public realtime client surface. (`codex-api/src/endpoint/realtime_websocket/mod.rs:1`)
- `protocol.rs` — Defines parser selection, session modes/context channels, session config, outbound message schema, and dispatches inbound parsing. (`codex-api/src/endpoint/realtime_websocket/protocol.rs:14`)
- `protocol_common.rs` — Shares realtime payload/session/transcript/error parsing helpers. (`codex-api/src/endpoint/realtime_websocket/protocol_common.rs:7`)
- `protocol_v1.rs` — Parses legacy Realtime V1 events into audio, transcript, handoff, item, and error events. (`codex-api/src/endpoint/realtime_websocket/protocol_v1.rs:12`)
- `protocol_v2.rs` — Parses Realtime V2 events into audio, transcripts, response lifecycle, item, handoff/noop, and error events. (`codex-api/src/endpoint/realtime_websocket/protocol_v2.rs:24`)
- `protocol_frameless_bidi.rs` — Parses Frameless Bidi events into sessions, audio, transcripts, delegations/handoffs, and errors. (`codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs:15`)
- `protocol_frameless_bidi_tests.rs` — Verifies frameless delegation, transcript, and audio events map to existing realtime events. (`codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs:7`)
- `methods.rs` — Implements realtime WebSocket transport/connection, writer, event stream, transcript state, and wire logging. (`codex-api/src/endpoint/realtime_websocket/methods.rs:211`)
- `methods_common.rs` — Chooses per-adapter outbound message shapes and session JSON across V1, V2, and Frameless Bidi. (`codex-api/src/endpoint/realtime_websocket/methods_common.rs:29`)
- `methods_common_tests.rs` — Tests adapter-specific session updates, handoff outputs, context channels, and V1 prefixes. (`codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs:16`)
- `methods_v1.rs` — Builds Realtime V1 session updates with PCM audio and voice, conversation item creates, handoff appends, and the `quicksilver` websocket intent. (`codex-api/src/endpoint/realtime_websocket/methods_v1.rs:18`)
- `methods_v2.rs` — Builds Realtime V2 conversational or transcription session updates, conversation items, function-call outputs, and websocket intent. (`codex-api/src/endpoint/realtime_websocket/methods_v2.rs:39`)
- `methods_frameless_bidi.rs` — Builds Frameless Bidi session updates, session/delegation context appends, and chunked context text. (`codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs:13`)
- `methods_frameless_bidi_tests.rs` — Tests Frameless Bidi context chunking and session JSON, including omission and role-bearing initial items. (`codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs:11`)
