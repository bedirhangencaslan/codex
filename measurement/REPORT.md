# Token report

A cached input token bills at a tenth of a fresh one, so a request
costs `e = (input - cached) + 0.1 * cached` and a run costs the sum.

## Side by side

| | .suffice-ab | .codex-baseline | delta |
|---|---|---|---|
| requests | 56 | 22 | 154.5% |
| input tokens | 3041222 | 861981 | 252.8% |
| cached | 2624000 | 708992 | 270.1% |
| cache hit rate | 86.3% | 82.3% | 4.9% |
| output tokens | 65923 | 26205 | 151.6% |
| effective bill B | 679622 | 223888 | 203.6% |
| tool calls | 82 | 20 | 310.0% |
| calls / request | 1.46 | 0.91 | 61.1% |
| compactions | 3 | 0 | - |
| B per tool call | 8288 | 11194 | -26.0% |

Delta is the first home relative to the second; negative means cheaper.
`B per tool call` is the only fair column when the two runs did
different amounts of work.

## Per run

| home | session | req | input | cached | bill | compactions |
|---|---|---|---|---|---|---|
| .suffice-ab | 2026-09-04T09-35-00-01a06b20-b518-7e | 19 | 1205937 | 89.0% | 240388 | 1 |
| .suffice-ab | 2026-09-04T09-53-54-01a06b32-0002-78 | 37 | 1835285 | 84.5% | 439234 | 2 |
| .codex-baseline | 2026-09-04T09-26-05-01a06b18-8b79-75 | 12 | 483737 | 86.0% | 109164 | 0 |
| .codex-baseline | 2026-09-04T09-48-20-01a06b2c-e8cb-75 | 10 | 378244 | 77.4% | 114724 | 0 |
