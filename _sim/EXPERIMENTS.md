# The transplant experiments, in full

Consolidated record of the two-direction block-transplant study. `BAND.md` and `REVERSE.md` hold the
individual write-ups; this is the method, both tables, and what each number does and does not
support.

## Why it was built this way

Running the two binaries moves every variable at once and costs six to twelve requests before it
says anything; five such runs identified nothing, and two of them stalled. Worse, the baseline
itself moved: the same `opencode.exe`, same config, same task, one day apart went from 8 requests
and $0.0086 to 25 requests and $0.0265 (`BASELINE-MOVED.md`).

So the unit of measurement became **one request, replayed**.

- The request is a **captured body** — the bytes an agent really sent — never a reconstruction.
  Reconstructions were tried and one of them differed in the `glob` result's ordering alone, which
  was enough to change the answer.
- The **known-correct answer** is read out of the *next* captured body, which carries the reply as
  its new assistant message. Same source, either agent, nothing rebuilt.
- It is validated: replaying OpenCode's real body 003 reproduces its own run's bounded rate (53%
  against 57%), and the fork's reproduces its own (54% against 49%). `FEED.md`.

## The decision point

Both agents reach the same situation — file listing in hand, nothing read yet — and this is the
request at which the model first writes a `limit`:

```
OpenCode  body 003   system + task + glob call + glob result                       -> read
Suffice   body 003   system + skills + AGENTS + task + prose + glob + result
                     + exec_command call + line-count result                       -> read
```

## The metric

**How much of OpenCode's own band survives.** All 132 reads across its six recorded runs fell in
40–150, centred on 80. So each variant is scored by the share of its reads landing in that band,
alongside lines per response — reads × window, an unbounded read counted at the documented 2000
default, because that is what it actually costs.

## Two rules learned the hard way

**Round-robin, never in blocks.** Running variants one after another put a false 29% → 93% on
`parallel_tool_calls`, which vanished when the arms were interleaved. Something drifts inside a
session; interleaving spreads it over every arm instead of dumping it on whichever ran first.

**A positive control in every batch.** `oc+counts` was known to destroy bounding, so it rides along:
if it fails to, the batch is not measuring anything. It reproduced exactly — 0/74 in one run, 0/56 in
another.

---

## Forward: the fork's pieces added to OpenCode's working window

14 reps each.

| variant | reads | limit | bounded | **in 40–150** | lines/resp | cost to base |
|---|---|---|---|---|---|---|
| `oc-base` | 4.0 | 100 | 53/56 | **53/56 — 95%** | 806 | — |
| `oc+sufglob` — our listing, sorted, 265 B longer | 4.0 | 80 | 40/55 | 40/55 — 73% | 2,381 | **−22** |
| `oc+sufskills` — our 5,592 B skills + permissions | 4.0 | 80 | 38/54 | 38/54 — 70% | 2,500 | **−25** |
| `oc+sufagents` — AGENTS.md as a user turn | 4.0 | 80 | 29/67 | 29/67 — 43% | 5,611 | **−52** |
| `oc+sufprose` — our 77-character opening sentence | 4.0 | 120 | 11/44 | 10/44 — 23% | 4,816 | **−72** |
| `oc+counts` — the line-count listing *(control)* | 5.0 | — | 0/56 | **0/56 — 0%** | 8,000 | **−95** |

All five damage it, the ordering is clean, and the control reproduced.

## Reverse: OpenCode's pieces put into the fork's window

12 reps each.

| variant | reads | limit | bounded | in 40–150 | **lines/resp** | against base |
|---|---|---|---|---|---|---|
| `suf-base` | 12.5 | 120 | 83/134 — 62% | 62/134 — 46% | 9,550 | — |
| **`suf-skills`** — drop our skills block | 10.5 | 140 | **108/129 — 84%** | 59/129 — 46% | **4,972** | **−48%** |
| `suf+ocread` | 8.0 | 120 | 50/82 — 61% | 30/82 — 37% | 7,820 | −18% |
| `suf-noprose` — drop our sentence | 8.5 | 120 | 61/115 — 53% | 45/115 — 39% | 9,695 | ±0 |
| `suf+ocprompt` | 9.0 | 161 | 45/106 — 42% | 20/106 — 19% | 11,767 | +23% |
| `suf+ocglob` | 11.5 | 140 | 45/116 — 39% | 31/116 — 27% | 12,468 | +31% |
| `suf+ocagents` — AGENTS.md into the system message | 11.5 | 175 | 56/128 — 44% | 25/128 — 20% | 12,888 | +35% |
| `suf-nocounts` — drop the line-count step | 10.0 | **2000** | 23/82 — 28% | **0/82 — 0%** | 13,667 | +43% |
| `suf+octools` | 9.0 | 140 | **7/101 — 7%** | 6/101 — 6% | **15,743** | **+65%** |

