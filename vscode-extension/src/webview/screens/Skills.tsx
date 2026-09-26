// Goal item 2: pick the skills a project uses. Two levels, both existing mechanisms:
// - On/Off is skills/config/write, the same call the TUI's skills toggle makes (it writes the
//   user config, so it applies everywhere);
// - "Use in this project" is stored per workspace; a new chat here starts with Codex's own
//   `skills.config` rules for it (selected on, the rest off), so the model sees only those.
import type { SkillMetadata } from "@protocol/v2/SkillMetadata";
import { useEffect, useState } from "react";
import type { MessageKey } from "../../shared/i18n";
import { useApp } from "../app/context";
import { Badge, Empty, Notice, SearchBox, Toggle } from "../components/ui";

const SCOPE_ORDER: SkillMetadata["scope"][] = ["repo", "user", "admin", "system"];

export function SkillsScreen() {
  const { state, ctl, t } = useApp();
  const [filter, setFilter] = useState("");
  useEffect(() => {
    if (state.server.state === "ready") void ctl.loadSkills();
  }, [state.server.state]);

  const attached = new Set(state.init?.attachedSkills.map((s) => s.path));
  const q = filter.toLowerCase();
  const skills = state.skills
    .filter((s) => !q || s.name.toLowerCase().includes(q) || s.description.toLowerCase().includes(q))
    .sort((a, b) => SCOPE_ORDER.indexOf(a.scope) - SCOPE_ORDER.indexOf(b.scope) || a.name.localeCompare(b.name));

  return (
    <div className="sf-screen">
      <header className="sf-screen-head">
        <h1>{t("skills.title")}</h1>
        <p className="sf-muted">{t("skills.intro")}</p>
      </header>
      <SearchBox value={filter} onChange={setFilter} placeholder={t("skills.search")} />
      {state.skillErrors.length > 0 && (
        <Notice tone="warning">
          {t("skills.errors")}
          <ul>
            {state.skillErrors.map((e) => (
              <li key={e}>{e}</li>
            ))}
          </ul>
        </Notice>
      )}
      {skills.length === 0 ? (
        <Empty>{t("skills.empty")}</Empty>
      ) : (
        <ul className="sf-skill-list">
          {skills.map((skill) => (
            <li key={skill.path} className={`sf-skill ${skill.enabled ? "" : "is-disabled"}`}>
              <div className="sf-skill-head">
                <span className="sf-skill-name">{skill.interface?.displayName ?? skill.name}</span>
                <Badge tone={skill.scope === "repo" ? "accent" : "neutral"}>{t(`skills.scope.${skill.scope}` as MessageKey)}</Badge>
              </div>
              <p className="sf-skill-desc">{skill.shortDescription ?? skill.description}</p>
              <div className="sf-row sf-wrap">
                <Toggle checked={skill.enabled} onChange={(v) => void ctl.setSkillEnabled(skill, v)} label={skill.enabled ? t("common.on") : t("common.off")} />
                <label className="sf-check">
                  <input type="checkbox" checked={attached.has(skill.path)} onChange={() => ctl.toggleAttachedSkill(skill)} />
                  <span className="sf-check-label">{t("skills.attach")}</span>
                </label>
              </div>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
