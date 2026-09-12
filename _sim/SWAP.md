# Transplanting one block at a time, both directions

The rule: put one of the fork's blocks into OpenCode's decision-point request — if the lines go up,
that block is implicated. Then put OpenCode's version of the same block into the fork's — if the
lines go down, it is confirmed. A block that moves one way is a suspect; both ways is the answer.

Both hosts are captured bodies (`003.json` from `wire-stock-rep11` and `wire-sufficefork-rep22`),
validated in `FEED.md` against what each binary really received. All variants run **round-robin**,
one rep of each in turn, because running them in blocks put a fake 29% → 93% on the first thing
measured.

10 reps each. `lines/resp` = reads × window, an unbounded read counted at the documented 2000.

| host | variant | reads/resp | limit median | bounded | **lines/resp** |
|---|---|---|---|---|---|
| OpenCode | `oc-base` | 4.0 | 120 | 30/41 — 73% | **2,514** |
| OpenCode | `oc+suftools` | 4.5 | 80 | **63/63 — 100%** | **546** |
| OpenCode | `oc+sufread` | 4.5 | 80 | **50/50 — 100%** | **392** |
| OpenCode | `oc+sufprompt` | 4.5 | 60 | **57/58 — 98%** | **628** |
| Suffice | `suf-base` | **12.5** | 160 | 45/103 — 44% | **12,352** |
| Suffice | `suf+octools` | 9.5 | 180 | 21/76 — 28% | 11,462 |
| Suffice | `suf+ocread` | 9.0 | 120 | 26/80 — 33% | 11,280 |
| Suffice | `suf+ocprompt` | **6.0** | 140 | 15/60 — 25% | **9,261** |

## What it says, and it is consistent in both directions

**The fork's reading text is not the problem. It is the best thing in either agent.**

Put the fork's `read` spec into OpenCode and OpenCode bounds **50 of 50** reads, against 30 of 41 on
its own — 2,514 lines a response down to **392**, a sixfold improvement. Its whole tool array does
the same (63/63). Its prompt does the same (57/58).

Take them out of the fork and the fork gets worse: 44% bounded down to 33% with OpenCode's `read`
spec, down to 25% with OpenCode's prompt.

Both directions agree. Three changes were made to that text over this investigation on the theory
that it was causing the cost. It was holding the line.

**What the prompt does control is batch size.** Swapping OpenCode's prompt into the fork halves the
reads per response, 12.5 → 6.0, and that is the single largest improvement measured on the fork
host: 12,352 lines a response down to 9,261. The same swap in the other direction barely moves
OpenCode, 4.0 → 4.5, so this is not symmetric and is not settled.

## What no longer stands

`parallel_tool_calls` does nothing. Measured round-robin, both directions: OpenCode 61% bounded
against 60% with it added, the fork 58% against 53% with it removed. The 29% → 93% recorded for it
in the first, block-ordered pass was an artefact of running order.

## What is still unexplained

The fork's base is 12,352 lines a response against OpenCode's 2,514 — **4.9×** — and no single block
transplant closes more than a quarter of it. The three blocks tested account for:

- the tool array: 12,352 → 11,462 (−7%)
- the `read` spec alone: → 11,280 (−9%)
- the prompt: → 9,261 (−25%)

Which leaves most of the gap in what was not transplanted: the fork's conversation is **9 messages
against OpenCode's 4** at the same decision point — an extra assistant turn carrying 77 characters of
prose, an extra `exec_command` call and its 2,024-character result, AGENTS.md as a user turn rather
than inside the system message, and the skills block as a second system message. Those are shape
changes rather than block substitutions, which is why they were left out of this pass and why they
are what the next one should move.

## Cost

$0.056 for 18 variant-runs at 10–15 reps. A conversation arm was six to fifteen cents for one
sample.
