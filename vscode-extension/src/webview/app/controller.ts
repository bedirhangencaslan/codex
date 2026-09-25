// Every action the screens can take, in one place. Each one is a documented app-server call
// (through the host) or a host message; nothing here reimplements what the server does.
import type { ServerNotification } from "@protocol/ServerNotification";
import type { ReasoningEffort } from "@protocol/ReasoningEffort";
import type { SandboxPolicy } from "@protocol/v2/SandboxPolicy";
import type { AskForApproval } from "@protocol/v2/AskForApproval";
import type { SkillMetadata } from "@protocol/v2/SkillMetadata";
import type { UserInput } from "@protocol/v2/UserInput";
import type { JsonValue } from "@protocol/serde_json/JsonValue";
import type { ThreadGoalStatus } from "@protocol/v2/ThreadGoalStatus";
import { AppServerSession } from "../../protocol/session";
import type { TurnStartParamsWithMode } from "../../protocol/experimental";
import type { CompactionScope } from "../../shared/compactionSync";
import type { MessageKey, Params } from "../../shared/i18n";
import { clampedAutoCompactLimit } from "../../shared/catalog";
import { compactionRange } from "./compaction";
import { EMPTY_PROGRESS, lessonSets, recordPracticed, type LearningProgress } from "../../shared/lessons";
import type { AttachedSkill, FileAccessMode, HostToWebview, InitState, PersistedState, ThreadFileAccess } from "../../shared/messages";
import type { ModelPrice } from "../../shared/pricing";
import { findCommand, parseSlashInput } from "../../shared/slashCommands";
import { lastAgentText } from "../state/chatReducer";
import type { HostBridge } from "../host";
import { appReducer, type AppAction, type AppState, type PermissionPreset, type Screen } from "./state";

/** codex_utils_approval_presets, as the TUI's /permissions offers them. */
export const PERMISSION_PRESETS: Record<PermissionPreset, { approvalPolicy: AskForApproval; sandboxPolicy: SandboxPolicy }> = {
  readOnly: { approvalPolicy: "on-request", sandboxPolicy: { type: "readOnly", networkAccess: false } },
  auto: {
    approvalPolicy: "on-request",
    sandboxPolicy: { type: "workspaceWrite", writableRoots: [], networkAccess: false, excludeTmpdirEnvVar: false, excludeSlashTmp: false },
  },
  full: { approvalPolicy: "never", sandboxPolicy: { type: "dangerFullAccess" } },
};

/** TUI chat_composer.rs LARGE_PASTE_CHAR_THRESHOLD. */
export const LARGE_PASTE_CHARS = 1000;

type Translate = (key: MessageKey, params?: Params) => string;

/**
 * The upstream refusal to start a chat whose profile denies reads on the unelevated Windows sandbox
 * (it cannot enforce them and will not run unconfined; windows-sandbox-rs / sandboxing windows.rs).
 */
export function isUnelevatedDenyReadRefusal(error: unknown): boolean {
  const text = error instanceof Error ? error.message : String(error);
  return /unelevated/i.test(text) && /deny-read/i.test(text);
}

/** Model-facing tag around the conversation preference, in the style of Codex's other context
 * sections (`<skills_instructions>`, `<permissions instructions>`). Owner-approved; English on
 * purpose, it is prompt text, not interface text. Only sent with a non-empty preference. */
export const PREFERENCE_TAG = "users_conversation_preferences";

/** `developerInstructions` for a chat's preference, or undefined when there is none. */
export function preferenceInstructions(preference: string | undefined): string | undefined {
  const text = preference?.trim();
  return text ? `<${PREFERENCE_TAG}>\n${text}\n</${PREFERENCE_TAG}>` : undefined;
}

/** The permission profile a chat that blocks files runs under. */
export const BLOCK_PROFILE_ID = "suffice-block";

