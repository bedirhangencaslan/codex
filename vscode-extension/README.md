# Suffice for VS Code

A sidebar chat for Suffice, laid out like the Claude Code extension. It is a client of
`suffice app-server`, the same JSON-RPC server the TUI uses. The extension adds no model logic:
every cost mechanism (reasoning retention, read/grep/glob budgets, apply_patch, exec output
shrinking, code mode, tool-output shrinking, parallel tool calls, invisible turns, compaction)
runs inside the binary exactly as it does for the TUI.

## Audit: which user interfaces exist (goal item 4)

On `suffice` @ `dabf4dac3e` (after the GUI revert) the only interactive UI is the TUI
(`codex-rs/tui`). `suffice exec` is non-interactive. The TypeScript SDK (`sdk/typescript`)
spawns `exec --experimental-json` per turn and cannot drive a live session. The Python SDK
talks to the app-server but has no UI.

The TUI itself is an app-server client (`tui/src/app_server_session.rs`), so the app-server
protocol is the existing seam for a second UI. This extension uses it and changes no Rust code.

## Architecture

```
VS Code ── extension host (Node) ───────────── suffice app-server (stdio, JSONL)
             src/extension/                     spawned per window, env carries API keys
             │ postMessage (src/shared/messages.ts)
           webview (React)
             src/webview/
```

| Folder | Role |
|---|---|
| `src/protocol/` | JSON-RPC connection, stdio transport, typed `AppServerSession`. Types come from `codex-rs/app-server-protocol/schema/typescript` through the `@protocol/*` path. The experimental fields that are used are declared in `experimental.ts`. |
| `src/extension/` | Activation, binary lookup (setting → PATH → workspace `codex-rs/target`), `SessionHost` (spawn, initialize, relay, pending approvals replayed when the view reopens), SecretStorage keys, view provider with CSP nonce. |
| `src/shared/` | Pure logic shared by both sides and covered by tests: i18n dictionaries, pricing, meters (the TUI formulas), compaction sync check, themes, slash command registry. |
| `src/webview/` | React UI. `state/chatReducer.ts` ports `tui/src/chatwidget/protocol.rs`. `app/controller.ts` is the only place that sends requests. |

The webview may call only the methods listed in `ALLOWED_RPC_METHODS` (`src/shared/messages.ts`).

## From the TUI to the webview (goal item 12)

| TUI | Extension |
|---|---|
| `chatwidget/protocol.rs` event → state | `webview/state/chatReducer.ts` (turn/item lifecycle, deltas, token usage, plan, diff) |
| `history_cell/*` (user, reasoning, agent markdown, exec, patch, plan, notices) | `webview/chat/cells.tsx` (same cell kinds; invisible user turns are dimmed) |
| `bottom_pane/chat_composer.rs` (Enter/Shift+Enter, history, `@` files, `$` skills, `/` popup, paste > 1000 chars collapses) | `webview/chat/Composer.tsx` |
| `bottom_pane/approval_overlay.rs`, request_user_input | `webview/chat/ServerRequests.tsx` |
| `bottom_pane/footer.rs` context/cache meters | `webview/chat/Meters.tsx` plus `shared/meters.ts` (same baseline of 12000 and the same formulas), plus total cost |
| model, plan and permission pickers, `/skills`, goal menu | `webview/chat/Toolbar.tsx` panels |
| `resume_picker.rs` | History screen |
| `slash_command.rs` | `shared/generated/slash-commands.json` (generated from the Rust source) plus the Commands screen |
| `theme_picker.rs` names | `shared/themes.ts` (catppuccin, nord, gruvbox, solarized, one-half, plus following VS Code) |

## Goal items → where they live

| # | Feature | Implementation |
|---|---|---|
| 1 | Interface list with previews | Home screen. Each card is a live, scaled, inert render of the real screen. |
| 2 | Skill selection | Skills screen: global on/off via `skills/config/write` (as in the TUI) and a per-workspace "attach" list sent as `skill` input items (like `$skill`). |
| 3 | Slash command course | Commands screen, two tabs. **Lessons** (`shared/lessons.ts`): six short lessons, each with explanations, practice steps that are ticked only when the extension actually runs the command, and quick-check quizzes. Progress is kept in globalState. **Reference**: every command with its availability, when to use it and an example. More lessons come from `registerLessonSet()` and more commands from `registerCommandSet()`, without changing the screen. |
| 7 | i18n | `shared/i18n/{en,tr}.ts` plus `package.nls{,.tr}.json`. `tests/noHardcodedText.test.ts` fails on Turkish text outside the dictionaries. |
| 8 | Conversation preferences | Settings text box, stored in globalState. **Not sent to the model yet**; see `PROMPT-CHANGE-PLAN.md`. |
| 9 | API keys and prices | API screen. Keys live in VS Code SecretStorage and are passed to the app-server as env vars (default `ZAI_API_KEY`); the server restarts on change. Built-in glm-5.3-flash prices can be overridden. |
| 10 | Compaction slider | Settings. Writes `model_auto_compact_token_limit` via `config/value/write`; "model default" removes it. |
| 11 | Retention ↔ compaction sync | `shared/compactionSync.ts`. It only warns and never changes anything. Findings: the running thread froze a different limit; the value differs from the retention window of 80000; the value is clamped to 9/10 of the context window; the `body_after_prefix` scope is unclamped. |
| 13 | Meters | Context left, cache % of the last request, speed, total cost. |
| 14 | Themes | Nine palettes as `--sf-*` CSS variables. |
| 15 | Panels under the composer | Invisible, mode (default/plan/goal/review/permissions), skills, file tree (`mention` items like `@file`), model + reasoning effort. |

## Cost guarantees (tested in `tests/controller.test.ts`)

- A plain message sends only `{threadId, input, turnTrigger, invisible:false}`, with no model,
  effort, mode or permission overrides. It matches what `exec` sends, so the prompt prefix and
  cache stay the same.
- `collaborationMode` is sent only when the mode changes: into Plan, and once back to Default.
  The TUI sends Default on every turn, which was measured to add about 1.3K characters per
  request. See `PROMPT-CHANGE-PLAN.md` for the parity option.
- The preferences text never reaches the server.
- Invisible mode uses the existing `turn/start.invisible`.

## Develop

```sh
pnpm install                      # from the repo root (pnpm workspace member)
cd vscode-extension
npm run build                     # dist/extension.js + dist/webview.js/.css  (--production to minify)
npm run typecheck
npx jest                          # unit tests
node scripts/gen-slash-commands.ts  # refresh the slash catalog after slash_command.rs changes
```

Integration tests run against a real binary with a throwaway `SUFFICE_HOME`:

```sh
SUFFICE_BIN=<path>/suffice.exe npx jest tests/appserver.integration.test.ts tests/controller.integration.test.ts
SUFFICE_BIN=... LIVE_TURN=1 ZAI_API_KEY=... npx jest tests/controller.integration.test.ts   # one small paid turn
```

Browser preview without VS Code: build, then open `preview.html` with
`?screen=home|chat|history|skills|commands|api|settings&lang=tr|en&theme=<id>&width=380`.
Sending a message plays back a recorded real turn. Add `&demo=approval` to also show an
approval and a question.

Install locally: copy `package.json`, `package.nls*.json`, `dist/` and `media/` to
`~/.vscode/extensions/suffice.suffice-vscode-0.1.0/` and reload VS Code. The view appears in the
activity bar and can be moved to the secondary sidebar. Set `suffice.binaryPath` if `suffice` is
not on PATH.
