# OpenCode oturum dokumu - playful-otter (ses_f7e41f811ffeB2HLZHAfTiWFyI)

- istek: 15 (200) + 0 hatali
- fresh 44477 / cached 428352 / output 5065 token
- fiyat: $0.0110
- prompt serisi: [12417, 657, 13765, 18165, 21857, 26003, 28585, 30907, 34620, 38828, 42135, 45786, 50379, 52915, 55810]

---

## mesaj

"Read every `.rs` file under `codex-api/src/` and write `WIRE.md`.

For each file, write one line: the path, what it translates or handles, and one `file:line`
citation backing it. Group the lines under the directory each file lives in.

Do not modify any file except `WIRE.md`.

STOP CONDITION. When `WIRE.md` has a line for every `.rs` file under `codex-api/src/`, count the
files you covered and print `CHECKPOINT REACHED: <n> files`. Then stop. Do not review anything
else and do not write any other file."

---

### dusunce

Let me find all .rs files under codex-api/src/.

### arac: glob

```json
{
 "pattern": "codex-api/src/**/*.rs"
}
```

#### cikti

```
C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\telemetry.rs
C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\sse\responses.rs
C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\sse\mod.rs
C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\sse\chat.rs
C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\search.rs
C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\safety_buffering.rs
C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\requests\responses.rs
C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\requests\mod.rs
C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\requests\headers.rs
C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\requests\chat.rs
C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\rate_limits.rs
C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\provider.rs
C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\lib.rs
C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\images.rs
C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\files.rs
C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\error.rs
C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\endpoint\session.rs
C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\endpoint\search.rs
C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\endpoint\responses_websocket.rs
C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\endpoint\responses.rs
C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\endpoint\realtime_websocket\protocol_v2.rs
C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\endpoint\realtime_websocket\protocol_v1.rs
C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\endpoint\realtime_websocket\protocol_frameless_bidi_tests.rs
C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\endpoint\realtime_websocket\protocol_frameless_bidi.rs
C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\endpoint\realtime_websocket\protocol_common.rs
C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\endpoint\realtime_websocket\protocol.rs
C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\endpoint\realtime_websocket\mod.rs
C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\endpoint\realtime_websocket\methods_v2.rs
C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\endpoint\realtime_websocket\methods_v1.rs
C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\endpoint\realtime_websocket\methods_frameless_bidi_tests.rs
C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\endpoint\realtime_websocket\methods_frameless_bidi.rs
C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\endpoint\realtime_websocket\methods_common_tests.rs
C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\endpoint\realtime_websocket\methods_common.rs
C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\endpoint\realtime_websocket\methods.rs
C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\endpoint\realtime_call.rs
C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\endpoint\models.rs
C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\endpoint\mod.rs
C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\endpoint\memories.rs
C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\endpoint\images.rs
C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\endpoint\compact.rs
C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\common.rs
C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\auth.rs
C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\api_bridge_tests.rs
C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\api_bridge.rs
```

---

### dusunce

44 files. I need to read each and write one line per file with a file:line citation. Let me read them. I'll read in parallel batches. Some may be long; I just 
need enough to understand what each handles plus a line citation.

Let me read them in batches.

## mesaj

44 files found. Reading them now.

### arac: read

```json
{
 "filePath": "C:\\Users\\Bedirhan\\Desktop\\agent\\ab\\par-opencode\\codex-api\\src\\lib.rs",
 "limit": 80
}
```

#### cikti

```
<path>C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\lib.rs</path>
<type>file</type>
<content>
1: pub(crate) mod api_bridge;
2: pub(crate) mod auth;
3: pub(crate) mod common;
4: pub(crate) mod endpoint;
5: pub(crate) mod error;
6: pub(crate) mod files;
7: pub(crate) mod images;
8: pub(crate) mod provider;
9: pub(crate) mod rate_limits;
10: pub(crate) mod requests;
11: pub(crate) mod safety_buffering;
12: pub(crate) mod search;
13: pub(crate) mod sse;
14: pub(crate) mod telemetry;
15: 
16: pub use crate::requests::headers::build_session_headers;
17: pub use codex_client::RequestTelemetry;
18: pub use codex_client::ReqwestTransport;
19: pub use codex_client::TransportError;
20: 
21: pub use crate::api_bridge::map_api_error;
22: pub use crate::auth::AgentIdentityTelemetry;
23: pub use crate::auth::AuthError;
24: pub use crate::auth::AuthHeaderTelemetry;
25: pub use crate::auth::AuthHeadersFuture;
26: pub use crate::auth::AuthProvider;
27: pub use crate::auth::AuthProviderFuture;
28: pub use crate::auth::SharedAuthProvider;
29: pub use crate::auth::auth_header_telemetry;
30: pub use crate::common::AccessPrograms;
31: pub use crate::common::CompactionInput;
32: pub use crate::common::MemorySummarizeInput;
33: pub use crate::common::MemorySummarizeOutput;
34: pub use crate::common::OpenAiVerbosity;
35: pub use crate::common::RawMemory;
36: pub use crate::common::RawMemoryMetadata;
37: pub use crate::common::Reasoning;
38: pub use crate::common::ReasoningContext;
39: pub use crate::common::ReasoningSummaryDelivery;
40: pub use crate::common::ResponseCreateWsRequest;
41: pub use crate::common::ResponseEvent;
42: pub use crate::common::ResponseStream;
43: pub use crate::common::ResponsesApiRequest;
44: pub use crate::common::ResponsesApiTools;
45: pub use crate::common::ResponsesWsRequest;
46: pub use crate::common::StreamOptions;
47: pub use crate::common::TextControls;
... [36 satir daha, toplam 3718 karakter]
```

### arac: read

```json
{
 "filePath": "C:\\Users\\Bedirhan\\Desktop\\agent\\ab\\par-opencode\\codex-api\\src\\api_bridge.rs",
 "limit": 120
}
```

#### cikti

```
<path>C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\api_bridge.rs</path>
<type>file</type>
<content>
1: use crate::TransportError;
2: use crate::error::ApiError;
3: use crate::rate_limits::parse_promo_message;
4: use crate::rate_limits::parse_rate_limit_for_limit;
5: use crate::rate_limits::parse_rate_limit_reached_type;
6: use base64::Engine;
7: use chrono::DateTime;
8: use chrono::Utc;
9: use codex_protocol::auth::PlanType;
10: use codex_protocol::error::CodexErr;
11: use codex_protocol::error::CodexErrorDetails;
12: use codex_protocol::error::ConnectionFailedError;
13: use codex_protocol::error::RetryLimitReachedError;
14: use codex_protocol::error::UnexpectedResponseError;
15: use codex_protocol::error::UsageLimitReachedError;
16: use codex_protocol::protocol::MisalignmentErrorDetails;
17: use http::HeaderMap;
18: use serde::Deserialize;
19: use serde_json::Value;
20: 
21: pub fn map_api_error(err: ApiError) -> CodexErr {
22:     match err {
23:         ApiError::ContextWindowExceeded => CodexErr::ContextWindowExceeded,
24:         ApiError::QuotaExceeded => CodexErr::QuotaExceeded,
25:         ApiError::UsageNotIncluded => CodexErr::UsageNotIncluded,
26:         ApiError::Retryable { message, delay } => {
27:             let error = CodexErr::Stream(message);
28:             match delay {
29:                 Some(delay) => error.with_retry_delay(delay),
30:                 None => error,
31:             }
32:         }
33:         ApiError::RateLimitExceeded { message, delay } => {
34:             let error = CodexErr::new(CodexErrorDetails::RateLimitExceeded(message));
35:             match delay {
36:                 Some(delay) => error.with_retry_delay(delay),
37:                 None => error,
38:             }
39:         }
40:         ApiError::Stream(msg) => CodexErr::Stream(msg),
41:         ApiError::ServerOverloaded => CodexErr::ServerOverloaded,
42:         ApiError::Api { status, message } => {
43:             let user_message = api_error_user_message(status, &message);
44:             CodexErr::UnexpectedStatus(UnexpectedResponseError {
45:                 status,
46:                 body: message,
47:                 user_message,
... [76 satir daha, toplam 5862 karakter]
```

### arac: read

```json
{
 "filePath": "C:\\Users\\Bedirhan\\Desktop\\agent\\ab\\par-opencode\\codex-api\\src\\api_bridge_tests.rs",
 "limit": 80
}
```

#### cikti

```
<path>C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\api_bridge_tests.rs</path>
<type>file</type>
<content>
1: use super::*;
2: use base64::Engine;
3: use codex_protocol::protocol::CodexErrorInfo;
4: use codex_protocol::protocol::RateLimitReachedType;
5: use pretty_assertions::assert_eq;
6: 
7: #[test]
8: fn map_api_error_maps_server_overloaded() {
9:     let err = map_api_error(ApiError::ServerOverloaded);
10:     assert!(matches!(err.details(), CodexErrorDetails::ServerOverloaded));
11: }
12: 
13: #[test]
14: fn map_api_error_preserves_retry_delay() {
15:     let retry_delay = std::time::Duration::from_secs(17);
16:     for (error, expected_code, expected_message) in [
17:         (
18:             ApiError::Retryable {
19:                 message: "retry later".to_string(),
20:                 delay: Some(retry_delay),
21:             },
22:             CodexErrorInfo::Other,
23:             "stream disconnected before completion: retry later",
24:         ),
25:         (
26:             ApiError::RateLimitExceeded {
27:                 message: "retry later".to_string(),
28:                 delay: Some(retry_delay),
29:             },
30:             CodexErrorInfo::RateLimitExceeded,
31:             "rate limit exceeded: retry later",
32:         ),
33:     ] {
34:         let err = map_api_error(error);
35:         assert_eq!(
36:             (
37:                 err.to_codex_protocol_error(),
38:                 err.retry_delay(),
39:                 err.is_retryable(),
40:                 err.http_status_code_value(),
41:                 err.to_string(),
42:             ),
43:             (
44:                 expected_code,
45:                 Some(retry_delay),
46:                 true,
47:                 None,
... [36 satir daha, toplam 2999 karakter]
```

### arac: read

```json
{
 "filePath": "C:\\Users\\Bedirhan\\Desktop\\agent\\ab\\par-opencode\\codex-api\\src\\auth.rs",
 "limit": 120
}
```

#### cikti

```
<path>C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\auth.rs</path>
<type>file</type>
<content>
1: use codex_client::Request;
2: use codex_client::TransportError;
3: use http::HeaderMap;
4: use std::future::Future;
5: use std::pin::Pin;
6: use std::sync::Arc;
7: 
8: /// Error returned while applying authentication to an outbound request.
9: #[derive(Debug, thiserror::Error)]
10: pub enum AuthError {
11:     #[error("request auth build error: {0}")]
12:     Build(String),
13:     #[error("transient auth error: {0}")]
14:     Transient(String),
15: }
16: 
17: impl From<AuthError> for TransportError {
18:     fn from(error: AuthError) -> Self {
19:         match error {
20:             AuthError::Build(message) => TransportError::Build(message),
21:             AuthError::Transient(message) => TransportError::Network(message),
22:         }
23:     }
24: }
25: 
26: /// Applies authentication to API requests.
27: ///
28: /// Header-only providers can implement `add_auth_headers`; providers that sign
29: /// complete requests can override `apply_auth`.
30: pub trait AuthProvider: Send + Sync {
31:     /// Adds any auth headers that are available without request body access.
32:     ///
33:     /// Implementations should be cheap and non-blocking. This method is also
34:     /// used by telemetry and non-HTTP request paths.
35:     fn add_auth_headers(&self, headers: &mut HeaderMap);
36: 
37:     /// Returns any auth headers that are available without request body access.
38:     fn to_auth_headers(&self) -> HeaderMap {
39:         let mut headers = HeaderMap::new();
40:         self.add_auth_headers(&mut headers);
41:         headers
42:     }
43: 
44:     /// Resolves auth headers for an outbound request.
45:     ///
46:     /// Unlike [`Self::to_auth_headers`], implementations may perform asynchronous work to refresh
47:     /// credentials before returning. Header-only providers with static credentials can rely on the
... [60 satir daha, toplam 4238 karakter]
```

---

### arac: read

```json
{
 "filePath": "C:\\Users\\Bedirhan\\Desktop\\agent\\ab\\par-opencode\\codex-api\\src\\common.rs",
 "limit": 120
}
```

#### cikti

```
<path>C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\common.rs</path>
<type>file</type>
<content>
1: use crate::error::ApiError;
2: use codex_protocol::ResponseUsageMetadata;
3: use codex_protocol::config_types::ReasoningSummary as ReasoningSummaryConfig;
4: use codex_protocol::config_types::Verbosity as VerbosityConfig;
5: use codex_protocol::models::ResponseItem;
6: use codex_protocol::openai_models::ReasoningEffort as ReasoningEffortConfig;
7: use codex_protocol::protocol::ModelVerification;
8: use codex_protocol::protocol::RateLimitSnapshot;
9: use codex_protocol::protocol::TokenUsage;
10: use codex_protocol::protocol::TurnModerationMetadataEvent;
11: use codex_protocol::protocol::W3cTraceContext;
12: use codex_protocol::turn_input::CyberAccessProgram;
13: use futures::Stream;
14: use serde::Deserialize;
15: use serde::Serialize;
16: use serde_json::Value;
17: use serde_json::value::RawValue;
18: use std::collections::HashMap;
19: use std::pin::Pin;
20: use std::sync::Arc;
21: use std::task::Context;
22: use std::task::Poll;
23: use tokio::sync::mpsc;
24: 
25: pub const WS_REQUEST_HEADER_TRACEPARENT_CLIENT_METADATA_KEY: &str = "ws_request_header_traceparent";
26: pub const WS_REQUEST_HEADER_TRACESTATE_CLIENT_METADATA_KEY: &str = "ws_request_header_tracestate";
27: 
28: /// Explicit per-request access selection using the Responses API wire values.
29: #[derive(Debug, Clone, Copy, PartialEq, Serialize)]
30: pub struct AccessPrograms {
31:     cyber: &'static str,
32: }
33: 
34: impl From<CyberAccessProgram> for AccessPrograms {
35:     fn from(program: CyberAccessProgram) -> Self {
36:         Self {
37:             cyber: match program {
38:                 CyberAccessProgram::Standard => "standard",
39:                 CyberAccessProgram::DaybreakBlue => "daybreak_blue",
40:                 CyberAccessProgram::DaybreakRed => "daybreak_red",
41:             },
42:         }
43:     }
44: }
45: 
46: /// Canonical input payload for the compaction endpoint.
47: #[derive(Debug, Clone, Serialize)]
... [76 satir daha, toplam 5173 karakter]
```

