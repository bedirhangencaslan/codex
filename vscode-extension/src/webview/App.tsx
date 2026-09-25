import type { MessageKey } from "../shared/i18n";
import { useApp } from "./app/context";
import type { Screen } from "./app/state";
import { ChatScreen } from "./chat/ChatScreen";
import { Icon } from "./components/icons";
import { Button, IconButton } from "./components/ui";
import { ApiScreen } from "./screens/Api";
import { CommandsScreen } from "./screens/Commands";
import { HistoryScreen } from "./screens/History";
import { HomeScreen } from "./screens/Home";
import { SettingsScreen } from "./screens/Settings";
import { SkillsScreen } from "./screens/Skills";

const TITLES: Record<Screen, MessageKey> = {
  home: "nav.home",
  chat: "nav.chat",
  history: "nav.history",
  skills: "nav.skills",
  commands: "nav.commands",
  api: "nav.api",
  settings: "nav.settings",
};

export function App() {
  const { state, ctl, t } = useApp();
  if (!state.init) return null;
  const screen = state.screen;
  const title = screen === "chat" && state.chat.threadName ? state.chat.threadName : t(TITLES[screen]);
  return (
    <div className="sf-app">
      <header className="sf-topbar">
        {screen !== "chat" && <IconButton icon="back" label={t("common.back")} onClick={() => ctl.go("chat")} />}
        <span className="sf-topbar-title" title={title}>
          {title}
        </span>
        <span className="sf-topbar-actions">
          <IconButton icon="plus" label={t("chat.newChat")} onClick={() => ctl.newThread()} />
          <IconButton icon="history" label={t("nav.history")} active={screen === "history"} onClick={() => ctl.go("history")} />
          <IconButton icon="grid" label={t("nav.home")} active={screen === "home"} onClick={() => ctl.go("home")} />
          <IconButton icon="gear" label={t("nav.settings")} active={screen === "settings"} onClick={() => ctl.go("settings")} />
        </span>
      </header>
      <ServerBanner />
      <main className="sf-main">
        {screen === "chat" && <ChatScreen />}
        {screen === "home" && <HomeScreen />}
        {screen === "history" && <HistoryScreen />}
        {screen === "skills" && <SkillsScreen />}
        {screen === "commands" && <CommandsScreen />}
        {screen === "api" && <ApiScreen />}
        {screen === "settings" && <SettingsScreen />}
      </main>
      {state.toast && (
        <div className="sf-toast" role="status">
          {state.toast}
        </div>
      )}
    </div>
  );
}

function ServerBanner() {
  const { state, ctl, t } = useApp();
  const s = state.server;
  if (s.state === "ready") return null;
  if (s.state === "starting") {
    return (
      <div className="sf-banner">
        <span className="sf-spinner" aria-hidden="true" /> {t("server.connecting")}
      </div>
    );
  }
  return (
    <div className="sf-banner is-error" role="alert">
      <Icon name="warning" />
      <div>
        <strong>{s.state === "noBinary" ? t("server.noBinary") : t("server.error")}</strong>
        <p>{s.state === "noBinary" ? t("server.binaryHint") : t("server.exited", { reason: s.reason })}</p>
        <div className="sf-row">
          <Button variant="primary" icon="refresh" onClick={() => ctl.restartServer()}>
            {t("server.restart")}
          </Button>
          <Button variant="ghost" icon="gear" onClick={() => ctl.openSettings()}>
            {t("server.openSettings")}
          </Button>
        </div>
      </div>
    </div>
  );
}