/**
 * `thread/start.config` (and `thread/resume.config`) for a chat that blocks `paths`. It is Codex's
 * own mechanism, the shape its app-server tests use (thread_settings_update.rs): a session-scoped
 * `[permissions.<id>]` profile plus `default_permissions` selecting it. Nothing is written to the
 * user's config.toml, so the TUI and other chats are untouched.
 *
 * The profile extends `:workspace` (the only built-in a profile with deny entries can extend) and
 * keeps the network on; each blocked path is a `deny` entry, so Codex's enforcement refuses it for
 * `exec_command`, `read`, `view_image`, `apply_patch`, `grep` and `glob` alike. Codex lists the
 * denied paths to the model in its permissions instructions (approved by the owner).
 */
export function blockProfileConfig(paths: string[]): Record<string, JsonValue> {
  return {
    default_permissions: BLOCK_PROFILE_ID,
    [`permissions.${BLOCK_PROFILE_ID}`]: {
      extends: ":workspace",
      filesystem: Object.fromEntries(paths.map((path) => [path, "deny"])),
      network: { enabled: true },
    },
  };
}

export class Controller {
  readonly session: AppServerSession;
  private state: AppState;

  constructor(
    readonly bridge: HostBridge,
    private readonly render: (action: AppAction) => void,
    initial: AppState,
    private readonly t: () => Translate,
  ) {
    this.session = new AppServerSession(bridge);
    this.state = initial;
  }

  /**
   * Applies an action to the controller's own state at once (React only applies it at the next
   * render), then hands it to React. Both run the same appReducer, so they cannot disagree, and a
   * sequence like "/plan <text>" (switch mode, then send) sees the switch.
   */
  private dispatch(action: AppAction): void {
    this.state = appReducer(this.state, action);
    this.render(action);
  }

  /** Re-aligns with React's state after a render. */
  sync(state: AppState): void {
    this.state = state;
  }

  get current(): AppState {
    return this.state;
  }

  start(): () => void {
    const off = this.bridge.onMessage((m) => this.onHostMessage(m));
    this.bridge.post({ type: "ready" });
    return off;
  }

  private onHostMessage(m: HostToWebview): void {
    switch (m.type) {
      case "init":
        this.dispatch({ type: "init", state: m.state });
        return;
      case "stateChanged":
        this.dispatch({ type: "initPatch", patch: m.patch });
        return;
      case "server":
        this.dispatch({ type: "server", status: m.status });
        if (m.status.state === "ready") void this.loadCatalog();
        return;
      case "notification": {
        const notification = { method: m.method, params: m.params } as ServerNotification;
        this.dispatch({ type: "chat", action: { type: "notification", notification } });
        if (m.method === "skills/changed") void this.loadSkills();
        return;
      }
      case "serverRequest":
        this.dispatch({ type: "serverRequest", request: { requestId: m.requestId, method: m.method, params: m.params } });
        return;
      case "command":
        if (m.command === "newThread") this.newThread();
        else this.go("home");
        return;
      case "rpcResult":
        return;
    }
  }

  // --- navigation / host -----------------------------------------------------------------
  go(screen: Screen): void {
    this.dispatch({ type: "screen", screen });
  }
  toast(text: string | null): void {
    this.dispatch({ type: "toast", text });
    if (text) setTimeout(() => this.dispatch({ type: "toast", text: null }), 4000);
  }
  private fail(error: unknown): void {
    if (isUnelevatedDenyReadRefusal(error)) {
      this.toast(this.t()("panel.filesElevated"));
      return;
    }
    this.toast(error instanceof Error ? error.message : String(error));
  }
  restartServer(): void {
    this.bridge.post({ type: "restartServer" });
  }
  openSettings(): void {
    this.bridge.post({ type: "openSettings" });
  }
  openFile(path: string): void {
    this.bridge.post({ type: "openFile", path });
  }
  copy(text: string): void {
    this.bridge.post({ type: "copy", text });
    this.toast(this.t()("common.copied"));
  }
  private persist<K extends keyof InitState>(key: K & keyof PersistedState, value: InitState[K]): void {
    this.dispatch({ type: "initPatch", patch: { [key]: value } as Partial<InitState> });
    this.bridge.post({ type: "setState", key, value });
  }

