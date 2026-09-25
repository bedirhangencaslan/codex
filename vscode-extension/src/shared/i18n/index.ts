// Every string the interface shows comes from these dictionaries; nothing user-facing is written
// inline in a component. `en` is the reference: its keys define `MessageKey`, and every other
// locale is typed `Record<MessageKey, string>`, so a missing or extra key is a compile error.
//
// Adding a language: copy en.ts to <code>.ts, translate the values, then add it to LOCALES and
// give it an Intl tag below. The language picker lists LOCALES, nothing else needs to change.
import { en, type MessageKey } from "./en";
import { tr } from "./tr";

export type { MessageKey };

export interface LocaleDefinition {
  code: string;
  /** Name of the language in that language, shown in the picker. */
  nativeName: string;
  /** BCP-47 tag for Intl number/date formatting. */
  intlTag: string;
  messages: Record<MessageKey, string>;
}

export const LOCALES: LocaleDefinition[] = [
  { code: "en", nativeName: "English", intlTag: "en-US", messages: en },
  { code: "tr", nativeName: "Türkçe", intlTag: "tr-TR", messages: tr },
];

export const DEFAULT_LOCALE = "en";

export function localeByCode(code: string): LocaleDefinition {
  return LOCALES.find((l) => l.code === code) ?? LOCALES[0]!;
}

/** `setting` is the suffice.language value; `displayLanguage` is vscode.env.language (e.g. "tr"). */
export function resolveLocale(setting: string | undefined, displayLanguage: string | undefined): string {
  if (setting && setting !== "auto" && LOCALES.some((l) => l.code === setting)) return setting;
  const base = (displayLanguage ?? "").toLowerCase().split(/[-_]/)[0] ?? "";
  return LOCALES.some((l) => l.code === base) ? base : DEFAULT_LOCALE;
}

export type Params = Record<string, string | number>;

/** `{name}` placeholders are replaced from params; unknown placeholders are left visible. */
export function translate(locale: LocaleDefinition, key: MessageKey, params?: Params): string {
  const template = locale.messages[key] ?? en[key] ?? key;
  if (!params) return template;
  return template.replace(/\{(\w+)\}/g, (match, name: string) =>
    name in params ? String(params[name]) : match,
  );
}

export function formatNumber(locale: LocaleDefinition, value: number, options?: Intl.NumberFormatOptions): string {
  return new Intl.NumberFormat(locale.intlTag, options).format(value);
}

export function formatRelativeTime(locale: LocaleDefinition, epochSeconds: number, now = Date.now()): string {
  const deltaSeconds = Math.round(epochSeconds - now / 1000);
  const rtf = new Intl.RelativeTimeFormat(locale.intlTag, { numeric: "auto" });
  const abs = Math.abs(deltaSeconds);
  if (abs < 60) return rtf.format(deltaSeconds, "second");
  if (abs < 3600) return rtf.format(Math.round(deltaSeconds / 60), "minute");
  if (abs < 86400) return rtf.format(Math.round(deltaSeconds / 3600), "hour");
  return rtf.format(Math.round(deltaSeconds / 86400), "day");
}
