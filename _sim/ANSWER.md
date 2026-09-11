# What doubles the bill: one sentence in `read`'s description

> *"…and at most about 32000 bytes: `limit` counts lines but the ceiling is bytes, so a file of long
> lines stops earlier than the line count suggests."*
>
> — `codex-rs/core/src/tools/handlers/read_spec.rs`, added by `baeb79fc8`

Remove it and this model reads like OpenCode. Leave it and it reads whole files.

## The chain, measured end to end

**1. The 2× is bytes, not requests.** From the real runs' own meter:

| | requests | fresh tokens | cached | peak | cost |
|---|---|---|---|---|---|
| `wire-stock-rep2` (OpenCode) | 10 | 52,658 | 284,672 | 56,961 | $0.0098 |
| `wire-sufficefork-rep4` (fork) | 11 | **148,187** | 379,328 | 111,090 | **$0.0196** |

73% of the difference is fresh tokens — content entering the window for the first time. That content
is the source it read: 122 KB against 479 KB, **2,779 bytes a file against 9,398**.

**2. It is an input, and the simulator reproduces it.** Same loop, same corpus, same meter, read
phase run out over eight steps:

| | B/read | cost |
|---|---|---|
| OpenCode, real | 2,779 | $0.0086–0.0109 |
| OpenCode, simulated | 2,160–3,264 | $0.0037–0.0060 |
| the fork, real | 9,398 | $0.0196 |
| the fork, simulated | 6,502–9,706 | $0.0121–0.0178 |

**3. It is the `read` spec, not the prompt, layout, or the rest of the tool surface.** OpenCode
whole, plus exactly one of the fork's pieces:

| piece added to OpenCode | B/read, per rep |
|---|---|
| *(baseline, `f-walk` / `f-sorted`)* | 2,840 · 2,566 · 3,264 · 2,160 |
| the fork's prompt | 3,601 · 2,906 |
| the fork's message layout (skills + permissions + AGENTS.md as a user turn) | 3,798 · 3,887 |
| **the fork's `read` spec + envelope** | **9,102** · 4,020 |
| **the fork's twelve tool specs** | **10,731** · 2,729 |
| the fork's twelve tool specs, with OpenCode's `read` put back | 4,011 · 3,336 |

The two that spike both carry the fork's `read`; the two that do not, do not. Putting OpenCode's
`read` back into the fork's own tool surface removes the spike.

**4. Within the spec, it is that one sentence.** Everything else identical:

| | n | reps over 6,000 B/read | mean B/read | mean cost |
|---|---|---|---|---|
| carries the sentence | 8 | **5** | **6,939** | $0.0085 |
| does not | 13 | **0** | **3,184** | $0.0054 |

One-sided Fisher exact **p = 0.0028**. Ratio **2.18× the bytes**.

`f-p-readspec` with the sentence: 3,793 and 8,935. The same arm with the sentence deleted: 2,820,
3,338, 2,868 — and one of those three read all 44 files at `limit: 80`, which is precisely what
OpenCode does.

## Why it does this

The sentence tells the model a byte ceiling will stop the read for it. So it stops setting `limit`
itself, and takes the documented 2,000-line default.

That is the same mechanism as everything else found on this task. `exec_command`'s
`max_output_tokens` is described as an *"Output token budget"*, and the model read that as its
context budget — *"we have token budget 19k only"* — and rationed hard, down to 110-line windows.
Told a budget exists, it rations; told a ceiling is enforced for it, it stops. **This model sizes
its reads against whatever the tool surface says about limits, and both of our sentences say
something other than what they mean.**

The sentence was added in good faith: `baeb79fc8` says it is there so the model is not "made to
discover [the ceiling] by being cut". It costs about 2×.

## The fix

Delete the clause from `read_spec.rs`. The ceiling stays — it just stops being advertised, exactly
as OpenCode does not advertise its own 50 KB cap. A read that is cut still says so in its own
output, which is where the model can act on it.

Not yet verified in the binary: one line, one build, one run against `wire-sufficefork-rep4`
(11 requests, 479 KB, $0.0196, 44/44 citations).

## Verified in the binary — partly

The clause was removed from `read_spec.rs`, rebuilt, and run twice against
`wire-sufficefork-rep4`, the only fork run that had completed on the pre-fix tree.

| | rep4 (before) | rep6 (after) | rep7 (after) |
|---|---|---|---|
| requests | 11 | 13 | 12 |
| **fresh tokens** | 148,187 | **76,965** | 141,106 |
| cached | 379,328 | 569,664 | 347,136 |
| peak context | 111,090 | **76,458** | **79,800** |
| compactions | **1** | **0** | **0** |
| corpus into context | 484 KB | **224 KB** | 413 KB |
| `read` calls | 51 | 46 | 44 |
| **calls carrying a `limit`** | **32/51** | **46/46** | **44/44** |
| median window | 500 | **140** | **2000** |
| bytes per read | 9,398 | **4,719** | 9,281 |
| citations | 44/44 | 44/44 | 44/44 |
| wall time | 419 s | 218 s | 234 s |
| **cost** | **$0.0196** | **$0.0158** | **$0.0180** |

**What the fix did.** Every read is now bounded — 90 of 90 across both runs, against 32 of 51
before. Neither run compacted, where the pre-fix run did. Peak context fell by a third in both, and
wall time by almost half. Mean cost $0.0169 against $0.0196, **a 14% drop**.

**What it did not do.** rep7 wrote `limit: 2000` explicitly on 24 of its 44 calls. Removing the
sentence stopped the model *omitting* the limit and started it *stating the default* — the same
number, arrived at differently — so that run pulled 413 KB and cost $0.0180, no better than before.
rep6 did what the simulator predicted, median 140 and 224 KB. Two runs, two different behaviours.

**So the clause is real but not sufficient.** In the simulator it was isolated against OpenCode's
prefix, where it was the only fork-shaped thing present and removing it worked 3/3. In the binary
the rest of the fork is there too, and the effects are additive: `f-suf` (everything) read 6,502 and
9,706 where `f-p-tools-ocread` (the fork's tools with OpenCode's `read`) read 4,011 and 3,336. The
clause was the largest single contributor found, not the whole of it.

**Next lever, from these two runs specifically:** the documented `2000` default itself, which
survives in both the usage line and the `limit` parameter description. OpenCode documents 2000 too
and its model still picks 80, so the number alone is not the problem — but with the fork's prompt
and tool surface around it, it is what rep7 reached for.

n = 2, against a recorded fork spread of $0.0073–$0.0301 on configurations that differ by nothing.
Treat the 14% as a direction, not a measurement.

## Cost of finding it

$0.25 across roughly 140 simulator reps, against $0.13 for five binary runs that identified nothing
and two of which stalled before producing output.

## Caveats

- Eight steps, not a whole task: the read phase is run out, the write and verify steps are not.
- The simulator's absolute costs sit below the real runs' because of that; the ratios are what carry.
- `tools.py` re-implements the tools. `replay.py` serves the real recorded results where the call
  matches, which is how the ordering artefact was caught — but reads that the real run did not make
  still come from the re-implementation.
