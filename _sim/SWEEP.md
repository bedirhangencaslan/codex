# The piece-by-piece sweep, and what it ruled out

Every input the model receives, swapped into a faithful OpenCode one at a time, zero context each
run, 3–9 reps per piece. This is the sweep the coarse ladder in `RESULTS.md` skipped.

## The metric

Not the nominal `limit` the model writes — that counts an unlimited read of a four-line `mod.rs`
the same as 500 lines of a 3,000-line file. What is billed is **bytes returned per read call**, and
the outcome turns out to be **bimodal**, not continuous:

- **bounded mode** — 2,600–5,200 bytes a file, the model wrote a window
- **unbounded mode** — 8,800–16,800 bytes a file, it took whatever the default gave it

So the statistic is *how often an arm lands in the unbounded mode*, and a mean over reps hides it.

## Results

| piece swapped into OpenCode | reps | unbounded | median B/read |
|---|---|---|---|
| *(nothing — the baseline)* | 9 | **3/9** | 5,214 |
| `t1-goals` — add `create_goal`/`get_goal`/`update_goal` | 2 | 0/2 | 3,867 |
| `t2-updateplan` — `todowrite` → `update_plan` | 3 | 0/3 | 3,183 |
| `p1-prompt` — `default.txt` → `instructions_template` | 3 | 1/3 | 3,024 |
| `p4-env` — `<env>` → `<environment_context>` | 3 | 1/3 | 4,012 |
| `t3-exec` — `bash` → `exec_command` | 3 | 1/3 | 4,177 |
| `p2-skills` — OpenCode's skills tail → our skills+permissions block | 9 | 3/9 | 4,968 |
| `p3-agents-user` — AGENTS.md out of the system message into a user turn | 9 | 3/9 | 3,807 |
| `t4-applypatch` — `edit`+`write` → `apply_patch` | 9 | 4/9 | 3,992 |
| `h1-feed-reasoning` — hand the model's own `reasoning_content` back | 5 | 2/5 | — |

**Nothing separates from the baseline.** The baseline itself goes unbounded a third of the time, and
no piece moves that rate outside the noise.

## The control fails, and that is the finding

`v-oc1-real` runs the **byte-identical prefix of `wire-stock-rep1`** — same 32,457-character system
message, same ten tool specs, same 519-character task, same `max_tokens`. That is the real run in
which OpenCode read 44 files at 80 lines each, with **44 of 44 reads carrying an explicit window**,
and 132/132 across its six recorded runs.

This simulator, on that exact input: **4 of 6 reps unbounded**, up to 16,840 bytes a file.

So OpenCode's reading discipline **is not in anything it sends the model**. Not the prompt, not the
tool descriptions, not the message layout, not the environment block, not the request parameters —
this rig now holds all of those identical and still does not reproduce it.

## Where it must be instead

From OpenCode's own session store (`ocreplay.py`), the real run's second assistant turn:

```
assistant parts=17
    step-start
    reasoning:560
    text:78
    tool:read(... "limit": 80)      x13
    step-finish
```

Three differences from this loop, none of them input text:

1. **It batches 13 reads in one turn.** This loop gets 2–5. A model issuing thirteen calls at once
   is budgeting across them; one issuing four is not.
2. **It emits prose alongside the calls** (78 characters). This loop usually gets none.
3. **Its turns are wrapped in `step-start`/`step-finish`** parts, so the provider adapter is
   shaping the assistant message differently from the plain `{role, content, tool_calls}` sent here.

Feeding the model's own `reasoning_content` back — the most obvious of these — was tested and
changed nothing (2/5).

## What this cost

$0.10 across 100 simulator reps, against $0.13 for five inconclusive binary runs of the real fork.

## The answer: `glob` sorts its results, and OpenCode's does not

The sweep above is invalid, and the reason is the sweep's own tool layer. `tools.py` *re-implements*
each tool, and a re-implementation is a difference like any other. OpenCode stores every real tool
result in its session database, so `replay.py` can serve the bytes the real run actually received.
Doing that changed the baseline immediately:

| baseline, byte-identical `wire-stock-rep1` prefix | n | unbounded | median B/read |
|---|---|---|---|
| tool results re-implemented (alphabetical `glob`) | 6 | 4/6 | 8,882 |
| tool results replayed from the real run | 6 | 1/6 | 3,558 |

The two `glob` outputs are **the same 44 files in the same 3,344 bytes**. The only difference is the
order of the lines. So the experiment reduces to that one thing, with everything else held identical
— same prefix, same ten tool specs, same read envelope, same `max_tokens`, same replayed results:

| `glob` result order | n | unbounded | median B/read | mean |
|---|---|---|---|---|
| **walk order** — what `rg` streams, passed through untouched | 17 | **1/17** | **3,558** | 4,290 |
| **alphabetised** — sorted before returning | 24 | **10/24** | 4,968 | 7,095 |

One-sided Fisher exact **p = 0.011**.

For reference, the real OpenCode runs sit at **2,330–3,144** bytes a read. Walk order lands at 3,558;
alphabetised averages 7,095, which is 2.3× over.

### This is a real divergence in the fork, made on purpose

`codex-rs/core/src/tools/handlers/search.rs:174-176`:

```rust
// Deterministic order: `rg` streams in walk order, which is filesystem-dependent, and a search
// ...
files.sort();
```

OpenCode's `glob.ts:50` hands `ripgrep.glob(...)` straight back — no sort. The fork added the sort
for reproducibility, and reproducibility is a good reason; it just costs more than it looks.

Why ordering would matter at all is a guess, and the measurement does not need it: alphabetical
groups the list into a tidy directory hierarchy that reads like "traverse this tree completely", and
it puts the two largest root files first (`api_bridge.rs`, `api_bridge_tests.rs`), where walk order
opens with `error.rs` and `telemetry.rs` at 1.5 and 2.8 KB.

### What it does not explain

The noise floor is enormous and the earlier arms were all under it: `v-oc1-real` and
`v-oc1-replay-sorted` receive **byte-identical input** and answered 4/6 and 2/6. Nothing measured at
n ≤ 9 in the table above survives that, including every piece in the sweep. The ordering result
stands only because it was taken to n = 17 against n = 24.

## Honest status

The question *"which input makes our agent read whole files"* now has a negative answer: **none of
them, in isolation**. The next thing to test is the loop itself — how the assistant turn is shaped
and why the real agent batches thirteen calls where this one batches four — and that is a different
experiment from the one this file describes.
