# Did the 2026-09-20 merge keep this fork's work?

Companion to `merge-decisions-2026-09-20.md`. That file records what was *decided*; this
one records what was *verified*, by three methods that do not share an assumption, and
what they found.

All 256 conflicts in that merge were resolved by hand, by static inspection, and nothing
had been compiled. The risk was never that a resolution was argued badly — it was that a
line disappeared and nobody noticed, because no check in the repository could see it.

**Result: one real defect, in `8b5b39d51`. Everything else this fork owns survived.**

---

## What was measured

| | |
|---|---|
| base | `0a12b855a` |
| ours | `88a2b0c53` (130 commits, 125 non-merge) |
| upstream | `e29eceb75` (1,050 commits) |
| merged | `b75aeafca` (merge commit `658ddfd66`) |

Our commits touch 467 files. Comparing blob ids settles most of them before any analysis,
and that comparison is proof rather than inference:

| bucket | files | what it means |
|---|---:|---|
| `ours:F` and `head:F` are the same blob | **254** | byte-identical; nothing to check |
| we deleted it ourselves in a later commit | 21 | no obligation (e.g. `0b4672c5f` folded 16 `_sim/*.md` into one) |
| absent at HEAD | 2 | `compact_remote.rs`, `compact_remote_request.rs` — see below |
| generated, per `.gitattributes merge=generated` | 44 | content is an output; regenerate, do not resolve |
| **differing, shared, not generated** | **144** | the set that got the full four-way analysis |

Of the 188 files where ours and HEAD differ, **none is ours-only**. Every file upstream
has never seen — including all 141 of `_sim/` and 36 of `gui/` — came through untouched.

---

## Three methods, one answer

### 1. Four-way line accounting — `scripts/merge/survival.py`

For each of the 144 contested files it counts canonical lines in all four trees and asks
what the merge *should* contain. Additions and deletions are one formula, because a fork
removes things on purpose and a merge can put them back:

```
do = ours - base ;  du = upstream - base
expected = base + (max(do,du) if both added else min(do,du) if both deleted else do+du)
```

Every discrepancy is then sorted by whether upstream had touched that region of the base
file — `LOST-SILENTLY` if they had not, `SUPERSEDED` if they had.

```
86 findings — 68 SUPERSEDED, 18 LOST-SILENTLY (16 on strong evidence)
```

17 of the 18 are one cluster in one file. The 18th is a test assertion in
`context_manager/history_tests.rs` that `6a5e29fa1` had removed and the merge restored;
the method it asserts on still exists, so it compiles and is harmless.

### 2. Named assertions — `scripts/merge/assertions.py`

44 explicit checks, each counted on comment- and string-stripped code at **both** the
pre-merge branch and HEAD, so "still present" and "went from twelve to one" read
differently.

```
43 pass, 1 fails
```

The one failure is the same cluster.

### 3. Reachability — `scripts/merge/reachability.py`

530 symbols our commits introduced, counted for references outside their own file and
outside tests, at both revisions. The trigger is the delta, not zero: a symbol only ever
called from tests is not a finding, and upstream renaming something is not either.

```
1 ORPHANED, 2 WEAKENED
```

`ORPHANED` is `TOOL_OUTPUT_SUBDIR` — the same cluster again, seen from the other end: the
constant still exists, and nothing outside its own file uses it any more. Both `WEAKENED`
symbols (`display_path`, `response_header`) were checked by hand and are unchanged; they
are counting artefacts, which is why that tier reviews rather than fails.

Separately, `verify-names.py` reports no identifier living under two spellings, and
function-level survival is 494 of the 497 functions we introduced and still had.

---

## The defect

**`8b5b39d51` — "Keep the output the budget cuts, and tell the model where to find it".**

`codex-rs/core/src/session/handlers.rs` lost three things: the `use` of
`TOOL_OUTPUT_SUBDIR`, the definition of `remove_session_tool_output`, and its call inside
`shutdown_session_runtime`. Upstream did edit that function — they added a
`shell_snapshot_prewarm` block — but earlier in it. Our call sat between
`terminate_all_processes()` and `code_mode_service.shutdown()`, and at HEAD those two
lines are adjacent. Nothing decided this; it fell out during a hand resolution.

**Severity: low, and it is worth being exact about why.** This is disk housekeeping, not a
token-cost mechanism. Nothing about what the model receives changes. The spill writer,
its budget, and the notice that tells the model where to read are all intact and
verified. The mechanism that actually *bounds* the directory — `sweep_stale_tool_output`,
the age-based sweep at startup — survived whole, with its tests. What was lost is the
tidy-up on clean shutdown, which the code's own comment describes as "only what keeps it
empty in the ordinary case". Without it, `$SUFFICE_HOME/tmp/tool-output/<thread>` is
cleared on the next start rather than at exit.

It has not been fixed. Restoring three lines is trivial, but nothing here has been
compiled, and it should go in with the build rather than ahead of it.

---

## Positive confirmation — the number to quote

The three methods above all report *discrepancies*, so "no finding" means "nothing went
wrong that they can see". That is a weaker claim than "this commit's work is there", and
reading the first as the second overstates the result. `scripts/merge/confirm.py` asks the
other question directly: of the lines each commit added and our branch tip still had, how
many are in the merged tree?

