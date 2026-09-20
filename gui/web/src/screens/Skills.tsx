import { useEffect, useMemo, useState } from "react";
import { fmtNumber, makeT, type DataProvider, type Locale, type SkillEntry } from "@suffice/gui-core";

/** Per-project skill selection. Toggles are local state in Phase 1; writing
 * them back to project config lands with the `skills/…` write endpoint in a
 * follow-up PR — the prompt impact mechanism itself is the existing
 * `include_skill_instructions` path, nothing new reaches the model. */
export function Skills({ provider, locale }: { provider: DataProvider; locale: Locale }) {
  const t = useMemo(() => makeT(locale), [locale]);
  const [skills, setSkills] = useState<SkillEntry[] | null>(null);

  useEffect(() => {
    let live = true;
    provider.listSkills("~/codex").then((s) => live && setSkills(s));
    return () => {
      live = false;
    };
  }, [provider]);

  const toggle = (name: string) =>
    setSkills((cur) => cur && cur.map((s) => (s.name === name ? { ...s, enabled: !s.enabled } : s)));

  const total = (skills ?? []).reduce((a, s) => a + s.promptTokens, 0);
  const active = (skills ?? []).filter((s) => s.enabled).reduce((a, s) => a + s.promptTokens, 0);
  const lighterPct = total === 0 ? 0 : Math.round(((total - active) / total) * 100);

  return (
    <>
      <div className="pagehead">
        <h1>
          {t("skills.title")} <small>· ~/codex</small>
        </h1>
      </div>
      <div className="panel" style={{ maxWidth: 720 }}>
        <header>
          <h2>
            {t("skills.title")} <small>— {t("skills.subtitle")}</small>
          </h2>
        </header>
        {skills == null ? (
          <p className="loading">{t("common.loading")}</p>
        ) : (
          <>
            {skills.map((s) => (
              <div className="skrow" key={s.name}>
                <button
                  className="togg"
                  role="switch"
                  aria-checked={s.enabled}
                  aria-label={`${s.name} ${s.enabled ? "açık" : "kapalı"}`}
                  onClick={() => toggle(s.name)}
                />
                <span className="grow">
                  <span className="skname">{s.name}</span>
                  <br />
                  <span className="skdesc">{s.description}</span>
                </span>
                {s.promptTokens > 0 && (
                  <span className="sktok num">
                    +<b>{fmtNumber(locale, s.promptTokens)}</b> {t("skills.tokens")}
                  </span>
                )}
              </div>
            ))}
            <div className="budget">
              {t("skills.promptLoad")}: <span className="num">{fmtNumber(locale, active)}</span> /{" "}
              {fmtNumber(locale, total)} {t("skills.tokens")}
              {lighterPct > 0 && (
                <>
                  {" "}
                  — <span className="lighter">%{lighterPct} {t("skills.lighter")}</span>
                </>
              )}
              <div className="bar">
                <i style={{ width: `${total === 0 ? 0 : Math.max(2, (active / total) * 100)}%` }} />
              </div>
            </div>
          </>
        )}
      </div>
    </>
  );
}
