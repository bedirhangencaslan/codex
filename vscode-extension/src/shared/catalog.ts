// Model facts the extension needs that `model/list` does not carry (it has no context window,
// no compaction limit and no prices). Each constant mirrors one line of Rust or of the model
// catalog, and tests/catalog.test.ts reads that source and fails if they drift apart.

/** codex-rs/protocol/src/openai_models.rs `auto_compact_token_limit()`: the limit is clamped to
 * 9/10 of the context window. */
export const AUTO_COMPACT_CLAMP_NUMERATOR = 9;
export const AUTO_COMPACT_CLAMP_DENOMINATOR = 10;

/** codex-rs/protocol/src/protocol.rs `BASELINE_TOKENS`: tokens always present in a request, left
 * out of the "context left" percentage the TUI footer shows. */
export const CONTEXT_BASELINE_TOKENS = 12_000;

export interface ModelFacts {
  contextWindow: number;
  /** Catalog `auto_compact_token_limit`; null means "9/10 of the context window". */
  autoCompactTokenLimit: number | null;
}

/** From codex-rs/models-manager/models.json. */
export const MODEL_FACTS: Record<string, ModelFacts> = {
  "glm-5.3-flash": { contextWindow: 200_000, autoCompactTokenLimit: 80_000 },
};

export function clampedAutoCompactLimit(limit: number | null, contextWindow: number): number {
  const ceiling = Math.floor((contextWindow * AUTO_COMPACT_CLAMP_NUMERATOR) / AUTO_COMPACT_CLAMP_DENOMINATOR);
  return limit === null ? ceiling : Math.min(limit, ceiling);
}
