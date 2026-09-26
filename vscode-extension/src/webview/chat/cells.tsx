// Transcript cells. Each one is the webview counterpart of a TUI history cell
// (codex-rs/tui/src/history_cell/*): user message (with the invisible tag), agent markdown,
// reasoning summary, one-line exec cell whose command and coloured output open on hover, patch cell laid out like a GitHub diff, MCP tool
// call, web search, proposed plan, compaction and review markers.
import type { ThreadItem } from "@protocol/v2/ThreadItem";
import { useEffect, useLayoutEffect, useMemo, useRef, useState, type CSSProperties, type MouseEvent } from "react";
import { parseAnsi } from "../../shared/ansi";
import { useApp } from "../app/context";
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

/** The nearest scrolling ancestor, whose visible area decides where the details open. */
function scrollParent(el: HTMLElement): HTMLElement | null {
  for (let p = el.parentElement; p; p = p.parentElement) {
    if (/(auto|scroll)/.test(getComputedStyle(p).overflowY)) return p;
  }
  return null;
}

const HOVER_OPEN_MS = 300;
/** Horizontal shift of the bubble from the line, and the gap kept to the view's edges. */
const BUBBLE_SHIFT = 22;
const BUBBLE_MARGIN = 6;
const BUBBLE_TAIL = 7;

/**
 * Whether the view sits in the right half of the VS Code window (the secondary side bar, or the
 * side bar moved to the right). A webview is told nothing about where it is docked, so this
 * compares the pointer's screen and view coordinates with the window's own position.
 */
function dockedRight(e: { screenX: number; clientX: number }): boolean {
  if (!window.outerWidth) return false;
  const viewCenter = e.screenX - e.clientX + window.innerWidth / 2;
  return viewCenter > window.screenX + window.outerWidth / 2;
}

/**
 * One line per command, like the TUI's collapsed exec cell. The full command and its output open
 * in a speech bubble level with the line while the pointer rests on it, shifted to the right with
 * its tail on the line; in a view docked on the right of the window it is mirrored and opens to the
 * left. A click (or Enter) keeps it open. The bubble is up to 90% of the view's height, so most
 * output shows without scrolling; it starts level with the line and moves up as far as needed to
 * stay on screen, the tail still on the line. A webview cannot draw outside its own view, so the
 * bubble stays inside the side bar.
 */
