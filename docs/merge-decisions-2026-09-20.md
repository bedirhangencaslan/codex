# What was decided in the 2026-09-20 merge, and why

`git show 658ddfd66 -- <path>` already shows *what* each side had and what the merge kept: a
combined diff marks the lines that differ from both parents. What it cannot show is *why* one
side was chosen, or which of the two was load-bearing. That is what this file is for.

It covers the files where a decision was actually made. The other ~200 conflicts were the
product name meeting upstream's edit on the same line; those followed upstream, and
`scripts/merge/` now resolves that class automatically so the next merge does not repeat it.

Read with `CLAUDE.md`, which says what each module is supposed to contain, and
`scripts/merge/verify-names.py`, which fails if an identifier ends up half-renamed.

---

## Kept ours, because losing it changes the bill

### `core/src/compact.rs` — `tools: Arc::clone(&tools)`

Upstream still sends no tool list with the summarization request. Without it that request
falls off the cached prefix of the turns it follows and the whole history is re-priced at full
rate. The most fragile line in the merge.

Also **deliberately not adopted**: `executed_tool_calls::attach_to_compaction_prompt`. It
rewrites history items that earlier requests already sent, so the prefix stops matching at the
first rewritten item and everything after it is re-priced. `compact.rs` carries a comment
saying so, and `compaction_request_stays_on_the_turns_cached_prefix` in `compact_tests.rs`
fails if it comes back.

### `core/src/tools/context.rs` — the spill mechanism

`spill_notice`, `policy_less`, `output_budget`, `spill_path`, `write_spill`, and the
`response_text(&self, payload)` that uses them. When the budget cuts a command's output, the
untruncated text is written to `tool_<call_id>.txt` and the header names the path, so the model
reaches the cut part without spending a second request on it.

Scoped to `ExecCommandToolOutput` only: `read` returns a `FunctionToolOutput`, which has no
`spill_dir`, and reasoning never touches this path.

Upstream's only change in the contested region was extracting `(policy * 1.2)` into
`with_serialization_allowance`. That helper *is* `policy * 1.2`, so it was taken - a naming
change with no arithmetic in it. `policy_less` keeps the unit the policy was expressed in,
which decides whether the marker says `tokens truncated` or `chars truncated`.

### `history/src/lib.rs` — four fields on `CodexHarnessMetadata`

Both sides added fields to the same struct; it was accumulation, not disagreement. Ours:

| field | what it remembers |
|---|---|
| `invisible_turn` | the turn id of an invisible turn, so the exclusion survives a rollout round-trip |
| `reasoning_turn` | which turn owns this reasoning, so it stays visible while that turn runs |
| `reasoning_retained` | the verdict, priced once when the turn ends and never revisited |
| `tool_output_turn` | which turn produced this output |

**The trap:** our side of the region opened with `fallback_token_limit_override`, which upstream
had renamed to `history_truncation_token_limit` - and git had already carried the renamed field
in *above* the conflict. Keeping our side verbatim would have left two fields serialising to the
same key. The old one was dropped; the four after it were appended whole.

`serde(rename)` keeps the wire name, so old rollout files still read.

Note: `tool_output_turn` is written but never read. That is not merge damage - our own
`1eb41e32a` deleted the history-time shrinker that read it when the source-side one replaced it.

### `core/src/context_manager/history.rs` — `dropped_reason`

The decision that spends the fields above. Two stages, and the second is easy to get wrong:

1. turn still active → keep, unconditionally
2. turn ended → the frozen `reasoning_retained` verdict decides

`reasoning_turn`'s doc comment still describes the pre-verdict behaviour ("once the turn ends the
reasoning drops out of every later prompt"). That has been stale since `87ad58278` added the
pricing on 2026-09-03. **The code is authoritative.**

Also kept: `ResponseItem::Reasoning { .. } => 0` in the size estimate. Upstream now estimates it
from the encrypted payload. Neither has been priced against `_labs/logs/**/relay.jsonl`; ours
stays until measured, because not changing is safer than changing blind. It moves when
auto-compaction fires and what the context meter shows, not what any request contains.

### `codex-api/src/endpoint/responses.rs` — the `WireApi::Chat` arm

Upstream has neither this nor `ResponsesEndpoint`. GLM is reached through Z.ai, which does not
speak the Responses API. Lose it and the fork cannot reach its provider at all.

### `utils/output-truncation/src/lib.rs` — `never_worse`

Truncating adds a ~105-byte frame, so inputs just over budget grow instead of shrinking - six of
the module's own tests used to pin exactly that, 14 bytes coming back as 96. Upstream's new
`with_serialization_allowance` composes with it: theirs sets the budget, ours checks that
truncating under it actually saved anything. Both belong in the file.

### `models-manager/src/model_info.rs` — `align_apply_patch_section`

Our side was 46 lines against upstream's 7, and the difference was not only personality: the
call to `align_apply_patch_section` sits at the end of the `else` branch and upstream has no
equivalent. Dropping it would have done two things at once - the grammar-or-prose rule gone, and
GLM handed a Lark CFG its provider rejects.

