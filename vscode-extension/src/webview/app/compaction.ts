// The compaction limit slider's range, shared by the meters row and the Settings screen.
//
//   max      the model's real context window (models.dev), else the catalog's
//   min      the chat's current context + 1: a limit below what the chat already holds would
//            compact on the next request, so the slider stops there instead
//   default  80K, the project default the owner chose (also the catalog's value for GLM 5.3)
import { MODEL_FACTS } from "../../shared/catalog";
import type { AppState } from "./state";

export const DEFAULT_COMPACTION_LIMIT = 80_000;
export const MIN_COMPACTION_LIMIT = 10_000;
export const COMPACTION_STEP = 1_000;

export interface CompactionRange {
  model: string | null;
  min: number;
  max: number;
  /** What the slider shows: the configured limit, or the default, kept inside [min, max]. */
  value: number;
  configured: number | null;
  defaultValue: number;
  /** The window Suffice's catalog knows; the 9/10 clamp is taken from it (or from an override). */
  catalogWindow: number | null;
  /** The model's real window (models.dev), falling back to the override or the catalog. */
  realWindow: number | null;
}

export function currentModel(state: AppState): string | null {
  return state.composer.model ?? state.threadModel ?? state.config?.model ?? state.models.find((m) => m.isDefault)?.id ?? null;
}

export function compactionRange(state: AppState, contextTokens: number): CompactionRange {
  const model = currentModel(state);
  const catalogWindow = model ? (MODEL_FACTS[model]?.contextWindow ?? null) : null;
  const realWindow = (model ? state.modelLimits[model] : undefined) ?? state.config?.contextWindow ?? catalogWindow;
  const max = realWindow ?? 200_000;
  const min = Math.min(max, Math.max(MIN_COMPACTION_LIMIT, contextTokens + 1));
  const configured = state.config?.compactionLimit ?? null;
  const defaultValue = Math.min(DEFAULT_COMPACTION_LIMIT, max);
  const value = Math.min(max, Math.max(min, configured ?? defaultValue));
  return { model, min, max, value, configured, defaultValue, catalogWindow, realWindow };
}

export const COMMIT_KEYS = ["ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown", "Home", "End", "PageUp", "PageDown"];
