# What was decided in the 2026-11-02 merge, and why

Upstream `29f056c26c..0a2dda4b5e`, 1,655 commits, against 173 of ours. With `scripts/merge/`
in place the merge stopped on 106 files (60 content, 46 modify/delete, 147 hunks); a plain text
merge of the same commits stops on 233.

Read with `docs/merge-decisions-2026-09-20.md` and `docs/merge-decisions-2026-09-24.md`, whose
decisions this merge had to keep.

---

## Kept ours, adapted to upstream's new shape

### `context_manager/history.rs`: prompt building

Upstream made `for_prompt` async to replace unavailable file images. This fork adds
`active_turn_id` so reasoning and invisible items stay scoped to their turn. Both are kept:
`for_prompt(modalities, image_store, active_turn_id)`, with the measured variants carrying the
`ContextFilterReport` for `request_stats`. `prompt_input_for_step` now takes the history, because
upstream's sampling loop keeps it to count items.

Reasoning is still estimated at zero. Upstream added an arm estimating encrypted reasoning in
`estimate_response_item_model_visible_bytes`; it was removed, since reasoning only reaches the
model in its own turn, where the server reports real usage.

### `compact.rs`

- Kept the tool list on the summarization request and the retained assistant tail.
- Took upstream's rewrite of `build_compacted_history_with_limit`, which borrows the original
  items. Selected messages now carry their request index so the tail can follow them.
- Took upstream's `base_instructions` logic (empty when history already holds the fragment) and
  `cyber_access_program`.
- `attach_to_compaction_prompt` stays out of local compaction. The remote path was upstream's and
  stays as it was.

### `tools/context.rs`: spill and condense

Upstream replaced `raw_output: Vec<u8>` and `output_omitted_bytes` with `CapturedStream`, which
owns the omission marker. `truncated_output_with_policy` wraps condensed text in a
`CapturedStream`, so the marker logic is upstream's and the text is ours. `spill_dir` sits beside
upstream's `tool_cwd`.

### `tools/handlers/unified_exec/`

Upstream moved argument parsing to `preparation.rs` and dropped the fields from
`ExecCommandRequest`. `spill_dir` is resolved in the handler as before and kept on the request.
`resolve_tool_environment` now returns `Result`, so the unavailable-environment branches in
`read.rs` and `search.rs` became `?`.

### `tools/spec_plan.rs`: `apply_patch` gate kept

Upstream registers `apply_patch` regardless of environments. Following it sent the tool to
environment-less requests such as the TUI recap (`recap_generation_uses_bounded_structured_request`
caught it), so `environment_mode.has_environment()` was restored. `exec_command` and `view_image`
follow upstream; called without an environment they return `No usable execution environment`
to the model, pinned by `environment_tools_answer_the_model_when_no_environment_exists`.

### `tools/code_mode/mod.rs`

`budget_code_mode_result` runs only under upstream's `CodeModeOutputTruncation::Client`. When the
host truncates, ours does not cut a second time.

### `utils/output-truncation/src/lib.rs`: `never_worse`

Kept on top of upstream's boundary remapping. Boundaries are remapped only when the frame is
actually returned; if the content is returned whole, offsets are already right. The removal of
`formatted_truncate_text_content_items_with_policy` follows upstream.

### `client.rs`

- Keep-alive fields, methods and the `arm` call kept beside upstream's `last_inference_tools` and
  `with_runtime_auth`.
- `include` keeps our `supports_encrypted_reasoning` gate, then upstream's
  `output.encrypted_content` push.
- `prompt_cache_key_override` gives way to upstream's `PromptCacheKey` policy. `free_guardian_enabled`
  on `ModelClient` had no reader left and was dropped.

### Chat wire

Upstream moved instructions into `input` as a developer fragment, so `chat.rs` stopped reading
`request.instructions`; the existing developer-to-system mapping delivers them as the first
system message. The Chat branch now calls `ResponsesHttpExecutor`, replacing the removed
`stream_responses_api`.

### `models-manager/models.json`: rebuilt as data

As on 2026-09-24, never from the markers: upstream's catalog, every field merged three-way against
the base, templates merged on their own lines, `glm-5.3-flash`, `gpt-5.2`, `gpt-5.4` and
`gpt-5.4-mini` kept, `$CODEX_HOME` -> `$SUFFICE_HOME`.

`gpt-6-luna`: `tool_mode: direct`, `use_responses_lite: false`, `default_reasoning_summary: auto`.
Responses Lite is off because the API rejects `parallel_tool_calls: true` under it, and in direct
mode that flag is the only way to batch.

---

## Followed upstream

- Deleted files with only rename edits on our side: repository `.codex` guidance and skills,
  `AGENTS.md`, `plugin-creator`, zsh-fork, personality templates, old snapshots.
- `history_cell/session.rs` and `status/card.rs`: upstream's redesign replaces our rail header.
  Invisible-turn rendering in `messages.rs` is kept.
- `personality` on `AppCommand`, `is_default` in the model popup and `status_line_hyperlink_url`.

---

## Prompts changed after the merge, at the user's request

Each coding model other than `glm-5.3-flash`, `gpt-6-luna` and `codex-auto-review` received: the
Suffice identity, the `glob`/`grep` search line in place of the `rg` line, "Cut reads", "one call
at a time" and "short read-only inline script". Direct-mode models (`gpt-5.5`, `gpt-5.4`,
`gpt-5.4-mini`, `gpt-5.2`) also received "several tool calls in a single response". Code-mode
models keep the `functions.exec` batching lines, which are the way to batch in that mode.

---

## Building

`rusty_v8` moved to 152.2.0. Building `codex-code-mode-host` needs the release archive and binding
from `openai/codex` `rusty-v8-v152.2.0`, verified against
`third_party/v8/rusty_v8_152_2_0_release_manifests.sha256`, through `RUSTY_V8_ARCHIVE` and
`RUSTY_V8_SRC_BINDING_PATH`. Run cargo from inside `codex-rs`, or `.cargo/config.toml` is not read.
