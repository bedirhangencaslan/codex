# The reverse direction: single-piece transplants do not fix the fork

`BAND.md` took OpenCode's 80-producing window and added the fork's pieces one at a time. This does
the mirror: the fork's own decision-point request, with OpenCode's version of each piece put in.
Same metric — how much of OpenCode's 40–150 band survives — 12 reps each, round-robin.

| variant | in 40–150 | lines per response | against base |
|---|---|---|---|
| `suf-base` | 62/134 — 46% | 9,550 | — |
| **`suf-skills`** — drop our skills + permissions block | 59/129 — 46% | **4,972** | **−48%** |
| `suf+ocread` — OpenCode's `read` spec | 30/82 — 37% | 7,820 | −18% |
| `suf-noprose` — drop our opening sentence | 45/115 — 39% | 9,695 | ±0 |
| `suf+ocprompt` — OpenCode's prompt | 20/106 — 19% | 11,767 | +23% |
| `suf+ocglob` — OpenCode's file listing | 31/116 — 27% | 12,468 | +31% |
| `suf+ocagents` — AGENTS.md into the system message | 25/128 — 20% | 12,888 | +35% |
| `suf-nocounts` — drop the line-count step | 0/82 — 0% | 13,667 | +43% |
| `suf+octools` — OpenCode's ten tools | 6/101 — 6% | 15,743 | +65% |

## The two directions disagree

| piece | forward: added to OpenCode | reverse: fixed in the fork |
|---|---|---|
| line counts | −95 points, the worst | removing them is **worse** |
| opening sentence | −72 points | removing it changes nothing |
| AGENTS.md placement | −52 points | fixing it is **worse** |
| glob listing | −22 points | fixing it is **worse** |
| **skills block** | **−25 points** | **removing it halves the lines** ✓ |

Only the skills block agrees with itself. Everything else moves one way when added to OpenCode and
the same way — or nowhere — when removed from the fork.

## What that means

**Single-piece transplants do not fix this.** Seven of eight made the fork worse or left it
unchanged. Both baselines behave like local equilibria: perturb either one in any direction and it
degrades. The forward results in `BAND.md` should therefore be read as "this piece is enough to
break a working context", not as "this piece is what is broken in ours".

It also puts a limit on the method. A one-at-a-time transplant can show sufficiency and cannot show
necessity when the factors interact, and here they plainly do — the multiplication in `BAND.md` came
out at 5% against a measured 5%, which says the four factors are not independent so much as jointly
sufficient to reach the floor.

## The one thing to act on

**The skills block.** 5,592 characters, sent as a second system message, listing five skills
(`imagegen`, `openai-docs`, `plugin-creator`, `skill-creator`, `skill-installer`) none of which has
anything to do with reading Rust files, plus a permissions block. Removing it:

- takes bounded reads from 62% to **84%**
- halves lines per response, 9,550 → **4,972**
- and costs 25 points when added to OpenCode, which is the same answer from the other side

`config/defaults.toml` already has `[skills] include_instructions`; the archive's uncommitted patch
set it to `false` and that hunk was never applied. This is the measurement that justifies it.

## Caveats

- 12 reps a variant, bimodal outcome. The `suf-skills` improvement is a 2× move on the lines metric
  and 22 points on the bounded rate, which is above the noise seen elsewhere in this file; the
  ordering among the middle rows is not.
- One host each. Nothing here was replicated on a second captured body.

## Cost

$0.052 for 108 single-turn replays.
