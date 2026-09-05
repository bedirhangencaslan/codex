//! Per-request token statistics, so this fork's cost mechanisms can be measured rather than
//! estimated.
//!
//! Nine mechanisms trim what reaches the model, and every one of them is silent. Reasoning
//! retention writes a verdict nobody reads back, the output shrinker leaves a marker inside
//! text, the keep-alive deliberately emits nothing, and none of it appears in the rollout, which
//! records what the harness *stored* rather than what it *sent*. So the savings are estimated
//! from item sizes and the estimates have never met a real session.
//!
//! One line per request fixes that. Each row pairs the provider's billed usage with the
//! [`ContextFilterReport`] for the prompt that produced it, and those two together reconstruct
//! the counterfactual: the prompt an unoptimised harness would have sent is the one that was
//! sent plus everything the report says was removed. No second model call, no second wire
//! format, and nothing on the request changes.
//!
//! Rows are written next to the rollout rather than into it. A new `RolloutItem` variant would
//! propagate into the generated protocol schemas, the TypeScript bindings, the app-server's
//! thread-read surface and the store migrations, and would make measurement part of the
//! resumable-session contract; rollout writing is also governed by `ThreadHistoryMode`, so a
//! history-disabled session would silently lose everything. A sidecar keyed on the same thread
//! id joins from either side and costs none of that.
//!
//! Unlike [`crate::request_density`] this file is append-only, so there is no read-modify-write
//! to serialise and no file lock: one handle is opened for the session and each row is one
//! `write_all`.

use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Mutex as StdMutex;
use std::sync::PoisonError;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::time::Instant;

use chrono::SecondsFormat;
use chrono::Utc;
use codex_features::Feature;
use codex_protocol::openai_models::ReasoningEffort;
use codex_protocol::protocol::TokenUsage;
use codex_utils_absolute_path::AbsolutePathBuf;
use codex_utils_string::approx_token_count;
use serde::Deserialize;
use serde::Serialize;

use crate::client_common::Prompt;
use crate::config::Config;
use crate::context_manager::ContextFilterReport;
use crate::context_manager::measured_item_token_count;
use crate::session::turn_context::TurnContext;

/// Sidecar location, deliberately outside `sessions/`: the compression and pagination
/// migrations walk that tree and have no reason to meet these files.
const DIRECTORY: &str = "analytics";

/// The switches that change the bill. Recording which were live is what makes two runs an A/B
/// rather than two unrelated sessions.
const MEASURED_FEATURES: &[Feature] = &[Feature::PromptCacheKeepAlive, Feature::ReasoningCostModel];

/// Session-wide facts, written once as the first line of the file.
#[derive(Serialize, Deserialize)]
pub struct RequestStatsHeader {
    pub thread_id: String,
    pub started_at: String,
    pub model: String,
    pub provider: String,
    pub reasoning_effort: Option<String>,
    pub codex_version: String,
    pub features: Vec<String>,
    /// The density retention priced against. Held fixed for the session, so it belongs here
    /// rather than on every row.
    pub requests_per_window: f64,
}

/// One request: what it was billed, what was sent, and what was left out of it.
#[derive(Serialize, Deserialize)]
pub struct RequestStatsLine {
    pub sequence: u64,
    pub timestamp: String,
    pub turn_id: String,
    pub response_id: Option<String>,
    pub invisible: bool,

    // Provider truth. This is the entire bill; everything below only attributes it.
    pub input_tokens: i64,
    pub cached_input_tokens: i64,
    pub output_tokens: i64,
    /// The quality dial. Thoughts of a few dozen tokens at `high` effort mean the model is not
    /// thinking, and that the optimisations are not why the answers got worse.
    pub reasoning_output_tokens: i64,
    pub total_tokens: i64,

    // What was sent, measured on the exact `Prompt`. `None` for requests this module never saw
    // armed, such as compaction, which builds its own prompt.
    pub prompt_items: Option<usize>,
    pub prompt_tokens_estimated: Option<i64>,
    /// The fixed-prefix term, and a silent cache killer: an MCP server connecting mid-session
    /// changes the tool list and rewrites the prompt from its first token.
    pub prompt_tool_specs: Option<usize>,

    // The counterfactual. `prompt_tokens_estimated` plus the dropped and shrunk sizes is the
    // prompt an unoptimised harness would have sent.
    /// `null` means the prompt was byte-identical to the previous one and nothing was paid for
    /// it. The distribution of nulls is the direct answer to why the shrinker never fires.
    pub prefix_break: Option<usize>,
    pub prefix_break_tokens: i64,
    pub dropped_invisible_items: usize,
    pub dropped_invisible_tokens: i64,
    pub dropped_reasoning_items: usize,
    pub dropped_reasoning_tokens: i64,
    pub retained_reasoning_items: usize,
    pub retained_reasoning_tokens: i64,
    /// What `horizon()` predicted. The realised horizon is countable from the log, so this is
    /// the one number that says whether the measured density generalises. `None` where no
    /// verdict was frozen, which is every request with no budget left to amortise over.
    pub retention_horizon: Option<i64>,
    /// Join keys, so a renderer can name the tool calls a filter removed.
    pub dropped_call_ids: Vec<String>,

