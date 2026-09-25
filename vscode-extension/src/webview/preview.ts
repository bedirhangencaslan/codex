// Stand-in host for opening the webview bundle in a plain browser (design review, screenshots).
// Answers protocol calls with sample data and plays back the recorded real turn when a message
// is sent. URL parameters: ?lang=tr&theme=nord&screen=home|chat|history|skills|commands|api|settings
import recorded from "../../tests/fixtures/turn-notifications.json";
import type { HostToWebview, InitState, WebviewToHost } from "../shared/messages";
import { BaseBridge } from "./host";

const params = new URLSearchParams(typeof location === "undefined" ? "" : location.search);
const THREAD = "preview-thread";
const CWD = "C:\\work\\suffice-demo";
const now = Math.floor(Date.now() / 1000);

const sampleThread = (id: string, preview: string, ageSeconds: number, name: string | null = null) => ({
  id,
  sessionId: id,
  forkedFromId: null,
  parentThreadId: null,
  preview,
  ephemeral: false,
  modelProvider: "zai",
  model: "glm-5.3-flash",
  createdAt: now - ageSeconds - 600,
  updatedAt: now - ageSeconds,
  recencyAt: now - ageSeconds,
  status: { type: "notLoaded" },
  path: null,
  cwd: CWD,
  cliVersion: "0.0.0",
  source: "vscode",
  name,
  turns: [],
});

const responses: Record<string, (p: any) => unknown> = {
  "model/list": () => ({
    data: [
      {
        id: "glm-5.3-flash", model: "glm-5.3-flash", displayName: "GLM 5.3 Flash", description: "Fast GLM model via Z.ai",
        hidden: false, isDefault: true, defaultReasoningEffort: "high",
        supportedReasoningEfforts: [
          { reasoningEffort: "low", description: "" }, { reasoningEffort: "high", description: "" }, { reasoningEffort: "max", description: "" },
        ],
      },
      {
        id: "gpt-6-sol", model: "gpt-6-sol", displayName: "GPT-6 Sol", description: "Workhorse model", hidden: false,
        isDefault: false, defaultReasoningEffort: "medium",
        supportedReasoningEfforts: [{ reasoningEffort: "low", description: "" }, { reasoningEffort: "medium", description: "" }, { reasoningEffort: "high", description: "" }],
      },
    ],
    nextCursor: null,
  }),
  "skills/list": () => ({
    data: [
      {
        cwd: CWD,
        errors: [],
        skills: [
          { name: "pdf", description: "Read, merge and fill PDF files.", path: "C:\\Users\\me\\.suffice\\skills\\pdf\\SKILL.md", scope: "user", enabled: true, pluginId: null },
          { name: "release-notes", description: "Draft release notes from merged PRs.", path: `${CWD}\\.suffice\\skills\\release-notes\\SKILL.md`, scope: "repo", enabled: true, pluginId: null },
          { name: "skill-creator", description: "Create a new skill from a description.", path: "<system>/skill-creator/SKILL.md", scope: "system", enabled: false, pluginId: null },
        ],
      },
    ],
  }),
  "thread/list": () => ({
    data: [
      sampleThread("t-1", "Add rate limiting to the API client", 3_600, "Rate limiting"),
      sampleThread("t-2", "Why does the read tool cap at 96K bytes?", 86_400),
      sampleThread("t-3", "Write WIRE.md for codex-api/src", 3 * 86_400, "wire benchmark"),
    ],
    nextCursor: null,
  }),
  "config/read": () => ({
    config: {
      model: "glm-5.3-flash",
      model_provider: "zai",
      model_auto_compact_token_limit: null,
      model_auto_compact_token_limit_scope: null,
      model_context_window: null,
    },
    origins: {},
  }),
  "collaborationMode/list": () => ({
    data: [
      { name: "Default", mode: "default", model: null, reasoning_effort: null },
      { name: "Plan", mode: "plan", model: null, reasoning_effort: "medium" },
    ],
  }),
  "thread/start": () => ({
    thread: sampleThread(THREAD, "", 0),
    model: "glm-5.3-flash",
    modelProvider: "zai",
    cwd: CWD,
    reasoningEffort: "high",
  }),
  "thread/resume": (p) => ({ thread: sampleThread(p.threadId, "", 0), model: "glm-5.3-flash", modelProvider: "zai", cwd: CWD }),
  "fuzzyFileSearch": (p) => ({
    files: ["src/api/client.ts", "src/api/retry.ts", "README.md", "package.json"]
      .filter((f) => f.includes(String(p.query ?? "")))
      .map((path, i) => ({ root: CWD, path, match_type: "file", file_name: path.split("/").pop(), score: 10 - i, indices: [] })),
  }),
  "fs/readDirectory": (p) => {
    const tree: Record<string, Array<[string, boolean]>> = {
      [CWD]: [["src", true], ["tests", true], ["README.md", false], ["package.json", false]],
      [`${CWD}\\src`]: [["api", true], ["index.ts", false]],
      [`${CWD}\\src\\api`]: [["client.ts", false], ["retry.ts", false]],
      [`${CWD}\\tests`]: [["client.test.ts", false]],
    };
    return { entries: (tree[p.path] ?? []).map(([fileName, isDirectory]) => ({ fileName, isDirectory, isFile: !isDirectory })) };
  },
  "account/read": () => ({ account: null, requiresOpenaiAuth: false }),
  "thread/goal/get": () => ({ goal: null }),
};

