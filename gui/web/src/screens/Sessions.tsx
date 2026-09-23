import { useEffect, useMemo, useState } from "react";
import {
  fmtNumber,
  fmtPercent,
  intlTag,
  makeT,
  type DataProvider,
  type Locale,
  type ThreadDetail,
  type ThreadSummary,
  type Warmth,
  type WeekStats,
} from "@suffice/gui-core";
import { Gauge } from "../components/Gauge";

function WarmthPill({ warmth, badge, t }: { warmth: Warmth | null; badge?: string; t: ReturnType<typeof makeT> }) {
  if (badge && warmth == null) return <span className="pill ok"><i />{badge}</span>;
  if (warmth == null) return null;
  const label = badge ?? t(`warmth.${warmth}`);
  const cls = badge && badge.includes("/") ? "ok" : warmth;
  return (
    <span className={`pill ${cls}`}>
      <i />
      {label}
    </span>
  );
}

function relTime(iso: string, locale: Locale): string {
  if (!iso) return "";
  const then = new Date(iso).getTime();
  if (Number.isNaN(then)) return iso;
  const mins = Math.round((Date.now() - then) / 60_000);
  const rtf = new Intl.RelativeTimeFormat(intlTag(locale), { numeric: "auto" });
  if (mins < 60) return rtf.format(-mins, "minute");
  if (mins < 60 * 24) return rtf.format(-Math.round(mins / 60), "hour");
  return rtf.format(-Math.round(mins / (60 * 24)), "day");
}