    pub tool_calls: usize,
    /// With `timestamp`, this gives the idle gap before the next request, which is the only
    /// visible signature of the prompt-cache keep-alive.
    pub duration_ms: u64,
}

/// Facts about the request in flight, gathered as they become known.
///
/// The report is produced two frames above the usage that prices it, and `run_sampling_request`
/// rebuilds its input on every retry, so the last arm wins and there is no cancellation to
/// plumb: a superseded record is simply overwritten before anything reads it.
#[derive(Default)]
struct ArmedRequest {
    started_at: Option<Instant>,
    report: ContextFilterReport,
    prompt_items: usize,
    prompt_tokens_estimated: i64,
    prompt_tool_specs: usize,
    response_id: Option<String>,
    tool_calls: usize,
}

#[derive(Default)]
pub(crate) struct RequestStats {
    /// `None` when the feature is off or the file could not be opened, which is what makes
    /// every method below a no-op for ordinary users.
    sink: Option<StdMutex<File>>,
    armed: StdMutex<Option<ArmedRequest>>,
    /// The horizon the turn's retention verdict was priced against. Frozen once per turn, before
    /// the prompt is built, so it lives here rather than on a record the retries replace.
    retention_horizon: StdMutex<Option<i64>>,
    sequence: AtomicU64,
}

impl std::fmt::Debug for RequestStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RequestStats").finish_non_exhaustive()
    }
}

impl RequestStats {
    /// Opens the sidecar for a session, or returns a disabled recorder.
    ///
    /// Sub-agents get their own [`crate::session::Session`] and so their own file. Separate
    /// files mean concurrent helpers never contend; a reader that wants the whole picture has
    /// to look for the child thread ids as well.
    pub(crate) fn open(config: &Config, thread_id: &str, requests_per_window: f64) -> Self {
        if !config.features.enabled(Feature::RequestStats) {
            return Self::default();
        }
        let header = header(config, thread_id, requests_per_window);
        match open_sink(&config.codex_home, thread_id, header) {
            Ok(file) => Self {
                sink: Some(StdMutex::new(file)),
                ..Self::default()
            },
            Err(error) => {
                tracing::warn!("failed to open the request statistics sidecar: {error:#}");
                Self::default()
            }
        }
    }

    fn enabled(&self) -> bool {
        self.sink.is_some()
    }

    /// Adds to the record in flight, if there is one. There is no record on the paths that build
    /// their own prompt, and inventing an empty one there would report a prompt that was never
    /// built rather than admitting the prompt is unknown.
    fn with_armed(&self, edit: impl FnOnce(&mut ArmedRequest)) {
        if !self.enabled() {
            return;
        }
        let mut armed = self.armed.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(armed) = armed.as_mut() {
            edit(armed);
        }
    }

    /// Records what the retention pass priced this turn against, `None` where it declined to
    /// decide because there was no budget left to amortise a retained token over.
    pub(crate) fn record_retention_horizon(&self, horizon: Option<i64>) {
        if !self.enabled() {
            return;
        }
        *self
            .retention_horizon
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = horizon;
    }

    /// Records the history filtering that produced this request's input.
    pub(crate) fn arm(&self, report: ContextFilterReport) {
        if !self.enabled() {
            return;
        }
        let mut armed = self.armed.lock().unwrap_or_else(PoisonError::into_inner);
        *armed = Some(ArmedRequest {
            started_at: Some(Instant::now()),
            report,
            ..ArmedRequest::default()
        });
    }

    /// Records the shape of the prompt as finally assembled, which is the only point where the
    /// parts history never saw — the tool specs and the base instructions — are in one place.
    pub(crate) fn record_prompt(&self, prompt: &Prompt) {
        if !self.enabled() {
            return;
        }
        let tokens_estimated = prompt
            .input
            .iter()
            .map(measured_item_token_count)
            .fold(0i64, i64::saturating_add)
            .saturating_add(estimated_tokens(&prompt.base_instructions.text))
            .saturating_add(
                serde_json::to_string(&prompt.tools).map_or(0, |tools| estimated_tokens(&tools)),
            );
        self.with_armed(|armed| {
            armed.prompt_items = prompt.input.len();
            armed.prompt_tokens_estimated = tokens_estimated;
            armed.prompt_tool_specs = prompt.tools.len();
        });
    }

    /// Records what the model answered with. `tool_calls` is the batching measurement.
    pub(crate) fn record_response(&self, response_id: &str, tool_calls: usize) {
        self.with_armed(|armed| {
            armed.response_id = Some(response_id.to_string());
            armed.tool_calls = tool_calls;
        });
    }

