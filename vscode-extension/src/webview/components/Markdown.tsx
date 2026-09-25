// Markdown for agent messages and plans, with GFM tables like the TUI's markdown_render.rs.
// Raw HTML from the model is never rendered as HTML: it is shown as text.
import { Marked } from "marked";
import { memo, useMemo } from "react";

const escapeHtml = (text: string) =>
  text.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");

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
    table(token) {
      // Wrap tables so wide ones scroll instead of breaking the side bar layout.
      const header = token.header.map((cell) => `<th>${this.parser.parseInline(cell.tokens)}</th>`).join("");
      const rows = token.rows
        .map((row) => `<tr>${row.map((cell) => `<td>${this.parser.parseInline(cell.tokens)}</td>`).join("")}</tr>`)
        .join("");
      return `<div class="sf-table-wrap"><table><thead><tr>${header}</tr></thead><tbody>${rows}</tbody></table></div>`;
    },
  },
});

export const Markdown = memo(function Markdown({ text, onOpenFile }: { text: string; onOpenFile?: (path: string) => void }) {
  const html = useMemo(() => marked.parse(text, { async: false }) as string, [text]);
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
