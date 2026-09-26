// The Settings compaction check: does the open chat use the limit set with the slider, and will
// that limit be applied as set? It only reports; it never writes anything.
//
// - The slider writes the limit with a config reload, so an open chat takes it from its next turn;
//   a limit changed elsewhere (another client) can leave an open chat on an older value.
// - For the `total` scope the limit is clamped to 9/10 of the context window; for
//   `body_after_prefix` compaction uses the configured value unclamped while the read tool's
//   budget still uses the clamped one.
import { clampedAutoCompactLimit } from "./catalog";

export type CompactionScope = "total" | "body_after_prefix";

export interface CompactionSyncInput {
  /** Value the slider wrote to config; null = no override (the model's catalog value applies). */
  sliderValue: number | null;
  /** Config value the active thread was started or resumed with; undefined = no active thread. */
  threadStartValue?: number | null;
  scope: CompactionScope;
  contextWindow: number | null;
  /** Catalog auto_compact_token_limit for the model: null = none, undefined = unknown model. */
  catalogLimit: number | null | undefined;
}

export type CompactionSyncFinding =
  | { kind: "thread-frozen"; threadValue: number | null; sliderValue: number | null }
  | { kind: "clamped"; requested: number; applied: number }
  | { kind: "scope-unclamped"; requested: number; clampedForRead: number };

export interface CompactionSyncReport {
  /** Limit a thread started now would use for compaction. */
  effectiveLimit: number | null;
  inSync: boolean;
  findings: CompactionSyncFinding[];
}

export function checkCompactionSync(input: CompactionSyncInput): CompactionSyncReport {
  const findings: CompactionSyncFinding[] = [];
  const requested = input.sliderValue ?? input.catalogLimit ?? null;

  if (input.threadStartValue !== undefined && input.threadStartValue !== input.sliderValue) {
    findings.push({ kind: "thread-frozen", threadValue: input.threadStartValue, sliderValue: input.sliderValue });
  }

  let effectiveLimit: number | null = requested;
  if (input.contextWindow) {
    const clamped = clampedAutoCompactLimit(requested, input.contextWindow);
    if (requested !== null && requested > clamped) {
      if (input.scope === "total") {
        findings.push({ kind: "clamped", requested, applied: clamped });
      } else {
        findings.push({ kind: "scope-unclamped", requested, clampedForRead: clamped });
      }
    }
    effectiveLimit = input.scope === "total" ? clamped : (requested ?? clamped);
  }

  return { effectiveLimit, inSync: findings.length === 0, findings };
}
