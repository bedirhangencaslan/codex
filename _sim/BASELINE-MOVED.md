# The baseline moved

The whole investigation compared the fork against a number. That number is from 2026-09-11, and it
no longer reproduces.

## Same binary, same config, same task, one day apart

| run | date | requests | reads per message | reads carrying a `limit` | peak | cost |
|---|---|---|---|---|---|---|
| `wire-stock-rep1` | 09-11 | 8 | 13, 17, 14 | **44/44** | 52,602 | **$0.0086** |
| `wire-stock-rep2` | 09-11 | 10 | 12, 8, 8, 8, 8 | 44/44 | 56,961 | $0.0098 |
| `wire-stock-rep3` | 09-11 | 15 | 5, 5, 3, 5, … | 44/44 | 47,345 | $0.0109 |
| **`wire-stock-rep11`** | **09-12** | **25** | **1, 2, 3, 3, 3, 1, 1, 2, 1, …** | **25/44** | 83,245 | **$0.0265** |

Nothing on this side changed: the same `opencode.exe` 1.18.29, the same generated `opencode.json`,
the same seed corpus, the same task text, the same relay, the same model id, the same
`reasoning_effort`. Both produced `WIRE.md` with 44 valid citations.

**OpenCode today costs $0.0265.** The fork's best measured configuration, `wire-sufficefork-rep8`,
cost **$0.0159**.

## What broke, precisely

Two behaviours, together:

1. **Batching collapsed.** 13–17 `read` calls in one assistant message became 1–4. That alone turns
   8 requests into 25, and every request re-sends the whole conversation — which is why cached
   tokens went 182,976 → 1,216,896 while fresh only went 59,056 → 90,052.
2. **Bounding became unreliable.** 44 of 44 reads carried an explicit window; now 25 of 44 do.

Both match what the single-request replay found and could not explain. Sending OpenCode's real
turn-two request back at the provider — byte-identical body, identical parameters, identical tool
specs, and finally identical transport headers — bounded every read in only 15 of 52 responses:

| variant | responses fully bounded | reads bounded |
|---|---|---|
| `content: null` | 5/18 | 53/108 |
| `content: ""` (what OpenCode actually sends) | 4/17 | 35/86 |
| plus `User-Agent`, `x-session-id`, `x-session-affinity` | 6/17 | 50/102 |
| **pooled** | **15/52 — 29%** | **138/296 — 47%** |

That 29% was read as proof that something outside the request differed between the agents. It is
simpler than that: **29% is what the model does now, and the real agent does it too.** The replay
was not failing to reproduce OpenCode; it was reproducing today's OpenCode correctly, against an
archived number from yesterday's.

## What this invalidates

Every comparison in this investigation that set a fork run against OpenCode's $0.0086–$0.0109. The
control was not held. In particular:

- the "2× cost gap" that framed the whole effort — today it is 0.6×, the other way
- the `read` spec finding, the prompt/layout interaction, the piece sweep: all measured fork-side
  changes against a fixed archived baseline while the provider moved underneath
- the simulator's inability to batch 13–17 reads, treated for a day as a bug in the loop

The fork-side measurements against *each other* are unaffected — they were run within hours of one
another. What cannot be trusted is any statement of the form "still 2× OpenCode".

## The rule this earns

**Interleave.** A baseline is not a number in a file; it is a run taken beside the run it is
compared to. From here, every fork measurement gets a stock OpenCode run in the same session,
alternating, and the comparison is between those two — never against the archive.

The archive keeps its value as a record of *what was once possible*: 8 requests, 13–17 reads a
message, 44/44 bounded, $0.0086. That configuration existed. It is simply not what either agent gets
from the provider today.
