// Finds the suffice executable: the `suffice.binaryPath` setting, then PATH, then a build of
// Suffice inside an open workspace (for people working on Suffice itself).
import { existsSync, statSync } from "node:fs";
import { delimiter, join } from "node:path";

const executableNames = process.platform === "win32" ? ["suffice.exe", "suffice.cmd", "suffice"] : ["suffice"];

function isFile(path: string): boolean {
  try {
    return statSync(path).isFile();
  } catch {
    return false;
  }
}

export function findSufficeBinary(setting: string | undefined, workspaceFolders: string[]): string | undefined {
  if (setting && setting.trim()) {
    return isFile(setting.trim()) ? setting.trim() : undefined;
  }
  for (const dir of (process.env.PATH ?? "").split(delimiter)) {
    if (!dir) continue;
    for (const name of executableNames) {
      const candidate = join(dir, name);
      if (isFile(candidate)) return candidate;
    }
  }
  for (const folder of workspaceFolders) {
    for (const profile of ["release", "debug"]) {
      for (const name of executableNames) {
        const candidate = join(folder, "codex-rs", "target", profile, name);
        if (existsSync(candidate) && isFile(candidate)) return candidate;
      }
    }
  }
  return undefined;
}
