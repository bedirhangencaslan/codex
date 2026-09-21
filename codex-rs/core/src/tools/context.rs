use crate::context_manager::tool_output::CondenseReport;
use crate::context_manager::tool_output::condense_exec_output;
use crate::context_manager::tool_output::condense_exec_output_lossless;
use crate::original_image_detail::sanitize_original_image_detail;
use crate::session::session::Session;
use crate::session::step_context::StepContext;
use crate::session::turn_context::TurnContext;
use crate::turn_diff_tracker::TurnDiffTracker;
use crate::unified_exec::format_output_omission_marker;
use crate::unified_exec::resolve_max_tokens;
use codex_protocol::ResponseItemId;
use codex_protocol::mcp::CallToolResult;
use codex_protocol::models::FunctionCallOutputBody;
use codex_protocol::models::FunctionCallOutputContentItem;
use codex_protocol::models::FunctionCallOutputPayload;
use codex_protocol::models::ResponseInputItem;
use codex_protocol::models::ResponseItem;
use codex_protocol::models::function_call_output_content_items_to_text;
use codex_tools::LoadableToolSpec;
use codex_tools::ToolName;
use codex_utils_audio::estimate_audio_token_count;
use codex_utils_output_truncation::TruncationPolicy;
use codex_utils_output_truncation::approx_token_count;
use codex_utils_output_truncation::formatted_truncate_text;
use codex_utils_output_truncation::truncate_function_output_payload;
use codex_utils_output_truncation::truncate_text;
use codex_utils_output_truncation::with_serialization_allowance;
use serde::Serialize;
use serde_json::Value as JsonValue;
use std::num::NonZeroUsize;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

pub use codex_tools::ToolOutput;
pub use codex_tools::ToolPayload;

pub(crate) fn boxed_tool_output<T>(output: T) -> Box<dyn ToolOutput>
where
    T: ToolOutput + 'static,
{
    Box::new(output)
}

pub type SharedTurnDiffTracker = Arc<Mutex<TurnDiffTracker>>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ToolCallSource {
    Direct,
    DirectPlaintextMessage,
    CodeMode {
        /// Runtime cell that issued the nested tool request.
        cell_id: String,
        /// Code-mode's per-cell tool invocation id. This is useful for
        /// debugging the JS/runtime bridge, but it is not the Suffice tool call id
        /// because the runtime id only needs to be unique within one cell.
        runtime_tool_call_id: String,
    },
}

#[derive(Clone)]
pub struct ToolInvocation {
    pub session: Arc<Session>,
    // TODO(sayan): Remove this compatibility field once handlers use `step_context.turn`.
    pub turn: Arc<TurnContext>,
    pub(crate) step_context: Arc<StepContext>,
    pub cancellation_token: CancellationToken,
    pub tracker: SharedTurnDiffTracker,
    pub call_id: String,
    pub tool_name: ToolName,
    pub source: ToolCallSource,
    pub payload: ToolPayload,
}

/// Identity of the model call that started a tool invocation, retained across code-mode waits.
#[derive(Clone)]
pub(crate) struct ToolCallOrigin {
    /// Best-effort lookup: invocations without a matching history item still
    /// retain their known window ID.
    pub(crate) item_id: Option<ResponseItemId>,
    pub(crate) window_id: String,
}

impl ToolInvocation {
    /// Returns the item and window that requested this call or started its code-mode cell.
    pub(crate) async fn originating_call(&self) -> Option<ToolCallOrigin> {
        if let ToolCallSource::CodeMode { cell_id, .. } = &self.source {
            return self
                .session
                .services
                .code_mode_service
                .cell_originating_call(&codex_code_mode::CellId::new(cell_id.clone()));
        }

        let item_id = self
            .session
            .clone_history()
            .await
            .raw_items()
            .rev()
            .find_map(|item| match item {
                ResponseItem::FunctionCall { id, call_id, .. }
                | ResponseItem::CustomToolCall { id, call_id, .. }
                    if call_id == &self.call_id =>
                {
                    id.clone()
                }
                _ => None,
            });
        Some(ToolCallOrigin {
            item_id,
            window_id: self.session.current_window_id().await,
        })
    }
}

