import { useEffect, useMemo, useState } from "react";
import {
  AppServerProvider,
  MockProvider,
  availableLocales,
  detectLocale,
  makeT,
  persistLocale,
  type DataProvider,
  type Locale,
} from "@suffice/gui-core";
import { Commands } from "./screens/Commands";
import { Learn } from "./screens/Learn";
import { Sessions } from "./screens/Sessions";
import { Skills } from "./screens/Skills";
import { Styles } from "./screens/Styles";

type Screen = "sessions" | "learn" | "skills" | "commands" | "styles";

const ICONS: Record<Screen, JSX.Element> = {
  sessions: (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path d="M4 6h16M4 12h16M4 18h10" />
    </svg>
  ),
  learn: (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path d="M12 4L3 8.5l9 4.5 9-4.5z" />
      <path d="M7 11v5c0 1.5 2.2 3 5 3s5-1.5 5-3v-5" />
    </svg>
  ),
  skills: (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path d="M12 3l2.5 6.5L21 12l-6.5 2.5L12 21l-2.5-6.5L3 12l6.5-2.5z" />
    </svg>
  ),
  commands: (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path d="M5 7l5 5-5 5M13 17h6" />
    </svg>
  ),
  styles: (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <rect x="4" y="4" width="7" height="7" rx="1" />
      <rect x="13" y="4" width="7" height="7" rx="1" />
      <rect x="4" y="13" width="7" height="7" rx="1" />
      <rect x="13" y="13" width="7" height="7" rx="1" />
    </svg>
  ),
};

/** `?ws=ws://127.0.0.1:PORT` connects to a live `suffice app-server`;
 * without it the GUI runs on fixture data so design review needs no build. */
function useProvider(): { provider: DataProvider; ready: boolean } {
  const [provider, setProvider] = useState<DataProvider>(() => new MockProvider());
  const [ready, setReady] = useState(false);
  useEffect(() => {
    const ws = new URLSearchParams(window.location.search).get("ws");
    if (!ws) {
      setReady(true);
      return;
    }
    let cancelled = false;
    AppServerProvider.connect(ws, "~")
      .then((p) => {
        if (!cancelled) setProvider(p);
      })
      .catch((err) => console.warn("app-server connection failed, falling back to fixtures:", err))
      .finally(() => {
        if (!cancelled) setReady(true);
      });
    return () => {
      cancelled = true;
    };
  }, []);
  return { provider, ready };
}

export function App() {
  const [screen, setScreen] = useState<Screen>("sessions");
  const [locale, setLocale] = useState<Locale>(() => detectLocale());
  const { provider, ready } = useProvider();
  const t = useMemo(() => makeT(locale), [locale]);

  useEffect(() => {
    document.documentElement.lang = locale;
    persistLocale(locale);
  }, [locale]);

  const NAV: Array<{ id: Screen; label: string }> = [
    { id: "sessions", label: t("nav.sessions") },
    { id: "learn", label: t("nav.learn") },
    { id: "skills", label: t("nav.skills") },
    { id: "commands", label: t("nav.commands") },
    { id: "styles", label: t("nav.styles") },
  ];

  return (
    <div className="app">
      <nav className="side" aria-label={t("nav.aria")}>
        <div className="logo">
          <span className="mark" aria-hidden="true" />
          suffice
        </div>
        {NAV.map((n) => (
          <button
            key={n.id}
            className={`nav${screen === n.id ? " on" : ""}`}
            aria-current={screen === n.id ? "page" : undefined}
            onClick={() => setScreen(n.id)}
          >
            {ICONS[n.id]}
            {n.label}
          </button>
        ))}
        <div className="foot">
          <label className="langpick">
            <span className="visually-hidden">{t("nav.language")}</span>
            <select value={locale} onChange={(e) => setLocale(e.target.value as Locale)} aria-label={t("nav.language")}>
              {availableLocales().map(([id, name]) => (
                <option key={id} value={id}>
                  {name}
                </option>
              ))}
            </select>
          </label>
          <span className="num">glm-5.3-flash · zai</span>
          <br />
          <span className="badge-src">{provider.kind === "mock" ? t("common.mockBadge") : t("common.liveBadge")}</span>
        </div>
      </nav>
      <main className="main">
        {!ready ? (
          <p className="loading">{t("common.loading")}</p>
        ) : screen === "sessions" ? (
          <Sessions provider={provider} locale={locale} />
        ) : screen === "learn" ? (
          <Learn provider={provider} locale={locale} />
        ) : screen === "skills" ? (
          <Skills provider={provider} locale={locale} />
        ) : screen === "commands" ? (
          <Commands provider={provider} locale={locale} />
        ) : (
          <Styles provider={provider} locale={locale} />
        )}
      </main>
    </div>
  );
}
