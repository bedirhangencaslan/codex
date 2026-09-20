/** Warmth is derived from the latest request's cached share, mirroring the
 * TUI's response-speed indicator thresholds (>=0.8 instant, >=0.4 fast). */
export type Warmth = "instant" | "fast" | "cold";

export interface ThreadSummary {
  id: string;
  title: string;
  preview: string;
  updatedAt: string;
  costUsd: number | null;
  cachedPercent: number | null;
  requests: number | null;
  compactions: number | null;
  warmth: Warmth | null;
  badge?: string;
}

export interface RequestStat {
  index: number;
  /** 0..1 share of input tokens served from the provider prompt cache. */
  cachedShare: number;
}

export interface ThreadDetail {
  id: string;
  cachedPercent: number;
  freshTokens: number;
  cachedTokens: number;
  outputTokens: number;
  keepAlives: number;
  keepAliveCostUsd: number;
  timeline: RequestStat[];
}

export interface SkillEntry {
  name: string;
  description: string;
  enabled: boolean;
  /** Tokens this skill's instruction block adds to every request prompt. */
  promptTokens: number;
  pluginId: string | null;
}

export interface CommandEntry {
  name: string;
  description: string;
}

export interface WeekStats {
  costUsd: number;
  avgCachedPercent: number;
  freshTokens: number;
  sessionCount: number;
  cacheState: Warmth;
}

export interface StyleCard {
  id: string;
  name: string;
  tagline: string;
}

/** Read-only data surface the screens consume. Implementations: MockProvider
 * (fixture data) and AppServerProvider (JSON-RPC to `suffice app-server`). */
export interface DataProvider {
  readonly kind: "mock" | "app-server";
  listThreads(): Promise<ThreadSummary[]>;
  threadDetail(id: string): Promise<ThreadDetail | null>;
  weekStats(): Promise<WeekStats>;
  listSkills(cwd: string): Promise<SkillEntry[]>;
  listCommands(): Promise<CommandEntry[]>;
  listStyles(): Promise<StyleCard[]>;
}
