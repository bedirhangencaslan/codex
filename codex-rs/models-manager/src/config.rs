use codex_protocol::config_types::Personality;
use codex_protocol::openai_models::ModelsResponse;
use codex_protocol::openai_models::ToolMode;

#[derive(Debug, Clone, Default)]
pub struct ModelsManagerConfig {
    pub model_context_window: Option<i64>,
    pub model_auto_compact_token_limit: Option<i64>,
    pub tool_output_token_limit: Option<usize>,
    pub base_instructions: Option<String>,
    pub personality: Option<Personality>,
    pub model_catalog: Option<ModelsResponse>,
    /// Whether the code-mode host is installed. With the tool mode the model asks for, it decides
    /// whether `exec` is offered, and so which of the instructions' code-mode lines apply.
    pub code_mode_host_available: bool,
    /// The tool mode `features.code_mode_only` / `features.code_mode` ask for. It applies only to
    /// a model whose catalog entry leaves `tool_mode` unset, the same order `requested_tool_mode`
    /// in core resolves them in.
    pub config_tool_mode: Option<ToolMode>,
}