  get cwd(): string | null {
    const status = this.state.server;
    return this.state.threadCwd ?? (status.state === "ready" ? status.cwd : null) ?? this.state.init?.workspaceFolders[0]?.path ?? null;
  }

  // --- catalog ---------------------------------------------------------------------------
  async loadCatalog(): Promise<void> {
    await Promise.all([this.loadModels(), this.loadModes(), this.loadSkills(), this.loadConfig()]);
    await this.loadModelLimits();
  }

  /** The models' real context windows (models.dev via the host) for the compaction slider. */
  async loadModelLimits(): Promise<void> {
    const models = this.state.models.map((m) => m.id);
    if (models.length === 0) return;
    try {
      const limits = await this.bridge.modelLimits(this.state.config?.modelProvider ?? null, models);
      this.dispatch({ type: "modelLimits", limits });
    } catch {
      // Unknown windows only lose the slider's upper end; the catalog value stands in.
    }
  }

  async loadModels(): Promise<void> {
    try {
      const models = await this.session.modelList({ includeHidden: false });
      this.dispatch({ type: "models", models: models.data });
    } catch (error) {
      this.fail(error);
    }
  }

  async loadModes(): Promise<void> {
    try {
      const modes = await this.session.collaborationModeList();
      this.dispatch({ type: "modes", modes: modes.data });
    } catch {
      this.dispatch({ type: "modes", modes: [] });
    }
  }

  async loadSkills(): Promise<void> {
    const cwd = this.cwd;
    try {
      const result = await this.session.skillsList({ cwds: cwd ? [cwd] : [], forceReload: false });
      const skills = result.data.flatMap((e) => e.skills);
      const errors = result.data.flatMap((e) => e.errors.map((x) => `${x.path}: ${x.message}`));
      this.dispatch({ type: "skills", skills, errors });
    } catch (error) {
      this.fail(error);
    }
  }

  async loadConfig(): Promise<void> {
    try {
      const result = await this.session.configRead({ includeLayers: false, cwd: this.cwd });
      const c = result.config as Record<string, unknown>;
      const scope = c.model_auto_compact_token_limit_scope === "body_after_prefix" ? "body_after_prefix" : "total";
      this.dispatch({
        type: "config",
        config: {
          model: (c.model as string | null) ?? null,
          modelProvider: (c.model_provider as string | null) ?? null,
          compactionLimit: typeof c.model_auto_compact_token_limit === "number" ? c.model_auto_compact_token_limit : null,
          compactionScope: scope as CompactionScope,
          contextWindow: typeof c.model_context_window === "number" ? c.model_context_window : null,
          windowsSandbox: typeof (c.windows as { sandbox?: unknown } | undefined)?.sandbox === "string" ? String((c.windows as { sandbox: string }).sandbox) : null,
        },
      });
    } catch (error) {
      this.fail(error);
    }
  }

  // --- threads ---------------------------------------------------------------------------
  newThread(): void {
    this.dispatch({ type: "chat", action: { type: "threadCleared" } });
    this.dispatch({ type: "thread", cwd: null, model: null });
    this.dispatch({ type: "composer", patch: { panel: null } });
    this.go("chat");
  }

  /** Records the compaction value a thread starts with (goal item 11: it is frozen from here). */
  private async rememberStartCompaction(threadId: string): Promise<void> {
    await this.loadConfig();
    const value = this.state.config?.compactionLimit ?? null;
    const map = { ...(this.state.init?.threadStartCompaction ?? {}), [threadId]: value };
    this.persist("threadStartCompaction", map);
  }