It sits last so it sees the template the other overrides settled on, and inside the `else` so a
user-supplied `base_instructions` is left alone.

### `models-manager/models.json`

Merged field by field: where this fork moved a field away from the merge base, ours wins;
otherwise upstream's. All 11 templates verified byte-identical afterwards. `glm-5.3-flash` keeps
its 46 fields, `visibility: "list"`, `apply_patch_tool_type: "prose"`, and the reasoning levels
`922a73493` settled on (`high` by default, `low`/`high`/`max` offered, `none` and `medium`
removed because the provider rejects them).

`base_instructions` was dropped everywhere: the deserializer ignores it whenever
`instructions_template` exists, which is true of every model, so it was ~100 KB nothing read.

### `core/src/tools/spec_plan.rs`

Nothing had to be done by hand - git had already carried `lean_parameters`,
`include_shell_parameter` and `include_windows_shell_guidance` into upstream's new
`add_shell_tools()`. The conflict that remained was the now-duplicated inline call site.

### `tui/src/chatwidget/streaming.rs` — `stream_plaintext_reasoning_status()`

GLM returns its thinking as plain text, without the bold section headers OpenAI models use.
Without this the live tail never reaches the status line and thinking is invisible. Upstream's
side of the region was a bare comment.

### `tui/src/chatwidget/input_submission.rs` — `self.invisible_mode`

The fifth argument to `new_user_prompt`, which is where an invisible turn begins. Upstream's
`/*personality*/ None` was taken alongside it: our side passed a `personality` variable upstream
had removed, so keeping it would not have compiled.

### `core/src/config/mod.rs` — `free_guardian_enabled()`

Reads `features.guardianv2.free_guardian`. Upstream has nothing at that point, so it is pure
addition. The approval-policy default from `2950aa15a` (`AskForApproval::Never` outside untrusted
projects) sits outside the conflict and survived the auto-merge - checked, because it is the
change that removes approval round trips.

---

## Followed upstream, because the change was theirs to make

### `model_info.rs` — the personality half

All five constants are in the merge base; this fork's only touch was one word of one string.
Upstream removed them along with `config.personality_enabled`, which no longer exists anywhere
in the tree - so keeping our side would not have compiled either.

The same follow-through applies to `SlashCommand::Personality`, which upstream removed from the
enum, and to `core/tests/suite/personality.rs` and `chatwidget/settings_popups.rs`.

### `core/src/config/config_tests.rs`

Upstream renamed `can_write_path_with_cwd` to `can_write_local_path_with_cwd` and added a
Windows case. Our side called the old name. Taken, with `.suffice` restored in five places -
that directory name is what `config/src/loader` looks for and what `protocol/src/permissions.rs`
keeps the sandbox out of.

### `tui/src/thread_transcript.rs` and `pager_overlay/scrolling_tests.rs`

Both looked like large merges and were not: our total contribution to each was a single
`..Default::default()`, added because `UserHistoryCell` gained our `invisible` field. Upstream
rewrote 423 lines of the first and replaced the tests in the second.

In `thread_transcript.rs` the line still had to be re-added to upstream's literal - it sets
`spoken: false` but knows nothing of `invisible`, and a missing field does not compile. In
`scrolling_tests.rs` it did not: the literal it lived in belonged to a test upstream deleted.

### `session/mod.rs` — `build_model_client_beta_features_header`

Noted because it arrived with **no conflict** and changes behaviour. At the merge base every
key, including `remote_compaction_v2`, had to pass `config.features.enabled(spec.id)` before
being advertised. Upstream advertises it unconditionally. Kept theirs: it is a request header,
not prompt content, so no prefix moves, and both call sites go through
`build_responses_headers` while GLM is reached over `WireApi::Chat`.

### `mcp-server` and `.devcontainer`, deleted whole

Upstream removed both. The merged `Cargo.toml` had already dropped the workspace membership and
nothing depended on `codex-mcp-server`. Unlike the `thread/rollback` case, the deletion was
whole and consistent.

Of the 23 `UD` conflicts, 21 had nothing of ours beyond the rename. The two that did were
`tui/src/frames.rs` (our loading animation, `FRAMES_SUFFICE`) and one snapshot - both went with
upstream, which removed the animation files along with the module.

---

## What is still unverified

**Nothing here has been compiled.** Every resolution above is static inspection plus
`_labs/merge-2026-09-20/scripts/verify_savings.py`, which checks that 14 mechanism files are
byte-identical to the pre-merge branch and that the traces survive in the ten that conflicted.
It reports the text is in place. It does not report that it runs.

Three classes of breakage are still waiting, and only the compiler finds them:

- upstream call sites that do not match a signature this fork changed - `new_spoken_user_prompt`
  calls `new_user_prompt` with four arguments against our five
- our ours-only files (`read.rs`, `search.rs`, `reasoning_retention.rs`, `request_stats.rs`)
  never conflicted, but they call APIs upstream changed
- the decisions above

Snapshots and generated schemas carry upstream's content as a placeholder. Regenerate after the
build: `cargo insta accept`, `cargo run -p codex-config-schema`.