#[derive(Clone, Debug)]
pub struct McpToolOutput {
    pub result: CallToolResult,
    pub tool_input: JsonValue,
    // Keep the original metadata for hooks; this flag only controls analytics capture.
    pub(crate) result_metadata_capture_allowed: bool,
    pub wall_time: Duration,
    pub original_image_detail_supported: bool,
    pub truncation_policy: TruncationPolicy,
}

impl ToolOutput for McpToolOutput {
    fn log_output(&self) -> String {
        // Logging has its own budget; do not first apply the model-context budget.
        let output = self.result.log_output();
        let wall_time_seconds = self.wall_time.as_secs_f64();
        let header = format!("Wall time: {wall_time_seconds:.4} seconds\nOutput:");
        if output.is_empty() {
            header
        } else {
            format!("{header}\n{output}")
        }
    }

    fn success_for_logging(&self) -> bool {
        self.result.success()
    }

    fn fallback_token_limit_override(&self) -> Option<usize> {
        Some(with_serialization_allowance(self.truncation_policy).token_budget())
    }

    fn to_response_item(&self, call_id: &str, _payload: &ToolPayload) -> ResponseInputItem {
        ResponseInputItem::FunctionCallOutput {
            call_id: call_id.to_string(),
            output: self.response_payload(),
        }
    }

    fn code_mode_result(&self, payload: &ToolPayload) -> JsonValue {
        self.result.code_mode_result(payload)
    }

    fn tool_result_metadata(&self) -> Option<&JsonValue> {
        if !self.result_metadata_capture_allowed {
            return None;
        }
        self.result.meta.as_ref()
    }

    fn post_tool_use_input(&self, _payload: &ToolPayload) -> Option<JsonValue> {
        Some(self.tool_input.clone())
    }

    fn post_tool_use_response(&self, _call_id: &str, _payload: &ToolPayload) -> Option<JsonValue> {
        serde_json::to_value(&self.result).ok()
    }
}

impl McpToolOutput {
    fn response_payload(&self) -> FunctionCallOutputPayload {
        let mut payload = self.result.as_function_call_output_payload();
        if let Some(items) = payload.content_items_mut() {
            sanitize_original_image_detail(self.original_image_detail_supported, items);
        }

        let wall_time_seconds = self.wall_time.as_secs_f64();
        let header = format!("Wall time: {wall_time_seconds:.4} seconds\nOutput:");

        match &mut payload.body {
            FunctionCallOutputBody::Text(text) => {
                if text.is_empty() {
                    *text = header;
                } else {
                    *text = format!("{header}\n{text}");
                }
            }
            FunctionCallOutputBody::ContentItems(items) => {
                items.insert(0, FunctionCallOutputContentItem::InputText { text: header });
            }
        }

        // History receives this budget in tokens. Code Mode keeps the raw result.
        truncate_function_output_payload(
            &mut payload,
            with_serialization_allowance(self.truncation_policy),
            estimate_audio_token_count,
        );
        payload
    }
}

#[derive(Clone)]
pub struct ToolSearchOutput {
    pub tools: Vec<LoadableToolSpec>,
}

impl ToolOutput for ToolSearchOutput {
    fn log_output(&self) -> String {
        let tools = self
            .tools
            .iter()
            .map(|tool| {
                serde_json::to_value(tool).unwrap_or_else(|err| {
                    JsonValue::String(format!("failed to serialize tool_search output: {err}"))
                })
            })
            .collect();
        JsonValue::Array(tools).to_string()
    }

    fn success_for_logging(&self) -> bool {
        true
    }

    fn to_response_item(&self, call_id: &str, _payload: &ToolPayload) -> ResponseInputItem {
        ResponseInputItem::ToolSearchOutput {
            call_id: call_id.to_string(),
            status: "completed".to_string(),
            execution: "client".to_string(),
            tools: self
                .tools
                .iter()
                .map(|tool| {
                    serde_json::to_value(tool).unwrap_or_else(|err| {
                        JsonValue::String(format!("failed to serialize tool_search output: {err}"))
                    })
                })
                .collect(),
        }
    }
}

pub struct FunctionToolOutput {
    pub body: Vec<FunctionCallOutputContentItem>,
    pub success: Option<bool>,
    pub post_tool_use_response: Option<JsonValue>,
}