  private async ensureThread(): Promise<string> {
    if (this.state.chat.threadId) return this.state.chat.threadId;
    const mode = this.state.composer.fileMode;
    const picks = this.state.composer.files.map((f) => f.path);
    const denied = picks.length > 0 ? await this.deniedPathsFor(mode, picks) : [];
    // Goal item 8 (approved, PROMPT-CHANGE-PLAN P1): Codex's own field, rendered once into the
    // chat's opening developer instructions and cached from then on. Fixed for the chat.
    const preference = (this.state.init?.preferences ?? "").trim();
    const started = await this.session.threadStart({
      cwd: this.cwd,
      ...(denied.length > 0 ? { config: blockProfileConfig(denied) } : {}),
      ...(preference ? { developerInstructions: preferenceInstructions(preference) } : {}),
    });
    if (preference) this.persist("threadPreferences", { ...(this.state.init?.threadPreferences ?? {}), [started.thread.id]: preference });
    if (picks.length > 0) {
      const access: ThreadFileAccess = { mode, picks, denied };
      this.persist("threadBlocks", { ...(this.state.init?.threadBlocks ?? {}), [started.thread.id]: access });
    }
    this.dispatch({ type: "chat", action: { type: "threadLoaded", threadId: started.thread.id, name: started.thread.name, turns: [] } });
    this.dispatch({ type: "thread", cwd: started.cwd, model: started.model });
    await this.rememberStartCompaction(started.thread.id);
    return started.thread.id;
  }

  async resume(threadId: string): Promise<void> {
    try {
      // A chat that blocked files gets the same profile back; the panel shows its list.
      const access = this.threadAccess(threadId);
      const preference = this.state.init?.threadPreferences?.[threadId];
      const resumed = await this.session.threadResume({
        threadId,
        ...(access.denied.length > 0 ? { config: blockProfileConfig(access.denied) } : {}),
        ...(preference ? { developerInstructions: preferenceInstructions(preference) } : {}),
      });
      this.dispatch({
        type: "composer",
        patch: { fileMode: access.mode, files: access.picks.map((path) => ({ name: path.split(/[\\/]/).pop() ?? path, path })) },
      });
      this.dispatch({ type: "chat", action: { type: "threadLoaded", threadId, name: resumed.thread.name, turns: resumed.thread.turns } });
      this.dispatch({ type: "thread", cwd: resumed.cwd, model: resumed.model });
      // A thread left in Plan mode resumes in Plan; the composer follows it.
      const resumedMode = (resumed as { collaborationMode?: { mode?: string } | null }).collaborationMode?.mode;
      if (resumedMode === "plan") this.planThreads.add(threadId);
      this.dispatch({ type: "composer", patch: { mode: resumedMode === "plan" ? "plan" : null } });
      this.go("chat");
      await this.rememberStartCompaction(threadId);
      const goal = await this.session.threadGoalGet({ threadId }).catch(() => ({ goal: null }));
      this.dispatch({ type: "chat", action: { type: "goalLoaded", goal: goal.goal } });
    } catch (error) {
      this.fail(error);
    }
  }

  // --- turns -----------------------------------------------------------------------------
  /**
   * Builds the turn input the way the TUI does. Files picked in the tree are written into the
   * text as paths, exactly what the TUI's @ picker inserts (chat_composer.rs insert_selected_path:
   * the path, quoted when it has whitespace). A `mention` item would not work for files: core only
   * resolves app:// and plugin:// mentions and adds no content for anything else
   * (protocol/src/models.rs). Attached skills are `skill` items, as the TUI sends for $skill.
   */
  buildInput(text: string, pastes: Map<string, string>): UserInput[] {
    let expanded = text;
    for (const [placeholder, content] of pastes) expanded = expanded.split(placeholder).join(content);
    const input: UserInput[] = [{ type: "text", text: expanded, text_elements: [] }];
    for (const s of this.state.init?.attachedSkills ?? []) input.push({ type: "skill", name: s.name, path: s.path });
    return input;
  }

  async send(text: string, pastes: Map<string, string> = new Map()): Promise<boolean> {
    const slash = parseSlashInput(text);
    if (slash && pastes.size === 0) return this.runSlash(slash.name, slash.args);
    try {
      const threadId = await this.ensureThread();
      const c = this.state.composer;
      const params: TurnStartParamsWithMode = { threadId, input: this.buildInput(text, pastes), turnTrigger: "user", invisible: c.invisible };
      if (c.model) params.model = c.model;
      if (c.effort) params.effort = c.effort;
      if (c.permission) Object.assign(params, PERMISSION_PRESETS[c.permission]);
      const mode = this.collaborationMode(threadId);
      if (mode) params.collaborationMode = mode;
      const started = await this.session.turnStart(params);
      if (c.invisible) {
        const map = { ...(this.state.init?.invisibleTurns ?? {}) };
        map[threadId] = [...(map[threadId] ?? []), started.turn.id];
        this.persist("invisibleTurns", map);
      }
      return true;
    } catch (error) {
      this.fail(error);
      return false;
    }
  }

