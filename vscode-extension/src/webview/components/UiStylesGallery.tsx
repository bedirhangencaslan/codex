// The interface style gallery: a centred window opened from the Skills panel. Styles are grouped
// into families (shared/uiStyles.ts), each with a live CSS preview; ticking one selects its skill
// for new chats in this project, next to the other selected skills (controller skillRules).
import type { SkillMetadata } from "@protocol/v2/SkillMetadata";
import { useEffect, useRef, useState } from "react";
import type { MessageKey } from "../../shared/i18n";
import { UI_STYLE_GROUPS, UI_STYLES, skillName, type UiStyle } from "../../shared/uiStyles";
import { useApp } from "../app/context";
import { Icon } from "./icons";
import { Button, Notice, SearchBox } from "./ui";

/** A small mock interface (card, text lines, button, chip) drawn in the style itself. */
export function StylePreview({ id }: { id: string }) {
  const { t } = useApp();
  return (
    <div className={`sf-uip sf-uip-${id}`} aria-hidden="true">
      <div className="uip-deco">
        <i />
        <i />
        <i />
      </div>
      <div className="uip-card">
        <div className="uip-title">{t("uiStyles.previewTitle")}</div>
        <div className="uip-text" />
        <div className="uip-text is-short" />
        <div className="uip-actions">
          <span className="uip-btn">{t("uiStyles.previewButton")}</span>
          <span className="uip-chip" />
        </div>
      </div>
    </div>
  );
}

export function UiStylesGallery() {
  const { state, ctl, t } = useApp();
  const [filter, setFilter] = useState("");
  const [group, setGroup] = useState<string>("all");
  const closeRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    closeRef.current?.focus();
    const onKey = (e: KeyboardEvent) => e.key === "Escape" && ctl.closeModal();
    document.addEventListener("keydown", onKey);
    return () => document.removeEventListener("keydown", onKey);
  }, []);

  const skillsByName = new Map(state.skills.map((s) => [s.name, s]));
  const attached = new Set(state.init?.attachedSkills.map((s) => s.path));
  const skillOf = (style: UiStyle): SkillMetadata | undefined => skillsByName.get(skillName(style.id));
  const selected = UI_STYLES.filter((s) => {
    const skill = skillOf(s);
    return skill !== undefined && attached.has(skill.path);
  }).length;

  const q = filter.trim().toLowerCase();
  const matches = (s: UiStyle) =>
    !q || t(`uiStyle.${s.id}.name` as MessageKey).toLowerCase().includes(q) || t(`uiStyle.${s.id}.desc` as MessageKey).toLowerCase().includes(q);
  const groups = UI_STYLE_GROUPS.filter((g) => group === "all" || g.id === group)
    .map((g) => ({ group: g, styles: UI_STYLES.filter((s) => s.group === g.id && matches(s)) }))
    .filter((g) => g.styles.length > 0);

  // The open chat keeps the skills it started with.
  const chatSkills = ctl.chatSkills;
  const differsFromChat =
    chatSkills !== null && (chatSkills.length !== attached.size || chatSkills.some((p) => !attached.has(p)));

  return (
    <div className="sf-modal-backdrop" onMouseDown={(e) => e.target === e.currentTarget && ctl.closeModal()}>
      <div className="sf-modal sf-uistyles" role="dialog" aria-modal="true" aria-labelledby="sf-uistyles-title">
        <header className="sf-modal-head">
          <div>
            <h2 id="sf-uistyles-title">{t("uiStyles.title")}</h2>
            <p className="sf-muted sf-small">{t("uiStyles.intro")}</p>
          </div>
          <button ref={closeRef} type="button" className="sf-icon-btn" onClick={() => ctl.closeModal()} aria-label={t("uiStyles.close")}>
            <Icon name="x" />
          </button>
        </header>

        <div className="sf-modal-tools">
          <SearchBox value={filter} onChange={setFilter} placeholder={t("uiStyles.search")} />
          <div className="sf-uistyles-groups" role="tablist">
            {[{ id: "all", label: t("uiStyles.all") }, ...UI_STYLE_GROUPS.map((g) => ({ id: g.id, label: t(g.titleKey) }))].map((g) => (
              <button key={g.id} type="button" role="tab" aria-selected={group === g.id} className={`sf-chip ${group === g.id ? "is-active" : ""}`} onClick={() => setGroup(g.id)}>
                {g.label}
              </button>
            ))}
          </div>
        </div>

        <div className="sf-modal-body">
          {groups.map(({ group: g, styles }) => (
            <section key={g.id} className="sf-uistyles-section">
              <h3>{t(g.titleKey)}</h3>
              <p className="sf-muted sf-small">{t(g.descKey)}</p>
              <div className="sf-uistyles-grid">
                {styles.map((style) => {
                  const skill = skillOf(style);
                  const checked = skill !== undefined && attached.has(skill.path);
                  return (
                    <label key={style.id} className={`sf-uistyle-card ${checked ? "is-selected" : ""} ${skill ? "" : "is-pending"}`}>
                      <StylePreview id={style.id} />
                      <span className="sf-uistyle-meta">
                        <span className="sf-uistyle-name">{t(`uiStyle.${style.id}.name` as MessageKey)}</span>
                        <span className="sf-uistyle-desc">{t(`uiStyle.${style.id}.desc` as MessageKey)}</span>
                      </span>
                      <span className="sf-uistyle-use">
                        <input type="checkbox" checked={checked} disabled={!skill} onChange={() => skill && ctl.toggleAttachedSkill(skill)} />
                        <span>{skill ? t("uiStyles.use") : t("uiStyles.notInstalled")}</span>
                      </span>
                    </label>
                  );
                })}
              </div>
            </section>
          ))}
        </div>

        <footer className="sf-modal-foot">
          <span className="sf-muted">{selected > 0 ? t("uiStyles.selectedCount", { count: selected }) : t("uiStyles.none")}</span>
          {differsFromChat && <Notice tone="info">{t("uiStyles.locked")}</Notice>}
          <Button variant="primary" onClick={() => ctl.closeModal()}>
            {t("uiStyles.close")}
          </Button>
        </footer>
      </div>
    </div>
  );
}