```
29,083 evidence lines  ->  28,996 present  (99.70%)

  91 commits   every evidence line confirmed present
  13 commits   partly confirmed
  24 commits   nothing of their own left to confirm
```

The 24 are honest, not a gap: fourteen of them are prompt tweaks that rewrite the same
one-line template inside `models.json`, so only the last version survives to our branch
tip and the earlier commits have nothing of their own left to check. The rest are merge
commits, generated output, or formatting.

All 87 missing lines were read. Only `8b5b39d51`'s 17 are a loss:

| commit | missing | what it is |
|---|---|---|
| `8b5b39d51` | 17 in `session/handlers.rs` | **the defect** (below) |
| `8b5b39d51` | `(truncation_policy * 1.2)` | replaced by upstream's `with_serialization_allowance`, present and used |
| `256cf5c9c` | 11 `base_instructions` in `models.json` | the field the deserializer ignores whenever a template exists — which is every model here |
| `a65e71ff7` | 1 `lean_parameters:` | the *duplicate*; the real one branches inside `add_shell_tools()` |
| `87ad58278`, `ca5e2954f`, `722afda33` | 2 each in `history/src/tests.rs` | struct-literal initialisers in tests. All four `CodexHarnessMetadata` fields are intact at `history/src/lib.rs:92,100,112,120` |
| `1315edcd1` | 2 in `compact.rs` | restructured; `get_last_assistant_message_from_turn` is present three times, more than before |
| `04290217d` | 4 in `agents_md_manager.rs` | a cache-reset sequence; the fields it resets are still there |
| `2950aa15a`, `66c356eeb`, `af298167b`, `d4e2cb67f`, `6a5e29fa1` | 1–22 each | permission test assertions, TUI footer rendering, and the two files upstream deleted |

## Per-commit result

| | commits |
|---|---:|
| merge or empty, no content of their own | 5 |
| excluded by policy (`f06d1e476`, `db33ff3a1`) | 2 |
| **evaluated** | **123** |
|   no finding at all | **111** |
|   `SUPERSEDED` findings only, all reviewed and benign | 10 |
|   `LOST-SILENTLY` findings | **2** |

The two policy exclusions are the product rename and the TUI identity commit. They are
partially reverted *by design* — `rename-map.txt` states that the product name follows
upstream because that is what stopped nine in ten conflicts — so scoring them would drown
every real finding in 1,600 files of expected difference.

Every `SUPERSEDED` finding in a cost-critical file was read individually:

| what | why it is fine |
|---|---|
| `models.json` — 7 `base_instructions` fields | the deserializer ignores the field whenever a template exists, which is true of every model here. Dropping it cost nothing and saved 100 KB |
| `tools/context.rs` — `(truncation_policy * 1.2)` | replaced by upstream's named `with_serialization_allowance`, which is present and used. Same behaviour |
| `spec_plan.rs` — a `lean_parameters:` line | the *duplicate*. Auto-merge had already carried the real one into upstream's new `add_shell_tools()`, where it still branches |
| `config/config_tests.rs` — 8 permission assertions | upstream's permission model changed; recorded in the decisions file. Note `.suffice` does **not** appear here, which is the point — the tool masks it, so a reverted one would have shown |
| `tui/.../messages.rs` — `user_message_style()` | TUI styling |

---

## What none of this can tell you

Static comparison of text has hard limits, and they are the reason this file is not a
clean bill of health:

1. **A removal expressed as an addition.** `a65e71ff7` withholds seven of ten shell
   parameters by *adding* a boolean; its diff is `+232/-0`. No line-level deletion check
   can see that class — behaviour changed by a flag, a default, or a condition.
2. **A line that survives with its meaning changed**, because upstream changed the
   function it calls or the `Default` it relies on.
3. **Signature drift.** `new_spoken_user_prompt` calling our five-argument
   `new_user_prompt` with four: present, reachable, does not compile.
4. **Ordering.** `align_apply_patch_section` is correct only because it runs last in its
   branch. Multiset accounting is order-blind by construction.
5. **The 44 generated files**, which deliberately carry stale content until
   `cargo insta accept` and `cargo run -p codex-config-schema` have run.

There is also a limit found by testing the tool rather than reasoning about it. A
throwaway commit was made with `tools: Arc::clone(&tools)` deleted from `compact.rs` —
the line CLAUDE.md calls the most fragile in the merge — and `survival.py` was run against
it. It **did** report the line, attributed to `be1fab215`, but classified it `SUPERSEDED`
rather than `LOST-SILENTLY`, because upstream had rewritten that region too. So:

> **A loss inside a region upstream also edited is reported as `SUPERSEDED`.** Those
> findings in cost-critical files must be read, not filed. That is exactly why
> `assertions.py` exists and why its list is explicit rather than derived.

`cargo check --workspace` settles 1, 2 and 3 in one run and is the obvious next step.

---

## Running it again

```
python scripts/merge/confirm.py                    # are our lines there? (the headline)
python scripts/merge/survival.py --json out.json   # 0 unless something was lost silently
python scripts/merge/assertions.py                 # 0 unless a named mechanism moved
python scripts/merge/reachability.py               # review tier, always 0
python scripts/merge/verify-names.py               # 0 unless a name is half-changed
```

`confirm.py --detail <sha>` prints the exact lines a partly-confirmed commit is missing.

`survival.py --head <rev>` judges any merged revision, which is how the negative control
above was run, and how the next merge should be checked before it is trusted.
