// Goal item 7: no user-facing Turkish is written outside the dictionaries. Letters that only
// Turkish uses must not appear in code; the dictionaries (and the Turkish manifest strings) are
// the only place for them.
import { readdirSync, readFileSync, statSync } from "node:fs";
import { join, relative } from "node:path";

const root = join(__dirname, "..", "src");
const ALLOWED = [/[\\/]i18n[\\/]/, /[\\/]generated[\\/]/];

function files(dir: string): string[] {
  return readdirSync(dir).flatMap((name) => {
    const path = join(dir, name);
    return statSync(path).isDirectory() ? files(path) : /\.(ts|tsx|css)$/.test(name) ? [path] : [];
  });
}

test("no Turkish-only letters outside the i18n dictionaries", () => {
  const offenders = files(root)
    .filter((f) => !ALLOWED.some((a) => a.test(f)))
    .flatMap((f) =>
      readFileSync(f, "utf8")
        .split("\n")
        .map((line, i) => ({ file: relative(root, f), line: i + 1, text: line }))
        .filter((l) => /[ğĞşŞıİ]/.test(l.text)),
    );
  expect(offenders).toEqual([]);
});

test("every manifest string has a Turkish translation", () => {
  const en = JSON.parse(readFileSync(join(__dirname, "..", "package.nls.json"), "utf8"));
  const tr = JSON.parse(readFileSync(join(__dirname, "..", "package.nls.tr.json"), "utf8"));
  expect(Object.keys(tr).sort()).toEqual(Object.keys(en).sort());
  const manifest = readFileSync(join(__dirname, "..", "package.json"), "utf8");
  // (keys contain dots, so compare as strings rather than as property paths)
  for (const key of manifest.match(/%[\w.]+%/g) ?? []) expect(Object.keys(en)).toContain(key.slice(1, -1));
});
