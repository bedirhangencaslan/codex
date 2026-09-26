// Markdown for agent messages and plans, with GFM tables like the TUI's markdown_render.rs and
// highlighted code blocks like its render/highlight.rs. Raw HTML from the model is never rendered as
// HTML: it is shown as text.
import hljs from "highlight.js/lib/core";
import bash from "highlight.js/lib/languages/bash";
import c from "highlight.js/lib/languages/c";
import cpp from "highlight.js/lib/languages/cpp";
import csharp from "highlight.js/lib/languages/csharp";
import css from "highlight.js/lib/languages/css";
import diff from "highlight.js/lib/languages/diff";
import dockerfile from "highlight.js/lib/languages/dockerfile";
import go from "highlight.js/lib/languages/go";
import ini from "highlight.js/lib/languages/ini";
import java from "highlight.js/lib/languages/java";
import javascript from "highlight.js/lib/languages/javascript";
import json from "highlight.js/lib/languages/json";
import markdown from "highlight.js/lib/languages/markdown";
import powershell from "highlight.js/lib/languages/powershell";
import python from "highlight.js/lib/languages/python";
import rust from "highlight.js/lib/languages/rust";
import shell from "highlight.js/lib/languages/shell";
import sql from "highlight.js/lib/languages/sql";
import typescript from "highlight.js/lib/languages/typescript";
import xml from "highlight.js/lib/languages/xml";
import yaml from "highlight.js/lib/languages/yaml";
import { Marked } from "marked";
import { memo, useMemo } from "react";

// The languages agents write most; each registers its own aliases (ts, js, sh, py, rs, yml, html,
// ps1, ...). `toml` is highlighted with the ini grammar, as highlight.js itself does.
const LANGUAGES = { bash, c, cpp, csharp, css, diff, dockerfile, go, ini, java, javascript, json, markdown, powershell, python, rust, shell, sql, typescript, xml, yaml };
for (const [name, language] of Object.entries(LANGUAGES)) hljs.registerLanguage(name, language);

const escapeHtml = (text: string) =>
  text.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");

/** Highlighted HTML for a fenced block, or the escaped text when its language is unknown. */
export function highlightCode(code: string, lang: string | undefined): { html: string; language: string | null } {
  const name = (lang ?? "").trim().split(/\s+/)[0]!.toLowerCase();
  if (name && hljs.getLanguage(name)) {
    return { html: hljs.highlight(code, { language: name, ignoreIllegals: true }).value, language: name };
  }
  return { html: escapeHtml(code), language: null };
}

const alignStyle = (align: "left" | "right" | "center" | null | undefined) =>
  align ? ` style="text-align:${align}"` : "";

const marked = new Marked({
  gfm: true,
  breaks: false,
  renderer: {
    html({ text }) {
      return escapeHtml(text);
    },
    link({ href, text }) {
      return `<a href="${escapeHtml(href ?? "")}" title="${escapeHtml(href ?? "")}">${text}</a>`;
    },
    code({ text, lang }) {
      const { html, language } = highlightCode(text.replace(/\n$/, ""), lang);
      const cls = language ? ` class="hljs language-${escapeHtml(language)}"` : "";
      return `<pre><code${cls}>${html}\n</code></pre>\n`;
    },
    table(token) {
      // Wrap tables so wide ones scroll instead of breaking the side bar layout. Column alignment
      // (`:---:`, `---:`) is kept.
      const header = token.header
        .map((cell, i) => `<th${alignStyle(token.align[i])}>${this.parser.parseInline(cell.tokens)}</th>`)
        .join("");
      const rows = token.rows
        .map(
          (row) =>
            `<tr>${row.map((cell, i) => `<td${alignStyle(token.align[i])}>${this.parser.parseInline(cell.tokens)}</td>`).join("")}</tr>`,
        )
        .join("");
      return `<div class="sf-table-wrap"><table><thead><tr>${header}</tr></thead><tbody>${rows}</tbody></table></div>`;
    },
  },
});

export function renderMarkdown(text: string): string {
  return marked.parse(text, { async: false }) as string;
}

export const Markdown = memo(function Markdown({ text, onOpenFile }: { text: string; onOpenFile?: (path: string) => void }) {
  const html = useMemo(() => renderMarkdown(text), [text]);
  return (
    <div
      className="sf-md"
      onClick={(e) => {
        const anchor = (e.target as HTMLElement).closest("a");
        if (!anchor) return;
        e.preventDefault();
        const href = anchor.getAttribute("href") ?? "";
        if (onOpenFile && !/^[a-z]+:\/\//i.test(href)) onOpenFile(href.replace(/#L\d+.*$/, ""));
      }}
      dangerouslySetInnerHTML={{ __html: html }}
    />
  );
});
