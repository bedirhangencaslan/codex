// Application state of the webview: what the host told us (init, server status), what the
// server returned (models, modes, skills, config), the current chat, the server's pending
// requests and the composer's selections for the next turn.
import type { CollaborationModeMask } from "@protocol/v2/CollaborationModeMask";
import type { Model } from "@protocol/v2/Model";
import type { ReasoningEffort } from "@protocol/ReasoningEffort";
import type { SkillMetadata } from "@protocol/v2/SkillMetadata";
import type { CompactionScope } from "../../shared/compactionSync";
import type { FileAccessMode, InitState, ServerStatus } from "../../shared/messages";
import { chatReducer, emptyChat, type ChatAction, type ChatState } from "../state/chatReducer";

export type Screen = "home" | "chat" | "history" | "skills" | "commands" | "api" | "settings";
export const SCREENS: Screen[] = ["home", "chat", "history", "skills", "commands", "api", "settings"];

export type PermissionPreset = "readOnly" | "auto" | "full";
export type ModeChoice = "default" | "plan";
export type ComposerPanel = "mode" | "skills" | "files" | "model" | null;

export interface ConfigSnapshot {
  model: string | null;
  modelProvider: string | null;
  compactionLimit: number | null;
  compactionScope: CompactionScope;
  contextWindow: number | null;
  /** `[windows] sandbox` from config.toml; blocking files needs "elevated" on Windows. */
  windowsSandbox: string | null;
}

export interface PendingServerRequest {
  requestId: number;
  method: string;
  params: unknown;
}

export interface AttachedFile {
  name: string;
  path: string;
}

export interface ComposerState {
  model: string | null;
  effort: ReasoningEffort | null;
  mode: ModeChoice | null;
  permission: PermissionPreset | null;
  invisible: boolean;
  /** Files and folders the next chat blocks (the thread's permission profile denies them). The
   * list stays for later chats until it is changed; a running chat keeps the list it started with. */
  files: AttachedFile[];
  /** Block the picks, or select them and block everything else at their levels of the workspace. */
  fileMode: FileAccessMode;
  panel: ComposerPanel;
  /** Text to place in the composer (from the command guide's "Try it"). */
  draft: string | null;
}

export interface AppState {
  init: InitState | null;
  server: ServerStatus;
  screen: Screen;
  chat: ChatState;
  /** cwd of the current thread (from thread/start or thread/resume). */
  threadCwd: string | null;
  threadModel: string | null;
  models: Model[];
  modes: CollaborationModeMask[];
  skills: SkillMetadata[];
  skillErrors: string[];
  config: ConfigSnapshot | null;
  /** Real context windows by model id (models.dev); the compaction slider's upper end. */
  modelLimits: Record<string, number>;
  serverRequests: PendingServerRequest[];
  composer: ComposerState;
  toast: string | null;
}

export const initialState: AppState = {
  init: null,
  server: { state: "starting" },
  screen: "chat",
  chat: emptyChat,
  threadCwd: null,
  threadModel: null,
  models: [],
  modes: [],
  skills: [],
  skillErrors: [],
  config: null,
  modelLimits: {},
  serverRequests: [],
  composer: { model: null, effort: null, mode: null, permission: null, invisible: false, files: [], fileMode: "block", panel: null, draft: null },
  toast: null,
};

export type AppAction =
  | { type: "init"; state: InitState }
  | { type: "initPatch"; patch: Partial<InitState> }
  | { type: "server"; status: ServerStatus }
  | { type: "screen"; screen: Screen }
  | { type: "chat"; action: ChatAction }
  | { type: "thread"; cwd: string | null; model: string | null }
  | { type: "models"; models: Model[] }
  | { type: "modes"; modes: CollaborationModeMask[] }
  | { type: "skills"; skills: SkillMetadata[]; errors: string[] }
  | { type: "config"; config: ConfigSnapshot }
  | { type: "modelLimits"; limits: Record<string, number> }
  | { type: "serverRequest"; request: PendingServerRequest }
  | { type: "serverRequestDone"; requestId: number }
  | { type: "composer"; patch: Partial<ComposerState> }
  | { type: "toast"; text: string | null };

export function appReducer(state: AppState, action: AppAction): AppState {
  switch (action.type) {
    case "init":
      return { ...state, init: action.state };
    case "initPatch":
      return state.init ? { ...state, init: { ...state.init, ...action.patch } } : state;
    case "server":
      return {
        ...state,
        server: action.status,
        serverRequests: action.status.state === "ready" ? state.serverRequests : [],
      };
    case "screen":
      return { ...state, screen: action.screen, composer: { ...state.composer, panel: null } };
    case "chat":
      return { ...state, chat: chatReducer(state.chat, action.action) };
    case "thread":
      return { ...state, threadCwd: action.cwd, threadModel: action.model };
    case "models":
      return { ...state, models: action.models };
    case "modes":
      return { ...state, modes: action.modes };
    case "skills":
      return { ...state, skills: action.skills, skillErrors: action.errors };
    case "config":
      return { ...state, config: action.config };
    case "modelLimits":
      return { ...state, modelLimits: { ...state.modelLimits, ...action.limits } };
    case "serverRequest":
      return state.serverRequests.some((r) => r.requestId === action.request.requestId)
        ? state
        : { ...state, serverRequests: [...state.serverRequests, action.request] };
    case "serverRequestDone":
      return { ...state, serverRequests: state.serverRequests.filter((r) => r.requestId !== action.requestId) };
    case "composer":
      return { ...state, composer: { ...state.composer, ...action.patch } };
    case "toast":
      return { ...state, toast: action.text };
  }
}
