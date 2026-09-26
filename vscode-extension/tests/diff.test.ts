// File changes arrive in three shapes (item_builders.rs format_file_change_diff); each becomes
// numbered rows like a GitHub diff.
import { diffStatBlocks, parseDiff, relativePath } from "../src/shared/diff";

describe("file change diffs", () => {
  test("a unified diff gets old and new line numbers per hunk", () => {
    const diff = "@@ -4,3 +4,4 @@\n keep\n-old\n+new\n+more\n tail\n@@ -20,1 +21,1 @@ fn x()\n-a\n+b\n";
    const parsed = parseDiff("update", diff);
    expect(parsed.added).toBe(3);
    expect(parsed.removed).toBe(2);
    expect(parsed.rows.slice(0, 6)).toEqual([
      { type: "hunk", text: "@@ -4,3 +4,4 @@" },
      { type: "ctx", oldLine: 4, newLine: 4, text: "keep" },
      { type: "del", oldLine: 5, newLine: null, text: "old" },
      { type: "add", oldLine: null, newLine: 5, text: "new" },
      { type: "add", oldLine: null, newLine: 6, text: "more" },
      { type: "ctx", oldLine: 6, newLine: 7, text: "tail" },
    ]);
    expect(parsed.rows[7]).toEqual({ type: "del", oldLine: 20, newLine: null, text: "a" });
    expect(parsed.rows[8]).toEqual({ type: "add", oldLine: null, newLine: 21, text: "b" });
  });

  test("file headers before the first hunk and 'no newline' markers", () => {
    const parsed = parseDiff("update", "--- a/x\n+++ b/x\n@@ -1 +1 @@\n-a\n+b\n\\ No newline at end of file\n");
    expect(parsed.added).toBe(1);
    expect(parsed.removed).toBe(1);
    expect(parsed.rows[parsed.rows.length - 1]).toEqual({ type: "note", text: "No newline at end of file" });
  });

  test("an added file is its whole content, every line new", () => {
    const parsed = parseDiff("add", "# Title\n\n+ not a diff sign\n");
    expect(parsed.added).toBe(3);
    expect(parsed.removed).toBe(0);
    expect(parsed.rows[2]).toEqual({ type: "add", oldLine: null, newLine: 3, text: "+ not a diff sign" });
  });

  test("a deleted file is its whole content, every line removed", () => {
    const parsed = parseDiff("delete", "a\nb");
    expect(parsed.removed).toBe(2);
    expect(parsed.rows[1]).toEqual({ type: "del", oldLine: 2, newLine: null, text: "b" });
  });

  test("a moved file keeps its diff and reports the new path", () => {
    const parsed = parseDiff("update", "@@ -1 +1 @@\n-a\n+b\n\n\nMoved to: C:\\w\\new.ts");
    expect(parsed.movedTo).toBe("C:\\w\\new.ts");
    expect(parsed.rows).toHaveLength(3);
  });

  test("the five-block bar", () => {
    expect(diffStatBlocks(83, 33)).toEqual(["add", "add", "add", "add", "del"]);
    expect(diffStatBlocks(1, 100)).toEqual(["add", "del", "del", "del", "del"]);
    expect(diffStatBlocks(10, 0)).toEqual(["add", "add", "add", "add", "add"]);
    expect(diffStatBlocks(0, 0)).toEqual(["none", "none", "none", "none", "none"]);
  });

  test("paths inside the chat's folder are shown relative to it", () => {
    expect(relativePath("C:\\Users\\me\\proj\\src\\a.ts", "c:\\users\\me\\proj")).toBe("src/a.ts");
    expect(relativePath("/home/me/proj/a.ts", "/home/me/proj/")).toBe("a.ts");
    expect(relativePath("/etc/hosts", "/home/me/proj")).toBe("/etc/hosts");
    expect(relativePath("/home/me/project2/a.ts", "/home/me/proj")).toBe("/home/me/project2/a.ts");
  });
});
