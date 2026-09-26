// Installs the interface style skills (shared/uiStyles.ts) where Codex looks for user skills:
// `<SUFFICE_HOME>/skills/ui-styles/<id>/SKILL.md`. Codex scans that folder itself, so the skills
// work in any client, the terminal included.
//
// A style skill is turned off in the user config the first time it is installed (Codex's own
// skills/config/write), so its catalogue line is not sent with every request anywhere. A chat
// turns on the styles picked in the gallery through its session skill rules (controller
// skillRules). Files are only written when missing or different, and only inside that folder.
import { promises as fs } from "node:fs";
import * as path from "node:path";
import { UI_STYLE_ROOT, UI_STYLES, skillMarkdown } from "../shared/uiStyles";

export type Request = (method: string, params: unknown) => Promise<unknown>;

/** Writes the style skills; returns whether any file changed. */
export async function installUiStyleSkills(codexHome: string, request: Request, log: (text: string) => void): Promise<boolean> {
  const root = path.join(codexHome, "skills", UI_STYLE_ROOT);
  const fresh: string[] = [];
  let changed = false;
  for (const style of UI_STYLES) {
    const dir = path.join(root, style.id);
    const file = path.join(dir, "SKILL.md");
    const content = skillMarkdown(style);
    let existing: string | null = null;
    try {
      existing = await fs.readFile(file, "utf8");
    } catch {
      existing = null;
    }
    if (existing === content) continue;
    await fs.mkdir(dir, { recursive: true });
    await fs.writeFile(file, content, "utf8");
    changed = true;
    if (existing === null) fresh.push(file);
  }
  for (const file of fresh) {
    try {
      await request("skills/config/write", { path: file, enabled: false });
    } catch (error) {
      log(`could not turn off style skill ${file}: ${error instanceof Error ? error.message : String(error)}`);
    }
  }
  if (changed) {
    // Refresh the server's skill cache so the new files are listed at once.
    try {
      await request("skills/list", { cwds: [], forceReload: true });
    } catch {
      // The next skills/list reloads anyway.
    }
  }
  return changed;
}
