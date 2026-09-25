# Prompt change plan (NOT applied)

The goal says prompt changes are only planned, and are applied after every other step with the
owner's approval. Nothing below is implemented. Today the extension sends no text to the model
that `suffice exec` would not send (see `tests/controller.test.ts`).

Each item lists what would change in the model input, what it costs, and how to switch it on.

## P1: the conversation preference (APPROVED by the owner, 2026-09-26, applied)

The **Preference** button under the chat box, between Files and the model, opens a text box.

- **When it is sent:** a chat that starts with a preference gives it to the model once, as
  `thread/start.developerInstructions`. Codex renders that into the chat's opening developer
  instructions (`core/src/session/mod.rs` 4302-4310). After the first request it is read from the
  cache.
- **Tag:** the text is wrapped in `<users_conversation_preferences>` ... `</users_conversation_preferences>`,
  in the style of Codex's other context sections. The owner approved the tag, which exists only
  when there is a preference; without one nothing is sent. The panel and the per-chat record keep
  the bare text.
- **Stored per chat:** the preference is kept with the chat, and `thread/resume` gives it back.
  This matters because Codex rebuilds its developer instructions from config at every compaction.
- **Fixed for the chat:** Codex ignores `developerInstructions` for a running chat
  (`app-server thread_processor.rs` 220-224).
- **Why no mid-chat change:** a live test showed that `thread/fork` alone does not deliver a new
  preference. Only fork plus compaction does, and that summarises the chat. The owner chose "set at
  the start": an edit while a chat is open applies to a new chat, and the panel offers that.

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

## P3: retention window vs the compaction limit (WITHDRAWN, 2026-09-26: the finding was wrong)

The earlier note said reasoning retention does not follow the configured limit. It does:

- **Where the budget comes from:** `reasoning_retention::horizon` gets `budget_remaining` from
  `session/context_window.rs` `base_window_tokens_remaining`, which is the configured compaction
  limit minus the tokens used (session/mod.rs 4575-4589).
- **What the 80K is:** `WINDOW_TOKENS = 80_000` in `request_density.rs` is only the unit in which
  the user's request density is measured and persisted (`request_density.json`). In
  `budget_remaining / WINDOW_TOKENS * requests_per_window` it cancels out.
- **Why nothing changes:** tying it to the limit would mix units across the persisted samples and
  break the horizon. No change is made. The extension's misleading "retention window" warning is
  removed.

## P4: blocked files are listed to the model (APPROVED by the owner, 2026-09-25, applied)

A chat that blocks files runs under a permission profile with `deny` entries. Codex then renders its
own "## Denied filesystem reads" section into the `<permissions instructions>` developer message.
That section names every blocked path, and the sandbox-mode sentence changes to `workspace-write`.

- **Where it appears:** the text is Codex's own, not the extension's. The profile belongs to the
  chat, so the section is part of that chat's context from its first request.
- **Cost:** about 150-300 tokens depending on the number of paths.
- **Owner's instruction:** use Codex's pipe for this, not text written by the extension.

## Order when approved

1. P1 option A (small, cache-friendly).
2. Decide P2 (default: keep).
3. P3 as a separate core change with its own measurements (`_sim` / the wire benchmark).
