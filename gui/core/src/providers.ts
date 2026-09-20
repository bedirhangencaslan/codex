import { AppServerClient } from "./client";
import type {
  CommandEntry,
  DataProvider,
  SkillEntry,
  StyleCard,
  ThreadDetail,
  ThreadSummary,
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
  { name: "code-review", description: "review a PR with verified findings", enabled: true, promptTokens: 412, pluginId: null },
  { name: "babysit-pr", description: "watch CI, fix failures until green", enabled: true, promptTokens: 287, pluginId: null },
  { name: "codex-pr-body", description: "write a PR body in this repo's voice", enabled: false, promptTokens: 198, pluginId: null },
  { name: "imagegen", description: "generate images via API", enabled: false, promptTokens: 1104, pluginId: null },
  { name: "skill-creator", description: "create or update a Suffice skill", enabled: false, promptTokens: 866, pluginId: null },
  { name: "skill-installer", description: "install skills from a repo", enabled: false, promptTokens: 934, pluginId: null },
];

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
}

/* ------------------------------------------------------------ app-server */

interface WireThread {
  id?: string;
  threadId?: string;
  title?: string | null;
  preview?: string | null;
  updatedAt?: string;
}

interface WireSkill {
  name: string;
  description?: string | null;
  enabled?: boolean;
  pluginId?: string | null;
}

/** Real data source. Phase 1 wires the endpoints that exist today
 * (`thread/list`, `skills/list`); cost/cache analytics come from the
 * request-stats sidecar in a later PR, so those fields stay null here —
 * the screens render them as "—" rather than inventing numbers. */
export class AppServerProvider implements DataProvider {
  readonly kind = "app-server" as const;
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
    return rows.map((t) => ({
      id: t.id ?? t.threadId ?? "",
      title: t.title ?? "(adsız oturum)",
      preview: t.preview ?? "",
      updatedAt: t.updatedAt ?? "",
      costUsd: null,
      cachedPercent: null,
      requests: null,
      compactions: null,
      warmth: null,
    }));
  }

  async threadDetail(): Promise<ThreadDetail | null> {
    return null;
  }

  async weekStats(): Promise<WeekStats> {
    const threads = await this.listThreads();
    return { costUsd: 0, avgCachedPercent: 0, freshTokens: 0, sessionCount: threads.length, cacheState: "cold" };
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
    }));
  }

  async listCommands(): Promise<CommandEntry[]> {
    return new MockProvider().listCommands();
  }

  async listStyles(): Promise<StyleCard[]> {
    return new MockProvider().listStyles();
  }
}
