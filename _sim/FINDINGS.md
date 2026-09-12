# Why the fork read whole files, and what actually fixed it

The single record. Replaces `ANSWER.md`, `BAND.md`, `BASELINE-MOVED.md`, `CONTEXT-PER-REQUEST.md`,
`EXPERIMENTS.md`, `FEED.md`, `MOTIVE.md`, `PREFIX.md`, `PROBE.md`, `RESULTS.md`, `REVERSE.md`,
`STAGE0.md`, `SWAP.md` and `SWEEP.md`, several of which reached conclusions that later measurement
overturned. Those reversals are kept here on purpose — knowing what was disproved is most of the
value.

---

## 1. The problem

Same task both sides: read 44 `.rs` files under `codex-api/src/`, write `WIRE.md` with one
`file:line` citation each. The fork cost 2-3× OpenCode.

The bill decomposes as `Σ(fresh × $0.075 + cached × $0.015 + out × $0.25) / 1M`. On the one
interleaved pair where both ran in the same session (`rep30`):

| | fresh | cached | out | cost |
|---|---|---|---|---|
| fork | 97,460 | 275,776 | 8,700 | $0.0136 |
| OpenCode | 53,622 | 218,304 | 5,312 | $0.0086 |
| **excess** | **+43,838 → $0.0033 (66%)** | +57,472 → $0.0009 | +3,388 → $0.0009 | +$0.0050 |

Two thirds of the excess is **fresh tokens** — bytes entering context for the first time, which on
this task is almost entirely file content. The fixed prefix is *not* the problem: 12,553 tokens
against OpenCode's 12,586, a 0.3% difference, with our tool array *smaller* than theirs
(14,683 characters against 21,803).

So: the model was pulling roughly 62% of the corpus where OpenCode pulled 30%, for identical output.

---

## 2. The answer

**`exec_command`'s tool spec.** Not the prompt, not the tools we added, not the message layout, not
the shell. The spec rides the fixed prefix of every request and acts **whether or not the tool is
ever called**.

Replacing it wholesale with OpenCode's `bash` spec, everything else held:

| | reads whole file | median window |
|---|---|---|
| as it shipped | 0.85 | 220 |
| **with OpenCode's `bash` spec** | **0.45** | **80** |

Six replays across two independent batches: **0.40 · 0.42 · 0.39 · 0.45 · 0.44 · 0.42**, against a
base at 0.65-1.00. No overlap, ±0.03 within arm — the tightest effect in this archive.

The difference is how the tool is described: ours 1,814 characters and **ten** parameters, OpenCode's
5,265 and **three**. Not the shell — both run PowerShell (`exec-server/src/client.rs:2060`).

---

## 3. Method, and three rules learned the hard way

Four rigs were built. Each failed in a way worth recording.

**Rig 1 — run both binaries.** Moves every variable at once, six to twelve requests before it says
anything. Five runs identified nothing and two stalled.

**Rig 2 — `sim.py`, a conversation simulator.** Assembles a prefix from captured wire bytes and runs
a real loop with real tools. Right idea, but it stopped at the first `read` by default, so it
measured an opening move.

**Rig 3 — `swap.py`, single-request replay.** Replays one captured body N times and scores the
`limit` values in the reply. Cheapest, and the basis of the transplant study in §6. Still an opening
move.

**Rig 4 — `probe.py`, today.** Runs the read phase out against the real corpus, reports **bytes
returned per read** and **window ÷ file length**, saves `reasoning_content`, interleaves round-robin,
aborts on a dollar budget. This is the one that found the answer.

### Rule 1 — interleave, never run in blocks

Running variants one after another put a false 29% → 93% on `parallel_tool_calls`, which vanished
when arms were interleaved. Something drifts inside a session and lands entirely on whichever arm
ran first.

### Rule 2 — a baseline is a run taken beside the run it is compared to

Same `opencode.exe` 1.18.29, same config, same task, one day apart:

| run | date | requests | reads carrying a `limit` | cost |
|---|---|---|---|---|
| `wire-stock-rep1` | 09-11 | 8 | 44/44 | $0.0086 |
| `wire-stock-rep11` | 09-12 | 25 | 25/44 | $0.0265 |

3× on nothing. Every cross-session comparison in the early work is void.

### Rule 3 — a positive control in every batch, and it must be the right kind

