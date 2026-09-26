// Interface styles: the catalogue, the SKILL.md files Codex reads, where they are installed, and
// how picking a style combines with the other selected skills.
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import * as path from "node:path";
import type { SkillMetadata } from "@protocol/v2/SkillMetadata";
import { installUiStyleSkills } from "../src/extension/uiStyleSkills";
import { LOCALES } from "../src/shared/i18n";
import { isUiStyleSkill, skillMarkdown, skillName, UI_STYLE_GROUPS, UI_STYLE_ROOT, UI_STYLES } from "../src/shared/uiStyles";
import { skillRules } from "../src/webview/app/controller";

describe("the style catalogue", () => {
  test("ids and skill names are unique, every group has styles, every style a known group", () => {
    const ids = UI_STYLES.map((s) => s.id);
    expect(new Set(ids).size).toBe(ids.length);
    for (const g of UI_STYLE_GROUPS) expect(UI_STYLES.some((s) => s.group === g.id)).toBe(true);
    for (const s of UI_STYLES) expect(UI_STYLE_GROUPS.map((g) => g.id)).toContain(s.group);
    expect(UI_STYLES.length).toBeGreaterThanOrEqual(20);
  });

  test("every style and group has its name and description in every locale", () => {
    for (const locale of LOCALES) {
      const messages = locale.messages as Record<string, string>;
      for (const s of UI_STYLES) {
        expect(messages[`uiStyle.${s.id}.name`]).toBeTruthy();
        expect(messages[`uiStyle.${s.id}.desc`]).toBeTruthy();
      }
      for (const g of UI_STYLE_GROUPS) {
        expect(messages[g.titleKey]).toBeTruthy();
        expect(messages[g.descKey]).toBeTruthy();
      }
    }
  });

  test("SKILL.md front matter is what Codex's parser accepts (skills/src/parser.rs)", () => {
    for (const s of UI_STYLES) {
      const md = skillMarkdown(s);
      const front = /^---\nname: (.+)\ndescription: (.+)\n---\n/.exec(md);
      expect(front).not.toBeNull();
      const [, name, description] = front!;
      expect(name).toBe(skillName(s.id));
      expect(name!.length).toBeLessThanOrEqual(64);
      // A JSON string is a valid double-quoted YAML scalar, whatever colons the summary holds.
      const text = JSON.parse(description!) as string;
      expect(text.length).toBeLessThanOrEqual(1024);
      expect(text).toContain(s.skill.summary);
      expect(md).toContain("## Design tokens");
      expect(md).toContain("```css");
    }
  });

  test("style skills are recognised by name, other skills are not", () => {
    expect(isUiStyleSkill("ui-glassmorphism")).toBe(true);
    expect(isUiStyleSkill("ui-unknown")).toBe(false);
    expect(isUiStyleSkill("pdf")).toBe(false);
  });
});

describe("picking styles next to other skills", () => {
  const skill = (name: string): SkillMetadata => ({ name, description: name, path: `/s/${name}/SKILL.md`, scope: "user", enabled: true }) as SkillMetadata;
  const skills = [skill("pdf"), skill("docx"), skill("ui-glassmorphism"), skill("ui-neumorphism")];
  const pick = (...names: string[]) => names.map((n) => ({ name: n, path: `/s/${n}/SKILL.md` }));

  test("a style alone turns that style on and the other styles off, and leaves regular skills alone", () => {
    expect(skillRules(pick("ui-glassmorphism"), skills)).toEqual([
      { path: "/s/ui-glassmorphism/SKILL.md", enabled: true },
      { path: "/s/ui-neumorphism/SKILL.md", enabled: false },
    ]);
  });

  test("a regular skill still limits the regular skills; styles stay off unless picked", () => {
    expect(skillRules(pick("pdf", "ui-neumorphism"), skills)).toEqual([
      { path: "/s/pdf/SKILL.md", enabled: true },
      { path: "/s/docx/SKILL.md", enabled: false },
      { path: "/s/ui-glassmorphism/SKILL.md", enabled: false },
      { path: "/s/ui-neumorphism/SKILL.md", enabled: true },
    ]);
  });

  test("no selection, no rules", () => {
    expect(skillRules([], skills)).toEqual([]);
  });
});

describe("installing the style skills", () => {
  let home: string;
  beforeEach(() => (home = mkdtempSync(path.join(tmpdir(), "suffice-styles-"))));
  afterEach(() => rmSync(home, { recursive: true, force: true }));

  test("writes every SKILL.md under <home>/skills/ui-styles and turns new ones off once", async () => {
    const calls: Array<{ method: string; params: any }> = [];
    const request = async (method: string, params: unknown) => {
      calls.push({ method, params });
      return {};
    };
    expect(await installUiStyleSkills(home, request, () => {})).toBe(true);
    for (const s of UI_STYLES) {
      const file = path.join(home, "skills", UI_STYLE_ROOT, s.id, "SKILL.md");
      expect(readFileSync(file, "utf8")).toBe(skillMarkdown(s));
    }
    const offs = calls.filter((c) => c.method === "skills/config/write");
    expect(offs).toHaveLength(UI_STYLES.length);
    for (const c of offs) expect(c.params.enabled).toBe(false);
    expect(calls.at(-1)).toEqual({ method: "skills/list", params: { cwds: [], forceReload: true } });

    // A second run with nothing to do changes nothing and writes no config.
    calls.length = 0;
    expect(await installUiStyleSkills(home, request, () => {})).toBe(false);
    expect(calls).toEqual([]);
  });

  test("an outdated file is rewritten without touching the user's on/off choice", async () => {
    const calls: string[] = [];
    const request = async (method: string) => (calls.push(method), {});
    await installUiStyleSkills(home, request, () => {});
    const file = path.join(home, "skills", UI_STYLE_ROOT, UI_STYLES[0]!.id, "SKILL.md");
    writeFileSync(file, "old");
    calls.length = 0;
    expect(await installUiStyleSkills(home, request, () => {})).toBe(true);
    expect(readFileSync(file, "utf8")).toBe(skillMarkdown(UI_STYLES[0]!));
    expect(calls).toEqual(["skills/list"]);
  });
});
