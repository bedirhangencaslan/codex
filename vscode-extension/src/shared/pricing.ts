// Prices per million tokens and the cost of a token breakdown. The formula is the one this
// fork measures with (suffice-labs report.py, _sim/FINDINGS.md §1): input that was served from
// the provider's cache is billed at the cached rate, the rest of the input at the fresh rate,
// and output (which includes reasoning tokens) at the output rate.

export interface ModelPrice {
  /** USD per million fresh (uncached) input tokens. */
  inputPerMillion: number;
  /** USD per million input tokens served from the prompt cache. */
  cachedInputPerMillion: number;
  /** USD per million output tokens. */
  outputPerMillion: number;
  source: "builtin" | "user";
  /** Where a built-in figure comes from, shown next to it. */
  note?: string;
}

export const BUILTIN_PRICES: Record<string, ModelPrice> = {
  "glm-5.3-flash": {
    inputPerMillion: 0.075,
    cachedInputPerMillion: 0.015,
    outputPerMillion: 0.25,
    source: "builtin",
    note: "Z.ai card this fork measures against (_sim/FINDINGS.md §1)",
  },
};

export interface TokenCounts {
  inputTokens: number;
  cachedInputTokens: number;
  outputTokens: number;
}

export function priceFor(model: string, overrides: Record<string, ModelPrice>): ModelPrice | undefined {
  return overrides[model] ?? BUILTIN_PRICES[model];
}

export interface CostBreakdown {
  fresh: number;
  cached: number;
  output: number;
  total: number;
}

export function costOf(tokens: TokenCounts, price: ModelPrice): CostBreakdown {
  const freshTokens = Math.max(0, tokens.inputTokens - tokens.cachedInputTokens);
  const fresh = (freshTokens * price.inputPerMillion) / 1_000_000;
  const cached = (tokens.cachedInputTokens * price.cachedInputPerMillion) / 1_000_000;
  const output = (tokens.outputTokens * price.outputPerMillion) / 1_000_000;
  return { fresh, cached, output, total: fresh + cached + output };
}

/** Validates a user-entered price row; returns the parsed price or undefined. */
export function parsePrice(input: { input: string; cached: string; output: string }): ModelPrice | undefined {
  const values = [input.input, input.cached, input.output].map((v) => Number(v.replace(",", ".")));
  if (values.some((v) => !Number.isFinite(v) || v < 0)) return undefined;
  const [inputPerMillion, cachedInputPerMillion, outputPerMillion] = values as [number, number, number];
  return { inputPerMillion, cachedInputPerMillion, outputPerMillion, source: "user" };
}