impl FunctionToolOutput {
    pub fn from_text(text: String, success: Option<bool>) -> Self {
        Self {
            body: vec![FunctionCallOutputContentItem::InputText { text }],
            success,
            post_tool_use_response: None,
        }
    }

    pub fn from_content(
        content: Vec<FunctionCallOutputContentItem>,
        success: Option<bool>,
    ) -> Self {
        Self {
            body: content,
            success,
            post_tool_use_response: None,
        }
    }

    pub fn into_text(self) -> String {
        function_call_output_content_items_to_text(&self.body).unwrap_or_default()
    }
}

impl ToolOutput for FunctionToolOutput {
    fn log_output(&self) -> String {
        function_call_output_content_items_to_text(&self.body).unwrap_or_default()
    }

    fn success_for_logging(&self) -> bool {
        self.success.unwrap_or(true)
    }

    fn to_response_item(&self, call_id: &str, payload: &ToolPayload) -> ResponseInputItem {
        function_tool_response(call_id, payload, self.body.clone(), self.success)
    }

    fn post_tool_use_response(&self, _call_id: &str, _payload: &ToolPayload) -> Option<JsonValue> {
        self.post_tool_use_response.clone()
    }
}

pub struct ApplyPatchToolOutput {
    pub text: String,
}

impl ApplyPatchToolOutput {
    pub fn from_text(text: String) -> Self {
        Self { text }
    }
}

impl ToolOutput for ApplyPatchToolOutput {
    fn log_output(&self) -> String {
        self.text.clone()
    }

    fn success_for_logging(&self) -> bool {
        true
    }

    fn to_response_item(&self, call_id: &str, payload: &ToolPayload) -> ResponseInputItem {
        function_tool_response(
            call_id,
            payload,
            vec![FunctionCallOutputContentItem::InputText {
                text: self.text.clone(),
            }],
            Some(true),
        )
    }

    fn post_tool_use_response(&self, _call_id: &str, _payload: &ToolPayload) -> Option<JsonValue> {
        Some(JsonValue::String(self.text.clone()))
    }

    fn code_mode_result(&self, _payload: &ToolPayload) -> JsonValue {
        JsonValue::Object(serde_json::Map::new())
    }
}

pub struct AbortedToolOutput {
    pub message: String,
}

impl ToolOutput for AbortedToolOutput {
    fn log_output(&self) -> String {
        self.message.clone()
    }

    fn success_for_logging(&self) -> bool {
        false
    }

    fn to_response_item(&self, call_id: &str, payload: &ToolPayload) -> ResponseInputItem {
        match payload {
            ToolPayload::ToolSearch { .. } => ResponseInputItem::ToolSearchOutput {
                call_id: call_id.to_string(),
                status: "completed".to_string(),
                execution: "client".to_string(),
                tools: Vec::new(),
            },
            _ => function_tool_response(
                call_id,
                payload,
                vec![FunctionCallOutputContentItem::InputText {
                    text: self.message.clone(),
                }],
                /*success*/ None,
            ),
        }
    }
}

/// Ceiling for one `exec_command` result, in bytes.
///
/// `truncation_policy * 1.2` alone is 120,000 bytes for this model, while a `read` call may emit
/// 53,333 - so one `rg` sweep could put 2.25x the largest possible read into the window, and the
/// window, not the harness cut, is what compaction watches. Measured: a corpus-wide `rg` returned
/// 102,233 bytes, 43% of everything that entered the window in that run, and the model discarded it
/// as "truncated and disordered" while it went on being resent on every later request.
///
/// The number is OpenCode's. It applies one cap to every tool output - `MAX_BYTES = 51200`,
/// `MAX_LINES = 2000` in its bundle - and uses the same 51,200 for its read tool, so a shell result
/// can never cost more window than a file read. That symmetry is the point; this is not a new
/// restriction so much as the one already imposed on `read` finally reaching the other door.
const MAX_EXEC_OUTPUT_BYTES: usize = 51_200;

/// How much larger the body's budget has to be than the line naming the spill file.
///
/// The notice is only worth its bytes when what it points at is worth more than what it cost,
/// and on a small budget it is not: an eighth of the model's window spent on a path is a bad
/// trade against the output the path was meant to make recoverable.
const SPILL_NOTICE_BUDGET_RATIO: usize = 8;

