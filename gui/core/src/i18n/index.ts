/** Dictionary-based i18n. No UI string lives in a component: everything goes
 * through a locale dictionary, and adding a language is one file plus one
 * registry line (see gui/README.md).
 *
 * The Turkish dictionary is the reference key set; every other locale is
 * typed `Record<MessageKey, string>` against it, so a missing or extra key
 * fails compilation instead of shipping. */
import { en } from "./en";
import { tr } from "./tr";

export type MessageKey = keyof typeof tr;

const LOCALES = { tr, en } satisfies Record<string, Record<MessageKey, string>>;

export type Locale = keyof typeof LOCALES;

/** BCP-47 tags for `Intl` formatting per locale. */
const INTL_TAGS: Record<Locale, string> = { tr: "tr-TR", en: "en-US" };

/** For a language picker: [id, self-name] pairs, e.g. ["tr", "Türkçe"]. */
export function availableLocales(): Array<[Locale, string]> {
  return (Object.keys(LOCALES) as Locale[]).map((id) => [id, LOCALES[id]["locale.name"]]);
}

export function makeT(locale: Locale): (key: MessageKey) => string {
  const dict = LOCALES[locale];
  return (key) => dict[key];
}

export function fmtNumber(locale: Locale, n: number, opts?: Intl.NumberFormatOptions): string {
  return new Intl.NumberFormat(INTL_TAGS[locale], opts).format(n);
}

/** Locale-correct percent: Turkish puts the sign first ("%17"), English
 * last ("17%"). Takes whole percents (17), not fractions. */
export function fmtPercent(locale: Locale, percent: number): string {
  return new Intl.NumberFormat(INTL_TAGS[locale], { style: "percent", maximumFractionDigits: 0 }).format(percent / 100);
}

export function intlTag(locale: Locale): string {
  return INTL_TAGS[locale];
}

const STORAGE_KEY = "suffice.locale";

/** Persisted choice first, then the browser's language, then Turkish. */
export function detectLocale(): Locale {
  try {
    const saved = globalThis.localStorage?.getItem(STORAGE_KEY);
    if (saved != null && saved in LOCALES) return saved as Locale;
  } catch {
    // storage may be unavailable (webview restrictions); fall through
  }
  const nav = globalThis.navigator?.language?.slice(0, 2);
  if (nav != null && nav in LOCALES) return nav as Locale;
  return "tr";
}

export function persistLocale(locale: Locale): void {
  try {
    globalThis.localStorage?.setItem(STORAGE_KEY, locale);
  } catch {
    // best-effort only
  }
}
