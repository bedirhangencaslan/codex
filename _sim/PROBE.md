# The shell tool's spec is what sets the read window

## What was wrong with every earlier rig

`swap.py` replays **one** request and scores the `limit` values in the reply. `sim.py` closes the
loop but stops at the first `read`. Both measure an opening move, and two real runs showed that is
not the question:

| run | batching | reads bounded | median window | cost |
|---|---|---|---|---|
| `rep30` fork, skills off | 12-14 per message | 50/50 | 220 | $0.0136 |
| `rep33` fork, parallel removed | 1 per message | 46/46 | **280** | $0.0394 |
| OpenCode `rep30` | 2-3 per message | 44/44 | **80** | $0.0086 |

Serialising fixed *whether* a window is set — 46/46 — and left *how wide* untouched, at 280 against
80. The width is what costs, and it was never the thing being measured.

`probe.py` runs the read phase out against the real corpus with real tools and reports **bytes
returned per read** and **window ÷ file length**. It saves `reasoning_content`, which every earlier
batch discarded, and it interleaves arms round-robin with a dollar budget that aborts the batch.

## What was ruled out, cheaply

**Batch size.** 47 runs: corr(batch, limit) = **+0.06**, corr(batch, bytes/read) = **−0.27**. If
anything, larger batches read *less*. The best rows in the table are batch 8 at limit 65; the worst
are batch 2 at limit 800.

**The prompt.** Holding the fork's layout and OpenCode's tools fixed, replacing each section of
`instructions_template` with OpenCode's — or dropping it — made every arm worse than leaving it
alone, OpenCode's whole prompt worst of all (frac 1.0 against the base's 0.45). The earlier
"prompt × layout interaction" was measured with the skills block still on; with it off the prompt is
harmless.

**Knowledge of file length.** With the footer saying nothing at all about length, the model still
asks for "100-150" and then "140". And the decisive tally from OpenCode's own 44-read run: `offset`
was passed **zero times**, despite 25 footers naming totals up to 3,083. Both agents get the same
information and behave oppositely, so the information is not the cause.

**The tools we added.** `read`, `glob` and `grep` are ours (`d829b5f25`, `03bab7aa3`). Swapping all
three for OpenCode's left the model reading whole files. Swapping only `read`'s spec made it worse.

## What it is

`exec_command`'s spec — which is codex's own tool, and rides the fixed prefix of every request
whether or not it is ever called.

| arm | bytes/read | frac | limit | reps (frac) |
|---|---|---|---|---|
| `suf` | 5,186 | 0.85 | 220 | 1.00 · 0.99 · 0.65 |
| **`suf+swap:exec_command=bash`** | **2,873** | **0.45** | **80** | **0.45 · 0.44 · 0.42** |
| `suf+desc:exec_command=bash` | 4,835 | 1.0 | 170 | 0.79 · 1.00 · 1.00 |
| `suf+param:-max_output_tokens` | 5,566 | 1.0 | 200 | 1.00 · 1.00 · 0.97 |
| `suf+drop:exec_command` | 3,596 | 0.71 | 120 | 1.00 · — · 0.67 |

Six replays across two independent batches (E4, E5) land in **0.39-0.45** against a base at
0.85-1.00, with no overlap — the tightest effect in this archive. Nothing smaller reproduces it:
the description alone is unstable, one parameter is nothing, and removing the tool entirely is
nothing. It is the whole spec, and the parameter list is the half the description cannot carry.

Not the shell. Both tools run PowerShell (`exec-server/src/client.rs:2060`); the fork's own capture
shows `exec_command` executing `Get-ChildItem`. What differs is how the tool is described:
ours 1,814 characters and **ten** parameters, OpenCode's 5,265 and **three**.

## Applied

`shell_spec.rs` now offers `cmd`, `timeout_ms` and `workdir` under `approval_policy = never`, with a
description written in OpenCode's sections. Seven parameters are withheld from a run that cannot act
on them, and nothing is lost: every field of `ExecCommandArgs` except `cmd` is `#[serde(default)]`,
so a parameter left out of the schema takes its default and the handler is untouched. `timeout_ms`
was already deserialized and simply never advertised. Interactive runs keep the full set.

Two runs, `-NoSkills -NoPermissions`:

| | rep40 | rep41 | previous best | typical |
|---|---|---|---|---|
| requests | 11 | 10 | 9 | 11-19 |
| reads bounded | 44/44 | 45/45 | 50/50 | 27/46 · 1/47 |
| median window | 37 [35-45] | 60 [50-80] | 220 | 500-2000 |
| peak context | 39,897 | 57,105 | 93,640 | 76k-111k |
| compactions | 0 | 0 | 1 | 1-2 |
| citations | 44/44 valid | 44/44 valid | 44/44 | 44/44 |
| **cost** | **$0.0085** | **$0.0114** | $0.0136 | $0.0196-$0.0299 |

Read context, peak minus the ~12,550-token fixed prefix: **27,347** and **44,555** against
OpenCode's nine-run mean of **51,570** and median of 44,411.

The model chooses the number itself — nothing in `read.rs` clamps `limit` — and it now says why:

> "44 files. I need to read each file enough to understand what it does and cite a line. Reading
> whole files could be heavy; use read with limit ~80 lines each to get headers/docs, plus grep for
> key structs. Some files may be large; the first hundred lines usually suffice."

against the same model, same task, before the change:

> "44 files. Need read every .rs... We can read all maybe total size. Read 250 lines each maybe."

"The first hundred lines usually suffice" is our own sentence, from `read_spec.rs:67`. The guidance
was always there and always right; the ten-parameter spec was drowning it out.

## Caveats

- **n = 2** on the binary. The two runs differ by 63% on read context (27k against 44k), and they
  chose different strategies — rep40 used no `grep` at all, rep41 used nine.
- **Two changes rode together.** The lean spec and the permissions block going off landed in the
  same pair; the earlier runs reported `-NoPermissions` while silently dropping it, because the
  key was emitted below a table header and parsed as a member of that table.
- The positive control inverted on this base: `+counts`, which takes OpenCode from 95% to 0%, does
  nothing to the fork — a third confirmation of `REVERSE.md`. A base already at the floor needs a
  *lifting* control, and `tools:oc` proved bimodal, so E4 and E5 rest on within-arm tightness and on
  two independent arms moving together, not on a control.

## Cost

$0.81 over five batches (E1-E5), plus $0.12 on four binary runs.
