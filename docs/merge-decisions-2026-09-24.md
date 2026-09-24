# What was decided in the 2026-09-24 merge, and why

Upstream `e29eceb751..29f056c26c`, 259 commits, against 158 of ours. With `scripts/merge/`
installed the merge stopped on 5 files; a plain text merge of the same commits stops on 29.

Read with `docs/merge-decisions-2026-09-20.md`, whose decisions this merge had to keep, and
`CLAUDE.md` for what each module is supposed to contain.

---

## Resolved by hand

### `core/src/compact.rs`

Kept `let tools = sess.last_known_tool_specs().await;` - the summarization request still has to
carry the tool list of the turns it follows, or it falls off their cached prefix.

Took upstream's move of `responses_metadata` into the retry loop (`ac98537678`, MCP
attribution sent in `client_metadata`). It is outside model-visible content and, by that
commit's own account, only sent to OpenAI destinations, so it costs this fork nothing. Our
side's pre-loop copy was upstream's old line, not ours, and was dropped.

The conflict was an insertion next to a deletion: upstream removed the block before `loop`,
we had added `tools` just below it.

### `core/src/session/turn.rs`

Both. Upstream's per-request `responses_metadata` (same commit as above), then our
`sess.request_stats.record_prompt(&prompt)`, both before the guardian check.

### `codex-api/src/provider.rs` - the `WireApi::Chat` field

Upstream moved `Provider` and `RetryConfig` into `codex-client/src/provider.rs`
(`f5960fcc22`) and left `pub use` lines behind. **The moved file arrived without our `wire`
field and without a conflict**; only the stub conflicted. Taking the stub alone would have
removed the one field that lets the fork reach Z.ai.

`WireApi` and `Provider::wire` now live in `codex-client`, re-exported as
`codex_api::WireApi`, so the existing `crate::provider::WireApi` paths are unchanged.
`codex-api/tests/realtime_websocket_tls.rs` had been missing `wire` since the previous merge;
added.

### `core/src/config/config_tests.rs`

Upstream's test skeleton taken whole (`legacy` / `prefer_mxc` cases, the MXC feature setup).
Only the expectations follow this fork's code: a trusted project defaults to
`:danger-full-access` with approvals `Never` (`2950aa15a`), with or without MXC. No sandbox
code was touched - `default_builtin_permission_profile_name` is ours and did not conflict.

The previous version asserted "can write" and then, on Windows, "cannot write" in the same
test.

Not adopting upstream's sandbox default is deliberate: `lean_parameters` in `spec_plan.rs`
is `approval_policy == Never`, so upstream's `OnRequest` default would take `exec_command`
from three parameters to ten.

### `models-manager/models.json` - rebuilt as data

Line-merging this file is not safe, and this time it was worse than it looked:

- `glm-5.3-flash` sat at the top of our file, where upstream's `gpt-6-astra` sits (the
  previous merge had put astra last). Git aligned the two cards as one model. Where GLM's
  value happened to equal astra's old one, upstream's change to astra was applied to GLM
  **without a conflict**: `shell_type` became `shell_command` - GLM would have lost
  `exec_command` - plus `supports_experimental_context` and `available_in_plans`. Choosing
  "ours" in every conflict region did not undo it.
- The rename driver folds our side before merging, so the conflict's "ours" showed
  `You are Codex` where our file says `You are Suffice`. Taking it would have shortened GLM's
  template from 16,166 characters to 16,162.

So the file was built from the index stages, never from the markers: upstream's catalog; every
field merged three-way against the base; the seven templates both sides edited merged on the
template's own lines (every line either side added is present, every line either removed is
gone); `glm-5.3-flash` copied verbatim from the pre-merge branch; `gpt-5.2` and `gpt-5.4-mini`
kept; upstream's new `gpt-6-sol` and `gpt-6-luna` taken as upstream has them, with
`$CODEX_HOME` -> `$SUFFICE_HOME` as the previous merge did for astra.

GLM is now the **last** card. Upstream adds new models at the top; order has no runtime effect
because presets are sorted by `priority`.

Left open: the hidden OpenAI models follow upstream to `shell_type: shell_command` (only
upstream changed that field); `gpt-6-astra`, `gpt-6-sol`, `gpt-6-luna` are `visibility: list`.

### `Cargo.lock`

The `generated` driver kept ours. The plain three-way merge of it was clean and equals
upstream's package set exactly - this fork adds no dependency - so that was taken instead of
letting cargo fill the four missing packages at whatever version is newest.

---

## Found in files that merged without a conflict

| where | what happened | done |
|---|---|---|
| `cli/Cargo.toml` | the driver folds every lowercase `suffice`; `name` and `default-run` became `codex`, so the build would produce `codex.exe` | restored; scoped rule in `rename-map.txt`; two checks in `assertions.py` |
| six test lines | the `.codex => .suffice` rule renamed a line-leading `.codex` - a method chain reading the field `codex` - undoing `bbdd99a92a` | restored; the rule now skips a `.codex` that starts its line. On upstream's tree the old and new rule differ on 182 lines, all field accesses |
| upstream's new test files | `post-merge.py`: 25 lines in 8 files, `SUFFICE_HOME`, `.suffice`, `.suffice-plugin` | applied, read line by line first |
| plugin metadata file | upstream moved the constant to `core-plugin-common/src/installed.rs` as `.codex-remote-plugin-install.json`; the map's `.codex` rule does not match before `-` | **open**: the on-disk name changed, and two tests still expect `.suffice-...` |
| ~20 messages | "Run `suffice login`" became "Run `codex login`" - product name, by the driver's policy, but it names a command | **open** |

## Tooling that was wrong

- `git merge-tree` **does** run custom merge drivers in this git version. `test-driver.py`
  relies on the opposite and now reports a meaningless `0/35`.
- `survival.py` and `reachability.py` matched `POLICY_EXCLUDED` exactly against 9-character
  ids; `%h` now prints 10, so the rename commit was silently scored. Now a prefix match.
- `assertions.py` did not pin `WireApi` or `Provider::wire`, and its `request_stats` check also
  matched the end-of-turn flush. Both added.

## Measured

`confirm.py`'s method over every non-merge commit of ours upstream lacks (150): 32,816 evidence
lines, 32,795 present in the merge (99.94%). The 21: 12 are `WireApi`, moved with `Provider`;
6 are lines upstream rewrote around ours (a template line, a test's shape); 3 are script lines
changed on purpose above. Over the previous merge's window the merge moved 28,877 to 28,862,
for the same reasons.

## Not verified at the merge commit

- Compiled only in the commits that follow it.
- Snapshots and generated schemas are the pre-merge copies; regenerate.
- The first request body has not been diffed against the pre-merge tree.