### arac: read

```json
{
 "filePath": "C:\\Users\\Bedirhan\\Desktop\\agent\\ab\\par-opencode\\codex-api\\src\\error.rs",
 "limit": 120
}
```

#### cikti

```
<path>C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\error.rs</path>
<type>file</type>
<content>
1: use crate::rate_limits::RateLimitError;
2: use codex_client::TransportError;
3: use codex_protocol::protocol::MisalignmentErrorDetails;
4: use http::StatusCode;
5: use std::time::Duration;
6: use thiserror::Error;
7: 
8: #[derive(Debug, Error)]
9: pub enum ApiError {
10:     #[error(transparent)]
11:     Transport(#[from] TransportError),
12:     #[error("api error {status}: {message}")]
13:     Api { status: StatusCode, message: String },
14:     #[error("stream error: {0}")]
15:     Stream(String),
16:     #[error("context window exceeded")]
17:     ContextWindowExceeded,
18:     #[error("quota exceeded")]
19:     QuotaExceeded,
20:     #[error("usage not included")]
21:     UsageNotIncluded,
22:     #[error("retryable error: {message}")]
23:     Retryable {
24:         message: String,
25:         delay: Option<Duration>,
26:     },
27:     #[error("rate limit exceeded: {message}")]
28:     RateLimitExceeded {
29:         message: String,
30:         delay: Option<Duration>,
31:     },
32:     #[error("rate limit: {0}")]
33:     RateLimit(String),
34:     #[error("invalid request: {message}")]
35:     InvalidRequest { message: String },
36:     #[error("cyber policy: {message}")]
37:     CyberPolicy { message: String },
38:     #[error("misalignment policy violation: {message}")]
39:     MisalignmentPolicyViolation {
40:         message: String,
41:         misalignment: Option<MisalignmentErrorDetails>,
42:     },
43:     #[error("server overloaded")]
44:     ServerOverloaded,
45: }
46: 
47: impl From<RateLimitError> for ApiError {
... [7 satir daha, toplam 1827 karakter]
```

### arac: read

```json
{
 "filePath": "C:\\Users\\Bedirhan\\Desktop\\agent\\ab\\par-opencode\\codex-api\\src\\files.rs",
 "limit": 100
}
```

#### cikti

```
<path>C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\files.rs</path>
<type>file</type>
<content>
1: use std::time::Duration;
2: 
3: use crate::AuthProvider;
4: use bytes::Bytes;
5: use codex_http_client::HttpResponse;
6: use codex_http_client::RouteAwareClientPool;
7: use codex_http_client::RouteAwareRequestBuilder;
8: use codex_http_client::RouteAwareRequestError;
9: use futures::Stream;
10: use http::Method;
11: use http::StatusCode;
12: use http::header::CONTENT_LENGTH;
13: use serde::Deserialize;
14: use tokio::time::Instant;
15: use uuid::Uuid;
16: 
17: pub const OPENAI_FILE_URI_PREFIX: &str = "sediment://";
18: pub const OPENAI_FILE_UPLOAD_LIMIT_BYTES: u64 = 512 * 1024 * 1024;
19: 
20: const OPENAI_FILE_REQUEST_TIMEOUT: Duration = Duration::from_secs(60);
21: const OPENAI_FILE_FINALIZE_TIMEOUT: Duration = Duration::from_secs(30);
22: const OPENAI_FILE_FINALIZE_RETRY_DELAY: Duration = Duration::from_millis(250);
23: const OPENAI_FILE_USE_CASE: &str = "codex";
24: 
25: #[derive(Debug)]
26: pub struct HostedFileUploadContext {
27:     pub connector_id: String,
28:     pub action_name: String,
29:     pub model: String,
30: }
31: 
32: #[derive(Debug, Clone, PartialEq, Eq)]
33: pub struct UploadedOpenAiFile {
34:     pub file_id: String,
35:     pub uri: String,
36:     pub download_url: String,
37:     pub file_name: String,
38:     pub file_size_bytes: u64,
39:     pub mime_type: Option<String>,
40: }
41: 
42: #[derive(Debug, thiserror::Error)]
43: pub enum OpenAiFileError {
44:     #[error(
45:         "file `{file_name}` is too large: {size_bytes} bytes exceeds the limit of {limit_bytes} bytes"
46:     )]
47:     FileTooLarge {
... [56 satir daha, toplam 3682 karakter]
```

### arac: read

```json
{
 "filePath": "C:\\Users\\Bedirhan\\Desktop\\agent\\ab\\par-opencode\\codex-api\\src\\images.rs",
 "limit": 100
}
```

#### cikti

```
<path>C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\images.rs</path>
<type>file</type>
<content>
1: use serde::Deserialize;
2: use serde::Serialize;
3: 
4: #[derive(Debug, Clone, Serialize, PartialEq)]
5: pub struct ImageGenerationRequest {
6:     pub prompt: String,
7:     #[serde(skip_serializing_if = "Option::is_none")]
8:     pub background: Option<ImageBackground>,
9:     pub model: String,
10:     #[serde(skip_serializing_if = "Option::is_none")]
11:     pub n: Option<u64>,
12:     #[serde(skip_serializing_if = "Option::is_none")]
13:     pub quality: Option<ImageQuality>,
14:     #[serde(skip_serializing_if = "Option::is_none")]
15:     pub size: Option<String>,
16: }
17: 
18: #[derive(Debug, Clone, Serialize, PartialEq)]
19: pub struct ImageEditRequest {
20:     pub images: Vec<ImageUrl>,
21:     pub prompt: String,
22:     #[serde(skip_serializing_if = "Option::is_none")]
23:     pub background: Option<ImageBackground>,
24:     pub model: String,
25:     #[serde(skip_serializing_if = "Option::is_none")]
26:     pub n: Option<u64>,
27:     #[serde(skip_serializing_if = "Option::is_none")]
28:     pub quality: Option<ImageQuality>,
29:     #[serde(skip_serializing_if = "Option::is_none")]
30:     pub size: Option<String>,
31: }
32: 
33: #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
34: pub struct ImageUrl {
35:     pub image_url: String,
36: }
37: 
38: #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
39: #[serde(rename_all = "lowercase")]
40: pub enum ImageBackground {
41:     Transparent,
42:     Opaque,
43:     Auto,
44: }
45: 
46: #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
47: #[serde(rename_all = "lowercase")]
... [26 satir daha, toplam 2347 karakter]
```

---

### arac: read

```json
{
 "filePath": "C:\\Users\\Bedirhan\\Desktop\\agent\\ab\\par-opencode\\codex-api\\src\\provider.rs",
 "limit": 120
}
```

#### cikti

```
<path>C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\provider.rs</path>
<type>file</type>
<content>
1: use codex_client::Request;
2: use codex_client::RequestCompression;
3: use codex_client::RetryOn;
4: use codex_client::RetryPolicy;
5: use http::Method;
6: use http::header::HeaderMap;
7: use std::collections::HashMap;
8: use std::time::Duration;
9: use url::Url;
10: 
11: /// High-level retry configuration for a provider.
12: ///
13: /// This is converted into a `RetryPolicy` used by `codex-client` to drive
14: /// transport-level retries for both unary and streaming calls.
15: #[derive(Debug, Clone)]
16: pub struct RetryConfig {
17:     pub max_attempts: u64,
18:     pub base_delay: Duration,
19:     pub retry_429: bool,
20:     pub retry_5xx: bool,
21:     pub retry_transport: bool,
22: }
23: 
24: impl RetryConfig {
25:     pub fn to_policy(&self) -> RetryPolicy {
26:         RetryPolicy {
27:             max_attempts: self.max_attempts,
28:             base_delay: self.base_delay,
29:             retry_on: RetryOn {
30:                 retry_429: self.retry_429,
31:                 retry_5xx: self.retry_5xx,
32:                 retry_transport: self.retry_transport,
33:             },
34:         }
35:     }
36: }
37: 
38: /// Wire protocol a provider speaks on its inference route.
39: ///
40: /// Suffice is Responses-native. `Chat` exists for providers that only expose an
41: /// OpenAI-compatible `/chat/completions` endpoint (Z.ai/GLM, most OSS servers);
42: /// requests and SSE frames are translated at the `ResponsesClient` boundary so
43: /// the rest of the codebase never sees the difference.
44: #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
45: pub enum WireApi {
46:     #[default]
47:     Responses,
... [76 satir daha, toplam 4265 karakter]
```

### arac: read

```json
{
 "filePath": "C:\\Users\\Bedirhan\\Desktop\\agent\\ab\\par-opencode\\codex-api\\src\\rate_limits.rs",
 "limit": 120
}
```

#### cikti

```
<path>C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\rate_limits.rs</path>
<type>file</type>
<content>
1: use codex_protocol::account::PlanType;
2: use codex_protocol::protocol::CreditsSnapshot;
3: use codex_protocol::protocol::RateLimitReachedType;
4: use codex_protocol::protocol::RateLimitSnapshot;
5: use codex_protocol::protocol::RateLimitWindow;
6: use http::HeaderMap;
7: use serde::Deserialize;
8: use std::collections::BTreeSet;
9: use std::fmt::Display;
10: 
11: #[derive(Debug)]
12: pub struct RateLimitError {
13:     pub message: String,
14: }
15: 
16: impl Display for RateLimitError {
17:     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
18:         write!(f, "{}", self.message)
19:     }
20: }
21: 
22: /// Parses the default Suffice rate-limit header family into a `RateLimitSnapshot`.
23: pub fn parse_default_rate_limit(headers: &HeaderMap) -> Option<RateLimitSnapshot> {
24:     parse_rate_limit_for_limit(headers, /*limit_id*/ None)
25: }
26: 
27: /// Parses all known rate-limit header families into update records keyed by limit id.
28: pub fn parse_all_rate_limits(headers: &HeaderMap) -> Vec<RateLimitSnapshot> {
29:     let mut snapshots = Vec::new();
30:     if let Some(snapshot) = parse_default_rate_limit(headers) {
31:         snapshots.push(snapshot);
32:     }
33: 
34:     let mut limit_ids: BTreeSet<String> = BTreeSet::new();
35: 
36:     for name in headers.keys() {
37:         let header_name = name.as_str().to_ascii_lowercase();
38:         if let Some(limit_id) = header_name_to_limit_id(&header_name)
39:             && limit_id != "codex"
40:         {
41:             limit_ids.insert(limit_id);
42:         }
43:     }
44: 
45:     snapshots.extend(limit_ids.into_iter().filter_map(|limit_id| {
46:         let snapshot = parse_rate_limit_for_limit(headers, Some(limit_id.as_str()))?;
47:         has_rate_limit_data(&snapshot).then_some(snapshot)
... [76 satir daha, toplam 4485 karakter]
```

### arac: read

```json
{
 "filePath": "C:\\Users\\Bedirhan\\Desktop\\agent\\ab\\par-opencode\\codex-api\\src\\safety_buffering.rs",
 "limit": 100
}
```

#### cikti

```
<path>C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\safety_buffering.rs</path>
<type>file</type>
<content>
1: use crate::common::SafetyBufferingTreatment;
2: use http::HeaderMap;
3: 
4: pub(crate) const X_CODEX_SAFETY_BUFFERING_ENABLED_HEADER: &str = "x-codex-safety-buffering-enabled";
5: pub(crate) const X_CODEX_SAFETY_BUFFERING_FASTER_MODEL_HEADER: &str =
6:     "x-codex-safety-buffering-faster-model";
7: 
8: pub(crate) fn treatment_from_headers(headers: &HeaderMap) -> Option<SafetyBufferingTreatment> {
9:     if !headers.contains_key(X_CODEX_SAFETY_BUFFERING_ENABLED_HEADER)
10:         && !headers.contains_key(X_CODEX_SAFETY_BUFFERING_FASTER_MODEL_HEADER)
11:     {
12:         return None;
13:     }
14:     let faster_model = headers
15:         .get(X_CODEX_SAFETY_BUFFERING_FASTER_MODEL_HEADER)
16:         .and_then(|value| value.to_str().ok())
17:         .map(str::to_string);
18: 
19:     Some(SafetyBufferingTreatment { faster_model })
20: }
21: 
22: #[cfg(test)]
23: mod tests {
24:     use super::*;
25:     use http::HeaderValue;
26:     use pretty_assertions::assert_eq;
27: 
28:     #[test]
29:     fn reads_treatment_from_http_headers() {
30:         let mut headers = HeaderMap::new();
31:         headers.insert(
32:             X_CODEX_SAFETY_BUFFERING_ENABLED_HEADER,
33:             HeaderValue::from_static("true"),
34:         );
35:         headers.insert(
36:             X_CODEX_SAFETY_BUFFERING_FASTER_MODEL_HEADER,
37:             HeaderValue::from_static("faster-model"),
38:         );
39: 
40:         assert_eq!(
41:             treatment_from_headers(&headers),
42:             Some(SafetyBufferingTreatment {
43:                 faster_model: Some("faster-model".to_string()),
44:             })
45:         );
46:     }
47: 
... [23 satir daha, toplam 2531 karakter]
```

### arac: read

```json
{
 "filePath": "C:\\Users\\Bedirhan\\Desktop\\agent\\ab\\par-opencode\\codex-api\\src\\search.rs",
 "limit": 80
}
```

#### cikti

