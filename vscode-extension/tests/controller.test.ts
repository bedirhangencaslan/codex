// What the webview sends to the model is the whole point of rule 6: only existing protocol
// fields, and nothing extra when the user chose nothing. These tests drive the controller with a
// fake bridge and check every turn/start it makes.
import { BLOCK_PROFILE_ID, Controller, PERMISSION_PRESETS } from "../src/webview/app/controller";
import { appReducer, initialState, type AppAction, type AppState } from "../src/webview/app/state";
import { BaseBridge } from "../src/webview/host";
import { translate, localeByCode } from "../src/shared/i18n";
import type { HostToWebview, InitState, WebviewToHost } from "../src/shared/messages";

class FakeBridge extends BaseBridge {
  readonly isPreview = false;
  calls: Array<{ method: string; params: any }> = [];
  posts: WebviewToHost[] = [];
  /** fs/readDirectory answers, by directory. */
  tree: Record<string, Array<{ fileName: string; isDirectory: boolean; isFile: boolean }>> = {};
  post(message: WebviewToHost): void {
    this.posts.push(message);
    if (message.type !== "rpc") return;
    this.calls.push({ method: message.method, params: message.params });
    const result: Record<string, unknown> = {
      "thread/start": { thread: { id: "T1", name: null }, cwd: "/w", model: "glm-5.3-flash" },
      "turn/start": { turn: { id: "U-turn" } },
      "config/read": { config: { model_auto_compact_token_limit: 90000 } },
      "thread/goal/set": { goal: { objective: "ship", status: "active" } },
    };
    if (message.method === "fs/readDirectory") result["fs/readDirectory"] = { entries: this.tree[(message.params as { path: string }).path] ?? [] };
    queueMicrotask(() => this.receive({ type: "rpcResult", id: message.id, result: result[message.method] ?? {} } as HostToWebview));
  }
  viewState<T>(): T | undefined {
    return undefined;
  }
  setViewState(): void {}
  turnStarts() {
    return this.calls.filter((c) => c.method === "turn/start").map((c) => c.params);
  }
}

const init: InitState = {
  locale: "en",
  languageSetting: "auto",
  themeId: "vscode",
  workspaceFolders: [{ name: "w", path: "/w" }],
  envKeys: [],
  extensionVersion: "0",
  prices: {},
  preferences: "Answer in Turkish, please.",
  attachedSkills: [],
  invisibleTurns: {},
  threadStartCompaction: {},
  learning: { practiced: [], quizzes: {}, explored: [] }, threadBlocks: {}, threadPreferences: {},
};

function setup(patch: Partial<AppState> = {}) {
  const bridge = new FakeBridge();
  let state: AppState = { ...initialState, init, server: { state: "ready", codexHome: "/h", userAgent: "x", cwd: "/w" }, ...patch };
  const locale = localeByCode("en");
  let ctl: Controller;
  // Like React: the store applies the action; the controller keeps its own copy in step.
  const dispatch = (action: AppAction) => {
    state = appReducer(state, action);
  };
  ctl = new Controller(bridge, dispatch, state, () => (k, p) => translate(locale, k, p));
  return { bridge, ctl, get state() { return state; } };
}

