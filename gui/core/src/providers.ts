import { AppServerClient } from "./client";
import type {
  CommandEntry,
  DataProvider,
  SkillEntry,
  StyleCard,
  ThreadDetail,
  ThreadSummary,
  Warmth,
  WeekStats,
} from "./types";

/* ------------------------------------------------------------------ mock */

const THREADS: ThreadSummary[] = [
  {
    id: "t-116",
    title: "Reproduce the licences this repository actually ships",
    preview:
      "NOTICE now names OpenCode, WezTerm, path-absolutize and bubblewrap; commercial-licensing.md written…",
    updatedAt: "2026-09-18T10:08:00+03:00",
    costUsd: 0.0311,
    cachedPercent: 91,
    requests: 27,
    compactions: 0,
    warmth: "instant",
    badge: "INSTANT",
  },
  {
    id: "t-115",
    title: "wire: cite all 44 codex-api files in WIRE.md",
    preview: "rep40 — 11 requests, every read bounded, median window 37 lines, zero compactions.",
    updatedAt: "2026-09-17T22:41:00+03:00",
    costUsd: 0.0085,
    cachedPercent: 87,
    requests: 11,
    compactions: 0,
    warmth: "instant",
    badge: "44/44 CITED",
  },
  {
    id: "t-114",
    title: "Keep the prompt cache warm while a tool call runs",
    preview: "KeepAliveHold guard covers the whole dispatch; 420 s interval from the measured Z.ai TTL.",
    updatedAt: "2026-09-16T18:20:00+03:00",
    costUsd: 0.0177,
    cachedPercent: 64,
    requests: 18,
    compactions: 1,
    warmth: "fast",
  },
  {
    id: "t-113",
    title: "Route bulk extraction to a script instead of grep",
    preview: "grep stops at 100 matches; 6 of 10 runs wrote the script anyway and paid twice for 352 values.",
    updatedAt: "2026-09-15T14:02:00+03:00",
    costUsd: 0.0094,
    cachedPercent: 12,
    requests: 8,
    compactions: 0,
    warmth: "cold",
  },
];

const DETAILS: Record<string, ThreadDetail> = {
  "t-116": {
    id: "t-116",
    cachedPercent: 91,
    freshTokens: 21_406,
    cachedTokens: 216_714,
    outputTokens: 9_882,
    keepAlives: 3,
    keepAliveCostUsd: 0.0004,
    timeline: [
      { index: 1, cachedShare: 0 },
      { index: 2, cachedShare: 0.71 },
      { index: 3, cachedShare: 0.84 },
      { index: 4, cachedShare: 0.92 },
      { index: 5, cachedShare: 0.95 },
      { index: 6, cachedShare: 0.96 },
      { index: 27, cachedShare: 0.97 },
    ],
  },
};

const SKILLS: SkillEntry[] = [
  { name: "code-review", description: "review a PR with verified findings", enabled: true, promptTokens: 412, pluginId: null, path: null },
  { name: "babysit-pr", description: "watch CI, fix failures until green", enabled: true, promptTokens: 287, pluginId: null, path: null },
  { name: "codex-pr-body", description: "write a PR body in this repo's voice", enabled: false, promptTokens: 198, pluginId: null, path: null },
  { name: "imagegen", description: "generate images via API", enabled: false, promptTokens: 1104, pluginId: null, path: null },
  { name: "skill-creator", description: "create or update a Suffice skill", enabled: false, promptTokens: 866, pluginId: null, path: null },
  { name: "skill-installer", description: "install skills from a repo", enabled: false, promptTokens: 934, pluginId: null, path: null },
];