  /** Threads whose current collaboration mode is Plan (set by us, or found on resume). */
  private readonly planThreads = new Set<string>();

  /**
   * turn/start.collaborationMode, sent only when the thread's mode actually changes: into Plan,
   * or back to Default from Plan (the server keeps a mode for later turns, so leaving Plan needs
   * one explicit Default). Measured: every turn that carries a mode adds the mode's built-in
   * `<collaboration_mode>` block (~1.3K chars for Default) to the request, so a thread that never
   * used Plan sends nothing — the same model input as `suffice exec`. The TUI instead sends Default
   * on every turn; matching it is a model-input change left to the owner (PROMPT-CHANGE-PLAN.md).
   */
  private collaborationMode(threadId: string): TurnStartParamsWithMode["collaborationMode"] {
    const choice = this.state.composer.mode;
    const inPlan = this.planThreads.has(threadId);
    if (choice === "plan") this.planThreads.add(threadId);
    else if (choice === "default" && inPlan) this.planThreads.delete(threadId);
    else return undefined;
    const mask = this.state.modes.find((m) => m.mode === choice);
    const model = this.state.composer.model ?? this.state.threadModel ?? this.state.config?.model ?? this.state.models.find((m) => m.isDefault)?.id;
    if (!model) return undefined;
    return {
      mode: choice,
      settings: { model, reasoning_effort: this.state.composer.effort ?? mask?.reasoning_effort ?? null, developer_instructions: null },
    };
  }

  async interrupt(): Promise<void> {
    const { threadId, activeTurnId } = this.state.chat;
    if (!threadId || !activeTurnId) return;
    await this.session.turnInterrupt({ threadId, turnId: activeTurnId }).catch((e) => this.fail(e));
  }

  answerServerRequest(requestId: number, result: unknown): void {
    this.bridge.post({ type: "serverRequestResult", requestId, result });
    this.dispatch({ type: "serverRequestDone", requestId });
  }

  // --- slash commands typed in the composer ----------------------------------------------
  async runSlash(name: string, args: string): Promise<boolean> {
    const t = this.t();
    const command = findCommand(name);
    const notice = (text: string) => this.dispatch({ type: "chat", action: { type: "localNotice", notice: { kind: "info", text } } });
    if (!command) {
      notice(t("chat.slashUnknown", { name }));
      return true;
    }
    if (!command.guiAction) {
      notice(t("chat.slashUnsupported", { name: command.name }));
      return true;
    }
    this.recordPractice(command.name);
    switch (command.guiAction) {
      case "newThread":
        this.newThread();
        return true;
      case "openHistory":
        this.go("history");
        return true;
      case "openModelPanel":
        this.dispatch({ type: "composer", patch: { panel: "model" } });
        return true;
      case "openSkillsPanel":
        this.dispatch({ type: "composer", patch: { panel: "skills" } });
        return true;
      case "openFilesPanel":
        this.dispatch({ type: "composer", patch: { panel: "files" } });
        return true;
      case "openPermissions":
        this.dispatch({ type: "composer", patch: { panel: "mode" } });
        return true;
      case "openCommandGuide":
        this.go("commands");
        return true;
      case "toggleInvisible":
        this.toggleInvisible();
        return true;
      case "planMode":
        // dispatch() already synced this.state, so send() below sees Plan.
        this.dispatch({ type: "composer", patch: { mode: "plan" } });
        return args ? this.send(args) : true;
      case "goal": {
        if (args) {
          await this.setGoal(args);
          return true;
        }
        const goal = this.state.chat.goal;
        notice(
          goal
            ? t("chat.goalCurrent", { objective: goal.objective, status: t(`goal.${goal.status}` as MessageKey), tokens: goal.tokensUsed })
            : t("chat.goalUsage"),
        );
        return true;
      }
      case "compact":
        if (this.state.chat.threadId) await this.session.threadCompactStart({ threadId: this.state.chat.threadId }).catch((e) => this.fail(e));
        return true;
      case "review":
        await this.startReview(args);
        return true;
      case "rename":
        if (args && this.state.chat.threadId) {
          await this.session.threadSetName({ threadId: this.state.chat.threadId, name: args }).catch((e) => this.fail(e));
        } else notice(t("chat.renamePrompt"));
        return true;
      case "statusCard": {
        const usage = this.state.chat.tokenUsage;
        notice(
          t("chat.statusCard", {
            model: this.state.composer.model ?? this.state.threadModel ?? this.state.config?.model ?? "–",
            effort: this.state.composer.effort ?? "–",
            used: String(usage?.total.totalTokens ?? 0),
          }),
        );
        return true;
      }
      case "copyLast": {
        const text = lastAgentText(this.state.chat);
        if (text) this.copy(text);
        return true;
      }
      case "showCwd":
        notice(this.cwd ?? "–");
        return true;
    }
  }

