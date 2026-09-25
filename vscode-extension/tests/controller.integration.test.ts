// The webview controller against a real app-server, through a bridge backed by the extension's
// SessionHost — the same path as in VS Code minus postMessage. Uses a throwaway SUFFICE_HOME.
//   SUFFICE_BIN=<suffice.exe> npx jest tests/controller.integration.test.ts          (free)
//   + LIVE_TURN=1 ZAI_API_KEY=<key>   also sends one small invisible turn (paid)
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { SessionHost } from "../src/extension/sessionHost";
import { localeByCode, translate } from "../src/shared/i18n";
import type { HostToWebview, InitState, WebviewToHost } from "../src/shared/messages";
import { BLOCK_PROFILE_ID, blockProfileConfig, Controller, isUnelevatedDenyReadRefusal } from "../src/webview/app/controller";
import { appReducer, initialState, type AppAction, type AppState } from "../src/webview/app/state";
import { BaseBridge } from "../src/webview/host";
import { lastAgentText } from "../src/webview/state/chatReducer";

const binary = process.env.SUFFICE_BIN;
const maybe = binary ? describe : describe.skip;
const live = binary && process.env.LIVE_TURN && process.env.ZAI_API_KEY ? test : test.skip;

class HostBackedBridge extends BaseBridge {
  readonly isPreview = false;
  constructor(private readonly host: SessionHost) {
    super();
  }
  deliver(message: HostToWebview) {
    this.receive(message);
  }
  post(message: WebviewToHost): void {
    if (message.type !== "rpc") return;
    this.host.request(message.method, message.params).then(
      (result) => this.receive({ type: "rpcResult", id: message.id, result }),
      (error: Error) => this.receive({ type: "rpcResult", id: message.id, error: { message: error.message } }),
    );
  }
  viewState<T>(): T | undefined {
    return undefined;
  }
  setViewState(): void {}
}

maybe("controller on a real app-server", () => {
  let home: string;
  let work: string;
  let host: SessionHost;
  let bridge: HostBackedBridge;
  let ctl: Controller;
  let state: AppState;
  const notifications: string[] = [];

  beforeAll(async () => {
    home = mkdtempSync(join(tmpdir(), "suffice-ctl-"));
    writeFileSync(join(home, "config.toml"), ['model = "glm-5.3-flash"', 'model_provider = "zai"', 'approval_policy = "never"', ""].join("\n"));
    work = join(home, "work");
    mkdirSync(work);
    writeFileSync(join(work, "notes.txt"), "The secret word is PAPRIKA.\n");
    host = new SessionHost({
      status: () => {},
      notification: (method, params) => {
        notifications.push(method);
        bridge?.deliver({ type: "notification", method, params });
      },
      serverRequest: () => {},
      log: () => {},
    });
    await host.start({ binary: binary!, cwd: work, env: { ...process.env, SUFFICE_HOME: home }, version: "0" });
    bridge = new HostBackedBridge(host);
    const init: InitState = {
      locale: "en", languageSetting: "auto", themeId: "vscode", workspaceFolders: [{ name: "w", path: work }],
      envKeys: [], extensionVersion: "0", prices: {}, preferences: "", attachedSkills: [], invisibleTurns: {}, threadStartCompaction: {},
  learning: { practiced: [], quizzes: {}, explored: [] }, threadBlocks: {},
    };
    state = { ...initialState, init, server: host.currentStatus };
    const dispatch = (action: AppAction) => {
      state = appReducer(state, action);
      ctl.sync(state);
    };
    ctl = new Controller(bridge, dispatch, state, () => (k, p) => translate(localeByCode("en"), k, p));
    ctl.start();
    await ctl.loadCatalog();
  }, 60_000);

  afterAll(() => {
    host?.stop();
    try {
      rmSync(home, { recursive: true, force: true });
    } catch {
      // Windows may hold a file briefly
    }
  });

  test("catalog loads: glm model, plan mode, config", () => {
    expect(state.models.some((m) => m.id === "glm-5.3-flash")).toBe(true);
    expect(state.modes.some((m) => m.mode === "plan")).toBe(true);
    expect(state.config?.model).toBe("glm-5.3-flash");
    expect(state.config?.compactionLimit).toBeNull();
  });

  test("the compaction slider writes the limit and 'model default' clears it", async () => {
    expect(await ctl.writeCompactionLimit(120_000)).toBe(true);
    expect(state.config?.compactionLimit).toBe(120_000);
    expect(await ctl.writeCompactionLimit(null)).toBe(true);
    expect(state.config?.compactionLimit).toBeNull();
  });

  test("a chat that blocks files starts under the session profile that denies them", async () => {
    // Codex's own pipe: a session-scoped [permissions.<id>] profile selected by default_permissions.
    // Either Codex runs the chat under it, or - on the unelevated Windows sandbox, which upstream
    // Codex does not let enforce read denials - it refuses to start rather than run unconfined.
    try {
      const started = (await host.request("thread/start", {
        cwd: work,
        config: blockProfileConfig([join(work, "notes.txt")]),
      })) as { activePermissionProfile?: { id: string; extends?: string | null } | null };
      expect(started.activePermissionProfile?.id).toBe(BLOCK_PROFILE_ID);
      expect(started.activePermissionProfile?.extends).toBe(":workspace");
    } catch (error) {
      expect(isUnelevatedDenyReadRefusal(error)).toBe(true);
    }
  });

  live(
    "an invisible turn runs to completion",
    async () => {
      ctl.toggleInvisible();
      expect(await ctl.send("What is the secret word in notes.txt? Answer with the word only.")).toBe(true);
      const deadline = Date.now() + 120_000;
      while (!notifications.includes("turn/completed") && Date.now() < deadline) await new Promise((r) => setTimeout(r, 250));
      const turn = state.chat.turns[0]!;
      expect(turn.status).toBe("completed");
      const user = turn.items.find((i) => i.type === "userMessage");
      expect(user && user.type === "userMessage").toBe(true);
      // The last answer, not the commentary the model may send first.
      expect(lastAgentText(state.chat)?.toUpperCase()).toContain("PAPRIKA");
      expect(state.init!.invisibleTurns[state.chat.threadId!]).toEqual([turn.id]);
      expect(state.chat.tokenUsage?.total.inputTokens).toBeGreaterThan(0);
    },
    150_000,
  );
});