    /// Writes one row and clears the armed slot.
    ///
    /// Called wherever the provider's usage lands, which covers ordinary turns and compaction
    /// alike. Compaction builds its own prompt and never arms, so
    /// its row carries the bill with the prompt columns null rather than with a stale report.
    /// Taking the slot is what guarantees that: a superseded record can never outlive its
    /// request.
    pub(crate) fn flush(&self, turn_context: &TurnContext, token_usage: Option<&TokenUsage>) {
        let Some(sink) = self.sink.as_ref() else {
            return;
        };
        let armed = self
            .armed
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take();
        let Some(token_usage) = token_usage else {
            return;
        };
        let retention_horizon = *self
            .retention_horizon
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let line = line(
            self.sequence.fetch_add(1, Ordering::SeqCst),
            &turn_context.sub_id,
            turn_context.invisible,
            armed,
            retention_horizon,
            token_usage,
        );
        if let Err(error) = append(sink, &line) {
            tracing::warn!("failed to record request statistics: {error:#}");
        }
    }
}

fn header(config: &Config, thread_id: &str, requests_per_window: f64) -> RequestStatsHeader {
    RequestStatsHeader {
        thread_id: thread_id.to_string(),
        started_at: timestamp(),
        model: config.model.clone().unwrap_or_default(),
        provider: config.model_provider_id.clone(),
        reasoning_effort: config
            .model_reasoning_effort
            .as_ref()
            .map(ReasoningEffort::to_string),
        codex_version: env!("CARGO_PKG_VERSION").to_string(),
        features: MEASURED_FEATURES
            .iter()
            .filter(|feature| config.features.enabled(**feature))
            .map(|feature| feature.key().to_string())
            .collect(),
        requests_per_window,
    }
}

fn line(
    sequence: u64,
    turn_id: &str,
    invisible: bool,
    armed: Option<ArmedRequest>,
    retention_horizon: Option<i64>,
    token_usage: &TokenUsage,
) -> RequestStatsLine {
    // A request nobody armed still gets a row, but its prompt columns stay null rather than
    // reporting a zero-sized prompt that was never built.
    let was_armed = armed.is_some();
    let armed = armed.unwrap_or_default();
    let report = armed.report;
    RequestStatsLine {
        sequence,
        timestamp: timestamp(),
        turn_id: turn_id.to_string(),
        response_id: armed.response_id,
        invisible,
        input_tokens: token_usage.input_tokens,
        cached_input_tokens: token_usage.cached_input_tokens,
        output_tokens: token_usage.output_tokens,
        reasoning_output_tokens: token_usage.reasoning_output_tokens,
        total_tokens: token_usage.total_tokens,
        prompt_items: was_armed.then_some(armed.prompt_items),
        prompt_tokens_estimated: was_armed.then_some(armed.prompt_tokens_estimated),
        prompt_tool_specs: was_armed.then_some(armed.prompt_tool_specs),
        prefix_break: report.prefix_break,
        prefix_break_tokens: report.prefix_break_tokens,
        dropped_invisible_items: report.dropped_invisible_items,
        dropped_invisible_tokens: report.dropped_invisible_tokens,
        dropped_reasoning_items: report.dropped_reasoning_items,
        dropped_reasoning_tokens: report.dropped_reasoning_tokens,
        retained_reasoning_items: report.retained_reasoning_items,
        retained_reasoning_tokens: report.retained_reasoning_tokens,
        retention_horizon,
        dropped_call_ids: report.dropped_call_ids,
        tool_calls: armed.tool_calls,
        duration_ms: armed
            .started_at
            .map(|started_at| started_at.elapsed().as_millis().min(u128::from(u64::MAX)) as u64)
            .unwrap_or_default(),
    }
}

/// The same rule the rest of the context accounting uses, so this estimate is comparable with
/// the sizes the report carries.
fn estimated_tokens(text: &str) -> i64 {
    i64::try_from(approx_token_count(text)).unwrap_or(i64::MAX)
}

fn timestamp() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

fn open_sink(
    codex_home: &AbsolutePathBuf,
    thread_id: &str,
    header: RequestStatsHeader,
) -> std::io::Result<File> {
    let directory = codex_home.join(DIRECTORY);
    std::fs::create_dir_all(&directory)?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(directory.join(format!("{thread_id}.jsonl")))?;
    // A resumed session appends to the file it left behind, so the header repeats. It carries
    // the version and the feature set, both of which may have changed in between.
    write_line(&mut file, &header)?;
    Ok(file)
}

fn append(sink: &StdMutex<File>, line: &RequestStatsLine) -> std::io::Result<()> {
    let mut file = sink.lock().unwrap_or_else(PoisonError::into_inner);
    write_line(&mut file, line)
}

fn write_line(file: &mut File, value: &impl Serialize) -> std::io::Result<()> {
    let mut encoded = serde_json::to_vec(value)?;
    encoded.push(b'\n');
    file.write_all(&encoded)
}

#[cfg(test)]
#[path = "request_stats_tests.rs"]
mod tests;
