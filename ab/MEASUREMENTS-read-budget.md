---
name: read-budget
description: Why the same GLM 5.3 Flash budgets its reads under OpenCode and does not under Suffice, measured on the 2026-09-08 wire ablations
metadata:
  type: project
---

# The read-budget gap (measured 2026-09-08, `ab/logs-*`, task `wire`)

Same model, same task text, same relay (`--effort` defaults to `high`, so reasoning depth is
normalised on both sides and is **not** the cause). Deliverable is identical everywhere:
44 citations, 44 valid, 36 distinct basenames, on every run checked - so every dollar of the
spread buys nothing.

## The decision is made in the first two reasoning blocks, and it is a reading of the word "read"

OpenCode, all 6 sessions in `~/.local/share/opencode/opencode.db` (`par-opencode`), phrased
almost identically every time:

> "44 files. I need to read each and write one line per file with a `file:line` citation.
> Some may be long; **I just need enough to understand what each handles plus a line citation**."
> ... "I've read all 44 files now (**each at least partially, enough for one line description**)."

Suffice, every expensive run, phrased as a byte-level obligation:

> "43 files. Need read every. Can use read batches with maybe whole files; unknown size."
> "Must read every file. common truncated at 104 ... Need complete 'read every .rs'."
> "'read every .rs' can be interpreted fully."

A run's cost is decided there. Cheap runs open with "use windows first 100 each"; expensive runs
open with "Need read every".

## What the model actually did

| side | reads | re-reads | median `limit` | $ |
|---|---|---|---|---|
| OpenCode x6 | 44 every time | **0** | 60-120 every time (limits set on 259/264 calls) | $0.0088-$0.0137 (4 relay-logged) |
| Suffice, best (`logs-execcap`) | 2 calls / 48 slices | ~4 | 100 | **$0.0075** |
| Suffice, worst (`logs-cap100/rep2`, `logs-tpl56`) | 36-37 calls / 79-122 slices | 35-78 | 400-500 | $0.0428-$0.0434 |

**OpenCode's number is stable; ours is a lottery.** Three runs of one config (`logs-cap3`) came out
$0.0147 / $0.0261 / $0.0434. That is the finding: not "our model cannot budget" but "its budget
decision has no floor".

## Negative results - these do NOT close the gap on their own

- **Tool shape.** `logs-ocflow` gives our binary OpenCode's exact read schema (single `filePath`,
  `offset`, `limit`, default 2000) plus `glob`/`grep`. The model set a limit on **3 of 45** calls
  and pulled whole files (11.3 KB/file), 1 compaction, $0.0199.
- **Prompt.** `logs-ocprompt` runs OpenCode's system prompt on our tools: median limit 500,
  8 of 9 reads whole-file, $0.0182.
- **Per-file cut notices** ("Showing lines 1-30 of 305. Use offset=31 to continue.") are identical
  on both sides and are **not** the debt trigger: the cheapest runs collect the most of them
  (execcap 39, lean 36, cite 35) and ignore them.
- **Banning measurement backfires.** `logs-nosize` (prompt rule "do not measure files before
  reading them") pushed the median window **up** to 900-1400 lines: with no size information the
  model over-buys. $0.0329 / $0.0170.

## What did move it

- **The default anchor.** 2000 -> 100 (`DEFAULT_LINE_LIMIT`) moved the median window to 70-180
  (`logs-cap100`, `logs-lean`, `logs-cite`, `logs-batch`). Variance survived.
- **A ceiling on `exec_command` output** (51,200 bytes, OpenCode's number): a corpus-wide `rg`
  sweep fell from 102,233 to 47,776 bytes, and that run is the cheapest ever measured on this task
  ($0.0075, 7 requests, peak 47,971).
- **Batching pressure, correctly aimed.** Our `read` gives the model two levers - how many files
  per call and how many lines per file - and the budget trailer names the wrong one first:
  *"Ask for fewer files per call, or a smaller `limit` per file."* The traces take the first branch
  (fragment into more rounds, each resending the window). OpenCode's model has only the `limit`
  lever and therefore pulls it, 44 times, informed by the file's name (tests 50, protocol 70,
  api_bridge 120) - our limits are uniform within a call because they are one shared number.

Config attribution per `logs-*` dir is inferred from the artifacts (tool schema in the calls,
output footers, `ab/backup-20260908-2345/uncommitted.patch`), not from a committed runner, and
every variant is n=1. See [cost.md](cost.md) for the rate card and the standing rule against
one-run comparisons.

## The decisive run: full imitation, and it changed nothing (2026-09-09)

`ab/logs-filepath/filepath-rep1.*`. The fork was rebuilt with OpenCode's read contract copied whole
- single `filePath` schema, its description verbatim (only the image/PDF line dropped and one
`apply_patch` line added), its 2000-line default, its `<path>/<type>/<content>` envelope and both
footers, `glob` and `grep` registered so the description's sentences point at real tools, and
`exec_command` carrying OpenCode's output contract (2000 lines / 51,200 bytes, overflow spilled to
a file the text names, "do NOT use head/tail", dedicated-tool redirection list).

**The model set `limit` on 1 of 48 read calls.** 617 KB of tool output, 11 requests, 2 compactions,
peak 94,517, $0.0212, 44/44 valid citations. Same frame in the reasoning as always: "Need read
every... Read all fully, but output context huge", "Task says read every; should finish file to
EOF".

So the tool surface is not the cause, and neither is the prompt (`logs-ocprompt`) nor the schema
(`logs-ocflow`). Everything OpenCode's model sees about reading, ours now sees too, and it still
reads whole files. What is left, untested: the harness's own conversation shape - who says what
between tool results, what the assistant writes into history, how the turn is framed - and the
possibility that the OpenCode CLI's client-side handling (its own system prompt is 8,887 chars
against our 16,665 template plus a 5,629-char skills block) is what carries the sufficiency frame.

