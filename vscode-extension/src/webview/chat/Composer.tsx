// The chat input, evolved from the TUI composer (codex-rs/tui/src/bottom_pane/chat_composer.rs):
// Enter submits and Shift+Enter breaks the line; while a turn runs, Enter queues (the TUI's Tab);
// Up/Down recall earlier messages; `/` opens the command popup (command_popup.rs matching);
// `@` searches files like the file_search popup and writes the picked path into the text; pastes over
// LARGE_PASTE_CHARS become a "[Pasted Content N chars]" placeholder expanded on send.
import type { FuzzyFileSearchResult } from "@protocol/FuzzyFileSearchResult";
import { useEffect, useMemo, useRef, useState, type KeyboardEvent } from "react";
import { matchCommands, type SlashCommandEntry } from "../../shared/slashCommands";
import { useApp } from "../app/context";
import { filePathToken, LARGE_PASTE_CHARS } from "../app/controller";
import { Icon } from "../components/icons";

interface ViewState {
  history?: string[];
}

type Popup =
  | { kind: "slash"; items: SlashCommandEntry[]; index: number }
  | { kind: "file"; items: FuzzyFileSearchResult[]; index: number; start: number; query: string }
  | null;

const HISTORY_LIMIT = 100;

export function Composer() {
  const { state, ctl, t } = useApp();
  const [text, setText] = useState("");
  const [popup, setPopup] = useState<Popup>(null);
  const [queue, setQueue] = useState<string[]>([]);
  const pastes = useRef(new Map<string, string>());
  const history = useRef<string[]>(ctl.bridge.viewState<ViewState>()?.history ?? []);
  const historyIndex = useRef<number | null>(null);
  const area = useRef<HTMLTextAreaElement>(null);
  const busy = state.chat.activeTurnId !== null;
  const ready = state.server.state === "ready";

  // A draft handed over by the command guide ("Try it") lands here, once.
  useEffect(() => {
    const draft = ctl.takeDraft();
    if (draft === null) return;
    setText(draft);
    requestAnimationFrame(() => area.current?.focus());
  }, [state.composer.draft]);

  // Send queued messages once the running turn is over.
  useEffect(() => {
    if (!busy && queue.length > 0 && ready) {
      const [next, ...rest] = queue;
      setQueue(rest);
      void ctl.send(next!);
    }
  }, [busy, queue, ready]);

  // Auto-size the textarea.
  useEffect(() => {
    const el = area.current;
    if (!el) return;
    el.style.height = "auto";
    el.style.height = `${Math.min(el.scrollHeight, 240)}px`;
  }, [text]);

  // `/` popup and `@` file search follow the text and the caret.
  const fileSearchSeq = useRef(0);
  const updatePopups = (value: string, caret: number) => {
    if (/^\/[a-z0-9-]*$/i.test(value)) {
      const items = matchCommands(value.slice(1));
      setPopup(items.length ? { kind: "slash", items, index: 0 } : null);
      return;
    }
    const before = value.slice(0, caret);
    const at = /(^|\s)@([^\s@]*)$/.exec(before);
    if (at && ctl.cwd) {
      const query = at[2]!;
      const start = before.length - query.length - 1;
      const seq = ++fileSearchSeq.current;
      void ctl.session
        .fuzzyFileSearch({ query, roots: [ctl.cwd], cancellationToken: null })
        .then((r) => {
          if (seq !== fileSearchSeq.current) return;
          const items = r.files.slice(0, 8);
          setPopup(items.length ? { kind: "file", items, index: 0, start, query } : null);
        })
        .catch(() => setPopup(null));
      return;
    }
    setPopup(null);
  };

  const pick = (index: number) => {
    if (!popup) return;
    if (popup.kind === "slash") {
      const cmd = popup.items[index]!;
      setText(`/${cmd.name}${cmd.inlineArgs ? " " : ""}`);
      setPopup(null);
      if (!cmd.inlineArgs) void submit(`/${cmd.name}`);
      return;
    }
    // Like the TUI (chat_composer.rs insert_selected_path): the `@token` becomes the path itself.
    const file = popup.items[index]!;
    const rel = filePathToken(file.path.replace(/\\/g, "/"), null);
    const next = `${text.slice(0, popup.start)}${rel} ${text.slice(popup.start + 1 + popup.query.length)}`;
    setText(next);
    setPopup(null);
  };

  const submit = async (value = text) => {
    const trimmed = value.trim();
    if (!trimmed || !ready) return;
    history.current = [trimmed, ...history.current.filter((h) => h !== trimmed)].slice(0, HISTORY_LIMIT);
    historyIndex.current = null;
    // Merge: other screens keep their own view state (open lesson, tab) in the same object.
    ctl.bridge.setViewState<ViewState>({ ...(ctl.bridge.viewState<ViewState>() ?? {}), history: history.current });
    const usedPastes = new Map([...pastes.current].filter(([k]) => trimmed.includes(k)));
    setText("");
    setPopup(null);
    pastes.current.clear();
    if (busy && !trimmed.startsWith("/")) {
      setQueue((q) => [...q, expandPastes(trimmed, usedPastes)]);
      return;
    }
    await ctl.send(trimmed, usedPastes);
  };

  const onKeyDown = (e: KeyboardEvent<HTMLTextAreaElement>) => {
    if (popup) {
      if (e.key === "ArrowDown" || e.key === "ArrowUp") {
        e.preventDefault();
        const delta = e.key === "ArrowDown" ? 1 : -1;
        setPopup({ ...popup, index: (popup.index + delta + popup.items.length) % popup.items.length } as Popup);
        return;
      }
      if (e.key === "Enter" || e.key === "Tab") {
        e.preventDefault();
        pick(popup.index);
        return;
      }
      if (e.key === "Escape") {
        setPopup(null);
        return;
      }
    }
    if (e.key === "Enter" && !e.shiftKey && !e.nativeEvent.isComposing) {
      e.preventDefault();
      void submit();
      return;
    }
    const el = e.currentTarget;
    if ((e.key === "ArrowUp" && el.selectionStart === 0) || (e.key === "ArrowDown" && el.selectionEnd === el.value.length)) {
      if (history.current.length === 0) return;
      const current = historyIndex.current;
      const next = e.key === "ArrowUp" ? (current === null ? 0 : Math.min(current + 1, history.current.length - 1)) : current === null ? null : current - 1;
      if (next === null || next < 0) {
        historyIndex.current = null;
        setText("");
      } else {
        historyIndex.current = next;
        setText(history.current[next]!);
      }
      e.preventDefault();
    }
    if (e.key === "Escape" && busy) void ctl.interrupt();
  };

  const onPaste = (e: React.ClipboardEvent<HTMLTextAreaElement>) => {
    const pasted = e.clipboardData.getData("text");
    if (pasted.length <= LARGE_PASTE_CHARS) return;
    e.preventDefault();
    const base = t("chat.pastedContent", { count: pasted.length });
    let placeholder = base;
    for (let n = 2; pastes.current.has(placeholder) || text.includes(placeholder); n++) placeholder = `${base} #${n}`;
    pastes.current.set(placeholder, pasted);
    const el = e.currentTarget;
    setText(text.slice(0, el.selectionStart) + placeholder + text.slice(el.selectionEnd));
  };

  const placeholder = busy ? t("chat.placeholderBusy") : t("chat.placeholder");
  const popupView = useMemo(() => {
    if (!popup) return null;
    return (
      <ul className="sf-popup" role="listbox">
        {popup.kind === "slash"
          ? popup.items.slice(0, 12).map((c, i) => (
              <li key={c.name} role="option" aria-selected={i === popup.index} className={i === popup.index ? "is-active" : ""} onMouseDown={(e) => (e.preventDefault(), pick(i))}>
                <span className="sf-popup-main">/{c.name}</span>
                <span className="sf-popup-desc">{t(c.descKey)}</span>
                {!c.guiAction && <span className="sf-popup-tag">{t("commands.terminalOnly")}</span>}
              </li>
            ))
          : popup.items.map((f, i) => (
              <li key={f.path} role="option" aria-selected={i === popup.index} className={i === popup.index ? "is-active" : ""} onMouseDown={(e) => (e.preventDefault(), pick(i))}>
                <Icon name="file" size={13} />
                <span className="sf-popup-main">{f.file_name}</span>
                <span className="sf-popup-desc">{f.path}</span>
              </li>
            ))}
      </ul>
    );
  }, [popup, t]);

  return (
    <div className={`sf-composer ${busy ? "is-busy" : ""}`}>
      {popupView}
      {queue.length > 0 && (
        <div className="sf-queue">
          {queue.map((q, i) => (
            <span key={i} className="sf-queue-item" title={q}>
              {t("chat.queued")}: {q.length > 40 ? `${q.slice(0, 40)}…` : q}
            </span>
          ))}
        </div>
      )}
      <textarea
        ref={area}
        className="sf-composer-input"
        rows={1}
        value={text}
        placeholder={placeholder}
        aria-label={placeholder}
        disabled={!ready}
        onChange={(e) => {
          setText(e.target.value);
          updatePopups(e.target.value, e.target.selectionStart);
        }}
        onKeyDown={onKeyDown}
        onPaste={onPaste}
      />
      <div className="sf-composer-send">
        {busy ? (
          <button type="button" className="sf-send is-stop" onClick={() => void ctl.interrupt()} title={t("chat.stop")} aria-label={t("chat.stop")}>
            <Icon name="stop" />
          </button>
        ) : (
          <button type="button" className="sf-send" disabled={!text.trim() || !ready} onClick={() => void submit()} title={t("chat.send")} aria-label={t("chat.send")}>
            <Icon name="send" />
          </button>
        )}
      </div>
    </div>
  );
}

function expandPastes(text: string, pastes: Map<string, string>): string {
  let out = text;
  for (const [k, v] of pastes) out = out.split(k).join(v);
  return out;
}
