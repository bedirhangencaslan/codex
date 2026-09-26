// Transcript cells. Each one is the webview counterpart of a TUI history cell
// (codex-rs/tui/src/history_cell/*): user message (with the invisible tag), agent markdown,
// reasoning summary, one-line exec cell whose command and coloured output open on hover, patch cell laid out like a GitHub diff, MCP tool
// call, web search, proposed plan, compaction and review markers.
import type { ThreadItem } from "@protocol/v2/ThreadItem";
import { useEffect, useMemo, useRef, useState, type CSSProperties } from "react";
import { parseAnsi } from "../../shared/ansi";
import { useApp } from "../app/context";
import { ExpandBubble } from "../components/ExpandBubble";
import { Icon } from "../components/icons";
import { highlightCode, Markdown } from "../components/Markdown";
import { diffStatBlocks, parseDiff, relativePath, type DiffRow } from "../../shared/diff";

type Item<T extends ThreadItem["type"]> = Extract<ThreadItem, { type: T }>;

export function ItemCell({ item, invisible, streaming }: { item: ThreadItem; invisible: boolean; streaming: boolean }) {
  switch (item.type) {
    case "userMessage":
      return <UserCell item={item} invisible={invisible} />;
    case "agentMessage":
      return <AgentCell item={item} streaming={streaming} />;
    case "reasoning":
      return <ReasoningCell item={item} streaming={streaming} />;
    case "commandExecution":
      return <ExecCell item={item} />;
    case "fileChange":
      return <PatchCell item={item} />;
    case "mcpToolCall":
      return <ToolCell tool={`${item.server}.${item.tool}`} status={item.status} detail={item.error?.message ?? null} />;
    case "dynamicToolCall":
      return <ToolCell tool={item.tool} status={item.status} detail={null} />;
    case "webSearch":
      return <MarkerCell icon="search" textKey="chat.webSearch" detail={item.query} />;
    case "plan":
      return <PlanCell item={item} />;
    case "imageView":
      return <MarkerCell icon="file" textKey="chat.imageViewed" detail={item.path} />;
    case "contextCompaction":
      return <MarkerCell icon="layers" textKey="chat.compacted" />;
    case "enteredReviewMode":
      return <MarkerCell icon="search" textKey="chat.reviewStarted" />;
    case "exitedReviewMode":
      return <AgentCell item={{ type: "agentMessage", id: item.id, text: item.review, phase: null, memoryCitation: null, delivery: null, questions: null }} streaming={false} />;
    default:
      return null;
  }
}

function UserCell({ item, invisible }: { item: Item<"userMessage">; invisible: boolean }) {
  const { t, ctl } = useApp();
  const text = item.content.filter((c) => c.type === "text").map((c) => (c.type === "text" ? c.text : "")).join("\n");
  const attachments = item.content.filter((c) => c.type === "mention" || c.type === "skill");
  return (
    <div className={`sf-cell sf-user ${invisible ? "is-invisible" : ""}`}>
      <div className="sf-user-bubble">
        {invisible && (
          <span className="sf-invisible-tag" title={t("panel.invisibleTitle")}>
            <Icon name="eyeOff" size={12} /> {t("chat.invisibleTag")}
          </span>
        )}
        <div className="sf-user-text">{text}</div>
        {attachments.length > 0 && (
          <div className="sf-attachments">
            {attachments.map((a, i) =>
              a.type === "mention" || a.type === "skill" ? (
                <button key={i} type="button" className="sf-attachment" onClick={() => a.type === "mention" && ctl.openFile(a.path)}>
                  <Icon name={a.type === "skill" ? "sparkles" : "file"} size={12} /> {a.name}
                </button>
              ) : null,
            )}
          </div>
        )}
      </div>
    </div>
  );
}