```
<path>C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\search.rs</path>
<type>file</type>
<content>
1: use crate::common::Reasoning;
2: use codex_protocol::models::ResponseItem;
3: use schemars::JsonSchema;
4: use serde::Deserialize;
5: use serde::Serialize;
6: use serde_json::Value as JsonValue;
7: 
8: #[derive(Debug, Clone, Serialize, PartialEq)]
9: pub struct SearchRequest {
10:     pub id: String,
11:     pub model: String,
12:     #[serde(skip_serializing_if = "Option::is_none")]
13:     pub reasoning: Option<Reasoning>,
14:     #[serde(skip_serializing_if = "Option::is_none")]
15:     pub input: Option<SearchInput>,
16:     #[serde(skip_serializing_if = "Option::is_none")]
17:     pub commands: Option<SearchCommands>,
18:     #[serde(skip_serializing_if = "Option::is_none")]
19:     pub settings: Option<SearchSettings>,
20:     #[serde(skip_serializing_if = "Option::is_none")]
21:     pub max_output_tokens: Option<u64>,
22: }
23: 
24: #[derive(Debug, Clone, Serialize, PartialEq)]
25: #[serde(untagged)]
26: pub enum SearchInput {
27:     Text(String),
28:     Items(Vec<ResponseItem>),
29: }
30: 
31: #[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, JsonSchema)]
32: pub struct SearchCommands {
33:     /// Query the internet search engine for a given list of queries.
34:     #[serde(skip_serializing_if = "Option::is_none")]
35:     pub search_query: Option<Vec<SearchQuery>>,
36:     /// Query the image search engine for a given list of queries.
37:     #[serde(skip_serializing_if = "Option::is_none")]
38:     pub image_query: Option<Vec<SearchQuery>>,
39:     /// Open pages by reference id or URL.
40:     #[serde(skip_serializing_if = "Option::is_none")]
41:     pub open: Option<Vec<OpenOperation>>,
42:     /// Open links from previously opened pages.
43:     #[serde(skip_serializing_if = "Option::is_none")]
44:     pub click: Option<Vec<ClickOperation>>,
45:     /// Find text patterns in pages.
46:     #[serde(skip_serializing_if = "Option::is_none")]
47:     pub find: Option<Vec<FindOperation>>,
... [36 satir daha, toplam 3668 karakter]
```

---

### arac: read

```json
{
 "filePath": "C:\\Users\\Bedirhan\\Desktop\\agent\\ab\\par-opencode\\codex-api\\src\\telemetry.rs",
 "limit": 80
}
```

#### cikti

```
<path>C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\telemetry.rs</path>
<type>file</type>
<content>
1: use crate::error::ApiError;
2: use codex_client::Request;
3: use codex_client::RequestTelemetry;
4: use codex_client::Response;
5: use codex_client::RetryPolicy;
6: use codex_client::StreamResponse;
7: use codex_client::TransportError;
8: use codex_client::run_with_retry;
9: use http::StatusCode;
10: use std::future::Future;
11: use std::sync::Arc;
12: use std::time::Duration;
13: use tokio::time::Instant;
14: use tokio_tungstenite::tungstenite::Error;
15: use tokio_tungstenite::tungstenite::Message;
16: 
17: /// Generic telemetry.
18: pub trait SseTelemetry: Send + Sync {
19:     fn on_sse_poll(
20:         &self,
21:         result: &Result<
22:             Option<
23:                 Result<
24:                     eventsource_stream::Event,
25:                     eventsource_stream::EventStreamError<TransportError>,
26:                 >,
27:             >,
28:             tokio::time::error::Elapsed,
29:         >,
30:         duration: Duration,
31:     );
32: }
33: 
34: /// Telemetry for Responses WebSocket transport.
35: pub trait WebsocketTelemetry: Send + Sync {
36:     fn on_ws_request(&self, duration: Duration, error: Option<&ApiError>, connection_reused: bool);
37: 
38:     fn on_ws_event(
39:         &self,
40:         result: &Result<Option<Result<Message, Error>>, ApiError>,
41:         duration: Duration,
42:     );
43: }
44: 
45: pub(crate) trait WithStatus {
46:     fn status(&self) -> StatusCode;
47: }
... [36 satir daha, toplam 2586 karakter]
```

### arac: read

```json
{
 "filePath": "C:\\Users\\Bedirhan\\Desktop\\agent\\ab\\par-opencode\\codex-api\\src\\requests\\mod.rs",
 "limit": 60
}
```

#### cikti

```
<path>C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\requests\mod.rs</path>
<type>file</type>
<content>
1: pub(crate) mod chat;
2: pub(crate) mod headers;
3: pub(crate) mod responses;
4: 
5: pub use responses::Compression;

(End of file - total 5 lines)
</content>
```

### arac: read

```json
{
 "filePath": "C:\\Users\\Bedirhan\\Desktop\\agent\\ab\\par-opencode\\codex-api\\src\\requests\\headers.rs",
 "limit": 120
}
```

#### cikti

```
<path>C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\requests\headers.rs</path>
<type>file</type>
<content>
1: use codex_protocol::protocol::SessionSource;
2: use http::HeaderMap;
3: use http::HeaderValue;
4: 
5: pub fn build_session_headers(session_id: Option<String>, thread_id: Option<String>) -> HeaderMap {
6:     let mut headers = HeaderMap::new();
7:     if let Some(id) = session_id {
8:         insert_header(&mut headers, "session-id", &id);
9:     }
10:     if let Some(id) = thread_id {
11:         insert_header(&mut headers, "thread-id", &id);
12:     }
13:     headers
14: }
15: 
16: pub(crate) fn subagent_header(source: &Option<SessionSource>) -> Option<String> {
17:     let SessionSource::SubAgent(sub) = source.as_ref()? else {
18:         return None;
19:     };
20:     match sub {
21:         codex_protocol::protocol::SubAgentSource::Review => Some("review".to_string()),
22:         codex_protocol::protocol::SubAgentSource::Compact => Some("compact".to_string()),
23:         codex_protocol::protocol::SubAgentSource::MemoryConsolidation => {
24:             Some("memory_consolidation".to_string())
25:         }
26:         codex_protocol::protocol::SubAgentSource::ThreadSpawn { .. } => {
27:             Some("collab_spawn".to_string())
28:         }
29:         codex_protocol::protocol::SubAgentSource::Other(label) => Some(label.clone()),
30:     }
31: }
32: 
33: pub(crate) fn insert_header(headers: &mut HeaderMap, name: &str, value: &str) {
34:     if let (Ok(header_name), Ok(header_value)) = (
35:         name.parse::<http::HeaderName>(),
36:         HeaderValue::from_str(value),
37:     ) {
38:         headers.insert(header_name, header_value);
39:     }
40: }

(End of file - total 40 lines)
</content>
```

### arac: read

```json
{
 "filePath": "C:\\Users\\Bedirhan\\Desktop\\agent\\ab\\par-opencode\\codex-api\\src\\requests\\chat.rs",
 "limit": 100
}
```

#### cikti

```
<path>C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\requests\chat.rs</path>
<type>file</type>
<content>
1: //! Translation from a Responses-shaped request into an OpenAI-compatible
2: //! `/chat/completions` body.
3: //!
4: //! Suffice speaks the Responses API everywhere above this module. Providers that
5: //! only expose Chat Completions (Z.ai/GLM, most OSS servers) get their request
6: //! rewritten here, and their SSE frames rewritten back in `crate::sse::chat`.
7: 
8: use crate::common::ResponsesApiRequest;
9: use codex_protocol::models::ContentItem;
10: use codex_protocol::models::FunctionCallOutputContentItem;
11: use codex_protocol::models::ReasoningItemContent;
12: use codex_protocol::models::ResponseItem;
13: use codex_protocol::openai_models::ReasoningEffort;
14: use serde_json::Value;
15: use serde_json::json;
16: use std::collections::HashMap;
17: 
18: /// Builds the `/chat/completions` request body for `request`.
19: pub(crate) fn chat_body_from_responses_request(request: &ResponsesApiRequest) -> Value {
20:     let mut messages = Vec::<Value>::new();
21:     if !request.instructions.is_empty() {
22:         messages.push(json!({"role": "system", "content": request.instructions}));
23:     }
24: 
25:     let input = request.input.as_slice();
26:     let reasoning_by_anchor_index = anchor_reasoning(input);
27:     let mut last_assistant_text: Option<String> = None;
28: 
29:     for (idx, item) in input.iter().enumerate() {
30:         match item {
31:             ResponseItem::Message { role, content, .. } => {
32:                 let mut text = String::new();
33:                 let mut parts: Vec<Value> = Vec::new();
34:                 let mut saw_image = false;
35: 
36:                 for c in content {
37:                     match c {
38:                         ContentItem::InputText { text: t } | ContentItem::OutputText { text: t } => {
39:                             text.push_str(t);
40:                             parts.push(json!({"type": "text", "text": t}));
41:                         }
42:                         ContentItem::InputImage { image_url, .. } => {
43:                             saw_image = true;
44:                             parts.push(json!({"type": "image_url", "image_url": {"url": image_url}}));
45:                         }
46:                         // Audio has no Chat Completions equivalent.
47:                         _ => {}
... [56 satir daha, toplam 4858 karakter]
```

---

### arac: read

```json
{
 "filePath": "C:\\Users\\Bedirhan\\Desktop\\agent\\ab\\par-opencode\\codex-api\\src\\requests\\responses.rs",
 "limit": 120
}
```

#### cikti

```
<path>C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\requests\responses.rs</path>
<type>file</type>
<content>
1: #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
2: pub enum Compression {
3:     #[default]
4:     None,
5:     Zstd,
6: }

(End of file - total 6 lines)
</content>
```

### arac: read

```json
{
 "filePath": "C:\\Users\\Bedirhan\\Desktop\\agent\\ab\\par-opencode\\codex-api\\src\\sse\\mod.rs",
 "limit": 60
}
```

#### cikti

```
<path>C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\sse\mod.rs</path>
<type>file</type>
<content>
1: pub(crate) mod chat;
2: pub(crate) mod responses;
3: 
4: pub(crate) use chat::spawn_chat_stream;
5: pub(crate) use responses::ResponsesStreamEvent;
6: pub(crate) use responses::process_responses_event;
7: pub use responses::spawn_response_stream;

(End of file - total 7 lines)
</content>
```

### arac: read

```json
{
 "filePath": "C:\\Users\\Bedirhan\\Desktop\\agent\\ab\\par-opencode\\codex-api\\src\\sse\\chat.rs",
 "limit": 80
}
```

#### cikti

```
<path>C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\sse\chat.rs</path>
<type>file</type>
<content>
1: //! Translation of OpenAI-compatible `/chat/completions` SSE frames into the
2: //! Responses-shaped [`ResponseEvent`] stream the rest of Suffice consumes.
3: //!
4: //! See `crate::requests::chat` for the outbound half of the bridge.
5: 
6: use crate::common::ResponseEvent;
7: use crate::common::ResponseStream;
8: use crate::error::ApiError;
9: use crate::telemetry::SseTelemetry;
10: use codex_client::StreamResponse;
11: use codex_protocol::models::ContentItem;
12: use codex_protocol::models::ReasoningItemContent;
13: use codex_protocol::models::ResponseItem;
14: use codex_protocol::protocol::TokenUsage;
15: use eventsource_stream::Eventsource;
16: use futures::Stream;
17: use futures::StreamExt;
18: use serde_json::Value;
19: use std::collections::HashMap;
20: use std::sync::Arc;
21: use std::time::Duration;
22: use tokio::sync::mpsc;
23: use tokio::time::Instant;
24: use tokio::time::timeout;
25: use tracing::debug;
26: use tracing::trace;
27: 
28: const REQUEST_ID_HEADER: &str = "x-request-id";
29: 
30: pub(crate) fn spawn_chat_stream(
31:     stream_response: StreamResponse,
32:     idle_timeout: Duration,
33:     telemetry: Option<Arc<dyn SseTelemetry>>,
34: ) -> ResponseStream {
35:     let upstream_request_id = stream_response
36:         .headers
37:         .get(REQUEST_ID_HEADER)
38:         .and_then(|value| value.to_str().ok())
39:         .map(str::to_string);
40: 
41:     let (tx_event, rx_event) = mpsc::channel::<Result<ResponseEvent, ApiError>>(1600);
42:     tokio::spawn(async move {
43:         process_chat_sse(stream_response.bytes, tx_event, idle_timeout, telemetry).await;
44:     });
45: 
46:     ResponseStream {
47:         rx_event,
... [36 satir daha, toplam 2951 karakter]
```

### arac: read

```json
{
 "filePath": "C:\\Users\\Bedirhan\\Desktop\\agent\\ab\\par-opencode\\codex-api\\src\\sse\\responses.rs",
 "limit": 100
}
```

#### cikti

```
<path>C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\sse\responses.rs</path>
<type>file</type>
<content>
1: use crate::common::ResponseEvent;
2: use crate::common::ResponseStream;
3: use crate::common::SafetyBuffering;
4: use crate::common::SafetyBufferingTreatment;
5: use crate::error::ApiError;
6: use crate::rate_limits::parse_all_rate_limits;
7: use crate::safety_buffering::treatment_from_headers;
8: use crate::telemetry::SseTelemetry;
9: use codex_client::ByteStream;
10: use codex_client::StreamResponse;
11: use codex_protocol::ResponseUsageMetadata;
12: use codex_protocol::models::ResponseItem;
13: use codex_protocol::protocol::MisalignmentErrorDetails;
14: use codex_protocol::protocol::ModelVerification;
15: use codex_protocol::protocol::TokenUsage;
16: use codex_protocol::protocol::TurnModerationMetadataEvent;
17: use eventsource_stream::Eventsource;
18: use futures::StreamExt;
19: use serde::Deserialize;
20: use serde_json::Value;
21: use std::sync::Arc;
22: use std::sync::OnceLock;
23: use std::time::Duration;
24: use tokio::sync::mpsc;
25: use tokio::time::Instant;
26: use tokio::time::timeout;
27: use tracing::debug;
28: use tracing::trace;
29: 
30: const X_REASONING_INCLUDED_HEADER: &str = "x-reasoning-included";
31: const X_CODEX_TURN_STATE_HEADER: &str = "x-codex-turn-state";
32: const OPENAI_MODEL_HEADER: &str = "openai-model";
33: const REQUEST_ID_HEADER: &str = "x-request-id";
34: const TRUSTED_ACCESS_FOR_CYBER_VERIFICATION: &str = "trusted_access_for_cyber";
35: 
36: pub fn spawn_response_stream(
37:     stream_response: StreamResponse,
38:     idle_timeout: Duration,
39:     telemetry: Option<Arc<dyn SseTelemetry>>,
40:     turn_state: Option<Arc<OnceLock<String>>>,
41: ) -> ResponseStream {
42:     let rate_limit_snapshots = parse_all_rate_limits(&stream_response.headers);
43:     let models_etag = stream_response
44:         .headers
45:         .get("X-Models-Etag")
46:         .and_then(|v| v.to_str().ok())
47:         .map(ToString::to_string);
... [56 satir daha, toplam 4136 karakter]
```

---

### arac: read

```json
{
 "filePath": "C:\\Users\\Bedirhan\\Desktop\\agent\\ab\\par-opencode\\codex-api\\src\\endpoint\\mod.rs",
 "limit": 80
}
```

#### cikti

