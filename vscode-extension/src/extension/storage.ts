// Persistence for the webview. Global things (prices, the preferences text, course progress, the
// list of env-key names) live in globalState; per-project things (selected skills, invisible turns, the
// compaction value each thread started with) in workspaceState. API keys only ever live in
// SecretStorage and reach Suffice as environment variables of the spawned app-server.
import type * as vscode from "vscode";
import { EMPTY_PROGRESS } from "../shared/lessons";
import type { EnvKeyInfo, PersistedState } from "../shared/messages";

const GLOBAL_KEYS: Array<keyof PersistedState> = ["prices", "preferences", "learning"];
const WORKSPACE_KEYS: Array<keyof PersistedState> = ["attachedSkills", "invisibleTurns", "threadStartCompaction", "threadBlocks", "threadPreferences", "threadSkills"];

const DEFAULTS: PersistedState = {
  prices: {},
  preferences: "",
  attachedSkills: [],
  invisibleTurns: {},
  threadStartCompaction: {},
  learning: EMPTY_PROGRESS,
  threadBlocks: {},
  threadPreferences: {},
  threadSkills: {},
};

/** Env var names offered even before the user adds any: the provider this fork ships for. */
const DEFAULT_ENV_KEYS = ["ZAI_API_KEY"];
const ENV_KEY_NAMES = "suffice.envKeyNames";
const secretKey = (name: string) => `suffice.env.${name}`;

export class ExtensionStorage {
  constructor(private readonly context: vscode.ExtensionContext) {}

  read(): PersistedState {
    const state = { ...DEFAULTS };
    for (const key of GLOBAL_KEYS) {
      (state as Record<string, unknown>)[key] = this.context.globalState.get(key, DEFAULTS[key]);
    }
    for (const key of WORKSPACE_KEYS) {
      (state as Record<string, unknown>)[key] = this.context.workspaceState.get(key, DEFAULTS[key]);
    }
    return state;
  }

  async write(key: keyof PersistedState, value: unknown): Promise<void> {
    if (GLOBAL_KEYS.includes(key)) await this.context.globalState.update(key, value);
    else if (WORKSPACE_KEYS.includes(key)) await this.context.workspaceState.update(key, value);
    else throw new Error(`unknown state key ${String(key)}`);
  }

  private envKeyNames(): string[] {
    const stored = this.context.globalState.get<string[]>(ENV_KEY_NAMES, []);
    return [...new Set([...DEFAULT_ENV_KEYS, ...stored])];
  }

  async envKeys(): Promise<EnvKeyInfo[]> {
    const names = this.envKeyNames();
    return Promise.all(
      names.map(async (name) => ({ name, stored: Boolean(await this.context.secrets.get(secretKey(name))) })),
    );
  }

  /** Values to add to the app-server's environment. */
  async envOverrides(): Promise<Record<string, string>> {
    const env: Record<string, string> = {};
    for (const name of this.envKeyNames()) {
      const value = await this.context.secrets.get(secretKey(name));
      if (value) env[name] = value;
    }
    return env;
  }

  async setSecret(name: string, value: string | null): Promise<void> {
    if (!/^[A-Z_][A-Z0-9_]*$/.test(name)) throw new Error(`invalid environment variable name: ${name}`);
    if (value) {
      await this.context.secrets.store(secretKey(name), value);
      const names = this.context.globalState.get<string[]>(ENV_KEY_NAMES, []);
      if (!names.includes(name)) await this.context.globalState.update(ENV_KEY_NAMES, [...names, name]);
    } else {
      await this.context.secrets.delete(secretKey(name));
    }
  }
}
