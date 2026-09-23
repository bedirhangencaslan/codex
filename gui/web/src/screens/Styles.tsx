import { useEffect, useMemo, useState, type CSSProperties } from "react";
import { makeT, type DataProvider, type Locale, type StyleCard } from "@suffice/gui-core";

const THUMBS: Record<string, CSSProperties> = {
  thermal: {
    background:
      "radial-gradient(120px 60px at 12% 0%, rgba(240,146,60,.35), transparent 70%)," +
      "radial-gradient(120px 70px at 100% 110%, rgba(78,128,178,.4), transparent 70%)," +
      "linear-gradient(180deg,#0d1015,#0a0d12)",
  },
  blueprint: {
    background:
      "linear-gradient(#dfe5eb 1px, transparent 1px), linear-gradient(90deg, #dfe5eb 1px, transparent 1px), #eef1f4",
    backgroundSize: "16px 16px, 16px 16px, auto",
  },
  abyss: {
    background: "linear-gradient(180deg,#050607,#070907)",
    boxShadow: "inset 0 -30px 40px -30px rgba(51,209,122,.25)",
  },
};

/** Style gallery: refero.design-style cards whose selection swaps only the
 * CSS custom-property layer (tokens.css). Phase 1 ships the approved theme
 * as the single active card plus the two pitched alternates as previews. */
export function Styles({ provider, locale }: { provider: DataProvider; locale: Locale }) {
  const t = useMemo(() => makeT(locale), [locale]);
  const [styles, setStyles] = useState<StyleCard[] | null>(null);

  useEffect(() => {
    let live = true;
    provider.listStyles().then((s) => live && setStyles(s));
    return () => {
      live = false;
    };
  }, [provider]);

  // Known style ids localize from the dictionary; anything the wire adds
  // later falls back to the name it carries.
  const KNOWN = new Set(["thermal", "blueprint", "abyss"]);
  const styleName = (s: StyleCard) => (KNOWN.has(s.id) ? t(`styles.${s.id}.name` as Parameters<typeof t>[0]) : s.name);
  const styleTagline = (s: StyleCard) =>
    KNOWN.has(s.id) ? t(`styles.${s.id}.tagline` as Parameters<typeof t>[0]) : s.tagline;

  return (
    <>
      <div className="pagehead">
        <h1>{t("styles.title")}</h1>
      </div>
      <p style={{ color: "var(--sf-muted)", marginTop: 0, maxWidth: "68ch" }}>{t("styles.subtitle")}</p>
      {styles == null ? (
        <p className="loading">{t("common.loading")}</p>
      ) : (
        <div className="stylegrid">
          {styles.map((s, i) => (
            <div className="stylecard" key={s.id}>
              <div className="thumb" style={THUMBS[s.id]} aria-hidden="true" />
              <div className="info">
                <span className="grow">
                  <b>{styleName(s)}</b>
                  <span>{styleTagline(s)}</span>
                </span>
                <span className="state">{i === 0 ? t("styles.active") : t("styles.apply")}</span>
              </div>
            </div>
          ))}
        </div>
      )}
    </>
  );
}

