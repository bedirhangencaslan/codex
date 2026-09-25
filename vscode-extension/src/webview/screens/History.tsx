// Local chat history: the sessions Suffice saved (rollouts), listed by thread/list and resumed by
// thread/resume — the TUI's resume picker (codex-rs/tui/src/resume_picker.rs) as a screen.
import type { Thread } from "@protocol/v2/Thread";
import { useEffect, useState } from "react";
import { formatRelativeTime } from "../../shared/i18n";
import { useApp } from "../app/context";
import { Icon } from "../components/icons";
import { Button, Empty, SearchBox } from "../components/ui";

export function HistoryScreen() {
  const { state, ctl, t, locale } = useApp();
  const [threads, setThreads] = useState<Thread[] | null>(null);
  const [cursor, setCursor] = useState<string | null>(null);
  const [search, setSearch] = useState("");
  const ready = state.server.state === "ready";

  const load = async (append: boolean, term: string, after: string | null) => {
    try {
      const r = await ctl.session.threadList({ limit: 30, cursor: after, sortKey: "updated_at", sortDirection: "desc", searchTerm: term || null });
      setThreads((prev) => (append && prev ? [...prev, ...r.data] : r.data));
      setCursor(r.nextCursor);
    } catch {
      setThreads([]);
    }
  };

  useEffect(() => {
    if (!ready) return;
    const handle = setTimeout(() => void load(false, search, null), search ? 200 : 0);
    return () => clearTimeout(handle);
  }, [ready, search]);

  return (
    <div className="sf-screen">
      <header className="sf-screen-head">
        <h1>{t("history.title")}</h1>
      </header>
      <SearchBox value={search} onChange={setSearch} placeholder={t("history.searchPlaceholder")} />
      {threads === null ? (
        <Empty>{t("common.loading")}</Empty>
      ) : threads.length === 0 ? (
        <Empty>{t("history.empty")}</Empty>
      ) : (
        <ul className="sf-history">
          {threads.map((thread) => (
            <li key={thread.id}>
              <button type="button" className={`sf-history-row ${thread.id === state.chat.threadId ? "is-current" : ""}`} onClick={() => void ctl.resume(thread.id)}>
                <span className="sf-history-title">{thread.name || thread.preview || t("history.untitled")}</span>
                <span className="sf-history-meta">
                  <Icon name="history" size={12} /> {formatRelativeTime(locale, thread.updatedAt)}
                  {thread.model && <span> · {thread.model}</span>}
                  {thread.cwd && <span className="sf-history-cwd"> · {String(thread.cwd).split(/[\\/]/).pop()}</span>}
                </span>
              </button>
            </li>
          ))}
        </ul>
      )}
      {cursor && (
        <Button variant="ghost" onClick={() => void load(true, search, cursor)}>
          {t("history.loadMore")}
        </Button>
      )}
    </div>
  );
}