```
<path>C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\endpoint\mod.rs</path>
<type>file</type>
<content>
1: pub(crate) mod compact;
2: pub(crate) mod images;
3: pub(crate) mod memories;
4: pub(crate) mod models;
5: pub(crate) mod realtime_call;
6: pub(crate) mod realtime_websocket;
7: pub(crate) mod responses;
8: pub(crate) mod responses_websocket;
9: pub(crate) mod search;
10: mod session;
11: 
12: pub use compact::CompactClient;
13: pub use images::ImagesClient;
14: pub use memories::MemoriesClient;
15: pub use models::ModelsClient;
16: pub use realtime_call::RealtimeCallClient;
17: pub use realtime_call::RealtimeCallResponse;
18: pub use realtime_websocket::RealtimeContextAppendChannel;
19: pub use realtime_websocket::RealtimeEventParser;
20: pub use realtime_websocket::RealtimeOutputModality;
21: pub use realtime_websocket::RealtimeSessionConfig;
22: pub use realtime_websocket::RealtimeSessionMode;
23: pub use realtime_websocket::RealtimeTranscriptState;
24: pub use realtime_websocket::RealtimeWebsocketClient;
25: pub use realtime_websocket::RealtimeWebsocketConnection;
26: pub use realtime_websocket::RealtimeWebsocketEvents;
27: pub use realtime_websocket::RealtimeWebsocketWriter;
28: pub use realtime_websocket::session_update_session_json;
29: pub use responses::ResponsesClient;
30: pub use responses::ResponsesEndpoint;
31: pub use responses::ResponsesOptions;
32: pub use responses_websocket::ResponsesWebsocketClient;
33: pub use responses_websocket::ResponsesWebsocketClose;
34: pub use responses_websocket::ResponsesWebsocketConnection;
35: pub use responses_websocket::ResponsesWebsocketProbe;
36: pub use search::SearchClient;

(End of file - total 36 lines)
</content>
```

### arac: read

```json
{
 "filePath": "C:\\Users\\Bedirhan\\Desktop\\agent\\ab\\par-opencode\\codex-api\\src\\endpoint\\responses.rs",
 "limit": 120
}
```

#### cikti

```
<path>C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\endpoint\responses.rs</path>
<type>file</type>
<content>
1: use crate::auth::SharedAuthProvider;
2: use crate::common::ResponseStream;
3: use crate::common::ResponsesApiRequest;
4: use crate::endpoint::session::EndpointSession;
5: use crate::error::ApiError;
6: use crate::provider::Provider;
7: use crate::provider::WireApi;
8: use crate::requests::Compression;
9: use crate::requests::chat::chat_body_from_responses_request;
10: use crate::requests::headers::build_session_headers;
11: use crate::requests::headers::insert_header;
12: use crate::requests::headers::subagent_header;
13: use crate::sse::spawn_chat_stream;
14: use crate::sse::spawn_response_stream;
15: use crate::telemetry::SseTelemetry;
16: use codex_client::EncodedJsonBody;
17: use codex_client::HttpTransport;
18: use codex_client::RequestCompression;
19: use codex_client::RequestTelemetry;
20: use codex_protocol::protocol::SessionSource;
21: use http::HeaderMap;
22: use http::HeaderValue;
23: use http::Method;
24: use serde_json::Value;
25: use std::sync::Arc;
26: use std::sync::OnceLock;
27: use tracing::instrument;
28: 
29: /// Responses-compatible inference routes supported by Suffice backend.
30: #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
31: pub enum ResponsesEndpoint {
32:     /// Regular user-owned model inference.
33:     #[default]
34:     Responses,
35:     /// Full Guardian approval-review agent inference.
36:     Guardian,
37:     /// Lightweight asynchronous Guardian risk classification.
38:     GuardianClassifier,
39: }
40: 
41: impl ResponsesEndpoint {
42:     /// Returns the provider-relative path for this inference surface.
43:     pub const fn path(self) -> &'static str {
44:         match self {
45:             Self::Responses => "/responses",
46:             Self::Guardian => "/guardian",
47:             Self::GuardianClassifier => "/guardian-classifier",
... [76 satir daha, toplam 4559 karakter]
```

### arac: read

```json
{
 "filePath": "C:\\Users\\Bedirhan\\Desktop\\agent\\ab\\par-opencode\\codex-api\\src\\endpoint\\compact.rs",
 "limit": 100
}
```

#### cikti

```
<path>C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\endpoint\compact.rs</path>
<type>file</type>
<content>
1: use crate::auth::SharedAuthProvider;
2: use crate::common::CompactionInput;
3: use crate::endpoint::session::EndpointSession;
4: use crate::error::ApiError;
5: use crate::provider::Provider;
6: use codex_client::HttpTransport;
7: use codex_client::RequestTelemetry;
8: use codex_protocol::models::ResponseItem;
9: use http::HeaderMap;
10: use http::Method;
11: use serde::Deserialize;
12: use std::sync::Arc;
13: use std::sync::OnceLock;
14: use std::time::Duration;
15: 
16: const X_CODEX_TURN_STATE_HEADER: &str = "x-codex-turn-state";
17: 
18: pub struct CompactClient<T: HttpTransport> {
19:     session: EndpointSession<T>,
20: }
21: 
22: impl<T: HttpTransport> CompactClient<T> {
23:     pub fn new(transport: T, provider: Provider, auth: SharedAuthProvider) -> Self {
24:         Self {
25:             session: EndpointSession::new(transport, provider, auth),
26:         }
27:     }
28: 
29:     pub fn with_telemetry(self, request: Option<Arc<dyn RequestTelemetry>>) -> Self {
30:         Self {
31:             session: self.session.with_request_telemetry(request),
32:         }
33:     }
34: 
35:     fn path() -> &'static str {
36:         "responses/compact"
37:     }
38: 
39:     pub async fn compact(
40:         &self,
41:         body: serde_json::Value,
42:         extra_headers: HeaderMap,
43:         request_timeout: Duration,
44:         turn_state: Option<&OnceLock<String>>,
45:     ) -> Result<Vec<ResponseItem>, ApiError> {
46:         let resp = self
47:             .session
... [56 satir daha, toplam 3494 karakter]
```

### arac: read

```json
{
 "filePath": "C:\\Users\\Bedirhan\\Desktop\\agent\\ab\\par-opencode\\codex-api\\src\\endpoint\\images.rs",
 "limit": 100
}
```

#### cikti

```
<path>C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\endpoint\images.rs</path>
<type>file</type>
<content>
1: use crate::auth::SharedAuthProvider;
2: use crate::endpoint::session::EndpointSession;
3: use crate::error::ApiError;
4: use crate::images::ImageEditRequest;
5: use crate::images::ImageGenerationRequest;
6: use crate::images::ImageResponse;
7: use crate::provider::Provider;
8: use codex_client::HttpTransport;
9: use codex_client::RequestTelemetry;
10: use http::HeaderMap;
11: use http::Method;
12: use serde::Serialize;
13: use serde_json::to_value;
14: use std::sync::Arc;
15: 
16: const X_CODEX_IMAGEGEN_REQUEST_ID_HEADER: &str = "x-codex-imagegen-request-id";
17: 
18: pub struct ImagesClient<T: HttpTransport> {
19:     session: EndpointSession<T>,
20: }
21: 
22: impl<T: HttpTransport> ImagesClient<T> {
23:     pub fn new(transport: T, provider: Provider, auth: SharedAuthProvider) -> Self {
24:         Self {
25:             session: EndpointSession::new(transport, provider, auth),
26:         }
27:     }
28: 
29:     pub fn with_telemetry(self, request: Option<Arc<dyn RequestTelemetry>>) -> Self {
30:         Self {
31:             session: self.session.with_request_telemetry(request),
32:         }
33:     }
34: 
35:     pub async fn generate(
36:         &self,
37:         request: &ImageGenerationRequest,
38:         extra_headers: HeaderMap,
39:     ) -> Result<(ImageResponse, Option<String>), ApiError> {
40:         self.post_image_request(
41:             "images/generations",
42:             request,
43:             extra_headers,
44:             "image generation",
45:         )
46:         .await
47:     }
... [56 satir daha, toplam 3739 karakter]
```

---

### arac: read

```json
{
 "filePath": "C:\\Users\\Bedirhan\\Desktop\\agent\\ab\\par-opencode\\codex-api\\src\\endpoint\\memories.rs",
 "limit": 110
}
```

#### cikti

```
<path>C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\endpoint\memories.rs</path>
<type>file</type>
<content>
1: use crate::auth::SharedAuthProvider;
2: use crate::common::MemorySummarizeInput;
3: use crate::common::MemorySummarizeOutput;
4: use crate::endpoint::session::EndpointSession;
5: use crate::error::ApiError;
6: use crate::provider::Provider;
7: use codex_client::HttpTransport;
8: use codex_client::RequestTelemetry;
9: use http::HeaderMap;
10: use http::Method;
11: use serde::Deserialize;
12: use serde_json::to_value;
13: use std::sync::Arc;
14: 
15: pub struct MemoriesClient<T: HttpTransport> {
16:     session: EndpointSession<T>,
17: }
18: 
19: impl<T: HttpTransport> MemoriesClient<T> {
20:     pub fn new(transport: T, provider: Provider, auth: SharedAuthProvider) -> Self {
21:         Self {
22:             session: EndpointSession::new(transport, provider, auth),
23:         }
24:     }
25: 
26:     pub fn with_telemetry(self, request: Option<Arc<dyn RequestTelemetry>>) -> Self {
27:         Self {
28:             session: self.session.with_request_telemetry(request),
29:         }
30:     }
31: 
32:     fn path() -> &'static str {
33:         "memories/trace_summarize"
34:     }
35: 
36:     pub async fn summarize(
37:         &self,
38:         body: serde_json::Value,
39:         extra_headers: HeaderMap,
40:     ) -> Result<Vec<MemorySummarizeOutput>, ApiError> {
41:         let resp = self
42:             .session
43:             .execute(Method::POST, Self::path(), extra_headers, Some(body))
44:             .await?;
45:         let parsed: SummarizeResponse =
46:             serde_json::from_slice(&resp.body).map_err(|e| ApiError::Stream(e.to_string()))?;
47:         Ok(parsed.output)
... [66 satir daha, toplam 3877 karakter]
```

### arac: read

```json
{
 "filePath": "C:\\Users\\Bedirhan\\Desktop\\agent\\ab\\par-opencode\\codex-api\\src\\endpoint\\models.rs",
 "limit": 100
}
```

#### cikti

```
<path>C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\endpoint\models.rs</path>
<type>file</type>
<content>
1: use crate::auth::SharedAuthProvider;
2: use crate::endpoint::session::EndpointSession;
3: use crate::error::ApiError;
4: use crate::provider::Provider;
5: use codex_client::HttpTransport;
6: use codex_client::RequestTelemetry;
7: use codex_protocol::openai_models::ModelInfo;
8: use codex_protocol::openai_models::ModelsResponse;
9: use http::HeaderMap;
10: use http::Method;
11: use http::header::ETAG;
12: use std::sync::Arc;
13: 
14: pub struct ModelsClient<T: HttpTransport> {
15:     session: EndpointSession<T>,
16: }
17: 
18: impl<T: HttpTransport> ModelsClient<T> {
19:     pub fn new(transport: T, provider: Provider, auth: SharedAuthProvider) -> Self {
20:         Self {
21:             session: EndpointSession::new(transport, provider, auth),
22:         }
23:     }
24: 
25:     pub fn with_telemetry(self, request: Option<Arc<dyn RequestTelemetry>>) -> Self {
26:         Self {
27:             session: self.session.with_request_telemetry(request),
28:         }
29:     }
30: 
31:     fn path() -> &'static str {
32:         "models"
33:     }
34: 
35:     fn append_client_version_query(req: &mut codex_client::Request, client_version: &str) {
36:         let separator = if req.url.contains('?') { '&' } else { '?' };
37:         req.url = format!("{}{}client_version={client_version}", req.url, separator);
38:     }
39: 
40:     pub fn request_url(provider: &Provider, client_version: &str) -> String {
41:         let mut request = provider.build_request(Method::GET, Self::path());
42:         Self::append_client_version_query(&mut request, client_version);
43:         request.url
44:     }
45: 
46:     pub async fn list_models(
47:         &self,
... [56 satir daha, toplam 3575 karakter]
```

### arac: read

```json
{
 "filePath": "C:\\Users\\Bedirhan\\Desktop\\agent\\ab\\par-opencode\\codex-api\\src\\endpoint\\search.rs",
 "limit": 110
}
```

#### cikti

```
<path>C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\endpoint\search.rs</path>
<type>file</type>
<content>
1: use crate::auth::SharedAuthProvider;
2: use crate::endpoint::session::EndpointSession;
3: use crate::error::ApiError;
4: use crate::provider::Provider;
5: use crate::search::SearchRequest;
6: use crate::search::SearchResponse;
7: use codex_client::HttpTransport;
8: use codex_client::RequestTelemetry;
9: use http::HeaderMap;
10: use http::Method;
11: use serde_json::to_value;
12: use std::sync::Arc;
13: 
14: pub struct SearchClient<T: HttpTransport> {
15:     session: EndpointSession<T>,
16: }
17: 
18: impl<T: HttpTransport> SearchClient<T> {
19:     pub fn new(transport: T, provider: Provider, auth: SharedAuthProvider) -> Self {
20:         Self {
21:             session: EndpointSession::new(transport, provider, auth),
22:         }
23:     }
24: 
25:     pub fn with_telemetry(self, request: Option<Arc<dyn RequestTelemetry>>) -> Self {
26:         Self {
27:             session: self.session.with_request_telemetry(request),
28:         }
29:     }
30: 
31:     fn path() -> &'static str {
32:         "alpha/search"
33:     }
34: 
35:     pub async fn search(
36:         &self,
37:         request: &SearchRequest,
38:         extra_headers: HeaderMap,
39:     ) -> Result<SearchResponse, ApiError> {
40:         let body = to_value(request)
41:             .map_err(|e| ApiError::Stream(format!("failed to encode search request: {e}")))?;
42:         let resp = self
43:             .session
44:             .execute(Method::POST, Self::path(), extra_headers, Some(body))
45:             .await?;
46:         serde_json::from_slice(&resp.body)
47:             .map_err(|e| ApiError::Stream(format!("failed to decode search response: {e}")))
... [66 satir daha, toplam 4104 karakter]
```

### arac: read

```json
{
 "filePath": "C:\\Users\\Bedirhan\\Desktop\\agent\\ab\\par-opencode\\codex-api\\src\\endpoint\\session.rs",
 "limit": 100
}
```

