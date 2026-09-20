import { useEffect, useMemo, useRef, useState } from "react";
import { makeT, type CommandEntry, type DataProvider, type Locale } from "@suffice/gui-core";

type TrialState = { command: string; output: string; running: boolean; error: string | null };

/** Slash-command learning screen. "Try" runs the command in an *ephemeral*
 * thread (`thread/start {ephemeral:true}`), so experiments never touch
 * history; the text sent is exactly the command the user clicked. */
export function Commands({ provider, locale }: { provider: DataProvider; locale: Locale }) {
  const t = useMemo(() => makeT(locale), [locale]);
  const [commands, setCommands] = useState<CommandEntry[] | null>(null);
  const [trial, setTrial] = useState<TrialState | null>(null);
  const outRef = useRef<HTMLPreElement>(null);

  useEffect(() => {
    let live = true;
    provider.listCommands().then((c) => live && setCommands(c));
    return () => {
      live = false;
    };
  }, [provider]);

  useEffect(() => {
    outRef.current?.scrollTo({ top: outRef.current.scrollHeight });
  }, [trial?.output]);

  const run = (command: string) => {
    setTrial({ command, output: "", running: true, error: null });
    provider
      .runCommandTrial(command, (chunk) =>
        setTrial((cur) => (cur && cur.command === command ? { ...cur, output: cur.output + chunk } : cur)),
      )
      .then(() => setTrial((cur) => (cur && cur.command === command ? { ...cur, running: false } : cur)))
      .catch((e: unknown) =>
        setTrial((cur) =>
          cur && cur.command === command ? { ...cur, running: false, error: e instanceof Error ? e.message : String(e) } : cur,
        ),
      );
  };

  return (
    <>
      <div className="pagehead">
        <h1>{t("commands.title")}</h1>
      </div>
      <div className="cmdlayout">
        <div className="panel">
          <header>
            <h2>
              {t("commands.title")} <small>— {t("commands.subtitle")}</small>
            </h2>
          </header>
          {commands == null ? (
            <p className="loading">{t("common.loading")}</p>
          ) : (
            <div className="cmdgrid">
              {commands.map((c) => (
                <button className="cmdcard" key={c.name} onClick={() => run(c.name)}>
                  <code>{c.name}</code>
                  <span>{c.description}</span>
                  <span className="try">{t("commands.try").toLocaleUpperCase(locale)} ↵</span>
                </button>
              ))}
            </div>
          )}
        </div>
        <div className="panel trial" aria-live="polite">
          <header>
            <h2>
              {trial ? <code className="num">{trial.command}</code> : t("commands.trialIdle")}{" "}
              {trial?.running && <small>· {t("common.loading")}</small>}
            </h2>
          </header>
          <pre ref={outRef} className="trialout num">
            {trial == null ? t("commands.trialHint") : trial.output || (trial.running ? "…" : "")}
            {trial?.error && `\n⚠ ${trial.error}`}
          </pre>
        </div>
      </div>
    </>
  );
}
