# Measurement: real (10 sessions)

## Sessions
| home | session | task | req | turns | input | hit | output | compact | prefix | fresh $ | cached $ | output $ | total $ | min |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| real | 2026-09-02T21-13-07 | codex | 96 | 7 | 4,684,237 | 97.1% | 46,045 | 0 | 14,826 | 0.0102 | 0.0682 | 0.0115 | **0.0900** | 45.3 |
| real | 2026-09-02T22-18-14 | deneme | 10 | 4 | 146,230 | 75.4% | 2,523 | 0 | 9,162 | 0.0027 | 0.0017 | 0.0006 | **0.0050** | 6.0 |
| real | 2026-09-02T22-25-30 | agent_new | 9 | 4 | 96,646 | 81.1% | 1,574 | 0 | 9,196 | 0.0014 | 0.0012 | 0.0004 | **0.0029** | 2.1 |
| real | 2026-09-02T22-29-00 | agent_web | 168 | 13 | 7,883,005 | 94.6% | 63,897 | 1 | 9,191 | 0.0320 | 0.1118 | 0.0160 | **0.1598** | 96.1 |
| real | 2026-09-03T09-39-38 | agent_web | 47 | 6 | 1,238,894 | 92.2% | 13,109 | 0 | 8,207 | 0.0072 | 0.0171 | 0.0033 | **0.0276** | 37.3 |
| real | 2026-09-03T14-44-24 | agent_web | 4 | 1 | 52,135 | 72.2% | 3,672 | 0 | 11,763 | 0.0011 | 0.0006 | 0.0009 | **0.0026** | 2.7 |
| real | 2026-09-03T15-21-36 | agent_web | 297 | 15 | 15,701,691 | 92.8% | 171,125 | 4 | 11,653 | 0.0854 | 0.2185 | 0.0428 | **0.3466** | 132.2 |
| real | 2026-09-03T23-02-24 | agent_web | 3 | 1 | 27,122 | 64.4% | 288 | 0 | 8,687 | 0.0007 | 0.0003 | 0.0001 | **0.0011** | 0.2 |
| real | 2026-09-03T23-27-19 | codex-rs | 44 | 2 | 2,475,686 | 87.5% | 19,941 | 1 | 13,924 | 0.0232 | 0.0325 | 0.0050 | **0.0607** | 11.3 |
| real | 2026-09-04T00-51-16 | agent_web | 39 | 3 | 2,034,045 | 84.6% | 34,530 | 1 | 8,934 | 0.0235 | 0.0258 | 0.0086 | **0.0580** | 24.3 |

## Fixed prefix
| session | first request input | developer msgs | AGENTS.md | prefix re-read (tokens) | carry $ | of bill |
|---|---|---|---|---|---|---|
| 2026-09-02T21-13-07 | 14,826 | ~3,128 | ~5,879 | 1,408,470 | 0.0211 | 23.5% |
| 2026-09-02T22-18-14 | 9,162 | ~2,672 | ~0 | 82,458 | 0.0012 | 24.8% |
| 2026-09-02T22-25-30 | 9,196 | ~2,698 | ~0 | 73,568 | 0.0011 | 37.6% |
| 2026-09-02T22-29-00 | 9,191 | ~2,698 | ~0 | 1,534,897 | 0.0230 | 14.4% |
| 2026-09-03T09-39-38 | 8,207 | ~1,694 | ~0 | 377,522 | 0.0057 | 20.5% |
| 2026-09-03T14-44-24 | 11,763 | ~3,688 | ~0 | 35,289 | 0.0005 | 20.6% |
| 2026-09-03T15-21-36 | 11,653 | ~3,688 | ~0 | 3,449,288 | 0.0517 | 14.9% |
| 2026-09-03T23-02-24 | 8,687 | ~1,357 | ~573 | 17,374 | 0.0003 | 24.6% |
| 2026-09-03T23-27-19 | 13,924 | ~1,845 | ~5,757 | 598,732 | 0.0090 | 14.8% |
| 2026-09-04T00-51-16 | 8,934 | ~1,403 | ~700 | 339,492 | 0.0051 | 8.8% |
The first request is instructions + tool specs + developer messages + AGENTS.md + the prompt; every later request re-reads it at cached price.

## Cache losses
| where | events | premium paid $ | of bill |
|---|---|---|---|
| compaction request (summariser) | 7 | 0.0318 | 4.2% |
| turn boundary (user idle) | 18 | 0.0260 | 3.4% |
| invisible turn start | 4 | 0.0174 | 2.3% |
| first request after compaction | 7 | 0.0082 | 1.1% |
| same-turn miss | 10 | 0.0058 | 0.8% |
| session warm-up | 4 | 0.0045 | 0.6% |
Premium = tokens the previous request had cached but this one re-prefilled, at fresh minus cached price.