#### cikti

```
<path>C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\endpoint\session.rs</path>
<type>file</type>
<content>
1: use crate::auth::SharedAuthProvider;
2: use crate::error::ApiError;
3: use crate::provider::Provider;
4: use crate::telemetry::run_with_request_telemetry;
5: use codex_client::EncodedJsonBody;
6: use codex_client::HttpTransport;
7: use codex_client::Request;
8: use codex_client::RequestBody;
9: use codex_client::RequestTelemetry;
10: use codex_client::Response;
11: use codex_client::StreamResponse;
12: use codex_client::TransportError;
13: use http::HeaderMap;
14: use http::Method;
15: use serde_json::Value;
16: use std::sync::Arc;
17: use tracing::instrument;
18: 
19: pub(crate) struct EndpointSession<T: HttpTransport> {
20:     transport: T,
21:     provider: Provider,
22:     auth: SharedAuthProvider,
23:     request_telemetry: Option<Arc<dyn RequestTelemetry>>,
24: }
25: 
26: impl<T: HttpTransport> EndpointSession<T> {
27:     pub(crate) fn new(transport: T, provider: Provider, auth: SharedAuthProvider) -> Self {
28:         Self {
29:             transport,
30:             provider,
31:             auth,
32:             request_telemetry: None,
33:         }
34:     }
35: 
36:     pub(crate) fn with_request_telemetry(
37:         mut self,
38:         request: Option<Arc<dyn RequestTelemetry>>,
39:     ) -> Self {
40:         self.request_telemetry = request;
41:         self
42:     }
43: 
44:     pub(crate) fn provider(&self) -> &Provider {
45:         &self.provider
46:     }
47: 
... [56 satir daha, toplam 3318 karakter]
```

---

### arac: read

```json
{
 "filePath": "C:\\Users\\Bedirhan\\Desktop\\agent\\ab\\par-opencode\\codex-api\\src\\endpoint\\realtime_call.rs",
 "limit": 90
}
```

#### cikti

```
<path>C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\endpoint\realtime_call.rs</path>
<type>file</type>
<content>
1: use crate::auth::SharedAuthProvider;
2: use crate::endpoint::realtime_websocket::RealtimeEventParser;
3: use crate::endpoint::realtime_websocket::RealtimeSessionConfig;
4: use crate::endpoint::realtime_websocket::session_update_session_json;
5: use crate::endpoint::session::EndpointSession;
6: use crate::error::ApiError;
7: use crate::provider::Provider;
8: use bytes::Bytes;
9: use codex_client::HttpTransport;
10: use codex_client::Request;
11: use codex_client::RequestBody;
12: use codex_client::RequestTelemetry;
13: use http::HeaderMap;
14: use http::HeaderValue;
15: use http::Method;
16: use http::header::CONTENT_TYPE;
17: use http::header::LOCATION;
18: use serde::Serialize;
19: use serde_json::Value;
20: use serde_json::to_string;
21: use serde_json::to_value;
22: use std::sync::Arc;
23: use tracing::instrument;
24: use tracing::trace;
25: 
26: const MULTIPART_BOUNDARY: &str = "codex-realtime-call-boundary";
27: const MULTIPART_CONTENT_TYPE: &str = "multipart/form-data; boundary=codex-realtime-call-boundary";
28: 
29: pub struct RealtimeCallClient<T: HttpTransport> {
30:     session: EndpointSession<T>,
31: }
32: 
33: /// Answer from creating a WebRTC Realtime call.
34: ///
35: /// `sdp` configures the peer connection. `call_id` is parsed from the response `Location` header
36: /// and is later used by the server-side sideband WebSocket to join this exact call.
37: #[derive(Debug, Clone, PartialEq, Eq)]
38: pub struct RealtimeCallResponse {
39:     pub sdp: String,
40:     pub call_id: String,
41: }
42: 
43: #[derive(Serialize)]
44: struct BackendRealtimeCallRequest<'a> {
45:     sdp: &'a str,
46:     session: &'a Value,
47: }
... [46 satir daha, toplam 3299 karakter]
```

### arac: read

```json
{
 "filePath": "C:\\Users\\Bedirhan\\Desktop\\agent\\ab\\par-opencode\\codex-api\\src\\endpoint\\responses_websocket.rs",
 "limit": 100
}
```

#### cikti

```
<path>C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\endpoint\responses_websocket.rs</path>
<type>file</type>
<content>
1: use crate::auth::SharedAuthProvider;
2: use crate::common::ResponseEvent;
3: use crate::common::ResponseStream;
4: use crate::common::ResponsesWsRequest;
5: use crate::common::SafetyBufferingTreatment;
6: use crate::common::WS_REQUEST_HEADER_TRACEPARENT_CLIENT_METADATA_KEY;
7: use crate::endpoint::responses::ResponsesEndpoint;
8: use crate::error::ApiError;
9: use crate::provider::Provider;
10: use crate::rate_limits::parse_rate_limit_event;
11: use crate::safety_buffering::treatment_from_headers;
12: use crate::sse::ResponsesStreamEvent;
13: use crate::sse::process_responses_event;
14: use crate::telemetry::WebsocketTelemetry;
15: use codex_client::TransportError;
16: use codex_http_client::HttpClientFactory;
17: use codex_websocket_client::WebSocketConnection;
18: use codex_websocket_client::WebSocketConnector;
19: use futures::SinkExt;
20: use futures::StreamExt;
21: use http::HeaderMap;
22: use http::HeaderName;
23: use http::HeaderValue;
24: use http::StatusCode;
25: use serde::Deserialize;
26: use serde_json::Value;
27: use serde_json::map::Map as JsonMap;
28: use std::sync::Arc;
29: use std::sync::OnceLock;
30: use std::time::Duration;
31: use tokio::sync::Mutex;
32: use tokio::sync::mpsc;
33: use tokio::sync::oneshot;
34: use tokio::time::Instant;
35: use tokio_tungstenite::tungstenite::Error as WsError;
36: use tokio_tungstenite::tungstenite::Message;
37: use tokio_tungstenite::tungstenite::client::IntoClientRequest;
38: use tokio_tungstenite::tungstenite::protocol::CloseFrame;
39: use tracing::Instrument;
40: use tracing::Span;
41: use tracing::debug;
42: use tracing::error;
43: use tracing::info;
44: use tracing::instrument;
45: use tungstenite::extensions::ExtensionsConfig;
46: use tungstenite::extensions::compression::deflate::DeflateConfig;
47: use tungstenite::protocol::WebSocketConfig;
... [56 satir daha, toplam 4333 karakter]
```

### arac: read

```json
{
 "filePath": "C:\\Users\\Bedirhan\\Desktop\\agent\\ab\\par-opencode\\codex-api\\src\\endpoint\\realtime_websocket\\mod.rs",
 "limit": 80
}
```

#### cikti

```
<path>C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\endpoint\realtime_websocket\mod.rs</path>
<type>file</type>
<content>
1: pub(crate) mod methods;
2: mod methods_common;
3: mod methods_frameless_bidi;
4: mod methods_v1;
5: mod methods_v2;
6: pub(crate) mod protocol;
7: mod protocol_common;
8: mod protocol_frameless_bidi;
9: mod protocol_v1;
10: mod protocol_v2;
11: 
12: pub use methods::RealtimeTranscriptState;
13: pub use methods::RealtimeWebsocketClient;
14: pub use methods::RealtimeWebsocketConnection;
15: pub use methods::RealtimeWebsocketEvents;
16: pub use methods::RealtimeWebsocketWriter;
17: pub use methods_common::session_update_session_json;
18: pub use protocol::RealtimeContextAppendChannel;
19: pub use protocol::RealtimeEventParser;
20: pub use protocol::RealtimeOutputModality;
21: pub use protocol::RealtimeSessionConfig;
22: pub use protocol::RealtimeSessionMode;

(End of file - total 22 lines)
</content>
```

### arac: read

```json
{
 "filePath": "C:\\Users\\Bedirhan\\Desktop\\agent\\ab\\par-opencode\\codex-api\\src\\endpoint\\realtime_websocket\\protocol.rs",
 "limit": 80
}
```

#### cikti

```
<path>C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\endpoint\realtime_websocket\protocol.rs</path>
<type>file</type>
<content>
1: use crate::endpoint::realtime_websocket::protocol_frameless_bidi::parse_frameless_bidi_event;
2: use crate::endpoint::realtime_websocket::protocol_v1::parse_realtime_event_v1;
3: use crate::endpoint::realtime_websocket::protocol_v2::parse_realtime_event_v2;
4: use codex_protocol::protocol::ConversationTextParams;
5: use codex_protocol::protocol::ConversationTextRole;
6: pub use codex_protocol::protocol::RealtimeAudioFrame;
7: pub use codex_protocol::protocol::RealtimeEvent;
8: pub use codex_protocol::protocol::RealtimeOutputModality;
9: pub use codex_protocol::protocol::RealtimeTranscriptEntry;
10: pub use codex_protocol::protocol::RealtimeVoice;
11: use serde::Serialize;
12: use serde_json::Value;
13: 
14: #[derive(Debug, Clone, Copy, PartialEq, Eq)]
15: pub enum RealtimeEventParser {
16:     V1,
17:     FramelessBidi,
18:     RealtimeV2,
19: }
20: 
21: pub type RealtimeWireAdapter = RealtimeEventParser;
22: 
23: #[derive(Debug, Clone, Copy, PartialEq, Eq)]
24: pub enum RealtimeSessionMode {
25:     Conversational,
26:     Transcription,
27: }
28: 
29: /// Selects the semantic stream used for Frameless Bidi context appends.
30: #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
31: #[serde(rename_all = "snake_case")]
32: pub enum RealtimeContextAppendChannel {
33:     Speakable,
34:     Commentary,
35: }
36: 
37: #[derive(Debug, Clone, PartialEq, Eq)]
38: pub struct RealtimeSessionConfig {
39:     pub instructions: String,
40:     pub initial_items: Vec<ConversationTextParams>,
41:     pub delegation_ack_filler: Option<bool>,
42:     pub model: Option<String>,
43:     pub session_id: Option<String>,
44:     pub event_parser: RealtimeEventParser,
45:     pub session_mode: RealtimeSessionMode,
46:     pub output_modality: RealtimeOutputModality,
47:     pub voice: RealtimeVoice,
... [36 satir daha, toplam 3333 karakter]
```

---

### arac: read

```json
{
 "filePath": "C:\\Users\\Bedirhan\\Desktop\\agent\\ab\\par-opencode\\codex-api\\src\\endpoint\\realtime_websocket\\protocol_common.rs",
 "limit": 60
}
```

#### cikti

```
<path>C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\endpoint\realtime_websocket\protocol_common.rs</path>
<type>file</type>
<content>
1: use codex_protocol::protocol::RealtimeEvent;
2: use codex_protocol::protocol::RealtimeTranscriptDelta;
3: use codex_protocol::protocol::RealtimeTranscriptDone;
4: use serde_json::Value;
5: use tracing::debug;
6: 
7: pub(super) fn parse_realtime_payload(payload: &str, parser_name: &str) -> Option<(Value, String)> {
8:     let parsed: Value = match serde_json::from_str(payload) {
9:         Ok(message) => message,
10:         Err(err) => {
11:             debug!("failed to parse {parser_name} event: {err}, data: {payload}");
12:             return None;
13:         }
14:     };
15: 
16:     let message_type = match parsed.get("type").and_then(Value::as_str) {
17:         Some(message_type) => message_type.to_string(),
18:         None => {
19:             debug!("received {parser_name} event without type field: {payload}");
20:             return None;
21:         }
22:     };
23: 
24:     Some((parsed, message_type))
25: }
26: 
27: pub(super) fn parse_session_updated_event(parsed: &Value) -> Option<RealtimeEvent> {
28:     let session_id = parsed
29:         .get("session")
30:         .and_then(Value::as_object)
31:         .and_then(|session| session.get("id"))
32:         .and_then(Value::as_str)
33:         .map(str::to_string)?;
34:     let instructions = parsed
35:         .get("session")
36:         .and_then(Value::as_object)
37:         .and_then(|session| session.get("instructions"))
38:         .and_then(Value::as_str)
39:         .map(str::to_string);
40:     Some(RealtimeEvent::SessionUpdated {
41:         realtime_session_id: session_id,
42:         instructions,
43:     })
44: }
45: 
46: pub(super) fn parse_transcript_delta_event(
47:     parsed: &Value,
... [16 satir daha, toplam 2294 karakter]
```

### arac: read

```json
{
 "filePath": "C:\\Users\\Bedirhan\\Desktop\\agent\\ab\\par-opencode\\codex-api\\src\\endpoint\\realtime_websocket\\protocol_v1.rs",
 "limit": 70
}
```

#### cikti

```
<path>C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\endpoint\realtime_websocket\protocol_v1.rs</path>
<type>file</type>
<content>
1: use crate::endpoint::realtime_websocket::protocol_common::parse_error_event;
2: use crate::endpoint::realtime_websocket::protocol_common::parse_realtime_payload;
3: use crate::endpoint::realtime_websocket::protocol_common::parse_session_updated_event;
4: use crate::endpoint::realtime_websocket::protocol_common::parse_transcript_delta_event;
5: use crate::endpoint::realtime_websocket::protocol_common::parse_transcript_done_event;
6: use codex_protocol::protocol::RealtimeAudioFrame;
7: use codex_protocol::protocol::RealtimeEvent;
8: use codex_protocol::protocol::RealtimeHandoffRequested;
9: use serde_json::Value;
10: use tracing::debug;
11: 
12: pub(super) fn parse_realtime_event_v1(payload: &str) -> Option<RealtimeEvent> {
13:     let (parsed, message_type) = parse_realtime_payload(payload, "realtime v1")?;
14:     match message_type.as_str() {
15:         "session.updated" => parse_session_updated_event(&parsed),
16:         "conversation.output_audio.delta" => {
17:             let data = parsed
18:                 .get("delta")
19:                 .and_then(Value::as_str)
20:                 .or_else(|| parsed.get("data").and_then(Value::as_str))
21:                 .map(str::to_string)?;
22:             let sample_rate = parsed
23:                 .get("sample_rate")
24:                 .and_then(Value::as_u64)
25:                 .and_then(|value| u32::try_from(value).ok())?;
26:             let num_channels = parsed
27:                 .get("channels")
28:                 .or_else(|| parsed.get("num_channels"))
29:                 .and_then(Value::as_u64)
30:                 .and_then(|value| u16::try_from(value).ok())?;
31:             Some(RealtimeEvent::AudioOut(RealtimeAudioFrame {
32:                 data,
33:                 sample_rate,
34:                 num_channels,
35:                 samples_per_channel: parsed
36:                     .get("samples_per_channel")
37:                     .and_then(Value::as_u64)
38:                     .and_then(|value| u32::try_from(value).ok()),
39:                 item_id: None,
40:             }))
41:         }
42:         "conversation.input_transcript.delta"
43:         | "conversation.item.input_audio_transcription.delta" => {
44:             parse_transcript_delta_event(&parsed, "delta").map(RealtimeEvent::InputTranscriptDelta)
45:         }
46:         "conversation.input_transcript.turn_marked"
47:         | "conversation.item.input_audio_transcription.completed" => {
... [26 satir daha, toplam 3803 karakter]
```