const TRIAL_OUTPUTS: Record<string, string> = {
  "/status": "workdir: ~/codex\nmodel: glm-5.3-flash (zai) · effort high\ncache: warm ⚡ · 420 s TTL · 3 keep-alives this session\ntokens: 21,406 fresh · 216,714 cached · 9,882 out\n",
  "/diff": "gui/web/src/App.tsx        | 12 ++++++-----\ngui/core/src/providers.ts  | 48 ++++++++++++++++++++++++++\n2 files changed, 55 insertions(+), 5 deletions(-)\n",
  "/skills": "enabled for ~/codex: code-review, babysit-pr\navailable: codex-pr-body, imagegen, skill-creator, skill-installer\n",
};

const COMMANDS: CommandEntry[] = [
  { name: "/review", description: "review my current changes and find issues" },
  { name: "/compact", description: "summarize conversation to prevent hitting the context limit" },
  { name: "/fork", description: "fork the current chat" },
  { name: "/status", description: "show current session configuration and token usage" },
  { name: "/skills", description: "use skills to improve how Suffice performs specific tasks" },
  { name: "/diff", description: "show git diff (including untracked files)" },
  { name: "/resume", description: "resume a saved chat" },
  { name: "/init", description: "create an AGENTS.md file with instructions for Suffice" },
];

const STYLES: StyleCard[] = [
  { id: "thermal", name: "Termal Enstrüman Pro", tagline: "cache sıcaklığı arayüzün fiziği" },
  { id: "blueprint", name: "Blueprint Defteri", tagline: "milimetrik kağıt üstünde ölçüm günlüğü" },
  { id: "abyss", name: "Abis Terminali", tagline: "çift fosforlu OLED terminal" },
];

export class MockProvider implements DataProvider {
  readonly kind = "mock" as const;
  async listThreads(): Promise<ThreadSummary[]> {
    return THREADS;
  }
  async threadDetail(id: string): Promise<ThreadDetail | null> {
    return DETAILS[id] ?? null;
  }
  async weekStats(): Promise<WeekStats> {
    return { costUsd: 0.4109, avgCachedPercent: 87, freshTokens: 312_000, sessionCount: 14, cacheState: "instant" };
  }
  async listSkills(): Promise<SkillEntry[]> {
    return SKILLS;
  }
  async listCommands(): Promise<CommandEntry[]> {
    return COMMANDS;
  }
  async listStyles(): Promise<StyleCard[]> {
    return STYLES;
  }
  async setSkillEnabled(skill: SkillEntry, enabled: boolean): Promise<boolean> {
    const row = SKILLS.find((s) => s.name === skill.name);
    if (row) row.enabled = enabled;
    return false; // in-memory only; nothing persisted
  }
  async runCommandTrial(command: string, onDelta: (text: string) => void): Promise<void> {
    const body =
      TRIAL_OUTPUTS[command.trim()] ??
      `(örnek çıktı) ${command} geçici oturumda çalıştırıldı; canlı app-server bağlıyken gerçek yanıt burada akar.\n`;
    for (const chunk of body.match(/.{1,18}/gs) ?? []) {
      await new Promise((r) => setTimeout(r, 24));
      onDelta(chunk);
    }
  }
}

/* ------------------------------------------------------------ app-server */

interface WireThread {
  id?: string;
  threadId?: string;
  title?: string | null;
  preview?: string | null;
  updatedAt?: string;
}

interface WireAnalyticsPoint {
  sequence: number;
  inputTokens: number;
  cachedInputTokens: number;
  invisible: boolean;
}

interface WireThreadAnalytics {
  threadId: string;
  model: string | null;
  provider: string | null;
  startedAt: string | null;
  lastTimestamp: string | null;
  requests: number;
  invisibleRequests: number;
  inputTokens: number;
  cachedInputTokens: number;
  outputTokens: number;
  reasoningOutputTokens: number;
  timeline: WireAnalyticsPoint[];
  timelineTruncated: boolean;
}

/** Z.ai list prices per million tokens for glm-5.3-flash, as measured and
 * documented in _sim/FINDINGS.md §1 and reasoning_retention.rs. Cost is only
 * computed when the sidecar header names this provider; anything else renders
 * token counts without a currency figure. */