#[derive(Debug, Clone, PartialEq)]
pub struct ExecCommandToolOutput {
    pub event_call_id: String,
    pub chunk_id: String,
    pub wall_time: Duration,
    /// Raw bytes returned for this unified exec call before any truncation.
    pub raw_output: Vec<u8>,
    pub truncation_policy: TruncationPolicy,
    pub max_output_tokens: Option<usize>,
    pub process_id: Option<i32>,
    pub exit_code: Option<i32>,
    pub original_token_count: Option<usize>,
    /// Bytes omitted by the output collection cap before model-facing truncation.
    pub output_omitted_bytes: Option<NonZeroUsize>,
    pub hook_command: Option<String>,
    /// Where to write the output the model's budget is about to cut, so it can read the rest.
    ///
    /// `None` disables the whole mechanism and returns the response byte for byte to what it
    /// was before it existed. That is what makes the switch cheap to reason about: a caller
    /// that does not opt in cannot be changed by this, and the profiles that cannot read the
    /// file simply never get a path (see the handler).
    pub spill_dir: Option<PathBuf>,
}

impl ToolOutput for ExecCommandToolOutput {
    fn log_output(&self) -> String {
        // The telemetry budget must not inherit the model's output-token limit.
        let mut output = String::from_utf8_lossy(&self.raw_output).into_owned();
        if let Some(omitted_bytes) = self.output_omitted_bytes {
            let marker = format_output_omission_marker(omitted_bytes.get());
            if !output.contains(&marker) {
                output = format!("{marker}\n{output}");
            }
        }
        // Telemetry records what the command actually printed, so it carries no condense
        // notice: nothing was condensed on this path.
        format!(
            "{}\n{output}",
            self.response_header(&CondenseReport::default(), None)
        )
    }

    fn success_for_logging(&self) -> bool {
        true
    }

    fn to_response_item(&self, call_id: &str, payload: &ToolPayload) -> ResponseInputItem {
        function_tool_response(
            call_id,
            payload,
            vec![FunctionCallOutputContentItem::InputText {
                text: self.response_text(payload),
            }],
            Some(true),
        )
    }

    fn post_tool_use_id(&self, call_id: &str) -> String {
        if self.event_call_id.is_empty() {
            call_id.to_string()
        } else {
            self.event_call_id.clone()
        }
    }

    fn post_tool_use_input(&self, _payload: &ToolPayload) -> Option<JsonValue> {
        self.hook_command
            .as_ref()
            .map(|command| serde_json::json!({ "command": command }))
    }

    fn post_tool_use_response(&self, _call_id: &str, _payload: &ToolPayload) -> Option<JsonValue> {
        if self.process_id.is_some() || self.hook_command.is_none() {
            return None;
        }

        // A post-tool-use hook is shown what the command printed, not what the model was
        // shown: a hook that greps for a line the harness condensed away would misfire.
        let raw = String::from_utf8_lossy(&self.raw_output);
        Some(JsonValue::String(
            self.truncated_output_with_policy(&raw, self.model_output_policy()),
        ))
    }

    fn code_mode_result(&self, _payload: &ToolPayload) -> JsonValue {
        #[derive(Serialize)]
        struct UnifiedExecCodeModeResult {
            #[serde(skip_serializing_if = "Option::is_none")]
            chunk_id: Option<String>,
            wall_time_seconds: f64,
            #[serde(skip_serializing_if = "Option::is_none")]
            exit_code: Option<i32>,
            #[serde(skip_serializing_if = "Option::is_none")]
            session_id: Option<i32>,
            #[serde(skip_serializing_if = "Option::is_none")]
            original_token_count: Option<usize>,
            output: String,
        }

        // Noise only. This path used to serialize the raw bytes, so escape sequences and git's
        // line-ending warnings reached the model here after being filtered out everywhere else.
        // The lossy stages stay off: the caller is code the model wrote, and `harness_notice`
        // would be a line of prose to a parser. There is therefore nothing to announce, which
        // is why this struct has no field for it.
        let (condensed, _report) = self.condensed_output_lossless();
        let result = UnifiedExecCodeModeResult {
            chunk_id: (!self.chunk_id.is_empty()).then(|| self.chunk_id.clone()),
            wall_time_seconds: self.wall_time.as_secs_f64(),
            exit_code: self.exit_code,
            session_id: self.process_id,
            original_token_count: self.original_token_count,
            output: match self.max_output_tokens {
                Some(max_tokens) => self
                    .truncated_output_with_policy(&condensed, TruncationPolicy::Tokens(max_tokens)),
                None => condensed,
            },
        };

        serde_json::to_value(result).unwrap_or_else(|err| {
            JsonValue::String(format!("failed to serialize exec result: {err}"))
        })
    }
}

