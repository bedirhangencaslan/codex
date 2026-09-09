# Four agents, one task (`patch`), one model (glm-5.3-flash via Z.ai)

Rate card: **promo (ends 2026-09-09)** — fresh 0.075, cached 0.015, output 0.25 per M tokens.
Reasoning depth was normalised to `high` for all four by the relay.

## Per run

| agent | rep | done | timeout | req | fresh in | cached in | hit | output | cost | min |
|---|---|---|---|---|---|---|---|---|---|---|
| Suffice (fork) | 1 | yes | no | 4 | 9,537 | 29,760 | 75.7% | 754 | **$0.0014** | 0.6 |
| Suffice (fork) | 2 | yes | no | 5 | 10,620 | 38,336 | 78.3% | 809 | **$0.0016** | 0.6 |
| Codex (baseline) | 1 | yes | no | 14 | 14,506 | 140,864 | 90.7% | 4,777 | **$0.0044** | 2.7 |
| Codex (baseline) | 2 | yes | no | 16 | 17,114 | 181,248 | 91.4% | 7,649 | **$0.0059** | 4.1 |
| Cline | 1 | yes | no | 5 | 13,086 | 22,272 | 63.0% | 1,802 | **$0.0018** | 1.2 |
| Cline | 2 | yes | no | 5 | 19,851 | 16,320 | 45.1% | 2,278 | **$0.0023** | 1.3 |
| OpenCode | 1 | yes | no | 6 | 10,093 | 32,448 | 76.3% | 1,122 | **$0.0015** | 0.5 |
| OpenCode | 2 | yes | no | 6 | 9,809 | 32,128 | 76.6% | 889 | **$0.0014** | 0.4 |

## Per agent

| agent | completed | req (mean) | fresh in | cached in | output | cost min-max | $ / completed run | min |
|---|---|---|---|---|---|---|---|---|
| Suffice (fork) | 2/2 | 4 | 10,078 | 34,048 | 782 | $0.0014-$0.0016 | **$0.0015** | 0.6-0.6 |
| Codex (baseline) | 2/2 | 15 | 15,810 | 161,056 | 6,213 | $0.0044-$0.0059 | **$0.0052** | 2.7-4.1 |
| Cline | 2/2 | 5 | 16,468 | 19,296 | 2,040 | $0.0018-$0.0023 | **$0.0020** | 1.2-1.3 |
| OpenCode | 2/2 | 6 | 9,951 | 32,288 | 1,006 | $0.0014-$0.0015 | **$0.0015** | 0.4-0.5 |

## Caveats

- reasoning_effort each agent asked for before normalisation: `base`=['high'], `cline`=['high'], `opencode`=['None'], `suffice`=['high'] (`None` means the agent sent nothing, which on GLM means max).