function ExecCell({ item }: { item: Item<"commandExecution"> }) {
  const { t } = useApp();
  const output = (item.aggregatedOutput ?? "").replace(/\n+$/, "");
  const [hover, setHover] = useState(false);
  const [pinned, setPinned] = useState(false);
  const [toLeft, setToLeft] = useState(false);
  const cellRef = useRef<HTMLDivElement>(null);
  const outputRef = useRef<HTMLPreElement>(null);
  const popRef = useRef<HTMLDivElement>(null);
  const hoverTimer = useRef<ReturnType<typeof setTimeout>>();
  const open = hover || pinned;
  const running = item.status === "inProgress";
  const failed = item.status === "failed" || (item.exitCode !== null && item.exitCode !== 0);
  const command = displayCommand(item);

  const measure = (e?: { screenX: number; clientX: number }) => {
    if (e) setToLeft(dockedRight(e));
  };
  /** Places the bubble in the view: level with the line, moved up only as far as it must be. */
  const place = () => {
    const cell = cellRef.current;
    const pop = popRef.current;
    if (!cell || !pop) return;
    const line = cell.getBoundingClientRect();
    const height = pop.offsetHeight;
    const top = Math.max(BUBBLE_MARGIN, Math.min(line.top + 2, window.innerHeight - BUBBLE_MARGIN - height));
    const tail = Math.max(6, Math.min(line.top + line.height / 2 - top - BUBBLE_TAIL, height - 2 * BUBBLE_TAIL - 6));
    const right = window.innerWidth - line.right;
    pop.style.top = `${top}px`;
    pop.style.left = `${toLeft ? line.left : line.left + BUBBLE_SHIFT}px`;
    pop.style.right = `${toLeft ? right + BUBBLE_SHIFT : right}px`;
    pop.style.setProperty("--tail-top", `${tail}px`);
  };
  useLayoutEffect(() => {
    if (open) place();
  });
  // Follow the line when the chat scrolls or the view is resized while the bubble is open.
  useEffect(() => {
    if (!open) return;
    const scroller = cellRef.current ? scrollParent(cellRef.current) : null;
    scroller?.addEventListener("scroll", place, { passive: true });
    window.addEventListener("resize", place);
    return () => {
      scroller?.removeEventListener("scroll", place);
      window.removeEventListener("resize", place);
    };
  }, [open, toLeft]);
  // Opens after a short dwell, so moving the pointer across a list of commands opens none of them.
  const enter = (e: MouseEvent<HTMLDivElement>) => {
    const at = { screenX: e.screenX, clientX: e.clientX };
    clearTimeout(hoverTimer.current);
    hoverTimer.current = setTimeout(() => (measure(at), setHover(true)), HOVER_OPEN_MS);
  };
  const leave = () => {
    clearTimeout(hoverTimer.current);
    setHover(false);
  };
  useEffect(() => () => clearTimeout(hoverTimer.current), []);
  // Live output stays scrolled to its newest line.
  useEffect(() => {
    if (open && running && outputRef.current) outputRef.current.scrollTop = outputRef.current.scrollHeight;
  }, [open, running, output]);

  return (
    <div
      ref={cellRef}
      className={`sf-cell sf-exec ${failed ? "is-failed" : ""} ${open ? "is-open" : ""}`}
      onMouseEnter={enter}
      onMouseLeave={leave}
    >
      <button
        type="button"
        className="sf-exec-head"
        onClick={(e) => (measure(e.detail > 0 ? e : undefined), setPinned(!pinned))}
        aria-expanded={open}
        title={t("chat.execExpand")}
      >
        <Icon name="terminal" size={13} />
        <span className="sf-exec-verb">{running ? t("chat.runningCommand") : t("chat.ranCommand")}</span>
        <code className="sf-exec-cmd">{command}</code>
        {item.exitCode !== null && item.exitCode !== 0 && <span className="sf-exec-exit">{t("chat.exitCode", { code: item.exitCode })}</span>}
        {item.durationMs !== null && <span className="sf-exec-time">{(item.durationMs / 1000).toFixed(1)}s</span>}
        {running && <span className="sf-spinner" aria-hidden="true" />}
      </button>
      {open && (
        <div ref={popRef} className={`sf-exec-pop ${toLeft ? "is-left" : ""}`} role="region" aria-label={command}>
          <div className="sf-exec-pop-body">
            <code className="sf-exec-pop-cmd">$ {command}</code>
            {output ? (
              <pre ref={outputRef} className="sf-exec-output">
                <AnsiText text={output} />
              </pre>
            ) : (
              <div className="sf-exec-empty">{running ? <span className="sf-spinner" aria-hidden="true" /> : t("chat.noOutput")}</div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}

/** Diffs longer than this start collapsed; the header still shows their size. */
const DIFF_OPEN_ROWS = 300;

/**
 * File changes laid out like a GitHub diff: one box per file with its path, kind, +/- counts and
 * the five-block bar, then numbered rows (old and new line), green and red lines, hunk headers and
 * the code highlighted by the file's extension. The TUI draws the same patch in diff_render.rs.
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
  const [open, setOpen] = useState(parsed.rows.length <= DIFF_OPEN_ROWS);
  const language = /\.([a-z0-9]+)$/i.exec(parsed.movedTo ?? change.path)?.[1];
  const shownPath = relativePath(change.path, cwd);
  return (
    <div className="sf-patch-file">
      <div className="sf-patch-file-head">
        <button type="button" className="sf-patch-toggle" onClick={() => setOpen(!open)} aria-expanded={open} aria-label={shownPath}>
          <Icon name={open ? "chevronDown" : "chevronRight"} size={12} />
        </button>
        <button type="button" className="sf-patch-path" onClick={() => ctl.openFile(change.path)} title={change.path}>
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
      </div>
      {open && parsed.rows.length > 0 && <DiffTable rows={parsed.rows} language={language} />}
    </div>
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