### arac: read

```json
{
 "filePath": "C:\\Users\\Bedirhan\\Desktop\\agent\\ab\\par-opencode\\codex-api\\src\\endpoint\\realtime_websocket\\protocol_v2.rs",
 "limit": 70
}
```

#### cikti

```
<path>C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\endpoint\realtime_websocket\protocol_v2.rs</path>
<type>file</type>
<content>
1: use crate::endpoint::realtime_websocket::protocol_common::parse_error_event;
2: use crate::endpoint::realtime_websocket::protocol_common::parse_realtime_payload;
3: use crate::endpoint::realtime_websocket::protocol_common::parse_session_updated_event;
4: use crate::endpoint::realtime_websocket::protocol_common::parse_transcript_delta_event;
5: use crate::endpoint::realtime_websocket::protocol_common::parse_transcript_done_event;
6: use codex_protocol::protocol::RealtimeAudioFrame;
7: use codex_protocol::protocol::RealtimeEvent;
8: use codex_protocol::protocol::RealtimeHandoffRequested;
9: use codex_protocol::protocol::RealtimeInputAudioSpeechStarted;
10: use codex_protocol::protocol::RealtimeNoopRequested;
11: use codex_protocol::protocol::RealtimeResponseCancelled;
12: use codex_protocol::protocol::RealtimeResponseCreated;
13: use codex_protocol::protocol::RealtimeResponseDone;
14: use serde_json::Map as JsonMap;
15: use serde_json::Value;
16: use tracing::debug;
17: 
18: const BACKGROUND_AGENT_TOOL_NAME: &str = "background_agent";
19: const SILENCE_TOOL_NAME: &str = "remain_silent";
20: const DEFAULT_AUDIO_SAMPLE_RATE: u32 = 24_000;
21: const DEFAULT_AUDIO_CHANNELS: u16 = 1;
22: const TOOL_ARGUMENT_KEYS: [&str; 5] = ["input_transcript", "input", "text", "prompt", "query"];
23: 
24: pub(super) fn parse_realtime_event_v2(payload: &str) -> Option<RealtimeEvent> {
25:     let (parsed, message_type) = parse_realtime_payload(payload, "realtime v2")?;
26: 
27:     match message_type.as_str() {
28:         "session.updated" => parse_session_updated_event(&parsed),
29:         "response.output_audio.delta" | "response.audio.delta" => {
30:             parse_output_audio_delta_event(&parsed)
31:         }
32:         "conversation.item.input_audio_transcription.delta" => {
33:             parse_transcript_delta_event(&parsed, "delta").map(RealtimeEvent::InputTranscriptDelta)
34:         }
35:         "conversation.item.input_audio_transcription.completed" => {
36:             parse_transcript_done_event(&parsed, "transcript")
37:                 .map(RealtimeEvent::InputTranscriptDone)
38:         }
39:         "response.output_text.delta" | "response.output_audio_transcript.delta" => {
40:             parse_transcript_delta_event(&parsed, "delta").map(RealtimeEvent::OutputTranscriptDelta)
41:         }
42:         "response.output_text.done" => {
43:             parse_transcript_done_event(&parsed, "text").map(RealtimeEvent::OutputTranscriptDone)
44:         }
45:         "response.output_audio_transcript.done" => {
46:             parse_transcript_done_event(&parsed, "transcript")
47:                 .map(RealtimeEvent::OutputTranscriptDone)
... [26 satir daha, toplam 4046 karakter]
```

### arac: read

```json
{
 "filePath": "C:\\Users\\Bedirhan\\Desktop\\agent\\ab\\par-opencode\\codex-api\\src\\endpoint\\realtime_websocket\\protocol_frameless_bidi.rs",
 "limit": 70
}
```

#### cikti

```
<path>C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\endpoint\realtime_websocket\protocol_frameless_bidi.rs</path>
<type>file</type>
<content>
1: use crate::endpoint::realtime_websocket::protocol_common::parse_error_event;
2: use crate::endpoint::realtime_websocket::protocol_common::parse_realtime_payload;
3: use crate::endpoint::realtime_websocket::protocol_common::parse_session_updated_event;
4: use codex_protocol::protocol::RealtimeAudioFrame;
5: use codex_protocol::protocol::RealtimeEvent;
6: use codex_protocol::protocol::RealtimeHandoffRequested;
7: use codex_protocol::protocol::RealtimeTranscriptDelta;
8: use codex_protocol::protocol::RealtimeTranscriptDone;
9: use serde_json::Value;
10: use tracing::debug;
11: 
12: const DEFAULT_AUDIO_SAMPLE_RATE: u32 = 24_000;
13: const DEFAULT_AUDIO_CHANNELS: u16 = 1;
14: 
15: pub(super) fn parse_frameless_bidi_event(payload: &str) -> Option<RealtimeEvent> {
16:     let (parsed, message_type) = parse_realtime_payload(payload, "frameless bidi")?;
17:     match message_type.as_str() {
18:         "session.started" | "session.updated" => parse_session_updated_event(&parsed),
19:         "output_audio.delta" => parse_output_audio_delta(&parsed),
20:         "input_transcript.added" => {
21:             parse_transcript_item(&parsed).map(RealtimeEvent::InputTranscriptDelta)
22:         }
23:         "output_transcript.added" => {
24:             parse_transcript_item(&parsed).map(RealtimeEvent::OutputTranscriptDelta)
25:         }
26:         "turn.done" => parse_turn_done(&parsed),
27:         "delegation.created" => parse_delegation_created(&parsed),
28:         "error" => parse_error_event(&parsed),
29:         _ => {
30:             debug!(
31:                 "received unsupported frameless bidi event type: {message_type}, data: {payload}"
32:             );
33:             None
34:         }
35:     }
36: }
37: 
38: fn parse_output_audio_delta(parsed: &Value) -> Option<RealtimeEvent> {
39:     Some(RealtimeEvent::AudioOut(RealtimeAudioFrame {
40:         data: parsed.get("audio").and_then(Value::as_str)?.to_string(),
41:         sample_rate: DEFAULT_AUDIO_SAMPLE_RATE,
42:         num_channels: DEFAULT_AUDIO_CHANNELS,
43:         samples_per_channel: None,
44:         item_id: None,
45:     }))
46: }
47: 
... [26 satir daha, toplam 3279 karakter]
```

---

### arac: read

```json
{
 "filePath": "C:\\Users\\Bedirhan\\Desktop\\agent\\ab\\par-opencode\\codex-api\\src\\endpoint\\realtime_websocket\\methods.rs",
 "limit": 90
}
```

#### cikti

```
<path>C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\endpoint\realtime_websocket\methods.rs</path>
<type>file</type>
<content>
1: use crate::endpoint::realtime_websocket::methods_common::conversation_function_call_output_message;
2: use crate::endpoint::realtime_websocket::methods_common::conversation_handoff_append_message;
3: use crate::endpoint::realtime_websocket::methods_common::conversation_item_create_message;
4: use crate::endpoint::realtime_websocket::methods_common::normalized_session_mode;
5: use crate::endpoint::realtime_websocket::methods_common::session_update_message;
6: use crate::endpoint::realtime_websocket::methods_common::standalone_handoff_message;
7: use crate::endpoint::realtime_websocket::methods_common::websocket_intent;
8: use crate::endpoint::realtime_websocket::methods_frameless_bidi::context_append_chunks;
9: use crate::endpoint::realtime_websocket::methods_frameless_bidi::delegation_context_append_message as frameless_delegation_context_append_message;
10: use crate::endpoint::realtime_websocket::methods_frameless_bidi::session_context_append_message as frameless_session_context_append_message;
11: use crate::endpoint::realtime_websocket::protocol::RealtimeAudioFrame;
12: use crate::endpoint::realtime_websocket::protocol::RealtimeContextAppendChannel;
13: use crate::endpoint::realtime_websocket::protocol::RealtimeEvent;
14: use crate::endpoint::realtime_websocket::protocol::RealtimeEventParser;
15: use crate::endpoint::realtime_websocket::protocol::RealtimeOutboundMessage;
16: use crate::endpoint::realtime_websocket::protocol::RealtimeOutputModality;
17: use crate::endpoint::realtime_websocket::protocol::RealtimeSessionConfig;
18: use crate::endpoint::realtime_websocket::protocol::RealtimeSessionMode;
19: use crate::endpoint::realtime_websocket::protocol::RealtimeTranscriptEntry;
20: use crate::endpoint::realtime_websocket::protocol::RealtimeVoice;
21: use crate::endpoint::realtime_websocket::protocol::parse_realtime_event;
22: use crate::error::ApiError;
23: use crate::provider::Provider;
24: use codex_client::backoff;
25: use codex_http_client::maybe_build_rustls_client_config_with_custom_ca;
26: use codex_protocol::protocol::ConversationTextParams;
27: use codex_protocol::protocol::ConversationTextRole;
28: use codex_protocol::protocol::RealtimeTranscriptDelta;
29: use codex_utils_rustls_provider::ensure_rustls_crypto_provider;
30: use futures::SinkExt;
31: use futures::StreamExt;
32: use http::HeaderMap;
33: use http::HeaderValue;
34: use http::StatusCode;
35: use std::collections::HashMap;
36: use std::collections::VecDeque;
37: use std::sync::Arc;
38: use std::sync::atomic::AtomicBool;
39: use std::sync::atomic::Ordering;
40: use tokio::net::TcpStream;
41: use tokio::sync::Mutex;
42: use tokio::sync::mpsc;
43: use tokio::sync::oneshot;
44: use tokio::time::sleep;
45: use tokio_tungstenite::MaybeTlsStream;
46: use tokio_tungstenite::WebSocketStream;
47: use tokio_tungstenite::tungstenite::Error as WsError;
... [46 satir daha, toplam 4639 karakter]
```

### arac: read

```json
{
 "filePath": "C:\\Users\\Bedirhan\\Desktop\\agent\\ab\\par-opencode\\codex-api\\src\\endpoint\\realtime_websocket\\methods_common.rs",
 "limit": 80
}
```

#### cikti

```
<path>C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\endpoint\realtime_websocket\methods_common.rs</path>
<type>file</type>
<content>
1: use crate::endpoint::realtime_websocket::methods_frameless_bidi::delegation_context_append_message as frameless_delegation_context_append_message;
2: use crate::endpoint::realtime_websocket::methods_frameless_bidi::session_context_append_message as frameless_session_context_append_message;
3: use crate::endpoint::realtime_websocket::methods_frameless_bidi::session_json as frameless_session_json;
4: use crate::endpoint::realtime_websocket::methods_frameless_bidi::session_update_message as frameless_session_update_message;
5: use crate::endpoint::realtime_websocket::methods_v1::conversation_handoff_append_message as v1_conversation_handoff_append_message;
6: use crate::endpoint::realtime_websocket::methods_v1::conversation_item_create_message as v1_conversation_item_create_message;
7: use crate::endpoint::realtime_websocket::methods_v1::session_update_session as v1_session_update_session;
8: use crate::endpoint::realtime_websocket::methods_v1::websocket_intent as v1_websocket_intent;
9: use crate::endpoint::realtime_websocket::methods_v2::conversation_function_call_output_message as v2_conversation_function_call_output_message;
10: use crate::endpoint::realtime_websocket::methods_v2::conversation_item_create_message as v2_conversation_item_create_message;
11: use crate::endpoint::realtime_websocket::methods_v2::session_update_session as v2_session_update_session;
12: use crate::endpoint::realtime_websocket::methods_v2::websocket_intent as v2_websocket_intent;
13: use crate::endpoint::realtime_websocket::protocol::RealtimeContextAppendChannel;
14: use crate::endpoint::realtime_websocket::protocol::RealtimeOutboundMessage;
15: use crate::endpoint::realtime_websocket::protocol::RealtimeOutputModality;
16: use crate::endpoint::realtime_websocket::protocol::RealtimeSessionConfig;
17: use crate::endpoint::realtime_websocket::protocol::RealtimeSessionMode;
18: use crate::endpoint::realtime_websocket::protocol::RealtimeVoice;
19: use crate::endpoint::realtime_websocket::protocol::RealtimeWireAdapter;
20: use codex_protocol::protocol::ConversationTextParams;
21: use codex_protocol::protocol::ConversationTextRole;
22: use serde_json::Result as JsonResult;
23: use serde_json::Value;
24: use serde_json::to_value;
25: 
26: pub(super) const REALTIME_AUDIO_SAMPLE_RATE: u32 = 24_000;
27: const AGENT_FINAL_MESSAGE_PREFIX: &str = "\"Agent Final Message\":\n\n";
28: 
29: pub(super) fn normalized_session_mode(
30:     wire_adapter: RealtimeWireAdapter,
31:     session_mode: RealtimeSessionMode,
32: ) -> RealtimeSessionMode {
33:     match wire_adapter {
34:         RealtimeWireAdapter::V1 | RealtimeWireAdapter::FramelessBidi => {
35:             RealtimeSessionMode::Conversational
36:         }
37:         RealtimeWireAdapter::RealtimeV2 => session_mode,
38:     }
39: }
40: 
41: pub(super) fn conversation_item_create_message(
42:     wire_adapter: RealtimeWireAdapter,
43:     text: String,
44:     role: ConversationTextRole,
45:     context_append_channel: Option<RealtimeContextAppendChannel>,
46: ) -> RealtimeOutboundMessage {
47:     match wire_adapter {
... [36 satir daha, toplam 4689 karakter]
```

### arac: read

```json
{
 "filePath": "C:\\Users\\Bedirhan\\Desktop\\agent\\ab\\par-opencode\\codex-api\\src\\endpoint\\realtime_websocket\\methods_v1.rs",
 "limit": 70
}
```