describe("turn/start is exactly what the TUI would send", () => {
  test("a plain message: text only, invisible false, no model/effort/mode/permission overrides", async () => {
    const { bridge, ctl } = setup();
    await ctl.send("hello");
    const [params] = bridge.turnStarts();
    expect(params).toEqual({
      threadId: "T1",
      input: [{ type: "text", text: "hello", text_elements: [] }],
      turnTrigger: "user",
      invisible: false,
    });
  });

  test("the preference goes once, as thread/start.developerInstructions, never with a turn", async () => {
    const env = setup();
    await env.ctl.send("hello");
    await env.ctl.send("again");
    expect(env.bridge.calls.find((c) => c.method === "thread/start")!.params.developerInstructions).toBe(
      "<users_conversation_preferences>\nAnswer in Turkish, please.\n</users_conversation_preferences>",
    );
    expect(JSON.stringify(env.bridge.turnStarts())).not.toContain("Answer in Turkish");
    expect(env.state.init!.threadPreferences).toEqual({ T1: "Answer in Turkish, please." });
  });

  test("no preference: thread/start carries none", async () => {
    const { bridge, ctl } = setup({ init: { ...init, preferences: "  " } });
    await ctl.send("hello");
    expect(bridge.calls.find((c) => c.method === "thread/start")!.params).not.toHaveProperty("developerInstructions");
  });

  test("resuming a chat gives Codex the preference it started with (compaction rebuilds from config)", async () => {
    const { bridge, ctl } = setup({ init: { ...init, threadPreferences: { OLD: "Be brief." } } });
    await ctl.resume("OLD");
    expect(bridge.calls.find((c) => c.method === "thread/resume")!.params.developerInstructions).toBe("<users_conversation_preferences>\nBe brief.\n</users_conversation_preferences>");
  });

  test("attached skills are skill items like $skill; blocked files add nothing to the input", async () => {
    const { bridge, ctl } = setup({ init: { ...init, attachedSkills: [{ name: "pdf", path: "/s/pdf/SKILL.md" }] } });
    ctl.attachFile({ name: "a.env", path: "/w/a.env" });
    await ctl.send("look");
    expect(bridge.turnStarts()[0].input).toEqual([
      { type: "text", text: "look", text_elements: [] },
      { type: "skill", name: "pdf", path: "/s/pdf/SKILL.md" },
    ]);
  });

  test("no blocked files: thread/start carries no config at all", async () => {
    const { bridge, ctl } = setup({ init: { ...init, preferences: "" } });
    await ctl.send("hi");
    expect(bridge.calls.find((c) => c.method === "thread/start")!.params).toEqual({ cwd: "/w" });
  });

  test("blocked files start the chat under a session profile that denies them (Codex's own mechanism)", async () => {
    const env = setup({ init: { ...init, preferences: "" } });
    env.ctl.attachFile({ name: "a.env", path: "/w/a.env" });
    env.ctl.attachFile({ name: "secrets", path: "/w/secrets" });
    await env.ctl.send("one");
    await env.ctl.send("two");
    const starts = env.bridge.calls.filter((c) => c.method === "thread/start");
    expect(starts).toHaveLength(1);
    expect(starts[0]!.params).toEqual({
      cwd: "/w",
      config: {
        default_permissions: BLOCK_PROFILE_ID,
        [`permissions.${BLOCK_PROFILE_ID}`]: {
          extends: ":workspace",
          filesystem: { "/w/a.env": "deny", "/w/secrets": "deny" },
          network: { enabled: true },
        },
      },
    });
    // Nothing per turn: the profile belongs to the thread.
    for (const p of env.bridge.turnStarts()) expect(Object.keys(p).sort()).toEqual(["input", "invisible", "threadId", "turnTrigger"]);
    expect(env.state.init!.threadBlocks).toEqual({ T1: { mode: "block", picks: ["/w/a.env", "/w/secrets"], denied: ["/w/a.env", "/w/secrets"] } });
    // The list stays for the next chat.
    expect(env.state.composer.files.map((f) => f.path)).toEqual(["/w/a.env", "/w/secrets"]);
  });

  test("resuming a chat gives it the same profile back (older bare-list entries too)", async () => {
    const { bridge, ctl } = setup({ init: { ...init, threadBlocks: { OLD: ["/w/a.env"] } } });
    await ctl.resume("OLD");
    const resume = bridge.calls.find((c) => c.method === "thread/resume")!.params;
    expect(resume.config[`permissions.${BLOCK_PROFILE_ID}`].filesystem).toEqual({ "/w/a.env": "deny" });
  });

  const dir = (fileName: string) => ({ fileName, isDirectory: true, isFile: false });
  const file = (fileName: string) => ({ fileName, isDirectory: false, isFile: true });

  test("Select: everything at the picks' levels except the picks and the way to them is denied", async () => {
    const env = setup();
    env.bridge.tree = {
      "/w": [dir("src"), dir("docs"), file("README.md"), file(".env"), file("AGENTS.md")],
      "/w/src": [dir("api"), file("index.ts")],
      "/w/src/api": [file("client.ts"), file("retry.ts")],
    };
    env.ctl.setFileMode("select");
    env.ctl.attachFile({ name: "client.ts", path: "/w/src/api/client.ts" });
    env.ctl.attachFile({ name: "docs", path: "/w/docs" });
    await env.ctl.send("go");
    const start = env.bridge.calls.find((c) => c.method === "thread/start")!.params;
    expect(Object.keys(start.config[`permissions.${BLOCK_PROFILE_ID}`].filesystem).sort()).toEqual(
      ["/w/.env", "/w/README.md", "/w/src/api/retry.ts", "/w/src/index.ts"].sort(),
    );
    // The picked folder is never listed, so its contents stay readable.
    expect(env.bridge.calls.some((c) => c.method === "fs/readDirectory" && c.params.path === "/w/docs")).toBe(false);
    expect(env.state.init!.threadBlocks.T1).toEqual({
      mode: "select",
      picks: ["/w/src/api/client.ts", "/w/docs"],
      denied: expect.arrayContaining(["/w/.env", "/w/README.md", "/w/src/api/retry.ts", "/w/src/index.ts"]),
    });
  });

  test("Select, resumed: the panel comes back in Select mode with the picks", async () => {
    const { ctl, state } = setup({ init: { ...init, threadBlocks: { OLD: { mode: "select", picks: ["/w/a.ts"], denied: ["/w/b.ts"] } } } });
    await ctl.resume("OLD");
    expect(ctl.current.composer.fileMode).toBe("select");
    expect(ctl.current.composer.files.map((f) => f.path)).toEqual(["/w/a.ts"]);
    void state;
  });

  test("large pastes are expanded before sending", async () => {
    const { bridge, ctl } = setup();
    await ctl.send("see [Pasted Content 1200 chars]", new Map([["[Pasted Content 1200 chars]", "x".repeat(1200)]]));
    expect(bridge.turnStarts()[0].input[0].text).toBe(`see ${"x".repeat(1200)}`);
  });

  test("invisible mode sets turn/start.invisible and records the turn", async () => {
    const env = setup();
    const { bridge, ctl } = env;
    ctl.toggleInvisible();
    await ctl.send("side question");
    expect(bridge.turnStarts()[0].invisible).toBe(true);
    expect(env.state.init!.invisibleTurns).toEqual({ T1: ["U-turn"] });
  });

  test("choosing Default on a fresh thread sends no mode (no extra <collaboration_mode> block)", async () => {
    const { bridge, ctl } = setup({ modes: [{ name: "Default", mode: "default", model: null, reasoning_effort: null }] });
    ctl.setMode("default");
    await ctl.send("hi");
    expect(bridge.turnStarts()[0].collaborationMode).toBeUndefined();
  });

  test("Plan, then Default: the switch back is sent once, later turns carry no mode", async () => {
    const { bridge, ctl } = setup({ modes: [{ name: "Plan", mode: "plan", model: null, reasoning_effort: "medium" }] });
    ctl.setMode("plan");
    await ctl.send("a");
    ctl.setMode("default");
    await ctl.send("b");
    await ctl.send("c");
    expect(bridge.turnStarts().map((p) => p.collaborationMode?.mode)).toEqual(["plan", "default", undefined]);
  });

  test("plan mode sends the collaboration mode with built-in instructions (null)", async () => {
    const { bridge, ctl } = setup({ modes: [{ name: "Plan", mode: "plan", model: null, reasoning_effort: "medium" }] });
    ctl.setMode("plan");
    await ctl.send("design it");
    expect(bridge.turnStarts()[0].collaborationMode).toEqual({
      mode: "plan",
      settings: { model: "glm-5.3-flash", reasoning_effort: "medium", developer_instructions: null },
    });
  });

  test("model, effort and a permission preset are passed only when chosen", async () => {
    const { bridge, ctl } = setup();
    ctl.setModel("gpt-6-sol");
    ctl.setEffort("high");
    ctl.setPermission("readOnly");
    await ctl.send("go");
    const p = bridge.turnStarts()[0];
    expect(p.model).toBe("gpt-6-sol");
    expect(p.effort).toBe("high");
    expect(p.approvalPolicy).toBe(PERMISSION_PRESETS.readOnly.approvalPolicy);
    expect(p.sandboxPolicy).toEqual({ type: "readOnly", networkAccess: false });
  });

  test("the thread's starting compaction value is remembered (goal item 11)", async () => {
    const env = setup();
    const { ctl } = env;
    await ctl.send("hi");
    expect(env.state.init!.threadStartCompaction).toEqual({ T1: 90000 });
  });
});

