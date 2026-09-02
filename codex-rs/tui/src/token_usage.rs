//! TUI token usage models and display formatting.

use std::fmt;

use codex_protocol::num_format::format_with_separators;
use serde::Deserialize;
use serde::Serialize;

const BASELINE_TOKENS: i64 = 12000;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenUsage {
    pub input_tokens: i64,
    pub cached_input_tokens: i64,
    pub output_tokens: i64,
    pub reasoning_output_tokens: i64,
    pub total_tokens: i64,
}

impl TokenUsage {
    pub fn is_zero(&self) -> bool {
        self.total_tokens == 0
    }

    pub(crate) fn cached_input(&self) -> i64 {
        self.cached_input_tokens.max(0)
    }

    pub(crate) fn non_cached_input(&self) -> i64 {
        (self.input_tokens - self.cached_input()).max(0)
    }

    pub(crate) fn blended_total(&self) -> i64 {
        (self.non_cached_input() + self.output_tokens.max(0)).max(0)
    }

    /// Returns the raw `total_tokens` value. For `last_token_usage`, this is the latest active
    /// context size; for `total_token_usage`, this is the accumulated session total.
    pub(crate) fn tokens_in_context_window(&self) -> i64 {
        self.total_tokens
    }

    /// Fraction (0.0-1.0) of this usage window's input that was served from
    /// already-warm server-side context. Higher values correspond to visibly
    /// faster responses and a lower bill; returns `None` before the first request.
    pub(crate) fn context_readiness(&self) -> Option<f64> {
        let input = self.input_tokens.max(0);
        (input > 0).then(|| self.cached_input().min(input) as f64 / input as f64)
    }

    pub(crate) fn percent_of_context_window_remaining(&self, context_window: i64) -> i64 {
        if context_window <= BASELINE_TOKENS {
            return 0;
        }
        let effective_window = context_window - BASELINE_TOKENS;
        let used = (self.tokens_in_context_window() - BASELINE_TOKENS).max(0);
        let remaining = (effective_window - used).max(0);
        ((remaining as f64 / effective_window as f64) * 100.0)
            .clamp(0.0, 100.0)
            .round() as i64
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct TokenUsageInfo {
    pub(crate) total_token_usage: TokenUsage,
    pub(crate) last_token_usage: TokenUsage,
    pub(crate) model_context_window: Option<i64>,
}

impl fmt::Display for TokenUsage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Token usage: total={} input={}{} output={}{}",
            format_with_separators(self.blended_total()),
            format_with_separators(self.non_cached_input()),
            if self.cached_input() > 0 {
                format!(
                    " (+ {} cached)",
                    format_with_separators(self.cached_input())
                )
            } else {
                String::new()
            },
            format_with_separators(self.output_tokens),
            if self.reasoning_output_tokens > 0 {
                format!(
                    " (reasoning {})",
                    format_with_separators(self.reasoning_output_tokens)
                )
            } else {
                String::new()
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_readiness_is_none_before_first_request() {
        let usage = TokenUsage::default();
        assert_eq!(usage.context_readiness(), None);
    }

    #[test]
    fn context_readiness_is_full_when_all_input_is_warm() {
        let usage = TokenUsage {
            input_tokens: 200,
            cached_input_tokens: 200,
            ..TokenUsage::default()
        };
        let ratio = usage.context_readiness().expect("ratio");
        assert!((ratio - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn context_readiness_is_partial_for_mixed_input() {
        let usage = TokenUsage {
            input_tokens: 1000,
            cached_input_tokens: 250,
            ..TokenUsage::default()
        };
        let ratio = usage.context_readiness().expect("ratio");
        assert!((ratio - 0.25).abs() < f64::EPSILON);
    }
}