function AgentCell({ item, streaming }: { item: Item<"agentMessage">; streaming: boolean }) {
  const { ctl, t } = useApp();
  if (!item.text && streaming) return <div className="sf-cell sf-thinking">{t("chat.thinking")}</div>;
  return (
    <div className="sf-cell sf-agent">
      <Markdown text={item.text} onOpenFile={(p) => ctl.openFile(p)} />
      {!streaming && item.text && (
        <button type="button" className="sf-cell-action" onClick={() => ctl.copy(item.text)} title={t("common.copy")}>
          <Icon name="copy" size={12} />
        </button>
      )}
    </div>
  );
}

function ReasoningCell({ item, streaming }: { item: Item<"reasoning">; streaming: boolean }) {
  const { t } = useApp();
  const [open, setOpen] = useState(false);
  const summary = item.summary.filter(Boolean).join("\n\n");
  if (!summary && !streaming) return null;
  return (
    <div className="sf-cell sf-reasoning">
      <button type="button" className="sf-disclosure" onClick={() => setOpen(!open)} aria-expanded={open}>
        <Icon name={open ? "chevronDown" : "chevronRight"} size={12} />
        {summary ? t("chat.reasoning") : t("chat.thinking")}
      </button>
      {open && summary && <Markdown text={summary} />}
    </div>
  );
}

/** The command the model asked for, not the shell wrapper (like the TUI's parsed commands). */
function displayCommand(item: Item<"commandExecution">): string {
  const actions = item.commandActions ?? [];
  const parsed = actions.map((a) => ("command" in a ? a.command : "")).filter(Boolean);
  if (parsed.length) return parsed.join(" && ");
  const m = /-Command\s+'([\s\S]*)'\s*$/.exec(item.command) ?? /-c\s+'([\s\S]*)'\s*$/.exec(item.command);
  return m ? m[1]! : item.command;
}

/** Terminal output with its colours; other escape sequences are dropped (shared/ansi.ts). */
function AnsiText({ text }: { text: string }) {
  const segments = useMemo(() => parseAnsi(text), [text]);
  return (
    <>
      {segments.map((s, i) => {
        const { fg, bg, bold, dim, italic, underline, inverse } = s.style;
        const style: CSSProperties = {
          color: inverse ? (bg ?? "var(--sf-code)") : fg,
          backgroundColor: inverse ? (fg ?? "var(--sf-text)") : bg ? `color-mix(in srgb, ${bg} 30%, transparent)` : undefined,
          fontWeight: bold ? 600 : undefined,
          opacity: dim ? 0.7 : undefined,
          fontStyle: italic ? "italic" : undefined,
          textDecoration: underline ? "underline" : undefined,
        };
        return Object.values(style).some((v) => v !== undefined) ? (
          <span key={i} style={style}>
            {s.text}
          </span>
        ) : (
          s.text
        );
      })}
    </>
  );
}

/** Command output that stays scrolled to its newest line while the command runs. */
function ExecOutput({ output, running }: { output: string; running: boolean }) {
  const ref = useRef<HTMLPreElement>(null);
  useEffect(() => {
    if (running && ref.current) ref.current.scrollTop = ref.current.scrollHeight;
  }, [running, output]);
  return (
    <pre ref={ref} className="sf-exec-output">
      <AnsiText text={output} />
    </pre>
  );
}

/**
 * One line per command, like the TUI's collapsed exec cell; the full command and its coloured
 * output open in a speech bubble beside it, from the expand button (ExpandBubble).
 */