impl ExecCommandToolOutput {
    fn model_output_policy(&self) -> TruncationPolicy {
        let requested_policy = TruncationPolicy::Tokens(resolve_max_tokens(self.max_output_tokens));
        if requested_policy.byte_budget() < self.truncation_policy.byte_budget() {
            requested_policy
        } else {
            self.truncation_policy
        }
    }

    /// The raw output under a token budget, for tests that assert on truncation alone.
    ///
    /// Test-only since `code_mode_result` started condensing: both model-facing paths now go
    /// through [`Self::condensed_output`], and nothing in the product wants the raw bytes with a
    /// budget applied. The cfg mirrors its only callers, `unified_exec/mod_tests.rs`, which
    /// `unified_exec/mod.rs` gates on `unix` -- without the same gate this is dead code on
    /// Windows.
    #[cfg(all(test, unix))]
    pub(crate) fn truncated_output(&self, max_tokens: usize) -> String {
        let text = String::from_utf8_lossy(&self.raw_output);
        self.truncated_output_with_policy(&text, TruncationPolicy::Tokens(max_tokens))
    }

    /// Applies the byte/token budget to `text`, which is the raw output for callers that want
    /// it verbatim and the condensed output on the path to the model.
    fn truncated_output_with_policy(&self, text: &str, policy: TruncationPolicy) -> String {
        let Some(omitted_bytes) = self.output_omitted_bytes else {
            return formatted_truncate_text(text, policy);
        };

        let marker = format_output_omission_marker(omitted_bytes.get());
        if text.len() <= policy.byte_budget() {
            return if text.contains(&marker) {
                text.to_string()
            } else {
                format!("{marker}\n{text}")
            };
        }

        let original_token_count = self
            .original_token_count
            .unwrap_or_else(|| approx_token_count(text));
        let truncated = truncate_text(text, policy);
        let omission_notice = if truncated.contains(&marker) {
            String::new()
        } else {
            format!("{marker}\n")
        };
        format!(
            "Warning: truncated output (original token count: {original_token_count})\n{omission_notice}\n{truncated}"
        )
    }

    fn response_header(&self, condensed: &CondenseReport, spill_path: Option<&Path>) -> String {
        let mut sections = Vec::new();

        if !self.chunk_id.is_empty() {
            sections.push(format!("Chunk ID: {}", self.chunk_id));
        }

        let wall_time_seconds = self.wall_time.as_secs_f64();
        sections.push(format!("Wall time: {wall_time_seconds:.4} seconds"));

        if let Some(exit_code) = self.exit_code {
            sections.push(format!("Process exited with code {exit_code}"));
        }

        if let Some(process_id) = &self.process_id {
            sections.push(format!("Process running with session ID {process_id}"));
        }

        if let Some(original_token_count) = self.original_token_count {
            sections.push(format!("Original token count: {original_token_count}"));
        }

        // The header is where the harness already speaks for itself, so the notice that it
        // altered the output belongs here rather than in a marker of its own.
        if let Some(summary) = condensed.summary() {
            sections.push(summary);
        }

        // OpenCode's sentence, and deliberately nothing but the path. What to do with the file
        // is said once in the tool description, where the session pays for it on the first
        // request and reads it from cache thereafter; saying it again on every truncated
        // command would charge for the same steer once per occurrence.
        if let Some(path) = spill_path {
            sections.push(Self::spill_notice(path));
        }

        sections.push("Output:".to_string());
        sections.join("\n")
    }

    fn spill_notice(path: &Path) -> String {
        format!("Full output saved to: {}", path.display())
    }

