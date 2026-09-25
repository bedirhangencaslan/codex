// Slash command sets for the chat composer and the command guide (goal item 3).
//
// A set is a named list of commands with their availability flags. The built-in set is the
// TUI's own enum (generated/slash-commands.json, regenerated from codex-rs/tui/src/
// slash_command.rs and checked by tests/slashCommands.test.ts). Further sets — custom prompts,
// a project's own commands — are added with registerCommandSet() and appear in the guide and the
// composer popup without changes to either screen.
//
// Texts: `slash.<name>.desc` / `.when` / `.example` in the i18n dictionaries. The English
// `.desc` is the TUI's description verbatim (enforced by the test).
import catalog from "./generated/slash-commands.json";
import type { MessageKey } from "./i18n";

export type CommandCategory = "session" | "model" | "context" | "tools" | "interface" | "account";

/** What the extension does when a command is typed in its composer; absent = terminal only. */
export type GuiAction =
  | "newThread"
  | "openHistory"
  | "openModelPanel"
  | "planMode"
  | "goal"
  | "toggleInvisible"
  | "compact"
  | "statusCard"
  | "openSkillsPanel"
  | "review"
  | "rename"
  | "openFilesPanel"
  | "openPermissions"
  | "copyLast"
  | "showCwd"
  | "openCommandGuide";

export interface SlashCommandEntry {
  name: string;
  aliases: string[];
  category: CommandCategory;
  inlineArgs: boolean;
  availableDuringTask: boolean;
  availableInSideConversation: boolean;
  availableWhenThreadUnavailable: boolean;
  guiAction?: GuiAction;
  descKey: MessageKey;
  whenKey?: MessageKey;
  exampleKey?: MessageKey;
}

export interface CommandSet {
  id: string;
  titleKey: MessageKey;
  /** Where the definitions come from, shown in the guide. */
  source: string;
  commands: SlashCommandEntry[];
}

/** Debug-only or internal in the TUI (is_visible / "DO NOT USE"); not taught. */
const HIDDEN = new Set(["rollout", "test-approval", "debug-m-drop", "debug-m-update"]);

const CATEGORY: Record<string, CommandCategory> = {
  new: "session", clear: "session", resume: "session", fork: "session", rename: "session",
  archive: "session", delete: "session", side: "session", btw: "session", export: "session",
  copy: "session", recap: "session", compact: "session", worktree: "session", app: "session",
  agents: "session", subagents: "session",
  model: "model", plan: "model", goal: "model", invisible: "model", permissions: "model",
  approve: "model", review: "model", experimental: "model", memories: "model",
  mention: "context", ide: "context", diff: "context", cd: "context", pwd: "context",
  init: "context", status: "context", skills: "context", import: "context",
  mcp: "tools", apps: "tools", plugins: "tools", hooks: "tools", ps: "tools", stop: "tools",
  voice: "tools", daemon: "tools", "setup-default-sandbox": "tools",
  keymap: "interface", vim: "interface", raw: "interface", tui: "interface", title: "interface",
  statusline: "interface", theme: "interface", pets: "interface",
  usage: "account", logout: "account", feedback: "account", warnings: "account",
  "debug-config": "account", quit: "account", exit: "account",
};

const GUI_ACTION: Record<string, GuiAction> = {
  new: "newThread",
  clear: "newThread",
  resume: "openHistory",
  model: "openModelPanel",
  plan: "planMode",
  goal: "goal",
  invisible: "toggleInvisible",
  compact: "compact",
  status: "statusCard",
  skills: "openSkillsPanel",
  review: "review",
  rename: "rename",
  mention: "openFilesPanel",
  permissions: "openPermissions",
  copy: "copyLast",
  pwd: "showCwd",
};

const key = (name: string, part: "desc" | "when" | "example") => `slash.${name}.${part}` as MessageKey;

const sufficeTui: CommandSet = {
  id: "suffice-tui",
  titleKey: "commands.setSuffice",
  source: catalog.source,
  commands: catalog.commands
    .filter((c) => !HIDDEN.has(c.name))
    .map((c) => ({
      name: c.name,
      aliases: c.aliases,
      category: CATEGORY[c.name] ?? "tools",
      inlineArgs: c.inlineArgs,
      availableDuringTask: c.availableDuringTask,
      availableInSideConversation: c.availableInSideConversation,
      availableWhenThreadUnavailable: c.availableWhenThreadUnavailable,
      guiAction: GUI_ACTION[c.name],
      descKey: key(c.name, "desc"),
      whenKey: key(c.name, "when"),
      exampleKey: key(c.name, "example"),
    })),
};

const sets: CommandSet[] = [sufficeTui];

export function commandSets(): readonly CommandSet[] {
  return sets;
}

export function registerCommandSet(set: CommandSet): void {
  const existing = sets.findIndex((s) => s.id === set.id);
  if (existing >= 0) sets[existing] = set;
  else sets.push(set);
}

export const CATEGORY_ORDER: CommandCategory[] = ["session", "model", "context", "tools", "interface", "account"];

/** Resolves `/name` or an alias across all sets. */
export function findCommand(name: string): SlashCommandEntry | undefined {
  const wanted = name.toLowerCase();
  for (const set of sets) {
    const hit = set.commands.find((c) => c.name === wanted || c.aliases.includes(wanted));
    if (hit) return hit;
  }
  return undefined;
}

/** Popup matching like the TUI's command_popup.rs: prefix first, then subsequence. */
export function matchCommands(query: string): SlashCommandEntry[] {
  const q = query.toLowerCase();
  const all = sets.flatMap((s) => s.commands);
  const prefix = all.filter((c) => c.name.startsWith(q));
  const fuzzy = all.filter((c) => !c.name.startsWith(q) && isSubsequence(q, c.name));
  return [...prefix, ...fuzzy];
}

function isSubsequence(needle: string, haystack: string): boolean {
  let i = 0;
  for (const ch of haystack) if (ch === needle[i]) i++;
  return i === needle.length;
}

/** Splits `/name rest of line` into the command and its argument text. */
export function parseSlashInput(text: string): { name: string; args: string } | undefined {
  const m = /^\/([a-z0-9-]+)(?:\s+([\s\S]*))?$/i.exec(text.trim());
  return m ? { name: m[1]!.toLowerCase(), args: (m[2] ?? "").trim() } : undefined;
}
