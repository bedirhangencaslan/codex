// Goal item 12: the chat, evolved from the TUI's ChatWidget — transcript of history cells, the
// server's pending requests inline, the meters, the composer and the panel row under it.
import { useEffect, useRef } from "react";
import type { Notice, TurnView } from "../state/chatReducer";
import { useApp } from "../app/context";
import { Icon } from "../components/icons";
import { ItemCell } from "./cells";
import { Composer } from "./Composer";
import { Meters } from "./Meters";
import { ServerRequests } from "./ServerRequests";
import { Panels, Toolbar } from "./Toolbar";

export function ChatScreen() {
  const { state, t, ctl } = useApp();
  const scroller = useRef<HTMLDivElement>(null);
  const stick = useRef(true);
  const invisible = new Set(state.chat.threadId ? (state.init?.invisibleTurns[state.chat.threadId] ?? []) : []);

  // Follow new output unless the reader scrolled up (the TUI's scrollback behaves the same).
  useEffect(() => {
    const el = scroller.current;
    if (el && stick.current) el.scrollTop = el.scrollHeight;
  }, [state.chat, state.serverRequests]);

  const empty = state.chat.turns.length === 0 && state.chat.notices.length === 0;
  return (
    <div className="sf-chat">
      <div
        className="sf-transcript"
        ref={scroller}
        onScroll={(e) => {
          const el = e.currentTarget;
          stick.current = el.scrollHeight - el.scrollTop - el.clientHeight < 40;
        }}
      >
        {empty ? (
          <div className="sf-chat-empty">
            <Icon name="chat" size={28} />
            <h2>{t("chat.emptyTitle")}</h2>
            <p className="sf-muted">{t("chat.emptyBody", { cwd: ctl.cwd ?? "–" })}</p>
          </div>
        ) : (
          <>
            {state.chat.notices.map((n, i) => (
              <NoticeCell key={`n${i}`} notice={n} />
            ))}
            {state.chat.turns.map((turn) => (
              <TurnBlock key={turn.id} turn={turn} invisible={invisible.has(turn.id)} active={turn.id === state.chat.activeTurnId} />
            ))}
          </>
        )}
        <ServerRequests />
      </div>
      <div className="sf-dock">
        <Panels />
        <Meters />
        <Composer />
        <Toolbar />
      </div>
    </div>
  );
}

function TurnBlock({ turn, invisible, active }: { turn: TurnView; invisible: boolean; active: boolean }) {
  const { t } = useApp();
  const lastItem = turn.items[turn.items.length - 1];
  return (
    <div className={`sf-turn ${invisible ? "is-invisible" : ""}`}>
      {turn.items.map((item) => (
        <ItemCell key={item.id} item={item} invisible={invisible && item.type === "userMessage"} streaming={active && item === lastItem} />
      ))}
      {turn.plan && turn.plan.length > 0 && (
        <div className="sf-cell sf-plan">
          <div className="sf-plan-title">
            <Icon name="layers" size={13} /> {t("chat.plan")}
          </div>
          {turn.planExplanation && <p className="sf-muted">{turn.planExplanation}</p>}
          <ul className="sf-plan-steps">
            {turn.plan.map((s, i) => (
              <li key={i} className={`is-${s.status}`}>
                <span className="sf-plan-box" aria-hidden="true">{s.status === "completed" ? "✓" : s.status === "inProgress" ? "•" : ""}</span>
                {s.step}
              </li>
            ))}
          </ul>
        </div>
      )}
      {turn.notices.map((n, i) => (
        <NoticeCell key={i} notice={n} />
      ))}
      {active && turn.items.every((i) => i.type === "userMessage") && <div className="sf-cell sf-thinking">{t("chat.thinking")}</div>}
      {turn.status === "interrupted" && <div className="sf-cell sf-marker">{t("chat.interrupted")}</div>}
      {turn.status === "failed" && <NoticeCell notice={{ kind: "error", text: t("chat.failed", { message: turn.errorMessage ?? "" }) }} />}
    </div>
  );
}

function NoticeCell({ notice }: { notice: Notice }) {
  const icon = notice.kind === "info" ? "info" : "warning";
  return (
    <div className={`sf-cell sf-notice-cell is-${notice.kind}`} role={notice.kind === "error" ? "alert" : undefined}>
      <Icon name={icon} size={13} />
      <span>{notice.text}</span>
    </div>
  );
}