  // --- slash command course (goal item 3) ------------------------------------------------
  get learning(): LearningProgress {
    return this.state.init?.learning ?? EMPTY_PROGRESS;
  }

  /** A command the extension actually ran; completes the lesson steps that ask for it. */
  private recordPractice(name: string): void {
    const next = recordPracticed(this.learning, name);
    if (next === this.learning) return;
    this.persist("learning", next);
    const taught = lessonSets().some((s) => s.lessons.some((l) => l.steps.some((st) => st.kind === "try" && st.command === name)));
    if (taught) this.toast(this.t()("learn.practicedToast", { name }));
  }

  answerQuiz(quizId: string, option: string): void {
    this.persist("learning", { ...this.learning, quizzes: { ...this.learning.quizzes, [quizId]: option } });
  }

  markExplored(key: string): void {
    if (this.learning.explored.includes(key)) return;
    this.persist("learning", { ...this.learning, explored: [...this.learning.explored, key] });
  }

  resetLearning(): void {
    this.persist("learning", EMPTY_PROGRESS);
  }

  toggleInvisible(): void {
    const on = !this.state.composer.invisible;
    this.dispatch({ type: "composer", patch: { invisible: on } });
    this.dispatch({
      type: "chat",
      action: { type: "localNotice", notice: { kind: "info", text: this.t()(on ? "chat.invisibleOnNotice" : "chat.invisibleOffNotice") } },
    });
  }

  // --- goal / review ---------------------------------------------------------------------
  async setGoal(objective: string): Promise<void> {
    try {
      const threadId = await this.ensureThread();
      const r = await this.session.threadGoalSet({ threadId, objective, status: "active" });
      this.dispatch({ type: "chat", action: { type: "goalLoaded", goal: r.goal } });
    } catch (error) {
      this.fail(error);
    }
  }

  async setGoalStatus(status: ThreadGoalStatus): Promise<void> {
    const threadId = this.state.chat.threadId;
    if (!threadId) return;
    try {
      const r = await this.session.threadGoalSet({ threadId, status });
      this.dispatch({ type: "chat", action: { type: "goalLoaded", goal: r.goal } });
    } catch (error) {
      this.fail(error);
    }
  }

  async clearGoal(): Promise<void> {
    const threadId = this.state.chat.threadId;
    if (!threadId) return;
    await this.session.threadGoalClear({ threadId }).catch((e) => this.fail(e));
    this.dispatch({ type: "chat", action: { type: "goalLoaded", goal: null } });
  }

  async startReview(instructions = ""): Promise<void> {
    try {
      const threadId = await this.ensureThread();
      await this.session.reviewStart({
        threadId,
        target: instructions ? { type: "custom", instructions } : { type: "uncommittedChanges" },
      });
      this.dispatch({ type: "composer", patch: { panel: null } });
    } catch (error) {
      this.fail(error);
    }
  }

