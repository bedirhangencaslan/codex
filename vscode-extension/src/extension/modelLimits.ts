// Context windows of the models in the picker, from models.dev (a public catalog of model limits).
// model/list carries no context window, and the compaction slider's upper end is the model's real
// window. Fetched by the extension host (the webview has no network access), cached for a day in
// globalState, and looked up by Suffice's provider id first (`zai` for glm-5.3-flash), then by the
// same model id under any provider. No user data is sent: it is one anonymous GET.
import type * as vscode from "vscode";

const SOURCE = "https://models.dev/api.json";
const CACHE_KEY = "suffice.modelLimits";
const CACHE_TTL_MS = 24 * 60 * 60 * 1000;
const FETCH_TIMEOUT_MS = 15_000;

interface Cached {
  fetchedAt: number;
  /** "provider/model" and "model" (lower case) -> context window in tokens. */
  limits: Record<string, number>;
}

type Catalog = Record<string, { models?: Record<string, { limit?: { context?: number } }> }>;

/** Flattens the models.dev catalog into lookup keys. The first provider to list a bare model id wins. */
export function indexCatalog(catalog: Catalog): Record<string, number> {
  const limits: Record<string, number> = {};
  for (const [provider, entry] of Object.entries(catalog)) {
    for (const [model, info] of Object.entries(entry.models ?? {})) {
      const context = info.limit?.context;
      if (typeof context !== "number" || context <= 0) continue;
      limits[`${provider}/${model}`.toLowerCase()] = context;
      const bare = model.toLowerCase();
      if (!(bare in limits)) limits[bare] = context;
    }
  }
  return limits;
}

export function lookupLimit(limits: Record<string, number>, provider: string | null, model: string): number | null {
  const byProvider = provider ? limits[`${provider}/${model}`.toLowerCase()] : undefined;
  return byProvider ?? limits[model.toLowerCase()] ?? null;
}

export class ModelLimits {
  private pending: Promise<Record<string, number>> | null = null;

  constructor(private readonly state: vscode.Memento) {}

  private async all(): Promise<Record<string, number>> {
    const cached = this.state.get<Cached>(CACHE_KEY);
    if (cached && Date.now() - cached.fetchedAt < CACHE_TTL_MS) return cached.limits;
    this.pending ??= (async () => {
      try {
        const response = await fetch(SOURCE, { signal: AbortSignal.timeout(FETCH_TIMEOUT_MS) });
        if (!response.ok) throw new Error(`HTTP ${response.status}`);
        const limits = indexCatalog((await response.json()) as Catalog);
        await this.state.update(CACHE_KEY, { fetchedAt: Date.now(), limits } satisfies Cached);
        return limits;
      } catch {
        // Offline or blocked: an old cache is better than nothing; nothing means "unknown".
        return cached?.limits ?? {};
      } finally {
        this.pending = null;
      }
    })();
    return this.pending;
  }

  async forModels(provider: string | null, models: string[]): Promise<Record<string, number>> {
    const limits = await this.all();
    const result: Record<string, number> = {};
    for (const model of models) {
      const limit = lookupLimit(limits, provider, model);
      if (limit !== null) result[model] = limit;
    }
    return result;
  }
}
