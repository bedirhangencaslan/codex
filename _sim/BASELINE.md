# Baseline — every measurement taken before the fix

The record as it stood on 2026-09-11, before the clause identified in `ANSWER.md` was removed from
`read_spec.rs`. Kept so any later claim of improvement has something to be measured against.

Task throughout is `wire`: read every `.rs` file under `codex-api/src/` (44 files, ~13,000 lines)
and write `WIRE.md` with one `file:line` citation each. One metering relay
(`measurement/relay.py`) in front of Z.ai for every agent, reading the provider's own `usage`.
Pricing $0.075 fresh / $0.015 cached / $0.25 output per million.

## Real binary runs

| run | req | err | fresh | cached | peak | cites | cost |
|---|---|---|---|---|---|---|---|
| `wire-stock-rep1` (OpenCode) | 8 | 0 | 59,056 | 182,976 | 52,602 | 44 | $0.0086 |
| `wire-stock-rep2` | 10 | 0 | 52,658 | 284,672 | 56,961 | 44 | $0.0098 |
| `wire-stock-rep3` | 15 | 0 | 53,910 | 369,792 | 47,345 | 44 | $0.0109 |
| `wire-suffice-rep1` (OpenCode + our prompt) | 10 | 0 | 62,518 | 266,880 | 52,616 | 44 | $0.0103 |
| `wire-suffice-rep2` | 9 | 0 | 54,817 | 226,880 | 53,578 | 44 | $0.0089 |
| `wire-suffice-rep3` | 11 | 0 | 59,929 | 298,112 | 54,080 | 44 | $0.0106 |
| `wire-sufficefork-rep1` (fork, HEAD) | 16 | 1 | 63,045 | 596,736 | 62,545 | 44 | $0.0150 |
| `wire-sufficefork-rep2` (fork + glob/grep) | 8 | 0 | 46,336 | 172,352 | 40,355 | 44 | $0.0073 |
| `wire-sufficefork-rep3` (same) | 16 | 2 | 304,580 | 461,632 | 76,065 | **0** | $0.0301 |
| `wire-sufficefork-rep4` (merged + imitations) | 11 | 1 | 148,187 | 379,328 | 111,090 | 44 | $0.0196 |
| `wire-sufficefork-rep5` (same) | 9 | 2 | 116,649 | 223,296 | 94,997 | **0** | $0.0135 |

OpenCode, six runs on two different prompts: **$0.0086–$0.0109**, median $0.0098, 44/44 citations
every time, never a failure.

The fork, five runs: **$0.0073–$0.0301**, and **two of the five produced no `WIRE.md` at all** —
rep3 and rep5 both stalled in a retry loop after a dropped connection, re-sending 76,065 and 94,997
tokens at `cached_tokens: 0` and getting one completion token back, roughly seven minutes apart, and
had to be killed by hand. rep3 spent $0.017 on three such retries alone.

### The run this is all about

`wire-sufficefork-rep4` is the reference: the only fork run that completed on the current tree.

| | OpenCode (`rep2`) | fork (`rep4`) |
|---|---|---|
| requests | 10 | 11 |
| **fresh tokens** | **52,658** | **148,187** |
| cached | 284,672 | 379,328 |
| peak context | 56,961 | 111,090 |
| compactions | 0 | **1** |
| corpus pulled into context | 122 KB | **479 KB** |
| **bytes per `read`** | **2,779** | **9,398** |
| citations | 44/44 | 44/44 |
| **cost** | **$0.0098** | **$0.0196** |

73% of the $0.0098 gap is fresh tokens; 14% cached, 12% output. Fresh tokens are the source it read.

## Reading shape, from each agent's own store

OpenCode, `read` arguments out of its session database — 132 of 132 calls carried an explicit window:

| run | reads | limits set | median | range |
|---|---|---|---|---|
| `wire-stock-rep1` | 44 | 44/44 | 80 | 40–100 |
| `wire-stock-rep2` | 44 | 44/44 | 80 | 40–150 |
| `wire-stock-rep3` | 44 | 44/44 | 60 | 40–80 |
| `wire-suffice-rep1` | 44 | 44/44 | 70 | 40–90 |
| `wire-suffice-rep2` | 44 | 44/44 | 80 | 40–100 |

The fork, from its rollouts:

| run | tool calls | reads | median limit |
|---|---|---|---|
| `rep1` | `exec_command` ×6, `read` ×8 (batched, 44 windows) | 44 windows | 110 |
| `rep2` | `exec_command` ×5, `glob` ×1, `read` ×0 | — | — (bulk shell head-read) |
| `rep4` | `glob` ×1, `read` ×51, `exec_command` ×2 | 51 | **500** |

## What was established, and what was retracted

**Established.**

- The prompt is not the cause. OpenCode carrying the fork's `instructions_template` word for word
  still costs $0.0089–$0.0106 and still bounds 132/132 reads.
- The fork's `read` does not re-read or window-chase. rep1 covered all 44 files in 8 batched calls,
  every window explicit, 0 re-reads — the archive's old "reads the corpus two to six times over" no
  longer reproduces.
- `rg` is **not installed on this machine**, and was named in three places (`instructions_template`
  line 80's count escape hatch, `grep`'s description, `exec_command`'s new text). Every fork run
  burned its first request on it.
- Adding `glob`/`grep` did not move the model off the shell; adding the anti-shell paragraph to
  `exec_command`'s description did, completely — shell file work went from 79,052 chars to 635.
- `exec_command`'s `max_output_tokens`, described as an *"Output token budget"*, is what made the
  model ration to 110-line windows: it read that as its **context** budget (*"we have token budget
  19k only"*) when the real window was three times larger. Rewording it removed the brake and the
  median window went 110 → 500.

**Retracted.** Both were measured with a two-step probe that stops at the model's first `read`, and
that probe does not predict cost:

- the three `*_goal` tool specs;
- `glob` result ordering (`search.rs:176` sorts where OpenCode streams `rg`'s walk order). Real at
  p = 0.011 on the opening window, indistinguishable once the read phase runs out.

## Method notes worth keeping

- **Never quote a single run.** The fork's recorded spread on this exact task is 8–18 requests and
  $0.0073–$0.0301 on configurations that differ by nothing.
- The outcome is **bimodal**, not continuous: either ~2,600–5,200 bytes a file, or ~8,800–16,800.
  Report the rate of the unbounded mode, not a mean.
- Measure **bytes returned per read**, not whether the model wrote a `limit`. An unlimited read of a
  four-line `mod.rs` costs nothing.
- Re-implemented tools are a variable. `replay.py` serves the bytes the real run received, from
  OpenCode's session database; that is how the ordering artefact was caught.
