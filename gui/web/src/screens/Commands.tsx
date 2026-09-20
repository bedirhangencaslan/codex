import { useEffect, useMemo, useState } from "react";
import { makeT, type CommandEntry, type DataProvider, type Locale } from "@suffice/gui-core";

/** Slash-command learning screen. "Try" will start an *ephemeral* thread via
 * `thread/start { ephemeral: true }` in the follow-up PR so experiments never
 * pollute history; in Phase 1 the button is a visual affordance only. */
export function Commands({ provider, locale }: { provider: DataProvider; locale: Locale }) {
  const t = useMemo(() => makeT(locale), [locale]);
  const [commands, setCommands] = useState<CommandEntry[] | null>(null);

  useEffect(() => {
    let live = true;
    provider.listCommands().then((c) => live && setCommands(c));
    return () => {
      live = false;
    };
  }, [provider]);

  return (
    <>
      <div className="pagehead">
        <h1>{t("commands.title")}</h1>
      </div>
      <div className="panel" style={{ maxWidth: 860 }}>
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
              <div className="cmdcard" key={c.name}>
                <code>{c.name}</code>
                <span>{c.description}</span>
                <span className="try">{t("commands.try").toLocaleUpperCase(locale)} ↵</span>
              </div>
            ))}
          </div>
        )}
      </div>
    </>
  );
}