Seven of eight made it worse or left it unchanged.

## Where the two directions meet

| piece | forward | reverse | verdict |
|---|---|---|---|
| **skills block** | **−25 points** | **bounded 62% → 84%, lines halved** | **confirmed both ways** |
| line counts | −95 points | removing is worse | one direction |
| opening sentence | −72 points | removing changes nothing | one direction |
| AGENTS.md placement | −52 points | fixing is worse | one direction |
| glob listing | −22 points | fixing is worse | one direction |
| `read` spec | improves OpenCode | improves the fork slightly | not a cause |
| tool array | improves OpenCode | **badly** worse in the fork | not a cause |
| `parallel_tool_calls` | nothing | nothing | cleared |

## What this supports, and what it does not

**Supported.** The skills block costs money in both directions. It is 5,592 characters, sent as its
own system message on every request, listing five skills — `imagegen`, `openai-docs`,
`plugin-creator`, `skill-creator`, `skill-installer` — none related to reading Rust files, plus a
permissions block. `config/defaults.toml` already carries `[skills] include_instructions`.

**Supported.** Each of the five forward pieces is *sufficient* to break a context that was working.
The line-count listing is the strongest: it takes a 95% baseline to zero.

**Not supported.** That any of them is *what is broken in ours*. The reverse direction says
otherwise for four of the five, and seven of eight single-piece repairs made the fork worse. Both
baselines behave like local equilibria — perturb either in any direction and it degrades.

**Not supported.** Independence. The forward multiplication landed on 5% against a measured 5%,
which reads as "jointly sufficient to reach the floor" rather than "five separable additive causes".

## Applied: the skills block turned off in the binary

`include_skill_instructions` is read from `[skills] include_instructions` and defaults to true
(`core/src/config/mod.rs:3894`), so this is a config change and needs no rebuild.
`run-suffice.ps1 -NoSkills` writes it.

One interleaved pair, same session, 2026-09-12:

| | fork, skills **on** (rep20 / 21 / 22) | **fork, skills off (rep30)** | OpenCode, same session |
|---|---|---|---|
| requests | 19 / 11 / 19 | **9** | 9 |
| reads bounded | 27/46 · 1/47 · 26/53 | **50/50 — 100%** | 44/44 |
| median window | 500 · 700 · 2000 | **220** [80–320] | 80 |
| measuring step | 2–6 `exec_command` | **none** | none |
| citations | 44/44 | 44/44 | 44/44 |
| **cost** | $0.0244 · $0.0196 · $0.0299 | **$0.0136** | $0.0086 |

The read shape changed in kind, not degree: every one of fifty reads carried a window, none took the
2000 default, and the model never went measuring. That is the first change in this investigation to
move the binary the way the simulator predicted.

The gap to OpenCode in that pair is 1.58×, against 1.30× measured on the earlier interleaved pairs —
but both absolute numbers moved too, so pair-to-pair ratios are not comparable across sessions. What
is comparable is the fork against itself: $0.0136 against a previous best of $0.0158 and a typical
$0.0196–$0.0299.

**n = 1.** A second pair was started and stopped for budget; its OpenCode half came in at $0.0259,
which is a reminder of how wide the session-to-session spread still is.

One caveat on fidelity: the config flag removes `<skills_instructions>` only. `<permissions
instructions>` rides the same system message under a separate flag and stays. The simulator's
`suf-skills` variant dropped both, so this applies roughly nine tenths of what was measured.

## Method limits worth carrying forward

- 12–14 reps on a bimodal outcome. Differences under roughly 15 points are noise; the sessions
  recorded the same body answering anywhere from 29% to 95%.
- One captured host per direction. Nothing was replicated on a second body.
- The metric counts an unbounded read at 2000 because that is the documented default, so a single
  flip moves `lines/resp` by 25×. Read the band column first and the lines column second.

## Cost

$0.029 forward, $0.052 reverse, $0.021 for the tool-choice study in `MOTIVE.md`. Against six to
fifteen cents for a single sample from a conversation arm.