function ExecCell({ item }: { item: Item<"commandExecution"> }) {
  const { t } = useApp();
  const output = (item.aggregatedOutput ?? "").replace(/\n+$/, "");
  const running = item.status === "inProgress";
  const failed = item.status === "failed" || (item.exitCode !== null && item.exitCode !== 0);
  const command = displayCommand(item);
  return (
    <ExpandBubble
      className={`sf-cell sf-exec ${failed ? "is-failed" : ""}`}
      label={command}
      expandLabel={t("chat.execExpand")}
      collapseLabel={t("chat.collapse")}
      head={({ open, toggle, expand }) => (
        <div
          className="sf-exec-head"
          role="button"
          tabIndex={0}
          aria-expanded={open}
          onClick={(e) => toggle(e)}
          onKeyDown={(e) => (e.key === "Enter" || e.key === " ") && (e.preventDefault(), toggle())}
        >
          <Icon name="terminal" size={13} />
          <span className="sf-exec-verb">{running ? t("chat.runningCommand") : t("chat.ranCommand")}</span>
          <code className="sf-exec-cmd">{command}</code>
          {item.exitCode !== null && item.exitCode !== 0 && <span className="sf-exec-exit">{t("chat.exitCode", { code: item.exitCode })}</span>}
          {item.durationMs !== null && <span className="sf-exec-time">{(item.durationMs / 1000).toFixed(1)}s</span>}
          {running && <span className="sf-spinner" aria-hidden="true" />}
          {expand}
        </div>
      )}
      bubble={() => (
        <>
          <code className="sf-bubble-title">$ {command}</code>
          {output ? (
            <ExecOutput output={output} running={running} />
          ) : (
            <div className="sf-exec-empty">{running ? <span className="sf-spinner" aria-hidden="true" /> : t("chat.noOutput")}</div>
          )}
        </>
      )}
    />
  );
}

/** Rows of a diff shown under a file while its bubble is closed. */
const DIFF_PREVIEW_ROWS = 4;

/** The preview: the first rows from the first change on, without hunk headers. */
function previewRows(rows: DiffRow[]): DiffRow[] {
  const content = rows.filter((r) => r.type !== "hunk" && r.type !== "note");
  const firstChange = Math.max(0, content.findIndex((r) => r.type === "add" || r.type === "del"));
  return content.slice(firstChange, firstChange + DIFF_PREVIEW_ROWS);
}

/**
 * File changes laid out like a GitHub diff: one box per file with its path, kind, +/- counts and
 * the five-block bar, then the first rows of the change. The whole diff (numbered rows, green and
 * red lines, hunk headers, code highlighted by the file's extension) opens in a speech bubble
 * beside it from the expand button, like a command's output. The TUI draws the same patch in diff_render.rs.
 */
function PatchCell({ item }: { item: Item<"fileChange"> }) {
  const { t, state } = useApp();
  const failed = item.status === "failed" || item.status === "declined";
  return (
    <div className="sf-cell sf-patch">
      <div className="sf-patch-head">
        <Icon name="file" size={13} />
        <span>{t("chat.edited", { count: item.changes.length })}</span>
        {failed && <span className="sf-exec-exit">{t(item.status === "declined" ? "chat.patchDeclined" : "chat.patchFailed")}</span>}
      </div>
      {item.changes.map((c) => (
        <PatchFile key={c.path} change={c} cwd={state.threadCwd ?? null} />
      ))}
    </div>
  );
}