const ZAI_USD_PER_M = { fresh: 0.075, cached: 0.015, output: 0.25 };

function isZai(provider: string | null): boolean {
  return provider != null && provider.toLowerCase().includes("z.ai");
}

function analyticsCostUsd(a: WireThreadAnalytics): number | null {
  if (!isZai(a.provider)) return null;
  const fresh = Math.max(0, a.inputTokens - a.cachedInputTokens);
  return (fresh * ZAI_USD_PER_M.fresh + a.cachedInputTokens * ZAI_USD_PER_M.cached + a.outputTokens * ZAI_USD_PER_M.output) / 1e6;
}

function cachedPercent(a: WireThreadAnalytics): number | null {
  if (a.inputTokens <= 0) return null;
  return Math.round((a.cachedInputTokens / a.inputTokens) * 100);
}

/** Same thresholds as the TUI's response-speed indicator, applied to the
 * latest visible request's cached share. */
function warmthOf(a: WireThreadAnalytics): Warmth | null {
  const last = [...a.timeline].reverse().find((p) => !p.invisible) ?? a.timeline.at(-1);
  if (!last || last.inputTokens <= 0) return null;
  const share = last.cachedInputTokens / last.inputTokens;
  return share >= 0.8 ? "instant" : share >= 0.4 ? "fast" : "cold";
}

interface WireSkill {
  name: string;
  description?: string | null;
  enabled?: boolean;
  pluginId?: string | null;
  path?: string | null;
}

/** Real data source. Phase 1 wires the endpoints that exist today
 * (`thread/list`, `skills/list`); cost/cache analytics come from the
 * request-stats sidecar in a later PR, so those fields stay null here —
 * the screens render them as "—" rather than inventing numbers. */
export class AppServerProvider implements DataProvider {
  readonly kind = "app-server" as const;
  private analytics = new Map<string, WireThreadAnalytics>();

  private constructor(
    private client: AppServerClient,
    private cwd: string,
  ) {}

  static async connect(url: string, cwd: string): Promise<AppServerProvider> {
    const client = new AppServerClient(url);
    await client.connect();
    return new AppServerProvider(client, cwd);
  }

  async listThreads(): Promise<ThreadSummary[]> {
    const res = await this.client.request<{ data?: WireThread[]; threads?: WireThread[] }>("thread/list", {
      cwd: this.cwd,
    });
    const rows = res.data ?? res.threads ?? [];
    const ids = rows.map((t) => t.id ?? t.threadId ?? "").filter(Boolean);
    await this.loadAnalytics(ids);
    return rows.map((t) => {
      const id = t.id ?? t.threadId ?? "";
      const a = this.analytics.get(id);
      return {
        id,
        title: t.title ?? "(adsız oturum)",
        preview: t.preview ?? "",
        updatedAt: t.updatedAt ?? a?.lastTimestamp ?? "",
        costUsd: a ? analyticsCostUsd(a) : null,
        cachedPercent: a ? cachedPercent(a) : null,
        requests: a ? a.requests : null,
        compactions: null,
        warmth: a ? warmthOf(a) : null,
      };
    });
  }

  private async loadAnalytics(ids: string[]): Promise<void> {
    const missing = ids.filter((id) => !this.analytics.has(id));
    if (missing.length === 0) return;
    try {
      const res = await this.client.request<{ data?: WireThreadAnalytics[] }>("analytics/threadStats", {
        threadIds: missing,
      });
      for (const a of res.data ?? []) this.analytics.set(a.threadId, a);
    } catch (err) {
      // Older servers without the endpoint: keep the fields absent.
      console.warn("analytics/threadStats kullanılamadı:", err);
    }
  }

