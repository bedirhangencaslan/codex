// Terminal output arrives with its escape sequences (colours from uvicorn, cargo, pytest, ...).
// The TUI hands it to a terminal that interprets them; the webview has to do it itself.
// SGR sequences (ESC [ ... m) become styled segments. Every other control sequence (cursor moves,
// erase such as ESC [ 31 X, OSC titles and hyperlinks, charset switches) is dropped, and a line
// redrawn with a carriage return (progress bars) keeps only what was drawn last.

export interface AnsiStyle {
  fg?: string;
  bg?: string;
  bold?: boolean;
  dim?: boolean;
  italic?: boolean;
  underline?: boolean;
  inverse?: boolean;
}

export interface AnsiSegment {
  text: string;
  style: AnsiStyle;
}

// CSI (ESC [ or the 8-bit 0x9B), OSC (ESC ] ... BEL | ESC \), then any other escape
// (optional intermediates and a final byte: ESC ( B, ESC =, ESC 7, ESC M, ...).
// eslint-disable-next-line no-control-regex
const SEQUENCE = /(?:\x1b\[|\x9b)([0-?]*)[ -/]*([@-~])|\x1b\][^\x07\x1b]*(?:\x07|\x1b\\)?|\x1b[ -/]*[0-~]?/g;

// The 16 standard colours in the theme's own palette, so output stays readable in every theme.
const BASIC = [
  "var(--sf-muted)",
  "var(--sf-danger)",
  "var(--sf-success)",
  "var(--sf-warning)",
  "var(--sf-accent)",
  "color-mix(in srgb, var(--sf-danger) 50%, var(--sf-accent))",
  "color-mix(in srgb, var(--sf-accent) 55%, var(--sf-success))",
  "var(--sf-text)",
];

function color256(n: number): string | undefined {
  if (!Number.isInteger(n) || n < 0 || n > 255) return undefined;
  if (n < 16) return BASIC[n % 8];
  if (n >= 232) {
    const v = 8 + (n - 232) * 10;
    return `rgb(${v},${v},${v})`;
  }
  const i = n - 16;
  const level = (x: number) => (x === 0 ? 0 : 55 + x * 40);
  return `rgb(${level(Math.floor(i / 36))},${level(Math.floor(i / 6) % 6)},${level(i % 6)})`;
}

/** Reads a 38/48 extended colour starting at codes[i]; returns the colour and the codes consumed. */
function extendedColor(codes: number[], i: number): [string | undefined, number] {
  if (codes[i + 1] === 5) return [color256(codes[i + 2]!), 3];
  if (codes[i + 1] === 2) {
    const [r, g, b] = [codes[i + 2], codes[i + 3], codes[i + 4]];
    const ok = [r, g, b].every((x) => x !== undefined && x >= 0 && x <= 255);
    return [ok ? `rgb(${r},${g},${b})` : undefined, 5];
  }
  return [undefined, 1];
}

function applySgr(style: AnsiStyle, params: string): AnsiStyle {
  const codes = params === "" ? [0] : params.split(/[;:]/).map((p) => (p === "" ? 0 : Number(p)));
  let next = { ...style };
  for (let i = 0; i < codes.length; i++) {
    const c = codes[i]!;
    if (c === 0) next = {};
    else if (c === 1) next.bold = true;
    else if (c === 2) next.dim = true;
    else if (c === 3) next.italic = true;
    else if (c === 4) next.underline = true;
    else if (c === 7) next.inverse = true;
    else if (c === 22) (next.bold = false), (next.dim = false);
    else if (c === 23) next.italic = false;
    else if (c === 24) next.underline = false;
    else if (c === 27) next.inverse = false;
    else if (c >= 30 && c <= 37) next.fg = BASIC[c - 30];
    else if (c >= 90 && c <= 97) next.fg = BASIC[c - 90];
    else if (c === 39) delete next.fg;
    else if (c >= 40 && c <= 47) next.bg = BASIC[c - 40];
    else if (c >= 100 && c <= 107) next.bg = BASIC[c - 100];
    else if (c === 49) delete next.bg;
    else if (c === 38 || c === 48) {
      const [color, used] = extendedColor(codes, i);
      if (color) next[c === 38 ? "fg" : "bg"] = color;
      i += used - 1;
    }
  }
  return next;
}

/** A line redrawn with carriage returns shows what was drawn last. */
function resolveCarriageReturns(text: string): string {
  return text
    .replace(/\r\n/g, "\n")
    .split("\n")
    .map((line) => {
      if (!line.includes("\r")) return line;
      const parts = line.split("\r").filter((p) => p.length > 0);
      return parts.length ? parts[parts.length - 1]! : "";
    })
    .join("\n");
}

/** Styled segments of terminal output; adjacent text with the same style is merged. */
export function parseAnsi(text: string): AnsiSegment[] {
  const input = resolveCarriageReturns(text);
  const segments: AnsiSegment[] = [];
  let style: AnsiStyle = {};
  let last = 0;
  const push = (chunk: string) => {
    // Other C0 controls (bell, backspace, ...) have no visible form; tabs and newlines stay.
    // eslint-disable-next-line no-control-regex
    const clean = chunk.replace(/[\x00-\x08\x0b-\x1f\x7f]/g, "");
    if (!clean) return;
    const prev = segments[segments.length - 1];
    if (prev && JSON.stringify(prev.style) === JSON.stringify(style)) prev.text += clean;
    else segments.push({ text: clean, style });
  };
  for (const match of input.matchAll(SEQUENCE)) {
    push(input.slice(last, match.index));
    last = match.index! + match[0].length;
    if (match[2] === "m") style = applySgr(style, match[1] ?? "");
  }
  push(input.slice(last));
  return segments;
}

/** Terminal output without any escape sequence. */
export function stripAnsi(text: string): string {
  return parseAnsi(text)
    .map((s) => s.text)
    .join("");
}
