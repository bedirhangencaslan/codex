// The postMessage contract between the extension host and the webview. The webview never talks
// to the app-server directly: it sends `rpc` messages the host forwards (to an allow-list of
// protocol methods), and receives the server's notifications and requests verbatim.
import type { LearningProgress } from "./lessons";
import type { ModelPrice } from "./pricing";

export interface WorkspaceFolderInfo {
  name: string;
  path: string;
}

export interface AttachedSkill {
  name: string;
  path: string;
}

export interface EnvKeyInfo {
  name: string;
  stored: boolean;
}

/** Everything the host persists for the webview. Keys of PersistedState are writable via setState. */
export interface PersistedState {
  /** User price overrides, by model id (globalState). */
  prices: Record<string, ModelPrice>;
  /** Goal item 8 text box (globalState). Stored only; see PROMPT-CHANGE-PLAN.md. */
  preferences: string;
  /** Skills this workspace attaches to every turn (workspaceState). */
  attachedSkills: AttachedSkill[];
  /** threadId -> turn ids that ran invisibly; the protocol does not record it (workspaceState). */
  invisibleTurns: Record<string, string[]>;
  /** threadId -> model_auto_compact_token_limit config value when the thread was started or
   * resumed, the value its reasoning shrink and compaction froze (workspaceState, item 11). */
  threadStartCompaction: Record<string, number | null>;
  /** Slash command course progress: commands run, quiz answers, reference entries opened (globalState). */
  learning: LearningProgress;
  /** threadId -> the file access a thread started with (workspaceState). `denied` is what its
   * permission profile denies, given again when the thread is resumed. Older entries are a bare
   * list of blocked paths. */
  threadBlocks: Record<string, ThreadFileAccess | string[]>;
}

/** What the files panel asked for: block the picks, or select them and block the rest. */
export type FileAccessMode = "block" | "select";

export interface ThreadFileAccess {
  mode: FileAccessMode;
  picks: string[];
  denied: string[];
}

export interface InitState extends PersistedState {
  locale: string;
  languageSetting: string;
  themeId: string;
  workspaceFolders: WorkspaceFolderInfo[];
  envKeys: EnvKeyInfo[];
  extensionVersion: string;
}

export type ServerStatus =
  | { state: "starting" }
  | { state: "ready"; codexHome: string; userAgent: string; cwd: string }
  | { state: "stopped"; reason: string }
  | { state: "noBinary" };

export type HostCommand = "newThread" | "showInterfaces";

export type HostToWebview =
  | { type: "init"; state: InitState }
  | { type: "server"; status: ServerStatus }
  | { type: "notification"; method: string; params: unknown }
  | { type: "serverRequest"; requestId: number; method: string; params: unknown }
  | { type: "rpcResult"; id: number; result?: unknown; error?: { message: string; code?: number } }
  | { type: "stateChanged"; patch: Partial<InitState> }
  | { type: "command"; command: HostCommand };

export type WebviewToHost =
  | { type: "ready" }
  | { type: "rpc"; id: number; method: string; params?: unknown }
  | { type: "serverRequestResult"; requestId: number; result: unknown }
  | { type: "setSetting"; key: "language" | "theme"; value: string }
  | { type: "setState"; key: keyof PersistedState; value: unknown }
  | { type: "setSecret"; name: string; value: string | null }
  | { type: "restartServer" }
  | { type: "openSettings" }
  | { type: "openFile"; path: string }
  | { type: "copy"; text: string };

/** Protocol methods the webview may call through the host. */
export const ALLOWED_RPC_METHODS = new Set([
  "thread/start",
  "thread/resume",
  "thread/list",
  "thread/read",
  "thread/turns/list",
  "thread/name/set",
  "thread/compact/start",
  "turn/start",
  "turn/interrupt",
  "review/start",
  "thread/goal/get",
  "thread/goal/set",
  "thread/goal/clear",
  "collaborationMode/list",
  "model/list",
  "skills/list",
  "skills/config/write",
  "config/read",
  "config/value/write",
  "config/batchWrite",
  "fuzzyFileSearch",
  "fs/readDirectory",
  "account/read",
  "account/login/start",
]);
