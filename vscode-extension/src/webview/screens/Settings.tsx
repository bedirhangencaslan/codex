// Goal items 7, 8, 10, 11 and 14: language, the conversation-preferences box, the compaction
// slider, the reasoning-shrink/compaction sync check and the theme picker.
import { useEffect, useState } from "react";
import { MODEL_FACTS } from "../../shared/catalog";
import { checkCompactionSync, type CompactionSyncFinding } from "../../shared/compactionSync";
import { formatNumber, LOCALES, type MessageKey } from "../../shared/i18n";
import { THEMES, type Palette } from "../../shared/themes";
import { COMMIT_KEYS, COMPACTION_STEP } from "../app/compaction";
import { useApp } from "../app/context";
import { useCompactionSlider } from "../app/useCompactionSlider";
import { Button, Notice, Section } from "../components/ui";

export function SettingsScreen() {
  const { t } = useApp();
  return (
    <div className="sf-screen">
      <header className="sf-screen-head">
        <h1>{t("settings.title")}</h1>
      </header>
      <Language />
      <Theme />
      <Preferences />
      <Compaction />
    </div>
  );
}

function Language() {
  const { state, ctl, t } = useApp();
  return (
    <Section title={t("settings.language")}>
      <select className="sf-select" value={state.init?.languageSetting ?? "auto"} onChange={(e) => ctl.setLanguage(e.target.value)} aria-label={t("settings.language")}>
        <option value="auto">{t("settings.languageAuto")}</option>
        {LOCALES.map((l) => (
          <option key={l.code} value={l.code}>
            {l.nativeName}
          </option>
        ))}
      </select>
    </Section>
  );
}

function Swatch({ palette }: { palette: Palette }) {
  return (
    <span className="sf-swatch" aria-hidden="true" style={{ background: palette.bg, borderColor: palette.border }}>
      <span style={{ background: palette.surfaceRaised }} />
      <span style={{ background: palette.accent }} />
      <span style={{ background: palette.text }} />
    </span>
  );
}

function Theme() {
  const { state, ctl, t } = useApp();
  const current = state.init?.themeId ?? "vscode";
  return (
    <Section title={t("settings.theme")}>
      <div className="sf-theme-grid" role="radiogroup" aria-label={t("settings.theme")}>
        {THEMES.map((theme) => (
          <button key={theme.id} type="button" role="radio" aria-checked={current === theme.id} className={`sf-theme ${current === theme.id ? "is-active" : ""}`} onClick={() => ctl.setTheme(theme.id)}>
            <Swatch palette={theme.palette} />
            <span>{t(theme.labelKey as MessageKey)}</span>
          </button>
        ))}
      </div>
    </Section>
  );
}

function Preferences() {
  const { state, ctl, t } = useApp();
  const [text, setText] = useState(state.init?.preferences ?? "");
  const [saved, setSaved] = useState(false);
  useEffect(() => setText(state.init?.preferences ?? ""), [state.init?.preferences]);
  return (
    <Section title={t("settings.prefsTitle")} description={t("settings.prefsDesc")}>
      <textarea className="sf-textarea" rows={4} value={text} onChange={(e) => (setText(e.target.value), setSaved(false))} placeholder={t("settings.prefsPlaceholder")} aria-label={t("settings.prefsTitle")} />
      <div className="sf-row">
        <Button variant="primary" onClick={() => (ctl.savePreferences(text), setSaved(true))} disabled={text === (state.init?.preferences ?? "")}>
          {t("common.save")}
        </Button>
        {saved && <span className="sf-muted">{t("settings.prefsSaved")}</span>}
      </div>
      <Notice tone="info">{t("settings.prefsPending")}</Notice>
    </Section>
  );
}