  // --- skills (goal item 2) --------------------------------------------------------------
  async setSkillEnabled(skill: SkillMetadata, enabled: boolean): Promise<void> {
    try {
      await this.session.skillsConfigWrite({ path: skill.path, enabled });
      await this.loadSkills();
    } catch (error) {
      this.fail(error);
    }
  }

  toggleAttachedSkill(skill: SkillMetadata): void {
    const current = this.state.init?.attachedSkills ?? [];
    const next: AttachedSkill[] = current.some((s) => s.path === skill.path)
      ? current.filter((s) => s.path !== skill.path)
      : [...current, { name: skill.name, path: skill.path }];
    this.persist("attachedSkills", next);
  }

  // --- config (goal items 10, 11) --------------------------------------------------------
  async writeCompactionLimit(value: number | null): Promise<boolean> {
    try {
      // Suffice clamps the limit to 9/10 of the context window it knows (its catalog). When the
      // chosen limit only fits the model's real window, that window goes to Codex's own
      // `model_context_window` setting so the limit is applied as chosen; below the clamp the
      // setting is removed again (only if it holds the value this slider wrote).
      const range = compactionRange(this.state, 0);
      const needsWindow =
        value !== null && range.catalogWindow !== null && range.realWindow !== null && range.realWindow > range.catalogWindow && value > clampedAutoCompactLimit(null, range.catalogWindow);
      const current = this.state.config?.contextWindow ?? null;
      const desired = needsWindow ? range.realWindow : current === range.realWindow ? null : current;
      if (desired !== current) {
        await this.session.configValueWrite({ keyPath: "model_context_window", value: desired, mergeStrategy: "replace" });
      }
      await this.session.configValueWrite({ keyPath: "model_auto_compact_token_limit", value, mergeStrategy: "replace" });
      await this.loadConfig();
      return true;
    } catch (error) {
      this.fail(error);
      return false;
    }
  }

  // --- API keys & prices (goal item 9) ---------------------------------------------------
  saveKey(name: string, value: string | null): void {
    this.bridge.post({ type: "setSecret", name, value });
  }

  async loginWithOpenAiKey(apiKey: string): Promise<void> {
    try {
      await this.session.accountLoginStart({ type: "apiKey", apiKey });
      this.toast(this.t()("settings.prefsSaved"));
    } catch (error) {
      this.fail(error);
    }
  }

  setPrice(model: string, price: ModelPrice | null): void {
    const prices = { ...(this.state.init?.prices ?? {}) };
    if (price) prices[model] = price;
    else delete prices[model];
    this.persist("prices", prices);
  }

  // --- preferences, language, theme (goal items 7, 8, 14) --------------------------------
  savePreferences(text: string): void {
    this.persist("preferences", text);
  }
  setLanguage(value: string): void {
    this.bridge.post({ type: "setSetting", key: "language", value });
  }
  setTheme(value: string): void {
    this.dispatch({ type: "initPatch", patch: { themeId: value } });
    this.bridge.post({ type: "setSetting", key: "theme", value });
  }

  // --- composer selections -----------------------------------------------------------------
  /** Opens a composer panel, or closes it when it is already open. */
  togglePanel(panel: AppState["composer"]["panel"]): void {
    this.dispatch({ type: "composer", patch: { panel: this.state.composer.panel === panel ? null : panel } });
  }
  setMode(mode: AppState["composer"]["mode"]): void {
    this.dispatch({ type: "composer", patch: { mode } });
  }
  setPermission(permission: PermissionPreset | null): void {
    this.dispatch({ type: "composer", patch: { permission } });
  }
  attachFile(file: { name: string; path: string }): void {
    if (this.state.composer.files.some((f) => f.path === file.path)) return;
    this.dispatch({ type: "composer", patch: { files: [...this.state.composer.files, file] } });
  }
  clearFiles(): void {
    this.dispatch({ type: "composer", patch: { files: [] } });
  }
  /** The preference the current chat started with ("" for none), or null before a chat has started. */
  get chatPreference(): string | null {
    const threadId = this.state.chat.threadId;
    return threadId ? (this.state.init?.threadPreferences?.[threadId] ?? "") : null;
  }