    /// `policy` tightened by `bytes`, keeping the unit it was expressed in.
    ///
    /// The unit matters beyond arithmetic: it is what decides whether the marker left in the
    /// text reads `tokens truncated` or `chars truncated`, so converting to bytes here would
    /// change what the model is told about its own output.
    fn policy_less(policy: TruncationPolicy, bytes: usize) -> TruncationPolicy {
        match policy {
            TruncationPolicy::Bytes(budget) => {
                TruncationPolicy::Bytes(budget.saturating_sub(bytes))
            }
            TruncationPolicy::Tokens(budget) => TruncationPolicy::Tokens(
                budget.saturating_sub(TruncationPolicy::Bytes(bytes).token_budget()),
            ),
        }
    }

    /// The body's share of the response, once the header has taken its own.
    fn output_budget(truncation_policy: TruncationPolicy, header_bytes: usize) -> usize {
        with_serialization_allowance(truncation_policy)
            .byte_budget()
            .min(MAX_EXEC_OUTPUT_BYTES)
            .saturating_sub(header_bytes.saturating_add(/*rhs*/ 1))
    }

    /// Where this call's untruncated output would go, if anywhere. Writes nothing.
    ///
    /// Named from the call rather than the clock for two reasons: rendering the same result
    /// twice rewrites one file instead of leaving a second copy behind, and the notice's length
    /// is knowable before the body's budget is fixed, which is what lets the budget reserve
    /// room for it rather than be exceeded by it.
    fn spill_path(&self) -> Option<PathBuf> {
        let dir = self.spill_dir.as_ref()?;
        let stem = if self.chunk_id.is_empty() {
            self.event_call_id.as_str()
        } else {
            self.chunk_id.as_str()
        };
        let mut name: String = stem
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
            .collect();
        if name.is_empty() {
            name.push_str("output");
        }
        // `.txt` because `read` refuses the extensions it treats as binary, and this file exists
        // only to be read back through it.
        Some(dir.join(format!("tool_{name}.txt")))
    }

    /// Writes the untruncated output, returning the path only if the model can be told of it.
    ///
    /// Every failure is silent and yields `None`, which drops the notice and leaves the response
    /// exactly as it was before this mechanism existed. A command's output must not be lost
    /// because the copy beside it could not be saved, and a path the model cannot open is worse
    /// than no path at all - that is the `rg -c` failure mode, where the harness advertised a
    /// capability the environment did not have.
    fn write_spill(path: &Path, text: &str) -> Option<PathBuf> {
        std::fs::create_dir_all(path.parent()?).ok()?;
        std::fs::write(path, text).ok()?;
        Some(path.to_path_buf())
    }

    /// The output as the model should read it: every stage, including the ones that drop
    /// content and say so.
    fn condensed_output(&self, payload: &ToolPayload) -> (String, CondenseReport) {
        let raw = String::from_utf8_lossy(&self.raw_output);
        let arguments = match payload {
            ToolPayload::Function { arguments } => arguments.as_str(),
            _ => "",
        };
        match condense_exec_output(arguments, self.exit_code, self.process_id, &raw) {
            Some((condensed_text, report)) => (condensed_text, report),
            None => (raw.into_owned(), CondenseReport::default()),
        }
    }

    /// The output as a program should parse it: noise removed, nothing dropped.
    ///
    /// Code mode's caller is code the model wrote, which may count lines or match the text
    /// exactly, so the lossy stages are left out even though they announce themselves.
    fn condensed_output_lossless(&self) -> (String, CondenseReport) {
        let raw = String::from_utf8_lossy(&self.raw_output);
        match condense_exec_output_lossless(&raw) {
            Some((condensed_text, report)) => (condensed_text, report),
            None => (raw.into_owned(), CondenseReport::default()),
        }
    }