function Compaction() {
  const { state, ctl, t, locale } = useApp();
  useEffect(() => {
    if (state.server.state === "ready") void ctl.loadConfig();
  }, [state.server.state]);

  const config = state.config;
  const contextTokens = state.chat.tokenUsage?.last.totalTokens ?? 0;
  const slider = useCompactionSlider(contextTokens);
  const { range } = slider;
  const model = range.model;
  const facts = model ? MODEL_FACTS[model] : undefined;
  // The clamp works on the raw window Suffice knows (an override, else the catalog), not the
  // effective one token usage reports.
  const rawWindow = config?.contextWindow ?? facts?.contextWindow ?? null;
  const modelDefault = range.defaultValue;
  const configured = range.configured;
  const value = slider.value;
  const [savedNote, setSavedNote] = useState(false);

  const fmt = (n: number) => formatNumber(locale, n);
  const commit = async (next: number | null) => {
    if (next === configured) return;
    setSavedNote(await slider.commit(next));
  };

  const threadId = state.chat.threadId;
  const report = checkCompactionSync({
    sliderValue: configured,
    threadStartValue: threadId ? (state.init?.threadStartCompaction[threadId] ?? null) : undefined,
    scope: config?.compactionScope ?? "total",
    contextWindow: rawWindow,
    catalogLimit: facts ? facts.autoCompactTokenLimit : undefined,
  });
  const describe = (f: CompactionSyncFinding): string => {
    const shown = (v: number | null) => (v === null ? `${t("settings.defaultValue")} (${modelDefault !== null ? fmt(modelDefault) : "–"})` : fmt(v));
    switch (f.kind) {
      case "thread-frozen":
        return t("settings.sync.threadFrozen", { thread: shown(f.threadValue), slider: shown(f.sliderValue) });
      case "retention-window":
        return t("settings.sync.retentionWindow", { window: fmt(f.window), limit: fmt(f.limit), ratio: formatNumber(locale, f.limit / f.window, { maximumFractionDigits: 2 }) });
      case "clamped":
        return t("settings.sync.clamped", { requested: fmt(f.requested), applied: fmt(f.applied) });
      case "scope-unclamped":
        return t("settings.sync.scopeUnclamped", { requested: fmt(f.requested), clamped: fmt(f.clampedForRead) });
    }
  };

  return (
    <>
      <Section title={t("settings.compactionTitle")} description={t("settings.compactionDesc")}>
        <div className="sf-slider-row">
          <input
            type="range"
            className="sf-slider"
            min={range.min}
            max={range.max}
            step={COMPACTION_STEP}
            value={value}
            onChange={(e) => (slider.setValue(Number(e.target.value)), setSavedNote(false))}
            onPointerUp={() => void commit(value)}
            onKeyUp={(e) => COMMIT_KEYS.includes(e.key) && void commit(value)}
            aria-label={t("settings.compactionTitle")}
            aria-valuetext={t("settings.compactionValue", { value: fmt(value) })}
            disabled={slider.disabled}
          />
          <output className="sf-slider-value">{t("settings.compactionValue", { value: fmt(value) })}</output>
        </div>
        <div className="sf-row sf-wrap">
          {modelDefault !== null && <span className="sf-muted">{t("settings.compactionDefault", { value: fmt(modelDefault) })}</span>}
          <Button variant="ghost" onClick={() => void commit(null)} disabled={configured === null}>
            {t("settings.compactionUseDefault")}
          </Button>
        </div>
        {!range.realWindow && <p className="sf-muted">{t("settings.compactionUnknownWindow", { model: model ?? "–", max: fmt(range.max) })}</p>}
        {savedNote && <Notice tone="success">{t("settings.compactionSaved")}</Notice>}
      </Section>

      <Section title={t("settings.syncTitle")}>
        {!threadId && <p className="sf-muted">{t("settings.syncNoThread")}</p>}
        {report.inSync ? (
          <Notice tone="success">{t("settings.syncOk", { value: report.effectiveLimit !== null ? fmt(report.effectiveLimit) : "–" })}</Notice>
        ) : (
          report.findings.map((f) => (
            <Notice key={f.kind} tone="warning">
              {describe(f)}
            </Notice>
          ))
        )}
      </Section>
    </>
  );
}