  /** The file access the current chat started with, or null before a chat has started. */
  get chatBlocks(): ThreadFileAccess | null {
    const threadId = this.state.chat.threadId;
    return threadId ? this.threadAccess(threadId) : null;
  }

  private threadAccess(threadId: string): ThreadFileAccess {
    const entry = this.state.init?.threadBlocks?.[threadId];
    if (!entry) return { mode: "block", picks: [], denied: [] };
    return Array.isArray(entry) ? { mode: "block", picks: entry, denied: entry } : entry;
  }

  setFileMode(fileMode: FileAccessMode): void {
    this.dispatch({ type: "composer", patch: { fileMode } });
  }

  /** What the thread's profile denies: the picks themselves, or in Select mode their complement. */
  private async deniedPathsFor(mode: FileAccessMode, picks: string[]): Promise<string[]> {
    return mode === "block" ? picks : this.selectionComplement(picks);
  }

  /**
   * Select mode: every entry at the picks' levels of the workspace that is neither picked nor on
   * the way to a pick. For `src/api/client.ts` that is the rest of `src/api`, the rest of `src` and
   * the rest of the workspace root. Nothing outside the workspace is touched, a picked folder keeps
   * all of its contents, and `AGENTS.md` stays readable because Codex reads it to build the chat's
   * instructions. Listings come from the app-server's own `fs/readDirectory`.
   */
  private async selectionComplement(picks: string[]): Promise<string[]> {
    const root = this.cwd;
    if (!root) return [];
    const sep = root.includes("\\") ? "\\" : "/";
    const norm = (path: string) => path.replace(/[\\/]+/g, sep).replace(/[\\/]$/, "");
    const key = (path: string) => (sep === "\\" ? norm(path).toLowerCase() : norm(path));
    const rootPath = norm(root);
    const rootKey = key(rootPath);
    const picked = new Set(picks.map(key));
    const onPath = new Set<string>();
    const dirs: string[] = [];
    for (const pick of picks) {
      const path = norm(pick);
      if (!key(path).startsWith(rootKey + sep.toLowerCase())) continue;
      let parent = path.slice(0, path.lastIndexOf(sep));
      while (parent.length >= rootPath.length) {
        if (!onPath.has(key(parent))) {
          onPath.add(key(parent));
          dirs.push(parent);
        }
        if (key(parent) === rootKey) break;
        parent = parent.slice(0, parent.lastIndexOf(sep));
      }
    }
    const denied: string[] = [];
    for (const dir of dirs) {
      const listing = await this.session.fsReadDirectory({ path: dir });
      for (const entry of listing.entries) {
        const full = `${dir}${sep}${entry.fileName}`;
        if (picked.has(key(full)) || onPath.has(key(full)) || entry.fileName === "AGENTS.md") continue;
        denied.push(full);
      }
    }
    return denied;
  }
  detachFile(path: string): void {
    this.dispatch({ type: "composer", patch: { files: this.state.composer.files.filter((f) => f.path !== path) } });
  }
  /** Puts text into the chat composer (the command guide's "Try it"). */
  setDraft(text: string): void {
    this.dispatch({ type: "composer", patch: { draft: text } });
    this.go("chat");
  }
  /** The composer takes the pending draft exactly once. */
  takeDraft(): string | null {
    const draft = this.state.composer.draft;
    if (draft !== null) this.dispatch({ type: "composer", patch: { draft: null } });
    return draft;
  }
  setModel(model: string | null): void {
    const entry = this.state.models.find((m) => m.id === model);
    const effort = this.state.composer.effort;
    const supported = entry?.supportedReasoningEfforts.map((e) => e.reasoningEffort) ?? [];
    const keep = effort && (supported.length === 0 || supported.includes(effort));
    this.dispatch({ type: "composer", patch: { model, effort: keep ? effort : null } });
  }
  setEffort(effort: ReasoningEffort | null): void {
    this.dispatch({ type: "composer", patch: { effort } });
  }
}
