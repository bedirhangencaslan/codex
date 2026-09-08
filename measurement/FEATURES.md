# What each fork mechanism cost, measured

Same task, same 80k budget, same model. Upstream Codex plus the four Z.ai
connection commits is the baseline; the fork is those plus its own 42 commits.
Numbers below are from `~/.suffice-ab/analytics` (the fork's own per-request
sidecar) and from the rollouts on both sides.

Sample so far: fork 2 runs / 53 requests, baseline 3 runs / 57 requests.

| mechanism | measurement | verdict | why |
|---|---|---|---|
| batching instruction (`models.json`) | 2.03 tool calls per visible request vs baseline 1.11 | **profit** | same work, ~1.8x fewer round trips, so the prefix is resent fewer times |
| compaction keeps the tool specs (`e00b04c0be`) | fork compaction billed 46,879 over 3; baseline 193,959 over 2 | **profit, large** | baseline re-prefills the whole 95-98k prompt at full price; the fork keeps the prefix cached. 15,626 vs 96,980 per compaction, 6.2x |
| `project_memory` (AGENTS.md refresh) | 19 requests, 268,199 = **39.5% of the fork bill** | **loss in-session** | runs at the peak of the window (77k-100k), makes its own tool calls, fired twice in one run. Its payoff is meant to land in the *next* session, which this A/B does not measure |
| reasoning cost model | dropped 96,078 reasoning tokens; one drop cost a **70,615-token full-price re-prefill** | **loss, mistuned** | horizon used the 70 requests/window cold-start default. These runs actually close a window in **7-22** requests (`request_density.json`: `[8,7,22]`). At 16,013 reasoning tokens against a 57,000 suffix the break-even horizon is 32, so the model chose drop where keep was correct |
| reasoning carried on the wire | retained peak 15,074 tokens = 19% of the 80k window | **loss here** | `anchor_reasoning` carries every reasoning item still in history; the baseline's `anchor_trailing_reasoning` keeps only what follows the last user message. Fuller context hits 80k sooner and compacts more often |
| tool-output shrinker | `shrunk_outputs` 0, `shrunk_tokens_removed` 0, `shrinkable_before_break` 0 | **dead** | never fired in any request |
| prompt cache keep-alive | never observed firing | **dead** | scripted runs have no idle gap for it to fill |

## The one number that decides it

Fork bill, 2 runs, 679,622 effective tokens:

| | requests | bill | share |
|---|---|---|---|
| visible work | 31 | 364,544 | 53.6% |
| `project_memory` | 19 | 268,199 | 39.5% |
| compaction | 3 | 46,879 | 6.9% |

Turn off `project_memory` and fix the horizon and most of the gap closes. The
mechanisms that win (batching, the compaction fix) win on the same runs where
the mechanisms that lose lose more.

## What this does not measure

- The next session. `project_memory` writes a file whose whole point is to make
  tomorrow's session cheaper; a single-shot `codex exec` run never reads it back.
- Run-to-run variance. One fork run read `package-lock.json` end to end and the
  baseline never did; with n=2 that alone moves the totals.
- The baseline's `anchor_trailing_reasoning` is not upstream code. Upstream has
  no Chat wire at all, so the comparison is "drop old reasoning unconditionally"
  against "price it", not "upstream" against "us".
