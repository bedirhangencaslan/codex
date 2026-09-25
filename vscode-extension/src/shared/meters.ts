// The chat indicators, computed exactly as the TUI footer computes them so both interfaces
// show the same numbers for the same thread:
//   context left  — codex-rs/protocol/src/protocol.rs percent_of_context_window_remaining
//                   (BASELINE_TOKENS excluded, based on the LAST request's total tokens)
//   cached share  — codex-rs/tui/src/bottom_pane/footer.rs ContextTokenBreakdown
//   speed label   — codex-rs/tui/src/token_usage.rs context_readiness
// and the cost, which the TUI does not show for Z.ai, from the same token counts.
import { CONTEXT_BASELINE_TOKENS } from "./catalog";
import { costOf, type CostBreakdown, type ModelPrice, type TokenCounts } from "./pricing";

export interface Breakdown extends TokenCounts {
  totalTokens: number;
  reasoningOutputTokens: number;
}

export interface Usage {
  total: Breakdown;
  last: Breakdown;
  modelContextWindow: number | null;
}

export function contextLeftPercent(lastTotalTokens: number, contextWindow: number): number {
  if (contextWindow <= CONTEXT_BASELINE_TOKENS) return 0;
  const effective = contextWindow - CONTEXT_BASELINE_TOKENS;
  const used = Math.max(0, lastTotalTokens - CONTEXT_BASELINE_TOKENS);
  const remaining = Math.max(0, effective - used);
  return Math.round(Math.min(100, Math.max(0, (remaining / effective) * 100)));
}

export type Readiness = "instant" | "fast" | "warming";

export interface Meters {
  /** Percentage of the context window in use (100 - left), null when the window is unknown. */
  contextUsedPercent: number | null;
  contextTokens: number;
  contextWindow: number | null;
  /** Cached share of the last request's input, 0..100, null before the first request. */
  cachedPercent: number | null;
  nextNewTokens: number;
  nextCachedTokens: number;
  readiness: Readiness | null;
  /** Cost of the whole thread so far, null when the model has no known price. */
  cost: CostBreakdown | null;
}

export function computeMeters(usage: Usage | null, fallbackWindow: number | null, price: ModelPrice | undefined): Meters {
  if (!usage) {
    return {
      contextUsedPercent: fallbackWindow ? 0 : null,
      contextTokens: 0,
      contextWindow: fallbackWindow,
      cachedPercent: null,
      nextNewTokens: 0,
      nextCachedTokens: 0,
      readiness: null,
      cost: price ? { fresh: 0, cached: 0, output: 0, total: 0 } : null,
    };
  }
  const window = usage.modelContextWindow ?? fallbackWindow;
  const last = usage.last;
  const newTokens = Math.max(0, last.inputTokens - last.cachedInputTokens);
  const cached = last.cachedInputTokens;
  const denominator = newTokens + cached;
  const ratio = last.inputTokens > 0 ? cached / last.inputTokens : 0;
  return {
    contextUsedPercent: window ? 100 - contextLeftPercent(last.totalTokens, window) : null,
    contextTokens: last.totalTokens,
    contextWindow: window,
    cachedPercent: denominator > 0 ? Math.round((cached / denominator) * 100) : null,
    nextNewTokens: newTokens,
    nextCachedTokens: cached,
    readiness: last.inputTokens > 0 ? (ratio >= 0.8 ? "instant" : ratio >= 0.4 ? "fast" : "warming") : null,
    cost: price ? costOf(usage.total, price) : null,
  };
}

/** Compact token count like the TUI's format_tokens_compact: 2 decimals under 10, 1 under 100. */
export function formatTokensCompact(value: number): string {
  const v = Math.max(0, Math.trunc(value));
  if (v < 1_000) return String(v);
  const units: Array<[number, string]> = [
    [1e12, "T"],
    [1e9, "B"],
    [1e6, "M"],
    [1e3, "K"],
  ];
  for (const [size, suffix] of units) {
    if (v >= size) {
      const scaled = v / size;
      const digits = scaled < 10 ? 2 : scaled < 100 ? 1 : 0;
      let text = scaled.toFixed(digits);
      if (text.includes(".")) text = text.replace(/0+$/, "").replace(/\.$/, "");
      return `${text}${suffix}`;
    }
  }
  return String(v);
}

export function formatUsd(value: number): string {
  if (value === 0) return "$0";
  if (value < 0.01) return `$${value.toFixed(4)}`;
  return `$${value.toFixed(value < 1 ? 3 : 2)}`;
}
