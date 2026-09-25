// Goal item 3: the slash command guide. It renders whatever command sets are registered
// (shared/slashCommands.ts), grouped by category; a new set appears here without changes.
// Opening a command marks it explored; "Try it" puts its example into the chat input.
import { useState } from "react";
import type { MessageKey } from "../../shared/i18n";
import { CATEGORY_ORDER, commandSets, type SlashCommandEntry } from "../../shared/slashCommands";
import { useApp } from "../app/context";
import { Icon } from "../components/icons";
import { Badge, Button, SearchBox } from "../components/ui";

interface ViewState {
  explored?: string[];
}

export function CommandsScreen() {
  const { t, ctl } = useApp();
  const [filter, setFilter] = useState("");
  const [open, setOpen] = useState<string | null>(null);
  const [explored, setExplored] = useState<Set<string>>(new Set(ctl.bridge.viewState<ViewState>()?.explored ?? []));
  const q = filter.toLowerCase().replace(/^\//, "");

  const expand = (key: string) => {
    setOpen(open === key ? null : key);
    if (!explored.has(key)) {
      const next = new Set(explored).add(key);
      setExplored(next);
      ctl.bridge.setViewState<ViewState>({ ...(ctl.bridge.viewState<ViewState>() ?? {}), explored: [...next] });
    }
  };

  const matches = (c: SlashCommandEntry) =>
    !q || c.name.includes(q) || c.aliases.some((a) => a.includes(q)) || t(c.descKey).toLowerCase().includes(q);

  return (
    <div className="sf-screen">
      <header className="sf-screen-head">
        <h1>{t("commands.title")}</h1>
        <p className="sf-muted">{t("commands.intro")}</p>
      </header>
      <SearchBox value={filter} onChange={setFilter} placeholder={t("commands.search")} />
      {commandSets().map((set) => {
        const total = set.commands.length;
        const done = set.commands.filter((c) => explored.has(`${set.id}:${c.name}`)).length;
        return (
          <section key={set.id} className="sf-command-set">
            <header className="sf-command-set-head">
              <h2>{t(set.titleKey)}</h2>
              <span className="sf-progress" title={set.source}>
                <span className="sf-progress-bar" aria-hidden="true">
                  <span style={{ width: `${(done / Math.max(1, total)) * 100}%` }} />
                </span>
                {t("commands.lessonProgress", { done, total })}
              </span>
            </header>
            {CATEGORY_ORDER.map((category) => {
              const commands = set.commands.filter((c) => c.category === category && matches(c));
              if (commands.length === 0) return null;
              return (
                <div key={category} className="sf-command-group">
                  <h3>{t(`commands.category.${category}` as MessageKey)}</h3>
                  <ul className="sf-command-list">
                    {commands.map((c) => {
                      const key = `${set.id}:${c.name}`;
                      const isOpen = open === key;
                      return (
                        <li key={c.name} className={`sf-command ${isOpen ? "is-open" : ""}`}>
                          <button type="button" className="sf-command-head" onClick={() => expand(key)} aria-expanded={isOpen}>
                            <code className="sf-command-name">/{c.name}</code>
                            {explored.has(key) && <Icon name="check" size={12} className="sf-command-seen" />}
                            <span className="sf-command-desc">{t(c.descKey)}</span>
                            <Icon name={isOpen ? "chevronDown" : "chevronRight"} size={12} />
                          </button>
                          {isOpen && (
                            <div className="sf-command-body">
                              <div className="sf-row sf-wrap">
                                <Badge tone={c.guiAction ? "success" : "neutral"}>{c.guiAction ? t("commands.inExtension") : t("commands.terminalOnly")}</Badge>
                                {c.availableDuringTask && <Badge>{t("commands.flagDuringTask")}</Badge>}
                                {c.inlineArgs && <Badge>{t("commands.flagArgs")}</Badge>}
                                {c.availableInSideConversation && <Badge>{t("commands.flagSide")}</Badge>}
                                {c.availableWhenThreadUnavailable && <Badge>{t("commands.flagNoThread")}</Badge>}
                                {c.aliases.map((a) => (
                                  <Badge key={a}>/{a}</Badge>
                                ))}
                              </div>
                              {c.whenKey && (
                                <p>
                                  <strong>{t("commands.whenToUse")}: </strong>
                                  {t(c.whenKey)}
                                </p>
                              )}
                              {c.exampleKey && (
                                <div className="sf-command-example">
                                  <span className="sf-muted">{t("commands.example")}</span>
                                  <code>{t(c.exampleKey)}</code>
                                  <Button variant="primary" icon="send" onClick={() => ctl.setDraft(t(c.exampleKey!))}>
                                    {t("commands.tryIt")}
                                  </Button>
                                </div>
                              )}
                            </div>
                          )}
                        </li>
                      );
                    })}
                  </ul>
                </div>
              );
            })}
          </section>
        );
      })}
    </div>
  );
}
