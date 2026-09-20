//! Read-only view over the request-stats sidecar (`<codex_home>/analytics/`).
//!
//! The sidecar pairs each request's billed usage with what the prompt left
//! out (see `codex-core/src/request_stats.rs`); this surface only aggregates
//! the billed side per thread so a client can render cost and cache-warmth
//! without inventing numbers. It never exposes prompt contents.

use crate::JsonSchema;
use crate::TS;
use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "v2/")]
pub struct AnalyticsThreadStatsParams {
    /// Thread ids to look up. Ids with no sidecar file are omitted from the
    /// response rather than reported as zeros.
    pub thread_ids: Vec<String>,
}

/// One request's billed shape, in file order.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "v2/")]
pub struct AnalyticsRequestPoint {
    pub sequence: u64,
    pub input_tokens: i64,
    pub cached_input_tokens: i64,
    pub invisible: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "v2/")]
pub struct ThreadAnalytics {
    pub thread_id: String,
    /// From the sidecar header; `None` when the header line was unreadable.
    pub model: Option<String>,
    pub provider: Option<String>,
    pub started_at: Option<String>,
    pub last_timestamp: Option<String>,
    pub requests: u64,
    pub invisible_requests: u64,
    pub input_tokens: i64,
    pub cached_input_tokens: i64,
    pub output_tokens: i64,
    pub reasoning_output_tokens: i64,
    pub timeline: Vec<AnalyticsRequestPoint>,
    /// True when the timeline was capped; the token sums above still cover
    /// every request in the file.
    pub timeline_truncated: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "v2/")]
pub struct AnalyticsThreadStatsResponse {
    pub data: Vec<ThreadAnalytics>,
}
