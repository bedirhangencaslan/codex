// The compaction slider's range and what writing it does to Codex's config.
import { indexCatalog, lookupLimit } from "../src/extension/modelLimits";
import { compactionRange, DEFAULT_COMPACTION_LIMIT } from "../src/webview/app/compaction";
import { Controller } from "../src/webview/app/controller";
import { appReducer, initialState, type AppAction, type AppState } from "../src/webview/app/state";
import { BaseBridge } from "../src/webview/host";
import { localeByCode, translate } from "../src/shared/i18n";
import type { HostToWebview, WebviewToHost } from "../src/shared/messages";

describe("models.dev lookup", () => {
  const limits = indexCatalog({
    zai: { models: { "glm-5.3-flash": { limit: { context: 1_000_000 } } } },
    other: { models: { "glm-5.3-flash": { limit: { context: 131_072 } }, "only-here": { limit: { context: 64_000 } } } },
  });
  test("the provider's own entry wins; a bare id falls back to the first provider listing it", () => {
    expect(lookupLimit(limits, "zai", "glm-5.3-flash")).toBe(1_000_000);
    expect(lookupLimit(limits, "unknown", "only-here")).toBe(64_000);
    expect(lookupLimit(limits, null, "missing")).toBeNull();
  });
});

function state(patch: Partial<AppState> = {}): AppState {
  return {
    ...initialState,
    config: { model: "glm-5.3-flash", modelProvider: "zai", compactionLimit: null, compactionScope: "total", contextWindow: null, windowsSandbox: null },
    modelLimits: { "glm-5.3-flash": 1_000_000 },
    ...patch,
  };
}

describe("compaction range", () => {
  test("up to the model's real window, 80K by default", () => {
    const r = compactionRange(state(), 0);
    expect(r.max).toBe(1_000_000);
    expect(r.value).toBe(DEFAULT_COMPACTION_LIMIT);
    expect(r.catalogWindow).toBe(200_000);
  });
  test("never below the chat's current context + 1", () => {
    const r = compactionRange(state({ config: { ...state().config!, compactionLimit: 30_000 } }), 52_000);
    expect(r.min).toBe(52_001);
    expect(r.value).toBe(52_001);
  });
  test("without models.dev the catalog window is the upper end", () => {
    expect(compactionRange(state({ modelLimits: {} }), 0).max).toBe(200_000);
  });
});

describe("writing the limit", () => {
  class Bridge extends BaseBridge {
    readonly isPreview = false;
    writes: Array<{ keyPath: string; value: unknown }> = [];
    post(m: WebviewToHost) {
      if (m.type !== "rpc") return;
      if (m.method === "config/value/write") this.writes.push(m.params as { keyPath: string; value: unknown });
      queueMicrotask(() => this.receive({ type: "rpcResult", id: m.id, result: {} } as HostToWebview));
    }
    viewState<T>(): T | undefined {
      return undefined;
    }
    setViewState(): void {}
  }
  function setup(s: AppState) {
    const bridge = new Bridge();
    let current = s;
    const ctl = new Controller(bridge, (a: AppAction) => (current = appReducer(current, a)), current, () => (k, p) => translate(localeByCode("en"), k, p));
    return { bridge, ctl };
  }

  test("a limit within Suffice's clamp writes only the limit", async () => {
    const { bridge, ctl } = setup(state());
    await ctl.writeCompactionLimit(120_000);
    expect(bridge.writes.map((w) => w.keyPath)).toEqual(["model_auto_compact_token_limit"]);
  });
  test("a limit past the catalog clamp also gives Codex the model's real window", async () => {
    const { bridge, ctl } = setup(state());
    await ctl.writeCompactionLimit(600_000);
    expect(bridge.writes).toEqual([
      { keyPath: "model_context_window", value: 1_000_000, mergeStrategy: "replace" },
      { keyPath: "model_auto_compact_token_limit", value: 600_000, mergeStrategy: "replace" },
    ].map(({ keyPath, value }) => expect.objectContaining({ keyPath, value })));
  });
  test("coming back under the clamp removes the window this slider wrote", async () => {
    const { bridge, ctl } = setup(state({ config: { ...state().config!, contextWindow: 1_000_000, compactionLimit: 600_000 } }));
    await ctl.writeCompactionLimit(80_000);
    expect(bridge.writes[0]).toEqual(expect.objectContaining({ keyPath: "model_context_window", value: null }));
  });
});
