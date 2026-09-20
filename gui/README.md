# Suffice GUI

Web (and later Tauri / VS Code webview) client for Suffice, built as a thin
consumer of the documented `suffice app-server` JSON-RPC API. No Rust code is
patched; the GUI reads `thread/list`, `skills/list` and (next PR) starts
ephemeral threads via `thread/start`.

## Layout

| Package | Contents |
| --- | --- |
| `gui/core` | app-server WebSocket client, shared types, design tokens (`tokens.css`), TR/EN i18n |
| `gui/web` | Vite + React app: Sessions, Skills, Commands, Styles screens |
| `gui/desktop` | Tauri shell around the same web bundle (standalone Cargo project, `bundle.active` off until icons land) |
| `gui/vscode` | VS Code extension: `Suffice: Open GUI` opens the bundle in a webview (`pnpm --filter suffice-gui build`) |
| `gui/design` | Phase 0 pitch (`mockups-v1.html`) — direction "Thermal Instrument Pro" approved |
| `gui/PROMPT-CHANGE-PROPOSALS.md` | Quarantine for anything that would touch model-visible tokens — planned, never applied |

## Run

```bash
pnpm install
pnpm --filter @suffice/gui-web dev     # http://localhost:7877 (fixture data)
```

To point at a live server:

```bash
suffice app-server --listen ws://127.0.0.1:4600
# then open http://localhost:7877/?ws=ws://127.0.0.1:4600
```

Without `?ws=`, the GUI renders fixture data (labelled "örnek veri" in the
sidebar) so design review needs no Rust build. In live mode the cost/cache
fields come from the read-only `analytics/threadStats` endpoint, which
aggregates the request-stats sidecar (`<codex_home>/analytics/<thread>.jsonl`,
written when `[features] request_stats` is on). Sessions without a sidecar
render those fields as absent, never invented. A currency figure is shown
only when the sidecar header names Z.ai — the prices are the measured ones
from `_sim/FINDINGS.md` §1 ($0.075/M fresh, $0.015/M cached, $0.25/M out);
other providers get token counts without money.

Live mode wires two write paths, both documented endpoints: skill toggles call
`skills/config/write`, and the Commands screen's "Try" runs the clicked
command in an ephemeral thread (`thread/start {ephemeral:true}` →
`turn/start`), streaming agent deltas into the trial pane. The text sent is
exactly the command string — the same bytes a TUI user typing it produces.

Shells: `gui/desktop` (Tauri, see its README) and `gui/vscode`
(`pnpm --filter suffice-gui build`, then F5 / "Suffice: Open GUI") wrap the
identical bundle and add no data paths of their own.

## Turkish support

Turkish is a first-class locale, not a translation pass: `gui/core/src/i18n.ts`
ships hand-written TR strings (correct dotted/dotless i), numbers and relative
times go through `Intl` with `tr-TR`, and the UI/data font stacks (IBM Plex
Sans / JetBrains Mono) cover latin-ext so İ ş ğ ü ö ç render natively. The
locale toggle sits at the bottom of the sidebar; `<html lang>` follows it.
Font files themselves are bundled in the Tauri/VS Code phase; the web dev build
uses system fallbacks.

## Design system

`gui/core/tokens.css` is the single source of visual truth (direction A,
"Thermal Instrument Pro", approved in Phase 0). Components never hardcode
colors; the Styles screen swaps themes by replacing the custom-property layer
only.

## Prompt isolation (project rule)

The GUI never constructs or alters model-visible tokens. Anything that would —
skill instruction block shaping, "Try command" framing text — is written up in
`PROMPT-CHANGE-PROPOSALS.md` with a measurement plan per `_sim/FINDINGS.md` §3
and waits for explicit approval.