## Compaction
| session | req | context before | summariser input | cached | context after | cost $ | if prefix cached $ |
|---|---|---|---|---|---|---|---|
| 2026-09-02T22-29-00 | 73 | 80,962 | 77,851 | 0.0% | 16,762 | 0.0060 | 0.0047 |
| 2026-09-03T15-21-36 | 77 | 79,086 | 73,413 | 0.0% | 22,928 | 0.0056 | 0.0044 |
| 2026-09-03T15-21-36 | 142 | 81,039 | 73,250 | 0.0% | 15,761 | 0.0059 | 0.0044 |
| 2026-09-03T15-21-36 | 198 | 89,092 | 82,387 | 0.0% | 19,821 | 0.0064 | 0.0049 |
| 2026-09-03T15-21-36 | 271 | 80,719 | 73,590 | 0.0% | 23,466 | 0.0058 | 0.0044 |
| 2026-09-03T23-27-19 | 28 | 83,912 | 75,740 | 0.0% | 20,512 | 0.0059 | 0.0045 |
| 2026-09-04T00-51-16 | 25 | 88,920 | 73,010 | 0.0% | 16,694 | 0.0058 | 0.0044 |
| **total** |  |  |  |  |  | **0.0414** | **0.0318** |
`if prefix cached` is the counterfactual saving of sending the summariser on the same cached prefix (the tool-specs line that was reverted).

## Reasoning retention (46 finished turns with requests after them)
| policy | cost $ |
|---|---|
| keep every finished turn's reasoning | 0.0251 |
| drop at turn end (upstream policy) | 0.0354 |
| per-turn optimum (what the cost model aims at) | 0.0149 |
| observed (sidecar incomplete, partial) | 0.0047 |
R from provider usage, S = context added by the turn after its first request, N = requests to the next compaction.

## Prompt-cache keep-alive (counterfactual)
| ttl s | interval s | budget | idle gaps | gaps > ttl | lost without $ | refresh cost $ | saved with $ | net $ | observed after long gaps |
|---|---|---|---|---|---|---|---|---|---|
| 600 | 420 | 4 | 46 | 2 | 0.0054 | 0.0021 | 0.0054 | 0.0033 | 1 hit / 1 miss |
`observed` says what actually happened after gaps longer than the TTL in these sessions: hits mean a keep-alive (or a longer real TTL) covered the gap. Measured TTL bounds on Z.ai: hits up to 448 s, miss at 1,023 s.

## Tool-output shrinker
|  | outputs | tokens removed | carry tokens | saved $ |
|---|---|---|---|---|
| observed (marker present) | 0 | 0 | 0 | 0.0000 |
| counterfactual: would have shrunk | 2 | 624 | 24,761 | 0.0004 |
Saved = removed tokens once at fresh price plus removed x remaining requests at cached price. The counterfactual keeps head 5 + tail 30 lines and assumes tokens are spread evenly over lines.

## Tool-call batching
| tool responses | calls | calls / response | rounds avoided | saved $ (counterfactual) | calls per response histogram |
|---|---|---|---|---|---|
| 658 | 783 | 1.19 | 125 | 0.0722 | 1:594 2:27 3:21 4:12 5:4 |
Counterfactual: every extra call in a response would otherwise have been its own round at the same context size.

## Tool failures
| class | calls | failed | rate |
|---|---|---|---|
| read | 222 | 24 | 10.8% |
| patch | 151 | 17 | 11.3% |
| run | 146 | 52 | 35.6% |
| search | 78 | 13 | 16.7% |
| other | 52 | 13 | 25.0% |
| write | 43 | 2 | 4.7% |
| stdin | 36 | 13 | 36.1% |
| list | 34 | 3 | 8.8% |
| git | 15 | 4 | 26.7% |
| tool:view_image | 3 | 0 | 0.0% |
| tool:request_user_input | 2 | 0 | 0.0% |
| tool:update_plan | 1 | 0 | 0.0% |

Responses with at least one failure: priced at the session's mean request cost = **0.1447** (upper bound on what the retries cost). Identical failing command retried immediately: 5.

| times | class | command |
|---|---|---|
| 10 | stdin |  |
| 5 | run | npm.cmd run build |
| 4 | run | docker compose up -d |
| 4 | run | python -c "from pathlib import Path; s=Path('api/seed.py').read_text(e |
| 3 | run | pnpm.cmd install --ignore-workspace |
| 3 | other | .\node_modules\.bin\next.cmd build |
| 2 | stdin |  |
| 2 | run | pnpm.cmd exec prisma db push |

## Memories pipeline

No `memories/` directory: the pipeline has not run in this home.
No consolidation threads found among these sessions.
