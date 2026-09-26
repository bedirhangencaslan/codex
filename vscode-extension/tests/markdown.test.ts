// Agent messages are rendered by the webview itself (the TUI draws the same markdown with
// markdown_render.rs): tables keep their column alignment and code blocks are highlighted.
import { highlightCode, renderMarkdown } from "../src/webview/components/Markdown";

describe("markdown rendering", () => {
  test("table columns keep their alignment", () => {
    const html = renderMarkdown("| a | b | c |\n|:---|:---:|---:|\n| 1 | 2 | 3 |\n");
    expect(html).toContain('<th style="text-align:left">a</th>');
    expect(html).toContain('<td style="text-align:center">2</td>');
    expect(html).toContain('<td style="text-align:right">3</td>');
  });

  test("columns without an alignment get no style", () => {
    expect(renderMarkdown("| a |\n|---|\n| 1 |\n")).toContain("<td>1</td>");
  });

  test("a code block in a known language is highlighted, aliases included", () => {
    const html = renderMarkdown("```ts\nconst x = 1;\n```\n");
    expect(html).toContain('<code class="hljs language-ts">');
    expect(html).toContain('<span class="hljs-keyword">const</span>');
    expect(highlightCode("[a]\nb = 1", "toml").language).toBe("toml");
    expect(highlightCode("ls -la", "sh").html).toContain("hljs-");
  });

  test("an unknown or missing language is shown as escaped plain text", () => {
    expect(renderMarkdown("```\n<b>x</b>\n```\n")).toContain("<pre><code>&lt;b&gt;x&lt;/b&gt;\n</code></pre>");
    expect(highlightCode("<i>", "brainfuck")).toEqual({ html: "&lt;i&gt;", language: null });
  });

  test("raw HTML from the model is still shown as text", () => {
    expect(renderMarkdown("hi <script>alert(1)</script>")).not.toContain("<script>");
  });
});
