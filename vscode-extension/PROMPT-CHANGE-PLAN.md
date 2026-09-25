# Prompt change plan (NOT applied)

The goal says prompt changes are only planned, and are applied after every other step with the
owner's approval. Nothing below is implemented. Today the extension sends no text to the model
that `suffice exec` would not send (see `tests/controller.test.ts`).

Each item lists what would change in the model input, what it costs, and how to switch it on.

## P1: send the conversation preferences box (goal item 8)

Today the Settings text box is stored in VS Code globalState and never leaves the extension.

**Option A (recommended): `thread/start.developerInstructions`**
- This is an existing protocol field (`ThreadStartParams.developerInstructions`). Core adds it
  as a developer-role fragment (`core/src/context/developer_instructions.rs`,
  `generic.developer_instructions`).
- It is sent only when a thread is created, so the prefix stays stable for the whole thread and
  prompt caching keeps working. Cost: the length of the text as input tokens once, then almost
  all of it at the cached price ($0.015/M on glm-5.3-flash).
- To change: in `webview/app/controller.ts`, where `thread/start` is built, add
  `developerInstructions: init.preferences.trim() || undefined`. Update the test "the preferences
  text is NOT sent anywhere" to expect it only in `thread/start`.
- Caveat: editing the box would not affect running threads. Replace the Settings notice
  `settings.prefsPending` (en/tr) with "Applies to new chats."

**Option B: `developer_instructions` in the user config** (`config/value/write`)
- This applies to every client, including the TUI and exec, which may be unwanted.
- It changes the prefix of every new session on the machine.

**Not recommended:** prepending the text to each user message. It changes every turn, adds its
tokens to every request, and shows up in the transcript.

## P2: TUI parity for the collaboration mode block

Measured: a turn that carries `collaborationMode = default` adds a `<collaboration_mode>` block
of about 1.3K characters to the request. The TUI sends Default on every turn; `exec` sends
nothing. The extension currently behaves like `exec` and only sends a mode when it changes: into
Plan, then once back to Default.

- Keep as is (recommended): the cheapest option, and a thread started in the extension reads
  exactly like an `exec` thread.
- Parity: send `{mode:"default", settings:{model, reasoning_effort, developer_instructions:null}}`
  on every turn, as the TUI does. Cost: about 1.3K characters on the first request, mostly cached
  afterwards. Change: `collaborationMode()` in `controller.ts` returns the Default mask when
  there is no change. Two tests in `controller.test.ts` would flip.

## P3: retention window vs the compaction limit (finding only, for the owner)

`core/src/request_density.rs` hardcodes `WINDOW_TOKENS = 80_000`, and the reasoning retention
horizon is computed from it. The horizon does not follow `model_auto_compact_token_limit`. When
the slider is set to anything other than 80000, retention and compaction plan for different
windows. The extension's sync check (goal item 11) reports this and changes nothing.

Possible fix (a Rust change inside a cost mechanism, so only with approval): derive the window
from the session's effective auto-compact limit. This is not a prompt change, but it changes how
much reasoning is kept, so it changes model input.

## Order when approved

1. P1 option A (small, cache-friendly).
2. Decide P2 (default: keep).
3. P3 as a separate core change with its own measurements (`_sim` / the wire benchmark).