One thing the imitation did change: the model called `exec_command` **zero** times (only `glob` and
`read`), against 186 calls across the earlier runs. The redirection list works; the window budget
does not.

## What actually moved it: the prompt's scope rules, not the tools (2026-09-09)

`ab/logs-scope/scope-rep{1,2,3}.*`, same binary as the imitation run plus three sentences in
`models.json`'s glm template. The user found the cause by reading the template: upstream's
`## Autonomy and persistence` section makes literal fidelity a rule *and* declares reading free -
"You avoid inferring authorization for a materially different action to the user's request",
"informed assumptions ... as long as they don't result in divergence from the user's intent and the
scope of the task", and 145(a) "Bias towards taking action ... the action is read-only, doesn't
change state". OpenCode's 8,887-char prompt contains **no** authorization/scope/divergence language
at all, and instead says "minimize output tokens" and "prefer the Task tool to reduce context usage".

The three sentences: authorization now covers only actions that change state, acquire authority or
leave scope ("how much of a file you read ... are judgement calls that belong to you, and taking the
smaller one is not a divergence"); a request "describes an outcome, not a procedure: 'read every
file' names the files that must be covered, not the number of bytes"; and, next to the read rules,
"Reading a file means reading enough of it to answer what you were asked, not all of its bytes."

| build | read calls | with `limit` | median window | tool output | req | compactions | $ |
|---|---|---|---|---|---|---|---|
| full tool imitation (`logs-filepath`) | 48 | **1** | - | 617 KB | 11 | 2 | 0.0212 |
| + scope rules, rep1 | 44 | **44** | 180 | 261 KB | 9 | 1 | 0.0144 |
| + scope rules, rep2 | 44 | **44** | 260 | 296 KB | 10 | 1 | 0.0131 |
| + scope rules, rep3 | 52 | 29 | 320 | 618 KB | 13 | 2 | 0.0234 |
| OpenCode x6 | 44 | 259/264 | 60-120 | ~150 KB | 11-15 | 0 | 0.0088-0.0137 |

Deliverable held: 44/44 valid citations in rep1 and rep2, 43/44 in rep3. The reasoning shows the
mechanism working in rep1 - *"We need not read every line? User says read every .rs file. 'Read
every .rs file' means covered; our developer says a read enough... Let's read files with limit
160/200"* - and not firing in rep3, which never invoked the rule and left 23 of 52 calls open.

So: **two of three runs land in OpenCode's range, and the variance is now inside the prompt rather
than the tool.** Caveat on the money: these three carry ~1,860 tokens more fixed prefix than the run
above, because fixing the memories 400 turned the `## Memory` guidance block on (13,084 chars of
second system message) while the memory content is still empty - the cost win is understated by
that, and that block is the next thing to price.

## Rewording the request too (2026-09-09) - `ab/logs-ask`, task `wire2`

`ab/prompt-wire2.txt` is `prompt-wire.txt` plus one paragraph: "You do not have to read any file in
full. Read as much of each one as you need to say what it does and to back it with a real line, and
no more. Every file must be covered; every line does not." Kept as a separate task (`wire2` in
`compare-par.sh`, `run-read.sh` now takes the task as its third argument) so `wire` stays
comparable to every run recorded against it.

| run | req | comp | peak | limit set | median window | lines/file | read to EOF | read tok | $ |
|---|---|---|---|---|---|---|---|---|---|
| tool imitation only | 11 | 2 | 94,517 | 1/48 | - | 342 | 44/44 | 151,110 | 0.0212 |
| + scope rules rep1 | 9 | 1 | 84,491 | 44/44 | 180 | 143 | 27/44 | 64,495 | 0.0144 |
| + scope rules rep2 | 10 | 1 | 87,510 | 44/44 | 260 | 163 | 29/44 | 72,550 | 0.0131 |
| + scope rules rep3 | 13 | 2 | 90,242 | 29/52 | 320 | 342 | 44/44 | 151,377 | 0.0234 |
| + reworded ask rep1 | 11 | 1 | 90,342 | 42/44 | 360 | 185 | 33/44 | 81,761 | 0.0155 |
| + reworded ask rep2 | 10 | 1 | 84,052 | 50/50 | 180 | 141 | 26/44 | 63,796 | 0.0133 |
| + reworded ask rep3 | **8** | **0** | **58,763** | 44/44 | **80** | **74** | 9/44 | **35,960** | **0.0091** |
| OpenCode x6 | 11-15 | 0 | 57,549 | 259/264 | 60-120 | ~70 | 0 re-reads | ~34,500 | 0.0088-0.0137 |

**`ask` rep3 is the first run that is OpenCode, not near it**: 8 requests, no compaction, peak 58,763,
80-line windows, 35,960 read tokens. All seven runs delivered 44/44 valid citations.

Medians of three: scope $0.0144 / 72,550 read tok, ask $0.0133 / 63,796. The ranges overlap
($0.0131-0.0234 against $0.0091-0.0155), so at n=3 the wording is **not** proven better than the
prompt rules alone - what is proven is that with the scope rules in the template the model sets a
`limit` at all (6 of 7 runs, against 1 of 48 calls before), and that the wording removes the
"read every / read enough" argument from the reasoning entirely.

The failure mode that remains is not the window size but **finishing files after compaction**:
scope rep3 wrote "should read remaining lines 1217-1247 for completeness" into its own handoff
summary, and the next turn inherited it as a task constraint. Both 342-line runs got there that
way. Next lever, untried: a line in `defaults.toml`'s `compact_prompt` saying an unread tail is not
unfinished work.