describe("slash commands typed in the composer", () => {
  // Practising a taught command shows a toast that clears itself after 4s.
  beforeEach(() => jest.useFakeTimers({ doNotFake: ["queueMicrotask", "nextTick"] }));
  afterEach(() => jest.useRealTimers());

  test("/invisible toggles without calling the model", async () => {
    const env = setup();
    const { bridge, ctl } = env;
    await ctl.send("/invisible");
    expect(env.state.composer.invisible).toBe(true);
    expect(bridge.turnStarts()).toHaveLength(0);
  });

  test("/plan <text> switches to plan and sends the text", async () => {
    const { bridge, ctl } = setup({ modes: [{ name: "Plan", mode: "plan", model: null, reasoning_effort: null }] });
    await ctl.send("/plan add caching");
    const p = bridge.turnStarts()[0];
    expect(p.input[0].text).toBe("add caching");
    expect(p.collaborationMode.mode).toBe("plan");
  });

  test("/goal <objective> sets the goal", async () => {
    const { bridge, ctl } = setup();
    await ctl.send("/goal ship it");
    expect(bridge.calls.find((c) => c.method === "thread/goal/set")!.params).toEqual({ threadId: "T1", objective: "ship it", status: "active" });
  });

  test("terminal-only commands are explained, not sent", async () => {
    const env = setup();
    const { bridge, ctl } = env;
    await ctl.send("/vim");
    expect(bridge.turnStarts()).toHaveLength(0);
    expect(env.state.chat.notices[0]!.text).toContain("/vim only works in the terminal UI");
  });
});
