// Reads the TUI's slash command enum (codex-rs/tui/src/slash_command.rs) and produces the
// catalog the extension teaches from, so the command list is never kept by hand. Used by
// scripts/gen-slash-commands.ts (writes src/shared/generated/slash-commands.json) and by
// tests/slashCommands.test.ts (fails when the JSON no longer matches the Rust source).
//
// It understands exactly the shapes that file uses: strum attributes on variants,
// `match self { A | B => "text" }` in description(), and `matches!(self, A | B)` or
// `match self { A | B => true, C => false }` in the boolean helpers.

export interface SlashCommandRecord {
  name: string;
  variant: string;
  aliases: string[];
  description: string;
  inlineArgs: boolean;
  availableDuringTask: boolean;
  availableInSideConversation: boolean;
  availableWhenThreadUnavailable: boolean;
}

export interface SlashCommandCatalog {
  source: string;
  commands: SlashCommandRecord[];
}

const kebab = (ident: string) => ident.replace(/([a-z0-9])([A-Z])/g, "$1-$2").toLowerCase();

function variants(source: string): Array<{ variant: string; name: string; aliases: string[] }> {
  const start = source.indexOf("pub enum SlashCommand {");
  if (start < 0) throw new Error("enum SlashCommand not found");
  const body = source.slice(start, source.indexOf("\n}", start));
  const out: Array<{ variant: string; name: string; aliases: string[] }> = [];
  let pendingToString: string | undefined;
  let pendingSerialize: string[] = [];
  for (const raw of body.split("\n").slice(1)) {
    const line = raw.trim();
    if (!line || line.startsWith("//")) continue;
    const attr = /^#\[strum\((.*)\)\]$/.exec(line);
    if (attr) {
      for (const [, key, value] of attr[1]!.matchAll(/(to_string|serialize)\s*=\s*"([^"]+)"/g)) {
        if (key === "to_string") pendingToString = value;
        else pendingSerialize.push(value!);
      }
      continue;
    }
    const ident = /^([A-Z][A-Za-z0-9]*),$/.exec(line);
    if (!ident) continue;
    const variant = ident[1]!;
    const name = pendingToString ?? pendingSerialize[0] ?? kebab(variant);
    const aliases = pendingSerialize.filter((s) => s !== name);
    out.push({ variant, name, aliases });
    pendingToString = undefined;
    pendingSerialize = [];
  }
  return out;
}

function fnBody(source: string, fn: string): string {
  const match = new RegExp(`fn ${fn}\\(self\\) -> [^{]+\\{`).exec(source);
  if (!match) throw new Error(`fn ${fn} not found`);
  const open = match.index + match[0].length;
  let depth = 1;
  for (let i = open; i < source.length; i++) {
    if (source[i] === "{") depth++;
    else if (source[i] === "}" && --depth === 0) return source.slice(open, i);
  }
  throw new Error(`fn ${fn} is not closed`);
}

const idents = (pattern: string) => [...pattern.matchAll(/SlashCommand::([A-Za-z0-9]+)/g)].map((m) => m[1]!);

function boolFn(source: string, fn: string): Map<string, boolean> {
  const body = fnBody(source, fn);
  const result = new Map<string, boolean>();
  if (body.includes("matches!(")) {
    for (const v of idents(body)) result.set(v, true);
    return result;
  }
  for (const arm of body.split(/,\s*\n/)) {
    const m = /([\s\S]*?)=>\s*(true|false)/.exec(arm);
    if (!m) continue;
    for (const v of idents(m[1]!)) result.set(v, m[2] === "true");
  }
  return result;
}

function descriptions(source: string): Map<string, string> {
  const body = fnBody(source, "description");
  const result = new Map<string, string>();
  const arm = /((?:SlashCommand::[A-Za-z0-9]+\s*\|?\s*)+)=>\s*\{?\s*"((?:[^"\\]|\\.)*)"/g;
  for (const m of body.matchAll(arm)) {
    for (const v of idents(m[1]!)) result.set(v, m[2]!.replace(/\\"/g, '"'));
  }
  return result;
}

export function parseSlashCommands(source: string, sourcePath: string): SlashCommandCatalog {
  const inline = boolFn(source, "supports_inline_args");
  const duringTask = boolFn(source, "available_during_task");
  const side = boolFn(source, "available_in_side_conversation");
  const noThread = boolFn(source, "available_when_thread_unavailable");
  const text = descriptions(source);
  const commands = variants(source).map(({ variant, name, aliases }) => {
    const description = text.get(variant);
    if (description === undefined) throw new Error(`no description for ${variant}`);
    if (!duringTask.has(variant)) throw new Error(`available_during_task does not cover ${variant}`);
    return {
      name,
      variant,
      aliases,
      description,
      inlineArgs: inline.get(variant) ?? false,
      availableDuringTask: duringTask.get(variant)!,
      availableInSideConversation: side.get(variant) ?? false,
      availableWhenThreadUnavailable: noThread.get(variant) ?? false,
    };
  });
  return { source: sourcePath, commands };
}

export const SLASH_COMMAND_SOURCE = "codex-rs/tui/src/slash_command.rs";

export function renderCatalog(catalog: SlashCommandCatalog): string {
  return JSON.stringify(catalog, null, 2) + "\n";
}
