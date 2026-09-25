// Goal item 1: the list of the extension's interfaces, each with a preview under its title. The
// preview is the real screen rendered small and inert (not a picture that can go stale), so it
// always shows what opening it will show, with the live data.
import { useEffect, useRef, useState, type ComponentType } from "react";
import type { MessageKey } from "../../shared/i18n";
import { useApp } from "../app/context";
import type { Screen } from "../app/state";
import { ChatScreen } from "../chat/ChatScreen";
import { Icon, type IconName } from "../components/icons";
import { ApiScreen } from "./Api";
import { CommandsScreen } from "./Commands";
import { HistoryScreen } from "./History";
import { SettingsScreen } from "./Settings";
import { SkillsScreen } from "./Skills";

interface InterfaceEntry {
  screen: Exclude<Screen, "home">;
  icon: IconName;
  title: MessageKey;
  desc: MessageKey;
  component: ComponentType;
}

export const INTERFACES: InterfaceEntry[] = [
  { screen: "chat", icon: "chat", title: "home.chat.title", desc: "home.chat.desc", component: ChatScreen },
  { screen: "history", icon: "history", title: "home.history.title", desc: "home.history.desc", component: HistoryScreen },
  { screen: "skills", icon: "sparkles", title: "home.skills.title", desc: "home.skills.desc", component: SkillsScreen },
  { screen: "commands", icon: "book", title: "home.commands.title", desc: "home.commands.desc", component: CommandsScreen },
  { screen: "api", icon: "key", title: "home.api.title", desc: "home.api.desc", component: ApiScreen },
  { screen: "settings", icon: "gear", title: "home.settings.title", desc: "home.settings.desc", component: SettingsScreen },
];

/** Width the previewed screen is laid out at before being scaled into the card. */
const PREVIEW_WIDTH = 380;
const PREVIEW_HEIGHT = 520;

function LivePreview({ component: Component }: { component: ComponentType }) {
  const frame = useRef<HTMLDivElement>(null);
  const [scale, setScale] = useState(0.5);
  useEffect(() => {
    const el = frame.current;
    if (!el) return;
    el.setAttribute("inert", "");
    const update = () => setScale(Math.min(1, el.clientWidth / PREVIEW_WIDTH));
    update();
    const observer = new ResizeObserver(update);
    observer.observe(el);
    return () => observer.disconnect();
  }, []);
  return (
    <div className="sf-preview" ref={frame} aria-hidden="true" style={{ height: Math.round(PREVIEW_HEIGHT * scale * 0.62) }}>
      <div className="sf-preview-inner" style={{ width: PREVIEW_WIDTH, height: PREVIEW_HEIGHT, transform: `scale(${scale})` }}>
        <Component />
      </div>
    </div>
  );
}

export function HomeScreen() {
  const { ctl, t } = useApp();
  return (
    <div className="sf-screen">
      <header className="sf-screen-head">
        <h1>{t("home.title")}</h1>
        <p className="sf-muted">{t("home.subtitle")}</p>
      </header>
      <ul className="sf-interface-list">
        {INTERFACES.map((entry) => (
          <li key={entry.screen}>
            <button type="button" className="sf-interface-card" onClick={() => ctl.go(entry.screen)}>
              <span className="sf-interface-head">
                <Icon name={entry.icon} />
                <span className="sf-interface-title">{t(entry.title)}</span>
                <Icon name="chevronRight" className="sf-interface-go" />
              </span>
              <span className="sf-interface-desc">{t(entry.desc)}</span>
              <LivePreview component={entry.component} />
            </button>
          </li>
        ))}
      </ul>
    </div>
  );
}
