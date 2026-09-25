// Goal item 13: context fill, cached share and the chat's total cost, as small indicators above
// the input. The numbers come from thread/tokenUsage/updated through the TUI footer's formulas
// (shared/meters.ts); the cost uses the price table of the API & Pricing screen.
import { MODEL_FACTS } from "../../shared/catalog";
import { computeMeters, formatTokensCompact, formatUsd } from "../../shared/meters";
import { priceFor } from "../../shared/pricing";
import { COMMIT_KEYS, COMPACTION_STEP } from "../app/compaction";
import { useApp } from "../app/context";
import { useCompactionSlider } from "../app/useCompactionSlider";
import { formatNumber } from "../../shared/i18n";

/** The compaction limit, at the left of the row: a short slider over [context + 1, model window]. */
function CompactionControl({ contextTokens }: { contextTokens: number }) {
  const { t, locale } = useApp();
  const slider = useCompactionSlider(contextTokens);
  const { range } = slider;
  return (
    <label
      className="sf-meter sf-compaction"
      title={t("meter.compactionTitle", { value: formatNumber(locale, slider.value), min: formatTokensCompact(range.min), max: formatTokensCompact(range.max) })}
    >
      <input
        type="range"
        className="sf-compaction-range"
        min={range.min}
        max={range.max}
        step={COMPACTION_STEP}
        value={slider.value}
        disabled={slider.disabled}
        aria-label={t("settings.compactionTitle")}
        aria-valuetext={t("settings.compactionValue", { value: formatNumber(locale, slider.value) })}
        onChange={(e) => slider.setValue(Number(e.target.value))}
        onPointerUp={() => void slider.commit(slider.value)}
        onKeyUp={(e) => COMMIT_KEYS.includes(e.key) && void slider.commit(slider.value)}
      />
      {t("meter.compaction", { value: formatTokensCompact(slider.value) })}
    </label>
  );
}

export function Meters() {
  const { state, t, locale } = useApp();
  const model = state.composer.model ?? state.threadModel ?? state.config?.model ?? null;
  const price = model ? priceFor(model, state.init?.prices ?? {}) : undefined;
  const fallbackWindow = model ? (MODEL_FACTS[model]?.contextWindow ?? state.config?.contextWindow ?? null) : null;
  const m = computeMeters(state.chat.tokenUsage, fallbackWindow, price);
  const pct = (v: number) => new Intl.NumberFormat(locale.intlTag).format(v);

  const context = m.contextUsedPercent;
  const tone = context === null ? "" : context >= 85 ? "is-danger" : context >= 65 ? "is-warning" : "";
  return (
    <div className="sf-meters" aria-live="polite">
      <CompactionControl contextTokens={m.contextTokens} />
      <span
        className={`sf-meter ${tone}`}
        title={m.contextWindow ? t("meter.contextTitle", { used: formatTokensCompact(m.contextTokens), window: formatTokensCompact(m.contextWindow) }) : undefined}
      >
        <span className="sf-meter-bar" aria-hidden="true">
          <span style={{ width: `${Math.min(100, context ?? 0)}%` }} />
        </span>
        {context === null ? t("meter.contextUnknown") : t("meter.context", { percent: pct(context) })}
      </span>
      {m.cachedPercent !== null && (
        <span className="sf-meter" title={t("meter.cachedTitle", { fresh: formatTokensCompact(m.nextNewTokens), cached: formatTokensCompact(m.nextCachedTokens) })}>
          {t("meter.cached", { percent: pct(m.cachedPercent) })}
        </span>
      )}
      {m.cost ? (
        <span
          className="sf-meter"
          title={t("meter.costTitle", { fresh: formatUsd(m.cost.fresh), cached: formatUsd(m.cost.cached), output: formatUsd(m.cost.output) })}
        >
          {t("meter.cost", { cost: formatUsd(m.cost.total) })}
        </span>
      ) : (
        <span className="sf-meter is-muted" title={t("meter.costUnknownTitle", { model: model ?? "–" })}>
          {t("meter.costUnknown")}
        </span>
      )}
      {m.readiness && <span className="sf-meter is-muted">{t(`meter.${m.readiness === "warming" ? "warming" : m.readiness}`)}</span>}
    </div>
  );
}
