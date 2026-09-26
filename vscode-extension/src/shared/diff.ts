// File changes as Codex sends them (app-server-protocol item_builders.rs format_file_change_diff):
// an added or deleted file carries its whole content, an updated one a unified diff, followed by
// "Moved to: <path>" when it was also moved. This turns each into numbered rows, the way a
// GitHub diff shows them.

export type DiffRow =
  | { type: "hunk"; text: string }
  | { type: "add" | "del" | "ctx"; oldLine: number | null; newLine: number | null; text: string }
  | { type: "note"; text: string };

export interface ParsedDiff {
  rows: DiffRow[];
  added: number;
  removed: number;
  movedTo: string | null;
}

export type ChangeKind = "add" | "delete" | "update";

const MOVED = /\n*\nMoved to: (.+)\s*$/;

function contentLines(content: string): string[] {
  if (content === "") return [];
  const lines = content.replace(/\r\n/g, "\n").split("\n");
  if (lines[lines.length - 1] === "") lines.pop();
  return lines;
}

export function parseDiff(kind: ChangeKind, diff: string): ParsedDiff {
  if (kind === "add" || kind === "delete") {
    const lines = contentLines(diff);
    const rows: DiffRow[] = lines.map((text, i) =>
      kind === "add" ? { type: "add", oldLine: null, newLine: i + 1, text } : { type: "del", oldLine: i + 1, newLine: null, text },
    );
    return { rows, added: kind === "add" ? lines.length : 0, removed: kind === "delete" ? lines.length : 0, movedTo: null };
  }

  const moved = MOVED.exec(diff);
  const body = moved ? diff.slice(0, moved.index) : diff;
  const rows: DiffRow[] = [];
  let added = 0;
  let removed = 0;
  let oldLine = 0;
  let newLine = 0;
  let inHunk = false;
  for (const line of contentLines(body)) {
    const hunk = /^@@ -(\d+)(?:,\d+)? \+(\d+)(?:,\d+)? @@/.exec(line);
    if (hunk) {
      oldLine = Number(hunk[1]);
      newLine = Number(hunk[2]);
      inHunk = true;
      rows.push({ type: "hunk", text: line });
      continue;
    }
    // File headers before the first hunk (--- a/x, +++ b/x, diff --git ...) are not content.
    if (!inHunk) continue;
    if (line.startsWith("\\")) {
      rows.push({ type: "note", text: line.slice(1).trim() });
    } else if (line.startsWith("+")) {
      rows.push({ type: "add", oldLine: null, newLine: newLine++, text: line.slice(1) });
      added++;
    } else if (line.startsWith("-")) {
      rows.push({ type: "del", oldLine: oldLine++, newLine: null, text: line.slice(1) });
      removed++;
    } else {
      rows.push({ type: "ctx", oldLine: oldLine++, newLine: newLine++, text: line.startsWith(" ") ? line.slice(1) : line });
    }
  }
  return { rows, added, removed, movedTo: moved ? moved[1]!.trim() : null };
}

/** GitHub's five-block bar: green and red shares of the changed lines, grey for the rest. */
export function diffStatBlocks(added: number, removed: number, blocks = 5): Array<"add" | "del" | "none"> {
  const total = added + removed;
  if (total === 0) return Array(blocks).fill("none");
  let adds = Math.round((added / total) * blocks);
  let dels = Math.round((removed / total) * blocks);
  if (added > 0 && adds === 0) adds = 1;
  if (removed > 0 && dels === 0) dels = 1;
  while (adds + dels > blocks) adds > dels ? adds-- : dels--;
  return [...Array(adds).fill("add"), ...Array(dels).fill("del"), ...Array(blocks - adds - dels).fill("none")];
}

/** A path shown relative to the chat's folder when it lies inside it. */
export function relativePath(path: string, cwd: string | null): string {
  if (!cwd) return path;
  const norm = (p: string) => p.replace(/\\/g, "/").replace(/\/+$/, "");
  const base = norm(cwd);
  const full = norm(path);
  const windows = /^[a-z]:\//i.test(base);
  const same = windows ? full.toLowerCase().startsWith(base.toLowerCase() + "/") : full.startsWith(base + "/");
  return same ? full.slice(base.length + 1) : path;
}