`oc+counts` (hand the model every file's length up front) takes OpenCode from 95% to 0% and
reproduced exactly, twice. But on the **fork's** base it does nothing — the fork is already at the
floor, and you cannot push down what is already down. A base at the floor needs a *lifting* control.
E1 and E2 of the probe study ran without a working control; their results rest on within-arm
tightness and on independent arms agreeing, which is weaker than it should have been.

### What the rig can and cannot do

Validated on both agents (`feed.py`): replaying OpenCode's real body 003 reproduces its own run's
bounded rate (53% against 57%), and the fork's reproduces its own (54% against 49%).

But **the request body does not determine the behaviour**. A byte-identical replay of OpenCode's
request — validated field by field, ten tool specs identical, only `content: null` against `""`
differing — gives 29% fully-bounded where the archive shows 132/132. Sampling variance at
temperature-default is larger than most effects being chased. This is why nothing under ~15 points
at 12 reps means anything.

---

## 4. Negative findings — what it is not

Every one of these was a live hypothesis, and killing them is most of what the work bought.

| ruled out | how | evidence |
|---|---|---|
| **Prefix size** | direct measurement | 12,553 tokens against OpenCode's 12,586. Our tool array is *smaller* |
| **The prompt text** | real runs, 3 reps | OpenCode's binary running *our* `instructions_template`: $0.0089 / $0.0103 / $0.0106, indistinguishable from its own $0.0086-$0.0109 |
| **The prompt, by section** | probe E1, 3 reps × 6 arms | Dropping `# Personality`, `# Working with the user`, `# Rules for getting work done` or `## Autonomy and persistence` all made it **worse**; swapping in OpenCode's whole prompt was worst of all (frac 1.0 against base 0.45) |
| **Batch size** | 47 probe runs | corr(batch, limit) = **+0.06**; corr(batch, bytes/read) = **−0.27**. Larger batches read *less*. Best rows are batch 8 at limit 65; worst are batch 2 at limit 800 |
| **Serialising reads** | binary, `rep33` | Took bounded reads to 46/46 and left the median window at 280. Cost **rose** to $0.0394 because 50 rounds × full context is a round tax: cached tokens 1.94M, 74% of the bill |
| **Telling the model how much context it has** | source audit | Nothing in either agent's prompt states a context size. `features.token_budget` defaults false; glm-5.3-flash has `"token_budget": null`. The model's `"context budget 98k"` is a hallucination |
| **Telling the model file lengths** | probe E3 + OpenCode's own trace | With the read footer silent about length the model still asks for "100-150" then 140. And OpenCode passed `offset` **zero times in 44 reads** despite 25 footers naming totals up to 3,083. Same information, opposite behaviour |
| **The line-count step** | three directions | Destroys OpenCode (95% → 0%) and does nothing to the fork: removing it made the fork worse (+43%), adding it changed nothing (probe E3) |
| **The tools we added** | git archaeology + probe E4 | `read` (`d829b5f25`), `glob`/`grep` (`03bab7aa3`) are ours. Swapping all three for OpenCode's left the model reading whole files (frac 1.0); swapping only `read`'s spec made it **worse** (limit 300) |
| **`max_output_tokens`** | probe E5 | Dropping it alone: frac 1.00 · 1.00 · 0.97. Nothing |
| **Removing `exec_command` entirely** | probe E5 | frac 1.00 · 0.67. Nothing — the earlier gain attributed to this came from the goal tools riding along |
| **`parallel_tool_calls`** | both transplant directions | Nothing either way |
| **The `read` spec's byte-ceiling clause** | `f-suf-nobytes`, 3 reps | 19% in the simulator, 14% in the binary. Real but small, and it was mistaken for the answer |

---

## 5. Findings that were retracted

Recorded because each was written up as settled and then failed to replicate.

**"The three `*_goal` tool descriptions"** (`RESULTS.md`). A 2-step probe showed dropping them took
the median window from 240 to 80. It does not predict cost, and in the full-task rig `drop:goals`
only reaches frac 0.80. Real but partial.

**"`glob` sorts its results and OpenCode's does not"** (`SWEEP.md`). From a batch whose own control
had failed. Did not survive.

**"One sentence in `read`'s description doubles the bill"** (`ANSWER.md`). The byte-ceiling clause,
measured at one-sided Fisher p = 0.0028. It was worth 14-19%, not 2×.

**"It is five causes and they multiply"** (`BAND.md`). Five pieces each damage OpenCode's working
window, and the product of four came to 5% against a measured 5%. But the reverse direction
disagreed for four of the five, and a one-at-a-time transplant cannot show necessity when factors
interact. "Jointly sufficient to reach the floor" is what the arithmetic actually said.

**"The prompt × layout interaction is the thing to fix"** (`PREFIX.md`). Real, 2.71× with no overlap
across three reps a side — but measured with the skills block still on. With skills off the prompt
is harmless, and probe E1 showed every prompt edit makes it worse.

**"The opening commentary sentence costs 72 points"** (`BAND.md`). It does, in OpenCode's window. The
follow-up separated content from the doubled turn: neither alone hurts (68% and 52% against a 46%
base), only both together (12%). The cause was our own wire serializer splitting one model turn into
two assistant messages, not the prompt rule — a prompt rewrite asking for the merged shape changed
nothing, because the model never made that choice. **The serializer fix was written, unit-tested and
then reverted unmeasured** when the direction changed; it is not in the shipped tree.

---

## 6. The transplant study (positive and negative directions)

Both directions on the captured decision-point request, 12-14 reps, round-robin, scored by how much
of OpenCode's own 40-150 band survives. These are real measurements; read them as "this piece is
enough to break a working context", **not** as "this is what is broken in ours".

### Positive: the fork's pieces added to OpenCode's working window

| variant | in 40-150 | lines/resp | cost to base |
|---|---|---|---|
| `oc-base` | **53/56 — 95%** | 806 | — |
| `oc+sufglob` — our listing, sorted, 265 B longer | 40/55 — 73% | 2,381 | **−22** |
| `oc+sufskills` — our 5,592 B skills + permissions | 38/54 — 70% | 2,500 | **−25** |
| `oc+sufagents` — AGENTS.md as a user turn | 29/67 — 43% | 5,611 | **−52** |
| `oc+sufprose` — our 77-character opening sentence | 10/44 — 23% | 4,816 | **−72** |
| `oc+counts` — the line-count listing *(control)* | **0/56 — 0%** | 8,000 | **−95** |

### Negative: OpenCode's pieces put into the fork's window

| variant | in 40-150 | lines/resp | against base |
|---|---|---|---|
| `suf-base` | 62/134 — 46% | 9,550 | — |
| **`suf-skills`** — drop our skills block | 59/129 — 46% | **4,972** | **−48%** |
| `suf+ocread` | 30/82 — 37% | 7,820 | −18% |
| `suf-noprose` | 45/115 — 39% | 9,695 | ±0 |
| `suf+ocprompt` | 20/106 — 19% | 11,767 | +23% |
| `suf+ocglob` | 31/116 — 27% | 12,468 | +31% |
| `suf+ocagents` | 25/128 — 20% | 12,888 | +35% |
| `suf-nocounts` | **0/82 — 0%** | 13,667 | +43% |
| `suf+octools` | 6/101 — 6% | 15,743 | +65% |

**Seven of eight repairs made the fork worse.** Only the skills block agrees with itself in both
directions. Both baselines behave like local equilibria: perturb either in any direction and it
degrades. That is the central methodological lesson of the whole study.

---

## 7. The probe study — what found it

Five batches, `e1`-`e5`, 3 reps each, round-robin, on the fork's real captured prefix at HEAD.

**E1 — the prompt, by section.** Base `suf+tools:oc` came in at 2,899 B/read, frac 0.45, limit 80,
130/132 bounded — i.e. our own prompt and layout, with skills off and OpenCode's tools, already
produce OpenCode's band. All six mutations made it worse. **The prompt was cleared.**

**E2 — which tool.** The whole OpenCode array drops B/read from 5,566 to 3,200 and the limit from
2000 to 80, but no single swap reproduces it. `drop:goals` reaches 0.80, `drop:exec_command` 0.81,
`tool:read=oc` is worse.

**E3 — tool outputs and the task text.** Uninterpretable: every arm sat at frac 1.0 and the control
inverted. Yielded two things anyway — the length-disclosure hypothesis died (see §4), and the model's
own words showed it picking round numbers, not computing.

**E4 — the tools we added, against the shell tool.**

| arm | B/read | frac | limit |
|---|---|---|---|
| `suf` | 6,000 | 1.0 | 240 |
| our `read`+`glob`+`grep` → OpenCode's | 4,603 | 1.0 | 180 |
| **`swap:exec_command=bash`** | **2,778** | **0.40** | **75** |
| `drop:goals+drop:exec_command` | 2,058 | 0.28 | 50 |

**E5 — inside the spec.** Description alone: 0.79 · 1.00 · 1.00, unstable. One parameter: nothing.
Dropping the tool: nothing. **It is the whole spec**, and the parameter list is the half the
description cannot carry.

---

## 8. What was applied

### The skills block, off

`[skills] include_instructions = false`. Config only — `include_skill_instructions` reads it with
`.unwrap_or(true)` (`core/src/config/mod.rs:3894`). 5,592 characters listing five skills — `imagegen`,
`openai-docs`, `plugin-creator`, `skill-creator`, `skill-installer` — none related to reading Rust
files. The only block that moved both transplant directions.

### The shell spec, lean

`shell_spec.rs` now offers `cmd`, `timeout_ms` and `workdir` under `approval_policy = never`, with a
description written in OpenCode's sections. Seven parameters are withheld from a run that cannot act
on them: `shell` and `login` (the environment's own shell is the right one), `tty` and
`yield_time_ms` (a resumable session needs a caller who will come back to it), `max_output_tokens`
(the cap belongs to the harness, and this model has twice been seen reading a per-command output cap
as its *context* budget), and `sandbox_permissions` / `justification` / `prefix_rule` (an escalation
nobody is present to approve).

**Nothing is lost.** Every field of `ExecCommandArgs` except `cmd` is `#[serde(default)]`
(`unified_exec.rs:30-53`), so a parameter left out of the schema takes its default and the handler is
untouched. `timeout_ms` was already deserialized and simply never advertised. Interactive runs keep
the full set. Commit `a65e71ff7`; locked by
`shell_spec_tests.rs::a_non_interactive_run_is_offered_three_parameters`.

### What it bought

| | rep40 | rep41 | previous best | typical | OpenCode |
|---|---|---|---|---|---|
| requests | 11 | 10 | 9 | 11-19 | 8-25 |
| reads bounded | 44/44 | 45/45 | 50/50 | 27/46 · 1/47 | 44/44 |
| median window | 37 [35-45] | 60 [50-80] | 220 | 500-2000 | 80 |
| peak context | 39,897 | 57,105 | 93,640 | 76k-111k | 47k-137k |
| compactions | 0 | 0 | 1 | 1-2 | 0 |
| citations | 44/44 valid | 44/44 valid | 44/44 | 44/44 | 44/44 |
| **cost** | **$0.0085** | **$0.0114** | $0.0136 | $0.0196-$0.0299 | $0.0086-$0.0265 |

Read context — peak minus the ~12,550-token fixed prefix — **27,347** and **44,555**, against
OpenCode's nine-run mean of **51,570** and median of 44,411.

Output quality held: 44 lines, 44 citations, **all in range**, on both sides.

### The model chooses the number itself

Nothing in `read.rs` clamps `limit`. Its reasoning, after:

> "44 files. I need to read each file enough to understand what it does and cite a line. Reading
> whole files could be heavy; use read with limit ~80 lines each to get headers/docs, plus grep for
> key structs. Some files may be large; the first hundred lines usually suffice."

Same model, same task, before:

> "44 files. Need read every .rs... We can read all maybe total size. Read 250 lines each maybe."

"The first hundred lines usually suffice" is our own sentence, from `read_spec.rs:67`. The guidance
was always there and always right; the ten-parameter spec was drowning it out. It also stopped
applying one flat number to a batch and started sizing per file — 50 for small test files, 60-70 for
mid, 80 for entry points — and in `rep41` it used `grep` on the nine largest files to pin exact
citation lines, which is the division of labour the tool descriptions ask for.

---

## 9. What is not settled

- **n = 2** on the binary. The two runs differ by 63% on read context and chose different strategies
  — `rep40` used no `grep` at all, `rep41` used nine.
- **Two changes rode together.** The lean spec and the permissions block going off landed in the same
  pair. Earlier runs reported `-NoPermissions` while silently dropping it: the key was emitted below
  a table header and TOML parsed it as a member of that table. Two runs were reported as testing
  something they were not.
- **Which third of the spec carries it** is unknown. Name, parameter list and description move
  together in the winning arm; description alone is unstable and one parameter is nothing. The arm
  that would separate them — our text with their parameters — was never run.
- **The goal tools** reach frac 0.80 on their own and are irrelevant to any reading task. Untouched.
- **The serializer split** (one model turn becoming two assistant messages) is measured and unfixed.

---

## 10. Cost

$0.81 on the probe study, $0.05 on the prose cut, $0.10 on the transplant study, and ~$0.12 on the
binary runs that checked them. About $1.10 in total, against $0.02-0.04 for a single binary pair.
