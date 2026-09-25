import { readFileSync } from "node:fs";
import { join } from "node:path";
import { parseSlashCommands, renderCatalog, SLASH_COMMAND_SOURCE } from "../scripts/slashCommandSource";
import catalog from "../src/shared/generated/slash-commands.json";
import { LOCALES, localeByCode, resolveLocale, translate, type MessageKey } from "../src/shared/i18n";
import { en } from "../src/shared/i18n/en";
import { commandSets, findCommand, matchCommands, parseSlashInput, registerCommandSet } from "../src/shared/slashCommands";

const repo = join(__dirname, "..", "..");

describe("slash command catalog", () => {
  test("the generated JSON matches the TUI source (rerun scripts/gen-slash-commands.ts)", () => {
    const source = readFileSync(join(repo, SLASH_COMMAND_SOURCE), "utf8");
    const onDisk = readFileSync(join(__dirname, "..", "src", "shared", "generated", "slash-commands.json"), "utf8");
    expect(onDisk.replace(/\r\n/g, "\n")).toBe(renderCatalog(parseSlashCommands(source, SLASH_COMMAND_SOURCE)));
  });

  const taught = commandSets().flatMap((s) => s.commands);

  test("every taught command has a description, a when and an example in every locale", () => {
    for (const locale of LOCALES) {
      for (const c of taught) {
        for (const k of [c.descKey, c.whenKey!, c.exampleKey!]) {
          expect(locale.messages[k]).toBeTruthy();
        }
      }
    }
  });

  test("the English description is the TUI's, verbatim", () => {
    for (const c of catalog.commands) {
      const k = `slash.${c.name}.desc` as MessageKey;
      if (k in en) expect(en[k]).toBe(c.description);
    }
    expect(taught.length).toBe(59);
  });

  test("aliases, fuzzy matching and argument parsing", () => {
    expect(findCommand("cwd")?.name).toBe("pwd");
    expect(findCommand("clean")?.name).toBe("stop");
    expect(findCommand("rollout")).toBeUndefined();
    expect(matchCommands("re")[0]?.name).toBe("review");
    expect(matchCommands("rsm").map((c) => c.name)).toContain("resume");
    expect(parseSlashInput("/plan add caching  ")).toEqual({ name: "plan", args: "add caching" });
    expect(parseSlashInput("hello")).toBeUndefined();
  });

  test("a new command set shows up without touching the built-in one", () => {
    registerCommandSet({
      id: "test-set",
      titleKey: "commands.setSuffice",
      source: "test",
      commands: [{ ...findCommand("model")!, name: "zz-custom", aliases: [] }],
    });
    expect(findCommand("zz-custom")?.name).toBe("zz-custom");
    expect(commandSets().map((s) => s.id)).toEqual(["suffice-tui", "test-set"]);
  });
});

describe("i18n dictionaries", () => {
  const placeholders = (s: string) => [...s.matchAll(/\{(\w+)\}/g)].map((m) => m[1]).sort();

  test("every locale has exactly the reference keys, none empty, same placeholders", () => {
    const keys = Object.keys(en).sort();
    for (const locale of LOCALES) {
      expect(Object.keys(locale.messages).sort()).toEqual(keys);
      for (const k of keys as MessageKey[]) {
        expect(locale.messages[k].trim()).not.toBe("");
        expect(placeholders(locale.messages[k])).toEqual(placeholders(en[k]));
      }
    }
  });

  test("locale resolution follows the setting, then VS Code, then English", () => {
    expect(resolveLocale("tr", "en")).toBe("tr");
    expect(resolveLocale("auto", "tr-TR")).toBe("tr");
    expect(resolveLocale(undefined, "de")).toBe("en");
    expect(resolveLocale("xx", "tr")).toBe("tr");
  });

  test("interpolation", () => {
    const tr = localeByCode("tr");
    expect(translate(tr, "meter.context", { percent: 42 })).toBe("bağlam %42");
    expect(translate(tr, "chat.exitCode", {})).toBe("çıkış {code}");
  });
});
