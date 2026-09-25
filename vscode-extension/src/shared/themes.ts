// Interface themes. The names are the TUI's syntax themes (codex-rs/tui/src/render/highlight.rs),
// so a user who picked `nord` in the terminal finds the same palette here; only the calm,
// low-contrast ones are carried over (no neon/high-saturation themes). `vscode` follows the
// editor's own theme through the --vscode-* variables the webview host provides.

export interface Palette {
  bg: string;
  surface: string;
  surfaceRaised: string;
  input: string;
  border: string;
  text: string;
  muted: string;
  accent: string;
  accentText: string;
  success: string;
  warning: string;
  danger: string;
  userBubble: string;
  code: string;
  selection: string;
}

export interface ThemeDefinition {
  id: string;
  /** i18n key of the display name. */
  labelKey: string;
  appearance: "dark" | "light" | "auto";
  palette: Palette;
}

const vscode: Palette = {
  bg: "var(--vscode-sideBar-background, var(--vscode-editor-background))",
  surface: "var(--vscode-editor-background)",
  surfaceRaised: "var(--vscode-editorWidget-background, var(--vscode-editor-background))",
  input: "var(--vscode-input-background)",
  border: "var(--vscode-widget-border, var(--vscode-panel-border, rgba(128,128,128,.25)))",
  text: "var(--vscode-foreground)",
  muted: "var(--vscode-descriptionForeground)",
  accent: "var(--vscode-button-background)",
  accentText: "var(--vscode-button-foreground)",
  success: "var(--vscode-testing-iconPassed, #3fb950)",
  warning: "var(--vscode-editorWarning-foreground, #d29922)",
  danger: "var(--vscode-errorForeground, #f85149)",
  userBubble: "var(--vscode-input-background)",
  code: "var(--vscode-textCodeBlock-background, rgba(128,128,128,.12))",
  selection: "var(--vscode-list-activeSelectionBackground)",
};

export const THEMES: ThemeDefinition[] = [
  { id: "vscode", labelKey: "theme.vscode", appearance: "auto", palette: vscode },
  {
    id: "catppuccin-mocha",
    labelKey: "theme.catppuccinMocha",
    appearance: "dark",
    palette: {
      bg: "#1e1e2e", surface: "#181825", surfaceRaised: "#313244", input: "#181825",
      border: "#45475a", text: "#cdd6f4", muted: "#a6adc8", accent: "#89b4fa", accentText: "#11111b",
      success: "#a6e3a1", warning: "#f9e2af", danger: "#f38ba8", userBubble: "#313244",
      code: "#11111b", selection: "#45475a",
    },
  },
  {
    id: "catppuccin-latte",
    labelKey: "theme.catppuccinLatte",
    appearance: "light",
    palette: {
      bg: "#eff1f5", surface: "#e6e9ef", surfaceRaised: "#ffffff", input: "#ffffff",
      border: "#ccd0da", text: "#4c4f69", muted: "#6c6f85", accent: "#1e66f5", accentText: "#ffffff",
      success: "#40a02b", warning: "#df8e1d", danger: "#d20f39", userBubble: "#dce0e8",
      code: "#e6e9ef", selection: "#ccd0da",
    },
  },
  {
    id: "nord",
    labelKey: "theme.nord",
    appearance: "dark",
    palette: {
      bg: "#2e3440", surface: "#2b303b", surfaceRaised: "#3b4252", input: "#272c36",
      border: "#434c5e", text: "#d8dee9", muted: "#a3acbd", accent: "#88c0d0", accentText: "#2e3440",
      success: "#a3be8c", warning: "#ebcb8b", danger: "#bf616a", userBubble: "#3b4252",
      code: "#272c36", selection: "#434c5e",
    },
  },
  {
    id: "gruvbox-dark",
    labelKey: "theme.gruvboxDark",
    appearance: "dark",
    palette: {
      bg: "#282828", surface: "#1d2021", surfaceRaised: "#3c3836", input: "#1d2021",
      border: "#504945", text: "#ebdbb2", muted: "#bdae93", accent: "#83a598", accentText: "#1d2021",
      success: "#b8bb26", warning: "#fabd2f", danger: "#fb4934", userBubble: "#3c3836",
      code: "#1d2021", selection: "#504945",
    },
  },
  {
    id: "gruvbox-light",
    labelKey: "theme.gruvboxLight",
    appearance: "light",
    palette: {
      bg: "#fbf1c7", surface: "#f2e5bc", surfaceRaised: "#fdf6dc", input: "#fdf6dc",
      border: "#d5c4a1", text: "#3c3836", muted: "#665c54", accent: "#076678", accentText: "#fbf1c7",
      success: "#79740e", warning: "#b57614", danger: "#9d0006", userBubble: "#ebdbb2",
      code: "#f2e5bc", selection: "#d5c4a1",
    },
  },
  {
    id: "solarized-dark",
    labelKey: "theme.solarizedDark",
    appearance: "dark",
    palette: {
      bg: "#002b36", surface: "#00252e", surfaceRaised: "#073642", input: "#00212b",
      border: "#0f4a57", text: "#93a1a1", muted: "#839496", accent: "#268bd2", accentText: "#fdf6e3",
      success: "#859900", warning: "#b58900", danger: "#dc322f", userBubble: "#073642",
      code: "#00212b", selection: "#0f4a57",
    },
  },
  {
    id: "solarized-light",
    labelKey: "theme.solarizedLight",
    appearance: "light",
    palette: {
      bg: "#fdf6e3", surface: "#f5eedb", surfaceRaised: "#fffbf0", input: "#fffbf0",
      border: "#e2dac3", text: "#586e75", muted: "#7b8a8f", accent: "#268bd2", accentText: "#fdf6e3",
      success: "#859900", warning: "#b58900", danger: "#dc322f", userBubble: "#eee8d5",
      code: "#f5eedb", selection: "#e2dac3",
    },
  },
  {
    id: "one-half-dark",
    labelKey: "theme.oneHalfDark",
    appearance: "dark",
    palette: {
      bg: "#282c34", surface: "#21252b", surfaceRaised: "#2f343d", input: "#21252b",
      border: "#3e4451", text: "#d4d8df", muted: "#9aa1ad", accent: "#61afef", accentText: "#1f2329",
      success: "#98c379", warning: "#e5c07b", danger: "#e06c75", userBubble: "#2f343d",
      code: "#1f2329", selection: "#3e4451",
    },
  },
];

export const DEFAULT_THEME_ID = "vscode";

export function themeById(id: string | undefined): ThemeDefinition {
  return THEMES.find((t) => t.id === id) ?? THEMES[0]!;
}

/** CSS custom properties for a palette: `--sf-bg`, `--sf-surface`, ... */
export function paletteToCss(palette: Palette): Record<string, string> {
  const css: Record<string, string> = {};
  for (const [key, value] of Object.entries(palette)) {
    css[`--sf-${key.replace(/[A-Z]/g, (c) => `-${c.toLowerCase()}`)}`] = value;
  }
  return css;
}
