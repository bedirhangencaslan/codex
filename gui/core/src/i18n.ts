/** Two locales ship from day one; Turkish is a first-class citizen, not an
 * afterthought — every string below is written by hand, no machine casing
 * (Turkish dotted/dotless i: "İstek", "ısınıyor"). */

export type Locale = "tr" | "en";

const tr = {
  "nav.sessions": "Oturumlar",
  "nav.skills": "Beceriler",
  "nav.commands": "Komutlar",
  "nav.styles": "Stiller",
  "sessions.search": "Oturumlarda ara…",
  "sessions.new": "Yeni oturum",
  "sessions.week": "bu hafta",
  "sessions.avgCached": "ort. önbellekli",
  "sessions.fresh": "taze token",
  "sessions.count": "oturum",
  "sessions.cacheNow": "önbellek · 420 sn TTL",
  "sessions.requests": "istek",
  "sessions.compactions": "sıkıştırma",
  "warmth.instant": "ANINDA",
  "warmth.fast": "HIZLI",
  "warmth.cold": "SOĞUK",
  "rail.selected": "Seçili Oturum",
  "rail.cachedInput": "ÖNBELLEKLİ GİRDİ",
  "rail.timeline": "İstek Zaman Çizelgesi",
  "rail.summary": "Özet",
  "rail.fresh": "taze",
  "rail.cached": "önbellekli",
  "rail.output": "çıktı",
  "rail.keepAlive": "canlı tutma",
  "skills.title": "Beceriler",
  "skills.subtitle": "prompt'a giren her blok ölçülür",
  "skills.promptLoad": "Prompt yükü",
  "skills.lighter": "hafifledi",
  "skills.tokens": "token",
  "commands.title": "Komutlar",
  "commands.subtitle": "güvenli denemeler geçici oturumda çalışır, geçmişe yazılmaz",
  "commands.try": "Dene",
  "styles.title": "Stiller",
  "styles.subtitle": "stil seçimi yalnız görünümü değiştirir; oturum verisine dokunmaz",
  "styles.apply": "Uygula",
  "styles.active": "Etkin",
  "common.loading": "Yükleniyor…",
  "common.empty": "Henüz içerik yok",
  "common.mockBadge": "örnek veri",
  "common.liveBadge": "canlı: app-server",
} as const;

const en: Record<keyof typeof tr, string> = {
  "nav.sessions": "Sessions",
  "nav.skills": "Skills",
  "nav.commands": "Commands",
  "nav.styles": "Styles",
  "sessions.search": "Search threads…",
  "sessions.new": "New session",
  "sessions.week": "this week",
  "sessions.avgCached": "avg cached",
  "sessions.fresh": "fresh tokens",
  "sessions.count": "sessions",
  "sessions.cacheNow": "cache · 420 s TTL",
  "sessions.requests": "req",
  "sessions.compactions": "compactions",
  "warmth.instant": "INSTANT",
  "warmth.fast": "FAST",
  "warmth.cold": "COLD",
  "rail.selected": "Selected Session",
  "rail.cachedInput": "CACHED INPUT",
  "rail.timeline": "Request Timeline",
  "rail.summary": "Summary",
  "rail.fresh": "fresh",
  "rail.cached": "cached",
  "rail.output": "output",
  "rail.keepAlive": "keep-alive",
  "skills.title": "Skills",
  "skills.subtitle": "every block that enters the prompt is measured",
  "skills.promptLoad": "Prompt load",
  "skills.lighter": "lighter",
  "skills.tokens": "tokens",
  "commands.title": "Commands",
  "commands.subtitle": "safe tries run in an ephemeral thread, never saved to history",
  "commands.try": "Try",
  "styles.title": "Styles",
  "styles.subtitle": "picking a style only changes the look; session data is untouched",
  "styles.apply": "Apply",
  "styles.active": "Active",
  "common.loading": "Loading…",
  "common.empty": "Nothing here yet",
  "common.mockBadge": "sample data",
  "common.liveBadge": "live: app-server",
};

export type MessageKey = keyof typeof tr;

const DICTS: Record<Locale, Record<MessageKey, string>> = { tr, en };

export function makeT(locale: Locale): (key: MessageKey) => string {
  const dict = DICTS[locale];
  return (key) => dict[key];
}

/** Locale-aware number formatting (Turkish uses comma decimals: 0,0085 USD). */
export function fmtNumber(locale: Locale, n: number, opts?: Intl.NumberFormatOptions): string {
  return new Intl.NumberFormat(locale === "tr" ? "tr-TR" : "en-US", opts).format(n);
}
