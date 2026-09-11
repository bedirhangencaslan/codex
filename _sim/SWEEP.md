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

## Honest status

The question *"which input makes our agent read whole files"* now has a negative answer: **none of
them, in isolation**. The next thing to test is the loop itself — how the assistant turn is shaped
and why the real agent batches thirteen calls where this one batches four — and that is a different
experiment from the one this file describes.