  async threadDetail(id: string): Promise<ThreadDetail | null> {
    await this.loadAnalytics([id]);
    const a = this.analytics.get(id);
    if (!a) return null;
    const fresh = Math.max(0, a.inputTokens - a.cachedInputTokens);
    return {
      id,
      cachedPercent: cachedPercent(a) ?? 0,
      freshTokens: fresh,
      cachedTokens: a.cachedInputTokens,
      outputTokens: a.outputTokens,
      keepAlives: a.invisibleRequests,
      keepAliveCostUsd: 0,
      timeline: a.timeline
        .filter((p) => !p.invisible && p.inputTokens > 0)
        .map((p) => ({ index: p.sequence, cachedShare: p.cachedInputTokens / p.inputTokens })),
    };
  }

  async weekStats(): Promise<WeekStats> {
    const threads = await this.listThreads();
    const all = [...this.analytics.values()];
    const input = all.reduce((s, a) => s + a.inputTokens, 0);
    const cached = all.reduce((s, a) => s + a.cachedInputTokens, 0);
    const cost = all.reduce((s, a) => s + (analyticsCostUsd(a) ?? 0), 0);
    const cacheStates = threads.map((t) => t.warmth).filter((w): w is Warmth => w != null);
    return {
      costUsd: cost,
      avgCachedPercent: input > 0 ? Math.round((cached / input) * 100) : 0,
      freshTokens: Math.max(0, input - cached),
      sessionCount: threads.length,
      cacheState: cacheStates[0] ?? "cold",
    };
  }

  async listSkills(cwd: string): Promise<SkillEntry[]> {
    const res = await this.client.request<{ data?: Array<{ cwd: string; skills: WireSkill[] }> }>("skills/list", {
      cwds: [cwd],
    });
    const entry = res.data?.[0];
    return (entry?.skills ?? []).map((s) => ({
      name: s.name,
      description: s.description ?? "",
      enabled: s.enabled ?? true,
      promptTokens: 0,
      pluginId: s.pluginId ?? null,
      path: s.path ?? null,
    }));
  }

  async listCommands(): Promise<CommandEntry[]> {
    return new MockProvider().listCommands();
  }

  async listStyles(): Promise<StyleCard[]> {
    return new MockProvider().listStyles();
  }

  /** Documented `skills/config/write` — by path when known, else by name. */
  async setSkillEnabled(skill: SkillEntry, enabled: boolean): Promise<boolean> {
    await this.client.request("skills/config/write", {
      path: skill.path,
      name: skill.path ? null : skill.name,
      enabled,
    });
    return true;
  }

  /** `thread/start {ephemeral:true}` + `turn/start` with exactly the user's
   * command text — the same bytes a TUI user typing the command produces
   * (PROMPT-CHANGE-PROPOSALS.md, Proposal 2's approved-by-construction
   * default). Deltas stream in via notifications until the turn ends. */
  async runCommandTrial(command: string, onDelta: (text: string) => void): Promise<void> {
    const started = await this.client.request<{ thread?: { id?: string }; threadId?: string }>("thread/start", {
      cwd: this.cwd,
      ephemeral: true,
    });
    const threadId = started.thread?.id ?? started.threadId;
    if (!threadId) throw new Error("thread/start kimlik döndürmedi");

    await new Promise<void>((resolve, reject) => {
      const unsub = this.client.onNotification((method, params) => {
        const p = params as { threadId?: string; delta?: unknown; text?: unknown } | undefined;
        if (p?.threadId != null && p.threadId !== threadId) return;
        if (method.includes("agentMessage") && method.endsWith("delta")) {
          const chunk = typeof p?.delta === "string" ? p.delta : typeof p?.text === "string" ? p.text : "";
          if (chunk) onDelta(chunk);
        } else if (method === "turn/completed") {
          unsub();
          resolve();
        } else if (method === "turn/failed" || method === "turn/aborted") {
          unsub();
          reject(new Error("tur tamamlanamadı"));
        }
      });
      this.client
        .request("turn/start", { threadId, input: [{ type: "text", text: command }] })
        .catch((e) => {
          unsub();
          reject(e instanceof Error ? e : new Error(String(e)));
        });
    });
  }
}