export class PreviewBridge extends BaseBridge {
  readonly isPreview = true;
  private state: unknown;

  constructor() {
    super();
    // ?lesson=<id> opens that lesson on the Commands screen; ?tab=reference opens the reference.
    if (params.get("lesson") || params.get("tab")) {
      this.state = { commandsTab: params.get("tab") ?? "lessons", openLesson: params.get("lesson") };
    }
  }

  private initState(): InitState {
    return {
      locale: params.get("lang") ?? "en",
      languageSetting: params.get("lang") ?? "auto",
      themeId: params.get("theme") ?? "catppuccin-mocha",
      workspaceFolders: [{ name: "suffice-demo", path: CWD }],
      envKeys: [{ name: "ZAI_API_KEY", stored: true }],
      extensionVersion: "preview",
      prices: {},
      preferences: "",
      attachedSkills: [],
      invisibleTurns: {},
      threadStartCompaction: {},
      learning: params.get("learning") === "some"
        ? { practiced: ["status", "pwd", "rename"], quizzes: { "basics.folder": "pwd", "sessions.unrelated": "compact" }, explored: [] }
        : { practiced: [], quizzes: {}, explored: [] },
      threadBlocks: {},
    };
  }

  /** The initial screen requested by ?screen=, read by the app on start. */
  static initialScreen(): string | null {
    return params.get("screen");
  }

  private emit(message: HostToWebview): void {
    this.receive(message);
  }

  post(message: WebviewToHost): void {
    // Like the real host: state and server status are sent in answer to "ready".
    if (message.type === "ready") {
      this.emit({ type: "init", state: this.initState() });
      this.emit({ type: "server", status: { state: "ready", codexHome: "C:\\Users\\me\\.suffice", userAgent: "suffice/preview", cwd: CWD } });
      return;
    }
    if (message.type === "modelLimits") {
      this.emit({ type: "modelLimitsResult", id: message.id, limits: { "glm-5.3-flash": 1_000_000 } });
      return;
    }
    if (message.type !== "rpc") return;
    if (message.method === "turn/start") {
      this.emit({ type: "rpcResult", id: message.id, result: { turn: { id: "preview-turn", items: [], status: "inProgress", error: null } } });
      this.playRecordedTurn();
      return;
    }
    const answer = responses[message.method];
    setTimeout(() => this.emit({ type: "rpcResult", id: message.id, result: answer ? answer(message.params ?? {}) : {} }), 30);
  }

  private playRecordedTurn(): void {
    const recordedThread = (recorded.find((n) => n.method === "thread/started")?.params as { thread: { id: string } }).thread.id;
    const steps = recorded.filter((n) => !["thread/started", "remoteControl/status/changed"].includes(n.method));
    steps.forEach((n, i) => {
      const text = JSON.stringify(n.params).split(recordedThread).join(THREAD).split("<home>").join(CWD.replace(/\\/g, "\\\\"));
      setTimeout(() => this.emit({ type: "notification", method: n.method, params: JSON.parse(text) }), 120 * (i + 1));
    });
    if (params.get("demo") === "approval") {
      const at = 120 * (steps.length + 1);
      setTimeout(() => {
        this.emit({
          type: "serverRequest",
          requestId: 1,
          method: "item/commandExecution/requestApproval",
          params: { kind: "command", threadId: THREAD, turnId: "t", itemId: "i", startedAtMs: 0, command: "npm install left-pad", reason: "needs network access", environmentId: null },
        });
        this.emit({
          type: "serverRequest",
          requestId: 2,
          method: "item/tool/requestUserInput",
          params: {
            threadId: THREAD, turnId: "t", itemId: "q", isBlocking: true, autoResolutionMs: null,
            questions: [{ id: "db", header: "Database", question: "Which database should the plan target?", isOther: true, isSecret: false, options: [{ label: "SQLite", description: "Single file, no server" }, { label: "Postgres", description: "The production database" }] }],
          },
        });
      }, at);
    }
  }

  viewState<T>(): T | undefined {
    return this.state as T | undefined;
  }

  setViewState<T>(state: T): void {
    this.state = state;
  }
}
