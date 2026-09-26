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
protocol is the existing seam for a second UI. This extension uses it. The only Rust change, which
the owner approved, puts the fork's `grep` and `glob` tools under Codex's sandbox (see *Blocked files*).

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
| 2 | Skill selection | Skills screen: global on/off via `skills/config/write` (as in the TUI) and a per-workspace selection that limits new chats to the ticked skills (see *Selected skills* below). |
| 3 | Suffice course (slash commands and everything else) | Course screen, opened from the cap button in the top bar, two tabs. **Lessons** (`shared/lessons.ts`): 14 lessons in three levels (Beginner, Intermediate, Advanced) covering the commands, the chat box, the meters and cells, chats, modes, model and permissions, file access, skills, the conversation preference, cost and cache, what Suffice does under the hood (reasoning retention, read/grep/glob budgets, output condensing, apply_patch, parallel tools, cache keep-alive), the compaction limit, API keys and prices, and terminal-only commands. Step kinds: explanations; practice steps ticked only when the extension actually runs the command; show steps that open the panel or screen being taught; command quizzes and statement quizzes. The screen shows overall progress, the next lesson and per-level progress. Progress is kept in globalState. **Reference**: every command with its availability, when to use it and an example. More lessons or levels come from `registerLessonSet()` and more commands from `registerCommandSet()`, without changing the screen. |
| 7 | i18n | `shared/i18n/{en,tr}.ts` plus `package.nls{,.tr}.json`. `tests/noHardcodedText.test.ts` fails on Turkish text outside the dictionaries. |
| 8 | Conversation preference | **Preference** button under the chat box. Sent once per chat as `thread/start.developerInstructions` and kept with the chat, including on resume. An edit while a chat is open applies to a new chat. See P1 in `PROMPT-CHANGE-PLAN.md`. |
| 9 | API keys and prices | API screen. Keys live in VS Code SecretStorage and are passed to the app-server as env vars (default `ZAI_API_KEY`); the server restarts on change. Built-in glm-5.3-flash prices can be overridden. |
| 10 | Compaction slider | Settings. Writes `model_auto_compact_token_limit` via `config/value/write`; "model default" removes it. |
| 11 | Retention ↔ compaction sync | `shared/compactionSync.ts`. It only warns and never changes anything. Findings: the running thread froze a different limit; the value is clamped to 9/10 of the context window; the `body_after_prefix` scope is unclamped. (Reasoning retention prices against the budget left under the same limit, so there is nothing else to be out of step with.) |
| 13 | Meters | Context left, cache % of the last request, speed, total cost. |
| 14 | Themes | Nine palettes as `--sf-*` CSS variables. |
| 15 | Panels under the composer | Invisible, mode (default/plan, permissions), skills, file and folder tree that **blocks** what you tick (see *Blocked files* below), model + reasoning effort. |

## Blocked files

The tree under the composer decides which files Suffice may read: **Block** the ticked ones, or
**Select** them and block the rest. It never sends them to the model. The block is
enforced by Codex's own permission machinery; no check is added outside it.

- **Profile per chat.** A chat that blocks files starts with a session-scoped permission profile in
  `thread/start.config`: `default_permissions = "suffice-block"` plus a `[permissions.suffice-block]`
  profile. This is the same shape Codex's app-server tests use. The profile extends `:workspace`,
  keeps the network on, and has one `deny` entry per blocked path. `thread/resume` gives a resumed
  chat the same profile. Nothing is written to config.toml, so the TUI and other chats are untouched.
- **Enforced by Codex.** Codex enforces the profile for `exec_command` (the OS sandbox), for `read`
  and `view_image` (the sandboxed file helper), and for `apply_patch`.
- **The fork's search tools.** `grep` and `glob` used to run ripgrep and the directory walker
  outside the sandbox. They now run ripgrep through Codex's own exec pipeline (`build_exec_request`
  and `execute_env`, the path `exec_command` uses) inside the turn's sandbox whenever the profile
  narrows reads (`core/src/tools/handlers/search_rg.rs` `run_in_turn_sandbox`). `glob` uses
  `rg --files`. If no sandbox can be selected they refuse to search instead of running unconfined.
  Without a narrowing profile they run exactly as before.
- **One list per chat.** A running chat keeps the list it started with. Edits apply to a new chat;
  the panel offers "Apply in a new chat".
- **What the model sees.** Codex lists the denied paths to the model in its permissions instructions
  ("Denied filesystem reads"). The owner approved this.
- **Windows.** Upstream Codex refuses read restrictions on the unelevated sandbox, and a chat that
  blocks files does not even start there. Blocking needs `[windows] sandbox = "elevated"`, set up
  once with `/setup-default-sandbox` in the terminal UI. The panel says so.
- **Codex quirk.** A blocked path that no longer exists is recreated as an empty folder by the
  Windows sandbox before it is locked.
- **Select mode.** The switch above the filter chooses Block or Select. Select turns the picks into
  their complement: at every level of the workspace on the way to a pick, each entry that is neither
  picked nor on the way to a pick becomes a `deny` entry. For `src/api/client.ts` that is the rest of
  `src/api`, the rest of `src` and the rest of the root. The listings come from the app-server's
  `fs/readDirectory`. A picked folder keeps all of its contents. Nothing outside the workspace is
  denied. `AGENTS.md` stays readable because Codex reads it to build the chat's instructions.

## Selected skills

The skills panel under the chat box, and "Use in this project" on the Skills screen, pick the
skills a new chat in this workspace may use. With nothing ticked a chat sees every skill that is on,
exactly as Codex would show it.

- **Mechanism:** Codex's own `[[skills.config]]` rules, given as session config in
  `thread/start.config` (`"skills.config": [{path, enabled}]`). Every ticked skill is `enabled: true`,
  every other known skill is `enabled: false`. Codex reads these rules from the session layer as
  well as from config.toml, the later layer winning (`config/src/skills_config.rs`
  `skill_config_rules_from_stack`). Nothing is written to config.toml, so the TUI and other chats
  are untouched.
- **Effect:** a disabled skill is left out of the "Available skills" list in
  `<skills_instructions>`, and Codex refuses it when it is named with `$skill` or sent as a skill
  item (`skills/src/selection.rs`). A ticked skill is on for the chat even when config.toml turns it
  off. Both verified live against the app-server.
- **Not blocked:** the skill's files stay on disk and readable. Codex has no skill-level read
  block; the file access switch can deny the folders if needed.
- **Fixed per chat:** the rules are stored with the chat and given again on `thread/resume`. A change
  while a chat is open applies to the next chat, and the panel offers one.
- **No per-turn cost:** skills are no longer sent as `skill` input items, which made Codex inject the
  whole SKILL.md into every turn. The model loads a listed skill itself when it needs it.

## Cost guarantees (tested in `tests/controller.test.ts`)

- A plain message sends only `{threadId, input, turnTrigger, invisible:false}`, with no model,
  effort, mode or permission overrides. It matches what `exec` sends, so the prompt prefix and
  cache stay the same.
- `collaborationMode` is sent only when the mode changes: into Plan, and once back to Default.
  The TUI sends Default on every turn, which was measured to add about 1.3K characters per
  request. See `PROMPT-CHANGE-PLAN.md` for the parity option.
- The conversation preference is sent once per chat, as `thread/start.developerInstructions` (P1).
- Selected skills and blocked files are thread config; nothing is added to the turn input.
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
