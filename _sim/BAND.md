# It is not one thing. It is five, and they multiply.

## The design

Take the exact request OpenCode answered with `limit: 80` — its captured body 003, everything up to
its first read: system message, task, `glob` call, `glob` result, ten tool specs, seven parameters.
Patch **one** of the fork's corresponding pieces into that window. Ask how much of OpenCode's own
band survives — 40–150, where all 132 of its recorded reads landed.

14 reps each, round-robin. `oc+counts` is the positive control: the line-count listing was already
shown to destroy bounding, so if it does not destroy it here the rig is broken.

## The result

| variant | in 40–150 | lines per response |
|---|---|---|
| `oc-base` | **53/56 — 95%** | 806 |
| `oc+sufglob` — our file listing, sorted and 265 bytes longer | 40/55 — 73% | 2,381 |
| `oc+sufskills` — our skills + permissions block | 38/54 — 70% | 2,500 |
| `oc+sufagents` — AGENTS.md moved out of system into a user turn | 29/67 — 43% | 5,611 |
| `oc+sufprose` — our opening sentence, as its own assistant turn | **10/44 — 23%** | 4,816 |
| `oc+counts` — the line-count listing *(control)* | **0/56 — 0%** | 8,000 |

The control reproduced exactly. The baseline held at 95%.

## Why every single fix failed

Five pieces damage it, and the fork carries all of them at once. Multiply the four that are not the
line-count step:

```
0.73 × 0.70 × 0.43 × 0.23 ≈ 0.05
```

Measured on the fork's own pure path — `suf-nocounts`, the same body with the line-count turn
removed — **5/94 = 5%**. In a real run that never measured, `wire-sufficefork-rep21`: **1/47 = 2%**.

So four stacking factors account for the fork's collapse on the plain glob → read path, and the
line-count step — which happens about a third of the time — finishes off whatever is left.

That is why the 32,000-byte clause, moving the window sentence into `read`'s description, and
stripping the prompt's reading bullets each changed so little. Every one of them touched a single
factor while the other four stayed in place.

## The surprise

`oc+sufprose` is **77 characters**:

> I'll inventory the Rust files, then read each one and produce only `WIRE.md`.

The fork says a sentence before it calls anything; OpenCode calls the tool directly. Adding that one
turn takes the band from 95% to 23% — second only to the line-count listing, and far ahead of the
5,592-byte skills block or the 23,958-byte AGENTS.md.

It is faithful to the fork, which really does send two consecutive assistant messages at this point:
one carrying prose, one carrying the `glob` call. Whether the damage comes from the sentence's
content or from the doubled assistant turn is not separated here, and that is the next cut.

## Order of work, by measured damage

1. **The line-count step** — stop the model measuring files before reading them. Worth 95 points
   when it happens, and it happens about a third of the time.
2. **The opening sentence** — 72 points. Cheapest thing on this list to change.
3. **AGENTS.md placement** — 52 points. Move it into the system message, as OpenCode does.
4. **The skills block** — 25 points.
5. **The glob listing** — 22 points. `search.rs:176` sorts where OpenCode streams walk order.

## Caveats

- 14 reps a variant on a bimodal outcome. The ranking is robust at the top and thin between
  `sufskills` (70%) and `sufglob` (73%).
- One direction only. Each piece was added to OpenCode; none was removed from the fork in this pass,
  and the earlier attempts at that direction did not confirm.
- The multiplication is a check that the numbers are consistent, not a proof that the factors are
  independent.

## Cost

$0.029 for 84 single-turn replays.
