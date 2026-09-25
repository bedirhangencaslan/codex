// Regenerates src/shared/generated/slash-commands.json from the TUI source.
//   node scripts/gen-slash-commands.ts      (Node >= 23 strips the types itself)
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { parseSlashCommands, renderCatalog, SLASH_COMMAND_SOURCE } from "./slashCommandSource.ts";

const here = dirname(fileURLToPath(import.meta.url));
const repo = join(here, "..", "..");
const catalog = parseSlashCommands(readFileSync(join(repo, SLASH_COMMAND_SOURCE), "utf8"), SLASH_COMMAND_SOURCE);
const out = join(here, "..", "src", "shared", "generated", "slash-commands.json");
mkdirSync(dirname(out), { recursive: true });
writeFileSync(out, renderCatalog(catalog));
console.log(`wrote ${catalog.commands.length} commands to ${out}`);