#### cikti

```
<path>C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\endpoint\realtime_websocket\methods_v1.rs</path>
<type>file</type>
<content>
1: use crate::endpoint::realtime_websocket::methods_common::REALTIME_AUDIO_SAMPLE_RATE;
2: use crate::endpoint::realtime_websocket::protocol::AudioFormatType;
3: use crate::endpoint::realtime_websocket::protocol::ConversationContentType;
4: use crate::endpoint::realtime_websocket::protocol::ConversationItemContent;
5: use crate::endpoint::realtime_websocket::protocol::ConversationItemPayload;
6: use crate::endpoint::realtime_websocket::protocol::ConversationItemType;
7: use crate::endpoint::realtime_websocket::protocol::ConversationMessageItem;
8: use crate::endpoint::realtime_websocket::protocol::RealtimeOutboundMessage;
9: use crate::endpoint::realtime_websocket::protocol::RealtimeVoice;
10: use crate::endpoint::realtime_websocket::protocol::SessionAudio;
11: use crate::endpoint::realtime_websocket::protocol::SessionAudioFormat;
12: use crate::endpoint::realtime_websocket::protocol::SessionAudioInput;
13: use crate::endpoint::realtime_websocket::protocol::SessionAudioOutput;
14: use crate::endpoint::realtime_websocket::protocol::SessionType;
15: use crate::endpoint::realtime_websocket::protocol::SessionUpdateSession;
16: use codex_protocol::protocol::ConversationTextRole;
17: 
18: pub(super) fn conversation_item_create_message(
19:     text: String,
20:     role: ConversationTextRole,
21: ) -> RealtimeOutboundMessage {
22:     let content_type = match role {
23:         ConversationTextRole::Assistant => ConversationContentType::OutputText,
24:         ConversationTextRole::User | ConversationTextRole::Developer => {
25:             ConversationContentType::InputText
26:         }
27:     };
28: 
29:     RealtimeOutboundMessage::ConversationItemCreate {
30:         item: ConversationItemPayload::Message(ConversationMessageItem {
31:             r#type: ConversationItemType::Message,
32:             role,
33:             content: vec![ConversationItemContent {
34:                 r#type: content_type,
35:                 text,
36:             }],
37:         }),
38:     }
39: }
40: 
41: pub(super) fn conversation_handoff_append_message(
42:     handoff_id: String,
43:     output_text: String,
44: ) -> RealtimeOutboundMessage {
45:     RealtimeOutboundMessage::ConversationHandoffAppend {
46:         handoff_id,
47:         output_text,
... [26 satir daha, toplam 3260 karakter]
```

### arac: read

```json
{
 "filePath": "C:\\Users\\Bedirhan\\Desktop\\agent\\ab\\par-opencode\\codex-api\\src\\endpoint\\realtime_websocket\\methods_v2.rs",
 "limit": 70
}
```

#### cikti

```
<path>C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\endpoint\realtime_websocket\methods_v2.rs</path>
<type>file</type>
<content>
1: use crate::endpoint::realtime_websocket::methods_common::REALTIME_AUDIO_SAMPLE_RATE;
2: use crate::endpoint::realtime_websocket::protocol::AudioFormatType;
3: use crate::endpoint::realtime_websocket::protocol::ConversationContentType;
4: use crate::endpoint::realtime_websocket::protocol::ConversationFunctionCallOutputItem;
5: use crate::endpoint::realtime_websocket::protocol::ConversationItemContent;
6: use crate::endpoint::realtime_websocket::protocol::ConversationItemPayload;
7: use crate::endpoint::realtime_websocket::protocol::ConversationItemType;
8: use crate::endpoint::realtime_websocket::protocol::ConversationMessageItem;
9: use crate::endpoint::realtime_websocket::protocol::NoiseReductionType;
10: use crate::endpoint::realtime_websocket::protocol::RealtimeOutboundMessage;
11: use crate::endpoint::realtime_websocket::protocol::RealtimeOutputModality;
12: use crate::endpoint::realtime_websocket::protocol::RealtimeSessionMode;
13: use crate::endpoint::realtime_websocket::protocol::RealtimeVoice;
14: use crate::endpoint::realtime_websocket::protocol::SessionAudio;
15: use crate::endpoint::realtime_websocket::protocol::SessionAudioFormat;
16: use crate::endpoint::realtime_websocket::protocol::SessionAudioInput;
17: use crate::endpoint::realtime_websocket::protocol::SessionAudioOutput;
18: use crate::endpoint::realtime_websocket::protocol::SessionAudioOutputFormat;
19: use crate::endpoint::realtime_websocket::protocol::SessionFunctionTool;
20: use crate::endpoint::realtime_websocket::protocol::SessionInputAudioTranscription;
21: use crate::endpoint::realtime_websocket::protocol::SessionNoiseReduction;
22: use crate::endpoint::realtime_websocket::protocol::SessionToolType;
23: use crate::endpoint::realtime_websocket::protocol::SessionTurnDetection;
24: use crate::endpoint::realtime_websocket::protocol::SessionType;
25: use crate::endpoint::realtime_websocket::protocol::SessionUpdateSession;
26: use crate::endpoint::realtime_websocket::protocol::TurnDetectionType;
27: use codex_protocol::protocol::ConversationTextRole;
28: use serde_json::json;
29: 
30: const REALTIME_V2_OUTPUT_MODALITY_AUDIO: &str = "audio";
31: const REALTIME_V2_OUTPUT_MODALITY_TEXT: &str = "text";
32: const REALTIME_V2_TOOL_CHOICE: &str = "auto";
33: const REALTIME_V2_BACKGROUND_AGENT_TOOL_NAME: &str = "background_agent";
34: const REALTIME_V2_BACKGROUND_AGENT_TOOL_DESCRIPTION: &str = "Send a user request to the background agent. Use this as the default action. Do not rephrase 
the user's ask or rewrite it in your own words; pass along the user's own words. If the background agent is idle, this starts a new task and returns the final 
result to the user. If the background agent is already working on a task, this sends the request as guidance to steer that previous task. If the user asks to do
 something next, later, after this, or once current work finishes, call this tool so the work is actually queued instead of merely promising to do it later.";
35: const REALTIME_V2_SILENCE_TOOL_NAME: &str = "remain_silent";
36: const REALTIME_V2_SILENCE_TOOL_DESCRIPTION: &str = "Call this when the best response is to say nothing. Use it instead of speaking after hidden 
system/control messages, after background agent updates in silent modes, or whenever acknowledging aloud would be distracting. This tool has no user-visible 
effect.";
37: const REALTIME_V2_INPUT_TRANSCRIPTION_MODEL: &str = "gpt-4o-mini-transcribe";
38: 
39: pub(super) fn conversation_item_create_message(
40:     text: String,
41:     role: ConversationTextRole,
42: ) -> RealtimeOutboundMessage {
43:     let content_type = match role {
44:         ConversationTextRole::Assistant => ConversationContentType::OutputText,
45:         ConversationTextRole::User | ConversationTextRole::Developer => {
46:             ConversationContentType::InputText
47:         }
... [26 satir daha, toplam 4908 karakter]
```

---

### arac: read

```json
{
 "filePath": "C:\\Users\\Bedirhan\\Desktop\\agent\\ab\\par-opencode\\codex-api\\src\\endpoint\\realtime_websocket\\methods_frameless_bidi.rs",
 "limit": 70
}
```

#### cikti

```
<path>C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\endpoint\realtime_websocket\methods_frameless_bidi.rs</path>
<type>file</type>
<content>
1: use crate::endpoint::realtime_websocket::protocol::FramelessContentType;
2: use crate::endpoint::realtime_websocket::protocol::FramelessInputTextContent;
3: use crate::endpoint::realtime_websocket::protocol::RealtimeContextAppendChannel;
4: use crate::endpoint::realtime_websocket::protocol::RealtimeOutboundMessage;
5: use crate::endpoint::realtime_websocket::protocol::RealtimeVoice;
6: use codex_protocol::protocol::ConversationTextParams;
7: use codex_protocol::protocol::ConversationTextRole;
8: use serde_json::Value;
9: use serde_json::json;
10: 
11: const CONTEXT_APPEND_MAX_BYTES: usize = 500;
12: 
13: pub(super) fn delegation_context_append_message(
14:     delegation_item_id: String,
15:     text: String,
16:     channel: Option<RealtimeContextAppendChannel>,
17: ) -> RealtimeOutboundMessage {
18:     RealtimeOutboundMessage::DelegationContextAppend {
19:         delegation_item_id,
20:         channel,
21:         content: input_text_content(text),
22:     }
23: }
24: 
25: pub(super) fn session_context_append_message(
26:     text: String,
27:     channel: Option<RealtimeContextAppendChannel>,
28: ) -> RealtimeOutboundMessage {
29:     RealtimeOutboundMessage::SessionContextAppend {
30:         channel,
31:         content: input_text_content(text),
32:     }
33: }
34: 
35: pub(super) fn session_update_message(
36:     instructions: String,
37:     initial_items: Vec<ConversationTextParams>,
38:     voice: RealtimeVoice,
39:     delegation_ack_filler: Option<bool>,
40: ) -> RealtimeOutboundMessage {
41:     RealtimeOutboundMessage::FramelessSessionUpdate {
42:         session: session_json(
43:             /*model*/ None,
44:             instructions,
45:             initial_items,
46:             voice,
47:             delegation_ack_filler,
... [26 satir daha, toplam 2601 karakter]
```

### arac: read

```json
{
 "filePath": "C:\\Users\\Bedirhan\\Desktop\\agent\\ab\\par-opencode\\codex-api\\src\\endpoint\\realtime_websocket\\methods_frameless_bidi_tests.rs",
 "limit": 50
}
```

#### cikti

```
<path>C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\endpoint\realtime_websocket\methods_frameless_bidi_tests.rs</path>
<type>file</type>
<content>
1: use super::CONTEXT_APPEND_MAX_BYTES;
2: use super::context_append_chunks;
3: use super::session_json;
4: use crate::endpoint::realtime_websocket::protocol::RealtimeVoice;
5: use codex_protocol::protocol::ConversationTextParams;
6: use codex_protocol::protocol::ConversationTextRole;
7: use pretty_assertions::assert_eq;
8: use serde_json::json;
9: 
10: #[test]
11: fn context_append_chunks_preserve_text_within_wire_limit() {
12:     for text in ["a".repeat(1_201), "🙂".repeat(200)] {
13:         let chunks = context_append_chunks(&text);
14:         assert_eq!(chunks.concat(), text);
15:         assert!(
16:             chunks
17:                 .iter()
18:                 .all(|chunk| chunk.len() <= CONTEXT_APPEND_MAX_BYTES)
19:         );
20:     }
21: }
22: 
23: #[test]
24: fn session_json_omits_initial_items_when_empty() {
25:     let session = session_json(
26:         Some("gpt-live".to_string()),
27:         "instructions".to_string(),
28:         Vec::new(),
29:         RealtimeVoice::Marin,
30:         /*delegation_ack_filler*/ None,
31:     );
32: 
33:     assert_eq!(
34:         session,
35:         json!({
36:             "model": "gpt-live",
37:             "instructions": "instructions",
38:             "audio": {
39:                 "output": {
40:                     "voice": "marin",
41:                 },
42:             },
43:             "delegation": {
44:                 "type": "client",
45:             },
46:         })
47:     );
... [6 satir daha, toplam 1732 karakter]
```

### arac: read

```json
{
 "filePath": "C:\\Users\\Bedirhan\\Desktop\\agent\\ab\\par-opencode\\codex-api\\src\\endpoint\\realtime_websocket\\protocol_frameless_bidi_tests.rs",
 "limit": 50
}
```

#### cikti

```
<path>C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\endpoint\realtime_websocket\protocol_frameless_bidi_tests.rs</path>
<type>file</type>
<content>
1: use super::parse_frameless_bidi_event;
2: use crate::endpoint::realtime_websocket::protocol_v1::parse_realtime_event_v1;
3: use codex_protocol::protocol::RealtimeEvent;
4: use codex_protocol::protocol::RealtimeHandoffRequested;
5: 
6: #[test]
7: fn legacy_and_frameless_delegations_decode_to_the_same_handoff() {
8:     let expected = Some(RealtimeEvent::HandoffRequested(RealtimeHandoffRequested {
9:         handoff_id: "handoff-123".to_string(),
10:         item_id: "handoff-123".to_string(),
11:         input_transcript: "check the weather".to_string(),
12:         active_transcript: Vec::new(),
13:     }));
14:     let legacy = r#"{
15:         "type": "conversation.handoff.requested",
16:         "handoff_id": "handoff-123",
17:         "item_id": "handoff-123",
18:         "input_transcript": "check the weather"
19:     }"#;
20:     let frameless = r#"{
21:         "type": "delegation.created",
22:         "offset_ms": 1000,
23:         "item": {
24:             "id": "handoff-123",
25:             "type": "delegation",
26:             "target": "client",
27:             "content": [{"type": "input_text", "text": "check the weather"}]
28:         }
29:     }"#;
30: 
31:     assert_eq!(parse_realtime_event_v1(legacy), expected);
32:     assert_eq!(parse_frameless_bidi_event(frameless), expected);
33: }
34: 
35: #[test]
36: fn frameless_transcript_and_audio_events_reuse_existing_internal_events() {
37:     let input = r#"{
38:         "type": "input_transcript.added",
39:         "item": {"id": "input-1", "type": "input_transcript", "text": "hello"}
40:     }"#;
41:     let done = r#"{
42:         "type": "turn.done",
43:         "turn": {"id": "turn-1", "role": "user", "transcript": "hello"}
44:     }"#;
45:     let audio = r#"{
46:         "type": "output_audio.delta",
47:         "audio": "AAE=",
... [6 satir daha, toplam 2131 karakter]
```

### arac: read

```json
{
 "filePath": "C:\\Users\\Bedirhan\\Desktop\\agent\\ab\\par-opencode\\codex-api\\src\\endpoint\\realtime_websocket\\methods_common_tests.rs",
 "limit": 50
}
```

#### cikti

