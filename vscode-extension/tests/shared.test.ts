import { readFileSync } from "node:fs";
import { join } from "node:path";
import {
  AUTO_COMPACT_CLAMP_DENOMINATOR,
  AUTO_COMPACT_CLAMP_NUMERATOR,
  clampedAutoCompactLimit,
  CONTEXT_BASELINE_TOKENS,
  MODEL_FACTS,
} from "../src/shared/catalog";
import { checkCompactionSync } from "../src/shared/compactionSync";
import { computeMeters, contextLeftPercent, formatTokensCompact, formatUsd } from "../src/shared/meters";
import { BUILTIN_PRICES, costOf, parsePrice, priceFor } from "../src/shared/pricing";
import { paletteToCss, THEMES, themeById } from "../src/shared/themes";

const codexRs = join(__dirname, "..", "..", "codex-rs");
const rust = (path: string) => readFileSync(join(codexRs, path), "utf8");

describe("catalog constants mirror the Rust they describe", () => {
  test("CONTEXT_BASELINE_TOKENS = protocol.rs BASELINE_TOKENS", () => {
    const m = /const BASELINE_TOKENS: i64 = ([\d_]+);/.exec(rust("protocol/src/protocol.rs"));
    expect(Number(m![1]!.replace(/_/g, ""))).toBe(CONTEXT_BASELINE_TOKENS);
  });

  test("the 9/10 clamp = openai_models.rs auto_compact_token_limit()", () => {
    const src = rust("protocol/src/openai_models.rs");
    expect(src).toContain(`(context_window * ${AUTO_COMPACT_CLAMP_NUMERATOR}) / ${AUTO_COMPACT_CLAMP_DENOMINATOR}`);
  });

  test("MODEL_FACTS = models.json", () => {
    const catalog = JSON.parse(rust("models-manager/models.json"));
    const models: Array<Record<string, unknown>> = Array.isArray(catalog) ? catalog : catalog.models;
    for (const [slug, facts] of Object.entries(MODEL_FACTS)) {
      const entry = models.find((m) => m.slug === slug)!;
      expect(entry).toBeDefined();
      expect(entry.context_window).toBe(facts.contextWindow);
      expect(entry.auto_compact_token_limit ?? null).toBe(facts.autoCompactTokenLimit);
    }
  });
});

describe("meters use the TUI footer formulas", () => {
  test("context left excludes the 12K baseline", () => {
    expect(contextLeftPercent(12_000, 200_000)).toBe(100);
    expect(contextLeftPercent(106_000, 200_000)).toBe(50);
    expect(contextLeftPercent(250_000, 200_000)).toBe(0);
    expect(contextLeftPercent(5_000, 10_000)).toBe(0);
  });

  const breakdown = (input: number, cached: number, output: number) => ({
    totalTokens: input + output,
    inputTokens: input,
    cachedInputTokens: cached,
    outputTokens: output,
    reasoningOutputTokens: 0,
  });

  test("cached share, next-request split and readiness come from the last request", () => {
    const meters = computeMeters(
      { total: breakdown(50_000, 40_000, 1_000), last: breakdown(10_000, 8_000, 200), modelContextWindow: 200_000 },
      null,
      BUILTIN_PRICES["glm-5.3-flash"],
    );
    expect(meters.cachedPercent).toBe(80);
    expect(meters.nextNewTokens).toBe(2_000);
    expect(meters.nextCachedTokens).toBe(8_000);
    expect(meters.readiness).toBe("instant");
    expect(meters.contextWindow).toBe(200_000);
  });

  test("thread cost = fresh + cached + output from the TOTAL breakdown", () => {
    // report.py: 43,326 fresh / 227,200 cached / 4,584 out -> $0.0078 (rep145)
    const cost = costOf(
      { inputTokens: 43_326 + 227_200, cachedInputTokens: 227_200, outputTokens: 4_584 },
      BUILTIN_PRICES["glm-5.3-flash"]!,
    );
    expect(cost.total).toBeCloseTo(0.0078, 4);
  });

  test("no price, no cost", () => {
    expect(computeMeters(null, 128_000, undefined).cost).toBeNull();
    expect(priceFor("unknown-model", {})).toBeUndefined();
  });

  test("user prices override built-in ones and are validated", () => {
    const user = parsePrice({ input: "0,1", cached: "0.02", output: "0.3" })!;
    expect(user.inputPerMillion).toBeCloseTo(0.1);
    expect(priceFor("glm-5.3-flash", { "glm-5.3-flash": user })).toBe(user);
    expect(parsePrice({ input: "-1", cached: "0", output: "0" })).toBeUndefined();
    expect(parsePrice({ input: "x", cached: "0", output: "0" })).toBeUndefined();
  });

  test("compact formatting matches format_tokens_compact", () => {
    expect(formatTokensCompact(999)).toBe("999");
    expect(formatTokensCompact(1_000)).toBe("1K");
    expect(formatTokensCompact(1_500)).toBe("1.5K");
    expect(formatTokensCompact(12_345)).toBe("12.3K");
    expect(formatTokensCompact(100_000)).toBe("100K");
    expect(formatTokensCompact(2_000_000)).toBe("2M");
    expect(formatUsd(0.0078)).toBe("$0.0078");
    expect(formatUsd(0.5)).toBe("$0.500");
  });
});

describe("compaction sync check (goal item 11)", () => {
  const glm = { contextWindow: 200_000, catalogLimit: 80_000, scope: "total" as const };

  test("the shipped setup is in sync", () => {
    const r = checkCompactionSync({ ...glm, sliderValue: null, threadStartValue: null });
    expect(r).toEqual({ effectiveLimit: 80_000, inSync: true, findings: [] });
  });

  test("a new slider value does not reach the running thread", () => {
    const r = checkCompactionSync({ ...glm, sliderValue: 80_000, threadStartValue: null });
    expect(r.findings).toEqual([{ kind: "thread-frozen", threadValue: null, sliderValue: 80_000 }]);
  });

  test("above 9/10 of the window the total scope clamps", () => {
    const r = checkCompactionSync({ ...glm, sliderValue: 195_000 });
    expect(r.effectiveLimit).toBe(180_000);
    expect(r.findings[0]).toEqual({ kind: "clamped", requested: 195_000, applied: 180_000 });
  });

  test("body_after_prefix compacts unclamped while read budgets clamped", () => {
    const r = checkCompactionSync({ ...glm, scope: "body_after_prefix", sliderValue: 195_000 });
    expect(r.effectiveLimit).toBe(195_000);
    expect(r.findings[0]).toEqual({ kind: "scope-unclamped", requested: 195_000, clampedForRead: 180_000 });
  });

  test("clamp helper: null limit means 9/10 of the window", () => {
    expect(clampedAutoCompactLimit(null, 128_000)).toBe(115_200);
  });
});

describe("themes", () => {
  test("every theme has a unique id and a full palette", () => {
    const ids = THEMES.map((t) => t.id);
    expect(new Set(ids).size).toBe(ids.length);
    const keys = Object.keys(THEMES[0]!.palette).sort();
    for (const theme of THEMES) expect(Object.keys(theme.palette).sort()).toEqual(keys);
    expect(themeById("nope").id).toBe("vscode");
    expect(paletteToCss(themeById("nord").palette)["--sf-surface-raised"]).toBe("#3b4252");
  });
});