    fn response_text(&self, payload: &ToolPayload) -> String {
        // Condensing runs before truncation so that noise never spends the model's budget: a
        // recursive listing whose dependency tree is dropped first fits whole, where the same
        // listing truncated first loses its own files to make room for `node_modules`.
        let (text, condensed) = self.condensed_output(payload);

        let bare_header = self.response_header(&condensed, None);
        // The notice's room is taken out of the budget before the body is measured against it,
        // so naming the file can never be what pushes the response over. When there is no spill
        // directory this reserves nothing and the arithmetic below is byte for byte what it was.
        // Which of the two limits actually binds decides where the notice's bytes have to come
        // from. `max_output_tokens` is usually the smaller one, and reserving only against the
        // serialization budget would leave the response longer with a notice than without.
        let bare_budget = Self::output_budget(self.truncation_policy, bare_header.len())
            .min(self.model_output_policy().byte_budget());
        // Asked against the budget the response would have had anyway, so that reserving room is
        // never itself the reason a body stops fitting. Without this an output landing in the
        // notice-wide band just under the cap would be cut and spilled where it used to arrive
        // whole, and the model would spend a `read` recovering the tail of something it had been
        // given for free - the mechanism causing the extra request it exists to prevent.
        // Two reasons to write the file, not one. The budget cutting the body is the original;
        // the other is a lossy condensing stage having dropped lines, which the budget knows
        // nothing about. Since a stage now *selects* what matters out of a failure, what it
        // decided against has to be recoverable, or the selection would be a guess the model
        // cannot check.
        let would_spill = text.len() > bare_budget || condensed.summary().is_some();
        // A path is ~115 bytes: nothing against the default budget, a large share of a small one.
        // Under a tight `max_output_tokens` the notice would buy the model a file by taking away
        // the very output it was asking about, so below this ratio there is no notice and no
        // file. Same reasoning as `MIN_LOSSY_TOKENS` in `tool_output.rs`: a line that announces
        // something costs tokens and has to earn them.
        let candidate = self.spill_path().filter(|_| would_spill).filter(|path| {
            Self::spill_notice(path)
                .len()
                .saturating_add(1)
                .saturating_mul(SPILL_NOTICE_BUDGET_RATIO)
                <= bare_budget
        });
        let reserved = candidate
            .as_deref()
            .map_or(0, |path| Self::spill_notice(path).len().saturating_add(1));
        let output_budget =
            Self::output_budget(self.truncation_policy, bare_header.len() + reserved);
        let mut policy = Self::policy_less(self.model_output_policy(), reserved);
        let mut output = self.truncated_output_with_policy(&text, policy);

        // History applies this same serialization budget to the complete response.
        // Reserve room for metadata, warning headers, and the truncation marker so
        // it does not truncate an already-truncated output a second time.
        while output.len() > output_budget && policy.byte_budget() > 0 {
            policy = Self::policy_less(policy, output.len() - output_budget);
            output = self.truncated_output_with_policy(&text, policy);
        }

        // Asked of the final policy, so this is the same question `truncated_output_with_policy`
        // answered rather than a guess about the string it returned. The file is written once,
        // here, and not inside the loop above, which would have written it on every pass.
        let spilled = (text.len() > policy.byte_budget() || condensed.summary().is_some())
            .then(|| {
                candidate.as_deref().and_then(|path| {
                    // The losslessly normalized output, not the condensed one. The file exists so
                    // the model can recover what it was not shown, and after a lossy stage the
                    // condensed text is precisely what it *was* shown. Escape sequences and
                    // carriage-return overwrites are still stripped, because `read` refuses a file
                    // that sniffs as binary and raw terminal bytes do.
                    let (full, _) = self.condensed_output_lossless();
                    Self::write_spill(path, &full)
                })
            })
            .flatten();
        let header = match spilled.as_deref() {
            Some(path) => self.response_header(&condensed, Some(path)),
            None => bare_header,
        };

        format!("{header}\n{output}")
    }
}

fn function_tool_response(
    call_id: &str,
    payload: &ToolPayload,
    body: Vec<FunctionCallOutputContentItem>,
    success: Option<bool>,
) -> ResponseInputItem {
    let body = match body.as_slice() {
        [FunctionCallOutputContentItem::InputText { text }] => {
            FunctionCallOutputBody::Text(text.clone())
        }
        _ => FunctionCallOutputBody::ContentItems(body),
    };

    if matches!(payload, ToolPayload::Custom { .. }) {
        return ResponseInputItem::CustomToolCallOutput {
            call_id: call_id.to_string(),
            name: None,
            output: FunctionCallOutputPayload { body, success },
        };
    }

    ResponseInputItem::FunctionCallOutput {
        call_id: call_id.to_string(),
        output: FunctionCallOutputPayload { body, success },
    }
}

#[cfg(test)]
#[path = "context_tests.rs"]
mod tests;