function PatchFile({ change, cwd }: { change: Item<"fileChange">["changes"][number]; cwd: string | null }) {
  const { t, ctl } = useApp();
  const kind = change.kind.type;
  const parsed = useMemo(() => parseDiff(kind, change.diff), [kind, change.diff]);
  const preview = useMemo(() => previewRows(parsed.rows), [parsed]);
  const language = /\.([a-z0-9]+)$/i.exec(parsed.movedTo ?? change.path)?.[1];
  const shownPath = relativePath(change.path, cwd);
  // Nothing more to show than the preview: no bubble.
  const complete = parsed.rows.every((r) => r.type === "hunk" || r.type === "note" || preview.includes(r));
  const header = (
    <>
      <button
        type="button"
        className="sf-patch-path"
        onClick={(e) => (e.stopPropagation(), ctl.openFile(change.path))}
        title={change.path}
      >
        {shownPath}
      </button>
      {kind === "add" && <span className="sf-patch-kind is-add">{t("chat.added")}</span>}
      {kind === "delete" && <span className="sf-patch-kind is-del">{t("chat.deleted")}</span>}
      {parsed.movedTo && <span className="sf-patch-kind" title={parsed.movedTo}>{t("chat.movedTo", { path: relativePath(parsed.movedTo, cwd) })}</span>}
      <span className="sf-patch-stat">
        <span className="sf-diff-add">+{parsed.added}</span>
        <span className="sf-diff-del">−{parsed.removed}</span>
        <span className="sf-diffstat" aria-hidden="true">
          {diffStatBlocks(parsed.added, parsed.removed).map((b, i) => (
            <i key={i} className={`is-${b}`} />
          ))}
        </span>
      </span>
    </>
  );
  return (
    <ExpandBubble
      className={`sf-patch-file ${complete ? "" : "has-more"}`}
      label={shownPath}
      expandLabel={t("chat.diffExpand")}
      collapseLabel={t("chat.collapse")}
      disabled={complete}
      head={({ open, toggle, expand }) => (
        <div
          className="sf-patch-file-head"
          role={complete ? undefined : "button"}
          tabIndex={complete ? undefined : 0}
          aria-expanded={complete ? undefined : open}
          onClick={(e) => toggle(e)}
          onKeyDown={(e) => (e.key === "Enter" || e.key === " ") && (e.preventDefault(), toggle())}
        >
          {header}
          {expand}
        </div>
      )}
      bubble={() => (
        <>
          <code className="sf-bubble-title">{shownPath}</code>
          <DiffTable rows={parsed.rows} language={language} />
        </>
      )}
    >
      {preview.length > 0 && <DiffTable rows={preview} language={language} />}
    </ExpandBubble>
  );
}

function DiffTable({ rows, language }: { rows: DiffRow[]; language: string | undefined }) {
  return (
    <div className="sf-diff-wrap">
      <table className="sf-diff">
        <tbody>
          {rows.map((row, i) => {
            if (row.type === "hunk" || row.type === "note") {
              return (
                <tr key={i} className={`is-${row.type}`}>
                  <td colSpan={3}>{row.text}</td>
                </tr>
              );
            }
            const sign = row.type === "add" ? "+" : row.type === "del" ? "-" : " ";
            return (
              <tr key={i} className={`is-${row.type}`}>
                <td className="sf-diff-num">{row.oldLine ?? ""}</td>
                <td className="sf-diff-num">{row.newLine ?? ""}</td>
                <td className="sf-diff-code">
                  <span className="sf-diff-sign">{sign}</span>
                  <span dangerouslySetInnerHTML={{ __html: highlightCode(row.text, language).html }} />
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}

function ToolCell({ tool, status, detail }: { tool: string; status: string; detail: string | null }) {
  const { t } = useApp();
  return (
    <div className="sf-cell sf-marker">
      <Icon name="chip" size={13} /> {t("chat.toolCall", { tool })}
      {status === "inProgress" && <span className="sf-spinner" aria-hidden="true" />}
      {detail && <span className="sf-muted"> · {detail}</span>}
    </div>
  );
}

function PlanCell({ item }: { item: Item<"plan"> }) {
  const { t } = useApp();
  return (
    <div className="sf-cell sf-plan">
      <div className="sf-plan-title">
        <Icon name="layers" size={13} /> {t("chat.proposedPlan")}
      </div>
      <Markdown text={item.text} />
    </div>
  );
}

function MarkerCell({ icon, textKey, detail }: { icon: "search" | "file" | "layers"; textKey: "chat.webSearch" | "chat.imageViewed" | "chat.compacted" | "chat.reviewStarted"; detail?: string }) {
  const { t } = useApp();
  return (
    <div className="sf-cell sf-marker">
      <Icon name={icon} size={13} /> {t(textKey)}
      {detail && <span className="sf-muted"> · {detail}</span>}
    </div>
  );
}
