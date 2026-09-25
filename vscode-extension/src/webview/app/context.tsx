import { createContext, useContext, useEffect, useMemo, useReducer, useRef, useState, type ReactNode } from "react";
import { localeByCode, translate, type LocaleDefinition, type MessageKey, type Params } from "../../shared/i18n";
import { paletteToCss, themeById } from "../../shared/themes";
import { createBridge, type HostBridge } from "../host";
import { Controller } from "./controller";
import { appReducer, initialState, SCREENS, type AppState, type Screen } from "./state";

export interface AppContextValue {
  state: AppState;
  ctl: Controller;
  t: (key: MessageKey, params?: Params) => string;
  locale: LocaleDefinition;
}

const AppContext = createContext<AppContextValue | null>(null);

export function useApp(): AppContextValue {
  const value = useContext(AppContext);
  if (!value) throw new Error("useApp outside AppProvider");
  return value;
}

/** Applies the selected theme as CSS custom properties on the document. */
function useTheme(themeId: string | undefined): void {
  useEffect(() => {
    const theme = themeById(themeId);
    const root = document.documentElement;
    for (const [name, value] of Object.entries(paletteToCss(theme.palette))) root.style.setProperty(name, value);
    root.dataset.appearance = theme.appearance;
    root.dataset.theme = theme.id;
  }, [themeId]);
}

/**
 * Browser preview only (never in VS Code): ?demo=1 sends one message through the real flow
 * (thread/start, turn/start, then the recorded turn plays back); ?panel=mode|skills|files|model
 * opens a composer panel. Used to screenshot states that need interaction.
 */
function usePreviewDemo(bridge: HostBridge, ctl: Controller, state: AppState): void {
  const done = useRef(false);
  useEffect(() => {
    if (!bridge.isPreview || done.current || state.server.state !== "ready" || state.models.length === 0) return;
    done.current = true;
    const params = new URLSearchParams(location.search);
    const panel = params.get("panel");
    if (params.get("demo")) void ctl.send("Run `echo hello-suffice` and tell me what it printed.");
    if (panel === "mode" || panel === "skills" || panel === "files" || panel === "model") setTimeout(() => ctl.togglePanel(panel), 200);
  }, [state.server.state, state.models.length]);
}

export function AppProvider({ children }: { children: ReactNode }) {
  const [bridge, setBridge] = useState<HostBridge | null>(null);
  useEffect(() => {
    void createBridge().then(setBridge);
  }, []);
  if (!bridge) return null;
  return <Provider bridge={bridge}>{children}</Provider>;
}

function Provider({ bridge, children }: { bridge: HostBridge; children: ReactNode }) {
  const [state, dispatch] = useReducer(appReducer, initialState, (s): AppState => {
    const requested = bridge.isPreview ? new URLSearchParams(location.search).get("screen") : null;
    return requested && (SCREENS as string[]).includes(requested) ? { ...s, screen: requested as Screen } : s;
  });
  const locale = localeByCode(state.init?.locale ?? "en");
  const tRef = useRef((key: MessageKey, params?: Params) => translate(locale, key, params));
  tRef.current = (key, params) => translate(locale, key, params);
  const ctl = useMemo(() => new Controller(bridge, dispatch, state, () => tRef.current), [bridge]);
  ctl.sync(state);

  useEffect(() => ctl.start(), [ctl]);
  useTheme(state.init?.themeId);
  usePreviewDemo(bridge, ctl, state);

  useEffect(() => {
    document.documentElement.lang = locale.code;
  }, [locale.code]);

  const value = useMemo<AppContextValue>(() => ({ state, ctl, t: tRef.current, locale }), [state, ctl, locale]);
  return <AppContext.Provider value={value}>{children}</AppContext.Provider>;
}