export function Sessions({ provider, locale }: { provider: DataProvider; locale: Locale }) {
  const t = useMemo(() => makeT(locale), [locale]);
  const [threads, setThreads] = useState<ThreadSummary[] | null>(null);
  const [stats, setStats] = useState<WeekStats | null>(null);
  const [selected, setSelected] = useState<string | null>(null);
  const [detail, setDetail] = useState<ThreadDetail | null>(null);
  const [query, setQuery] = useState("");

  useEffect(() => {
    let live = true;
    provider.listThreads().then((rows) => {
      if (!live) return;
      setThreads(rows);
      setSelected((cur) => cur ?? rows[0]?.id ?? null);
    });
    provider.weekStats().then((s) => live && setStats(s));
    return () => {
      live = false;
    };
  }, [provider]);

  useEffect(() => {
    if (!selected) return;
    let live = true;
    provider.threadDetail(selected).then((d) => live && setDetail(d));
    return () => {
      live = false;
    };
  }, [provider, selected]);

  const filtered = useMemo(
    () =>
      (threads ?? []).filter((th) =>
        query.trim() === "" ? true : (th.title + " " + th.preview).toLocaleLowerCase(locale).includes(query.toLocaleLowerCase(locale)),
      ),
    [threads, query, locale],
  );

  const money = (v: number) => fmtNumber(locale, v, { style: "currency", currency: "USD", maximumFractionDigits: 4 });

  return (
    <>
      <div className="pagehead">
        <h1>
          {t("nav.sessions")} <small>· ~/codex</small>
        </h1>
        <div style={{ display: "flex", gap: 10 }}>
          <input
            className="search"
            placeholder={t("sessions.search")}
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            aria-label={t("sessions.search")}
          />
          <button className="btn-primary">＋ {t("sessions.new")}</button>
        </div>
      </div>

      {stats && (
        <div className="stats">
          <div className="stat hot">
            <span className="v num">{money(stats.costUsd)}</span>
            <span className="l">{t("sessions.week")}</span>
          </div>
          <div className="stat">
            <span className="v num">{fmtPercent(locale, stats.avgCachedPercent)}</span>
            <span className="l">{t("sessions.avgCached")}</span>
          </div>
          <div className="stat">
            <span className="v num">{fmtNumber(locale, stats.freshTokens, { notation: "compact" })}</span>
            <span className="l">{t("sessions.fresh")}</span>
          </div>
          <div className="stat">
            <span className="v num">{stats.sessionCount}</span>
            <span className="l">{t("sessions.count")}</span>
          </div>
          <div className="stat">
            <span className="v" style={{ color: "var(--sf-warm-hi)" }}>
              ⚡ {t(`warmth.${stats.cacheState}`).toLocaleLowerCase(locale)}
            </span>
            <span className="l">{t("sessions.cacheNow")}</span>
          </div>
        </div>
      )}

      <div className="sessions">
        <div className="cards">
          {threads == null ? (
            <p className="loading">{t("common.loading")}</p>
          ) : filtered.length === 0 ? (
            <p className="empty">{t("common.empty")}</p>
          ) : (
            filtered.map((th) => (
              <button
                key={th.id}
                className={`card${selected === th.id ? " sel" : ""}`}
                onClick={() => setSelected(th.id)}
                aria-pressed={selected === th.id}
              >
                <span className="toprow">
                  {th.cachedPercent != null && <Gauge percent={th.cachedPercent} label={t("gauge.cachedInput")} />}
                  <span className="grow">
                    <span className="c-title">{th.title || t("sessions.untitled")}</span>
                    <span className="c-prev">{th.preview}</span>
                  </span>
                  <WarmthPill warmth={th.warmth} badge={th.badge} t={t} />
                </span>
                <span className="c-meta">
                  {th.costUsd != null && <span className="num">{money(th.costUsd)}</span>}
                  {th.requests != null && (
                    <span>
                      <span className="num">{th.requests}</span> {t("sessions.requests")}
                    </span>
                  )}
                  {th.compactions != null && (
                    <span>
                      <span className="num">{th.compactions}</span> {t("sessions.compactions")}
                    </span>
                  )}
                  <span className="when">{relTime(th.updatedAt, locale)}</span>
                </span>
              </button>
            ))
          )}
        </div>

        {detail && (
          <aside className="rail" aria-label={t("rail.selected")}>
            <p className="railh">{t("rail.selected")}</p>
            <div className="dial">
              <svg viewBox="0 0 132 76" width="132" height="76" role="img" aria-label={`${t("rail.cachedInput")} %${detail.cachedPercent}`}>
                <path d="M12 68 A54 54 0 0 1 120 68" fill="none" stroke="rgba(255,255,255,.08)" strokeWidth="7" strokeLinecap="round" />
                <path
                  d="M12 68 A54 54 0 0 1 111 41"
                  fill="none"
                  stroke="url(#sfDial)"
                  strokeWidth="7"
                  strokeLinecap="round"
                />
                <defs>
                  <linearGradient id="sfDial" x1="0" y1="0" x2="1" y2="0">
                    <stop offset="0" stopColor="var(--sf-cold)" />
                    <stop offset=".55" stopColor="var(--sf-cold-hi)" />
                    <stop offset="1" stopColor="var(--sf-warm)" />
                  </linearGradient>
                </defs>
                <text className="dialval" x="66" y="58" textAnchor="middle">
                  {fmtPercent(locale, detail.cachedPercent)}
                </text>
                <text className="diallbl" x="66" y="72" textAnchor="middle">
                  {t("rail.cachedInput")}
                </text>
              </svg>
            </div>
            <p className="railh">{t("rail.timeline")}</p>
            {detail.timeline.map((r) => (
              <div className="req" key={r.index}>
                <span className="id">{String(r.index).padStart(2, "0")}</span>
                <span className="track2">
                  <span className="cach" style={{ width: `${r.cachedShare * 100}%` }} />
                  <span className="fresh" style={{ width: `${(1 - r.cachedShare) * 100}%` }} />
                </span>
                <span className="pc">{fmtPercent(locale, Math.round(r.cachedShare * 100))}</span>
              </div>
            ))}
            <p className="railh" style={{ marginTop: 16 }}>
              {t("rail.summary")}
            </p>
            <div className="railsum">
              <span>{t("rail.fresh")}</span>
              <span className="num">{fmtNumber(locale, detail.freshTokens)}</span>
              <span>{t("rail.cached")}</span>
              <span className="num warmv">{fmtNumber(locale, detail.cachedTokens)}</span>
              <span>{t("rail.output")}</span>
              <span className="num">{fmtNumber(locale, detail.outputTokens)}</span>
              <span>{t("rail.keepAlive")}</span>
              <span className="num">
                {detail.keepAlives} · {money(detail.keepAliveCostUsd)}
              </span>
            </div>
          </aside>
        )}
      </div>
    </>
  );
}