```
<path>C:\Users\Bedirhan\Desktop\agent\ab\par-opencode\codex-api\src\endpoint\realtime_websocket\methods_common_tests.rs</path>
<type>file</type>
<content>
1: use super::conversation_function_call_output_message;
2: use super::conversation_handoff_append_message;
3: use super::session_update_message;
4: use super::standalone_handoff_message;
5: use crate::endpoint::realtime_websocket::protocol::RealtimeContextAppendChannel;
6: use crate::endpoint::realtime_websocket::protocol::RealtimeOutputModality;
7: use crate::endpoint::realtime_websocket::protocol::RealtimeSessionMode;
8: use crate::endpoint::realtime_websocket::protocol::RealtimeVoice;
9: use crate::endpoint::realtime_websocket::protocol::RealtimeWireAdapter;
10: use pretty_assertions::assert_eq;
11: use serde_json::Value;
12: use serde_json::json;
13: use serde_json::to_value;
14: 
15: #[test]
16: fn frameless_session_update_encodes_explicit_delegation_ack_filler() {
17:     for delegation_ack_filler in [false, true] {
18:         let message = session_update_message(
19:             RealtimeWireAdapter::FramelessBidi,
20:             "instructions".to_string(),
21:             Vec::new(),
22:             RealtimeSessionMode::Conversational,
23:             RealtimeOutputModality::Audio,
24:             RealtimeVoice::Marin,
25:             Some(delegation_ack_filler),
26:         );
27: 
28:         assert_eq!(
29:             to_value(message).expect("frameless session update should serialize"),
30:             json!({
31:                 "type": "session.update",
32:                 "session": {
33:                     "instructions": "instructions",
34:                     "audio": {
35:                         "output": {
36:                             "voice": "marin",
37:                         },
38:                     },
39:                     "delegation": {
40:                         "type": "client",
41:                         "ack_filler": delegation_ack_filler,
42:                     },
43:                 },
44:             })
45:         );
46:     }
47: }
... [6 satir daha, toplam 2231 karakter]
```

---

### dusunce

I've read all 44 files now (each at least partially, enough for one line description + citation). Let me compile WIRE.md.

Files by directory:
- src/: telemetry.rs, search.rs, safety_buffering.rs, rate_limits.rs, provider.rs, lib.rs, images.rs, files.rs, error.rs, common.rs, auth.rs, 
api_bridge_tests.rs, api_bridge.rs
- src/sse/: responses.rs, mod.rs, chat.rs
- src/requests/: responses.rs, mod.rs, headers.rs, chat.rs
- src/endpoint/: session.rs, search.rs, responses_websocket.rs, responses.rs, realtime_call.rs, models.rs, mod.rs, memories.rs, images.rs, compact.rs
- src/endpoint/realtime_websocket/: protocol_v2.rs, protocol_v1.rs, protocol_frameless_bidi_tests.rs, protocol_frameless_bidi.rs, protocol_common.rs, 
protocol.rs, mod.rs, methods_v2.rs, methods_v1.rs, methods_frameless_bidi_tests.rs, methods_frameless_bidi.rs, methods_common_tests.rs, methods_common.rs, 
methods.rs

Count: 13 + 3 + 4 + 10 + 14 = 44. ✓

Write WIRE.md at repo root (working directory).

## mesaj

All 44 files read. Writing `WIRE.md`.

### arac: write

```json
{
 "filePath": "C:\\Users\\Bedirhan\\Desktop\\agent\\ab\\par-opencode\\WIRE.md",
 "content": "# WIRE.md — codex-api wire map\n\nOne line per `.rs` file under `codex-api/src/`, grouped by directory.\n\n## codex-api/src/\n\n- `codex-
api/src/lib.rs` — Crate root: declares the wire modules and re-exports the public API surface (clients, auth, error, transport re-exports). `codex-
api/src/lib.rs:1`\n- `codex-api/src/api_bridge.rs` — Translates `ApiError` (wire errors, transport HTTP bodies, Cloudflare/cyber/misalignment codes) into 
`codex_protocol` `CodexErr`. `codex-api/src/api_bridge.rs:21`\n- `codex-api/src/api_bridge_tests.rs` — Tests for `map_api_error`: overloaded/retry/rate-limit 
mapping, 503 body sniffing, Cloudflare blocked responses. `codex-api/src/api_bridge_tests.rs:8`\n- `codex-api/src/auth.rs` — Auth wire layer: `AuthProvider` 
trait that attaches/refreshes auth headers or signs complete requests, plus auth telemetry types. `codex-api/src/auth.rs:30`\n- `codex-api/src/common.rs` — 
Canonical wire payload/event types: `ResponsesApiRequest` inputs, `CompactionInput`, memory summarize payloads, `ResponseEvent` stream enum, `ResponseStream`. 
`codex-api/src/common.rs:47`\n- `codex-api/src/error.rs` — `ApiError` enum: every error shape that can come off the wire (transport, HTTP status, stream, 
context window, quota, rate limits, policy violations, overload). `codex-api/src/error.rs:9`\n- `codex-api/src/files.rs` — Hosted OpenAI file upload wire flow: 
create file, blob upload to Azure, finalize/retry, `sediment://` URIs and 512 MiB limit. `codex-api/src/files.rs:17`\n- `codex-api/src/images.rs` — Image 
generation/edit request and response wire types (prompt, size, quality, background, b64 data). `codex-api/src/images.rs:5`\n- `codex-api/src/provider.rs` — 
Provider endpoint config: `RetryConfig`→`RetryPolicy`, `WireApi` (Responses vs Chat), URL/request builders, websocket URL conversion, Azure detection. `codex-
api/src/provider.rs:44`\n- `codex-api/src/rate_limits.rs` — Parses rate-limit header families (`x-<limit>-primary/secondary-...`, credits, promo messages, rate-
limit event bodies) into `RateLimitSnapshot`s. `codex-api/src/rate_limits.rs:23`\n- `codex-api/src/safety_buffering.rs` — Reads safety-buffering treatment from 
`x-codex-safety-buffering-*` response headers into `SafetyBufferingTreatment`. `codex-api/src/safety_buffering.rs:8`\n- `codex-api/src/search.rs` — Search 
endpoint wire types: `SearchRequest`, text/items input, search/image/click/find/finance/weather/sports commands, filters and settings. `codex-
api/src/search.rs:9`\n- `codex-api/src/telemetry.rs` — Telemetry plumbing for wire calls: `SseTelemetry` and `WebsocketTelemetry` traits and retry-wrapping with
 per-attempt request telemetry. `codex-api/src/telemetry.rs:18`\n\n## codex-api/src/requests/\n\n- `codex-api/src/requests/mod.rs` — Module root for request-
building: re-exports chat translation, session headers, and `Compression`. `codex-api/src/requests/mod.rs:1`\n- `codex-api/src/requests/chat.rs` — Translates a 
Responses-shaped request into an OpenAI-compatible `/chat/completions` body (roles, images, reasoning anchoring, tool calls). `codex-
api/src/requests/chat.rs:19`\n- `codex-api/src/requests/headers.rs` — Builds session headers (`session-id`, `thread-id`) and derives the subagent header from 
`SessionSource`. `codex-api/src/requests/headers.rs:5`\n- `codex-api/src/requests/responses.rs` — Defines the `Compression` option (`None`/`Zstd`) applied to 
Responses wire requests. `codex-api/src/requests/responses.rs:1`\n\n## codex-api/src/sse/\n\n- `codex-api/src/sse/mod.rs` — Module root for SSE handling: re-
exports `spawn_chat_stream`, `spawn_response_stream`, and shared event processing. `codex-api/src/sse/mod.rs:4`\n- `codex-api/src/sse/chat.rs` — Translates 
`/chat/completions` SSE delta frames back into Responses-shaped `ResponseEvent`s (rebuilds whole items at finish boundaries). `codex-api/src/sse/chat.rs:30`\n- 
`codex-api/src/sse/responses.rs` — Consumes Responses API SSE streams: emits header-derived events (model, rate limits, ETag, reasoning-included, turn state) 
and processes SSE events into `ResponseEvent`s. `codex-api/src/sse/responses.rs:36`\n\n## codex-api/src/endpoint/\n\n- `codex-api/src/endpoint/mod.rs` — Module 
root for endpoint clients; re-exports all client types (Compact, Images, Memories, Models, Realtime, Responses, Search, WebSockets). `codex-
api/src/endpoint/mod.rs:12`\n- `codex-api/src/endpoint/compact.rs` — `CompactClient`: POSTs `responses/compact`, captures `x-codex-turn-state`, decodes 
compacted `ResponseItem` history. `codex-api/src/endpoint/compact.rs:39`\n- `codex-api/src/endpoint/images.rs` — `ImagesClient`: POSTs `images/generations` and 
`images/edits`, returns `ImageResponse` plus the imagegen request id header. `codex-api/src/endpoint/images.rs:35`\n- `codex-api/src/endpoint/memories.rs` — 
`MemoriesClient`: POSTs `memories/trace_summarize` and decodes memory summarize outputs. `codex-api/src/endpoint/memories.rs:36`\n- `codex-
api/src/endpoint/models.rs` — `ModelsClient`: GETs `models` with `client_version` query, returns `ModelInfo` list and the `ETag` header. `codex-
api/src/endpoint/models.rs:46`\n- `codex-api/src/endpoint/realtime_call.rs` — `RealtimeCallClient`: creates WebRTC realtime calls (`realtime/calls`/`live`) with
 SDP + session config, parses `Location` for `call_id`. `codex-api/src/endpoint/realtime_call.rs:29`\n- `codex-api/src/endpoint/responses.rs` — 
`ResponsesClient`: streams inference over `/responses`, `/guardian`, `/guardian-classifier`, dispatching Chat vs Responses wire translation. `codex-
api/src/endpoint/responses.rs:112`\n- `codex-api/src/endpoint/responses_websocket.rs` — `ResponsesWebsocketClient`: WebSocket transport for Responses requests, 
pump/ping handling, WS telemetry, rate-limit events, and connection probing. `codex-api/src/endpoint/responses_websocket.rs:50`\n- `codex-
api/src/endpoint/search.rs` — `SearchClient`: POSTs `alpha/search` and decodes the `SearchResponse`. `codex-api/src/endpoint/search.rs:35`\n- `codex-
api/src/endpoint/session.rs` — `EndpointSession`: shared per-client request execution (build request, apply auth, retry policy, telemetry) for unary and 
streaming HTTP calls. `codex-api/src/endpoint/session.rs:19`\n\n## codex-api/src/endpoint/realtime_websocket/\n\n- `codex-
api/src/endpoint/realtime_websocket/mod.rs` — Module root for the realtime WebSocket stack; re-exports client, protocol, and method types. `codex-
api/src/endpoint/realtime_websocket/mod.rs:12`\n- `codex-api/src/endpoint/realtime_websocket/methods.rs` — `RealtimeWebsocketClient`/`Connection`: connects and 
manages the realtime WS session (connect, reconnect, event pump, transcript state, writes). `codex-api/src/endpoint/realtime_websocket/methods.rs:59`\n- `codex-
api/src/endpoint/realtime_websocket/methods_common.rs` — Dispatches outbound message construction across wire adapters (V1, Frameless Bidi, V2) and normalizes 
session mode. `codex-api/src/endpoint/realtime_websocket/methods_common.rs:29`\n- `codex-api/src/endpoint/realtime_websocket/methods_common_tests.rs` — Tests 
for adapter dispatch: frameless session update encoding, `ack_filler`, context-append channel on handoff outputs. `codex-
api/src/endpoint/realtime_websocket/methods_common_tests.rs:15`\n- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs` — Builds frameless bidi
 outbound messages: delegation/session context appends (chunked to 500 bytes) and session update JSON. `codex-
api/src/endpoint/realtime_websocket/methods_frameless_bidi.rs:13`\n- `codex-api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs` — Tests for 
frameless bidi methods: chunking within the wire byte limit and session JSON shape. `codex-
api/src/endpoint/realtime_websocket/methods_frameless_bidi_tests.rs:10`\n- `codex-api/src/endpoint/realtime_websocket/methods_v1.rs` — Builds realtime V1 
outbound messages: conversation item create, handoff append, and Quicksilver session update audio config. `codex-
api/src/endpoint/realtime_websocket/methods_v1.rs:18`\n- `codex-api/src/endpoint/realtime_websocket/methods_v2.rs` — Builds realtime V2 outbound messages: item 
create/function-call output and full session update with modalities, tools, transcription, noise reduction. `codex-
api/src/endpoint/realtime_websocket/methods_v2.rs:39`\n- `codex-api/src/endpoint/realtime_websocket/protocol.rs` — Realtime wire protocol types: parser 
selection (`RealtimeEventParser`), session config/mode, context-append channels, and the `RealtimeOutboundMessage` serde enum. `codex-
api/src/endpoint/realtime_websocket/protocol.rs:15`\n- `codex-api/src/endpoint/realtime_websocket/protocol_common.rs` — Shared realtime event parsing helpers: 
payload/type extraction, session-updated, transcript delta/done, and error events. `codex-api/src/endpoint/realtime_websocket/protocol_common.rs:7`\n- `codex-
api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs` — Parses frameless bidi WS events (`session.started`, `output_audio.delta`, 
`input/output_transcript.added`, `turn.done`, `delegation.created`). `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi.rs:15`\n- `codex-
api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs` — Tests that legacy V1 and frameless bidi delegations decode to the same `RealtimeEvent`s.
 `codex-api/src/endpoint/realtime_websocket/protocol_frameless_bidi_tests.rs:6`\n- `codex-api/src/endpoint/realtime_websocket/protocol_v1.rs` — Parses realtime 
V1 WS events into `RealtimeEvent`s (audio deltas, transcripts, items, handoffs). `codex-api/src/endpoint/realtime_websocket/protocol_v1.rs:12`\n- `codex-
api/src/endpoint/realtime_websocket/protocol_v2.rs` — Parses realtime V2 WS events into `RealtimeEvent`s, including response lifecycle and background-
agent/silence tool calls. `codex-api/src/endpoint/realtime_websocket/protocol_v2.rs:24`\n\nCHECKPOINT REACHED: 44 files\n"
}
```

#### cikti

```
Wrote file successfully.
```

---

### dusunce

Done. Verify count: lib 1 + api_bridge, api_bridge_tests, auth, common, error, files, images, provider, rate_limits, safety_buffering, search, telemetry = 13; 
requests 4; sse 3; endpoint 10; realtime_websocket 14. 13+4+3+10+14 = 44. Glob returned 44 entries. ✓

## mesaj

`WIRE.md` written at the repo root with one line per file (path, what it translates/handles, `file:line` citation), grouped under `src/`, `src/requests/`, 
`src/sse/`, `src/endpoint/`, and `src/endpoint/realtime_websocket/`. No other files were modified.

CHECKPOINT REACHED: 44 files
