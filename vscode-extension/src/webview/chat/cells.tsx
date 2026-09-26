// Transcript cells. Each one is the webview counterpart of a TUI history cell
// (codex-rs/tui/src/history_cell/*): user message (with the invisible tag), agent markdown,
// reasoning summary, one-line exec cell whose command and coloured output open on hover, patch cell with a coloured diff, MCP tool
// call, web search, proposed plan, compaction and review markers.
import type { ThreadItem } from "@protocol/v2/ThreadItem";
import { useEffect, useMemo, useRef, useState, type CSSProperties } from "react";
import { parseAnsi } from "../../shared/ansi";
import { useApp } from "../app/context";
import { Icon } from "../components/icons";
import { Markdown } from "../components/Markdown";

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

/**
 * One line per command, like the TUI's collapsed exec cell. The full command and its output open
 * in a box beside the line while the pointer rests on it, indented like the reasoning text; a
 * click (or Enter) keeps it open. It opens upwards when there is no room below.
 */
function ExecCell({ item }: { item: Item<"commandExecution"> }) {
  const { t } = useApp();
  const output = (item.aggregatedOutput ?? "").replace(/\n+$/, "");
  const [hover, setHover] = useState(false);
  const [pinned, setPinned] = useState(false);
  const [upwards, setUpwards] = useState(false);
  const cellRef = useRef<HTMLDivElement>(null);
  const outputRef = useRef<HTMLPreElement>(null);
  const hoverTimer = useRef<ReturnType<typeof setTimeout>>();
  const open = hover || pinned;
  const running = item.status === "inProgress";
  const failed = item.status === "failed" || (item.exitCode !== null && item.exitCode !== 0);
  const command = displayCommand(item);

  const measure = () => {
    const el = cellRef.current;
    if (!el) return;
    const box = el.getBoundingClientRect();
    const view = scrollParent(el)?.getBoundingClientRect() ?? { top: 0, bottom: window.innerHeight };
    setUpwards(view.bottom - box.bottom < 220 && box.top - view.top > view.bottom - box.bottom);
  };
  // Opens after a short dwell, so moving the pointer across a list of commands opens none of them.
  const enter = () => {
    clearTimeout(hoverTimer.current);
    hoverTimer.current = setTimeout(() => (measure(), setHover(true)), HOVER_OPEN_MS);
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
        onClick={() => (measure(), setPinned(!pinned))}
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
        <div className={`sf-exec-pop ${upwards ? "is-up" : ""}`} role="region" aria-label={command}>
          <code className="sf-exec-pop-cmd">$ {command}</code>
          {output ? (
            <pre ref={outputRef} className="sf-exec-output">
              <AnsiText text={output} />
            </pre>
          ) : (
            <div className="sf-exec-empty">{running ? <span className="sf-spinner" aria-hidden="true" /> : t("chat.noOutput")}</div>
          )}
        </div>
      )}
    </div>
  );
}

function PatchCell({ item }: { item: Item<"fileChange"> }) {
  const { t, ctl } = useApp();
  const [open, setOpen] = useState<string | null>(null);
  return (
    <div className="sf-cell sf-patch">
      <div className="sf-exec-head">
        <Icon name="file" size={13} />
        <span className="sf-exec-verb">{t("chat.edited", { count: item.changes.length })}</span>
      </div>
      <ul className="sf-patch-files">
        {item.changes.map((c) => {
          const added = (c.diff.match(/^\+(?!\+\+)/gm) ?? []).length;
          const removed = (c.diff.match(/^-(?!--)/gm) ?? []).length;
          const kind = c.kind.type === "add" ? t("chat.added") : c.kind.type === "delete" ? t("chat.deleted") : t("chat.updated");
          return (
            <li key={c.path}>
              <button type="button" className="sf-disclosure" onClick={() => setOpen(open === c.path ? null : c.path)}>
                <Icon name={open === c.path ? "chevronDown" : "chevronRight"} size={12} />
                <span className="sf-patch-path" onDoubleClick={() => ctl.openFile(c.path)}>{c.path}</span>
                <span className="sf-muted">{kind}</span>
                <span className="sf-diff-add">+{added}</span>
                <span className="sf-diff-del">−{removed}</span>
              </button>
              {open === c.path && <Diff diff={c.diff} />}
            </li>
          );
        })}
      </ul>
    </div>
  );
}

function Diff({ diff }: { diff: string }) {
  return (
    <pre className="sf-diff">
      {diff.split("\n").map((line, i) => {
        const cls = line.startsWith("+") ? "sf-diff-add" : line.startsWith("-") ? "sf-diff-del" : line.startsWith("@@") ? "sf-diff-hunk" : "";
        return (
          <span key={i} className={cls}>
            {line}
            {"\n"}
          </span>
        );
      })}
    </pre>
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
