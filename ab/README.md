# A/B and measurement harness

Everything here lives outside the product. It reads what the binaries already write
(rollouts, and the fork's optional `analytics/` sidecar) and never changes what a user gets.

## Setup

```
pip install -r requirements.txt         # httpx (relay) + tiktoken (token counts)
export ZAI_API_KEY=...                  # the relay holds the key; the agents get a placeholder
```

The fork itself is built from the sibling checkout, and the binary under test must be **pinned**
rather than discovered: `compare-par.sh` falls back to whatever `target/release/suffice.exe`
happens to be on disk, which is regularly older than the change being measured.

```
cargo build -p codex-cli --bin suffice --manifest-path ../codex/codex-rs/Cargo.toml
export FORK_BIN=.../codex/codex-rs/target/debug/suffice.exe
```

Rust toolchain is pinned by `codex-rs/rust-toolchain.toml` (1.95.0). `rg` is expected on PATH.
Cline and OpenCode are only needed for the four-way comparison: `npm i -g @cline/cli opencode-ai`.

## Running an A/B

```
./run.sh base|fork [reps] [task]     # one side, resets the workspace from the task's seed
./drive.sh [reps] [side ...]         # alternate sides so provider drift hits both
./queue-git.sh                       # base then fork on the short git task (~3 min a side)
```

Tasks: `sepet` (audit a FastAPI/React app), `rs` (trace filters in the Rust workspace),
`docs` (generative: 22 review documents), `git` (import a tree, one commit per crate; short,
exercises the tool-output shrinker). Every prompt carries a stop condition so both sides halt
at the same checkpoint. Homes: `base` -> `~/.codex-baseline`, `fork` -> `~/.suffice-ab`.

## Reading the results

```
python measure.py ab --n 3                       # base vs fork per task, newest n runs a side
python measure.py ledger --home real             # where the dollars actually went, by operation
python measure.py all --home real --md out.md    # every section for one home
python measure.py keepalive --home real --ttl 600 --interval 420 --budget 4
python cost-table.py [N] [home ...]              # one line per session, dollars
python window.py [home ...]                      # what occupies the window at its peak
python ../cost-report.py <home> <home>           # the older effective-token comparison
```

`window.py` answers a different question from `ledger`: not where the dollars went, but what is
in the context window at the moment it is largest — the number that decides whether shrinking a
given kind of output buys a denser window. It counts real tokens via `tiktoken`, because `len/4`
is wrong by roughly ten times on anything whitespace-heavy. See `memory/context-density.md`.

Prices are the Z.ai `glm-5.3-flash` promo rates: $0.075 fresh input, $0.015 cached input,
$0.25 output, per million tokens. A cached token is a fifth of a fresh one; `premium` below is
the difference, what a re-prefilled token costs over a cached one.

### What each section measures, and what it assumes

| section | observed from | counterfactual |
|---|---|---|
| `overview` | provider usage per request | - |
| `ledger` | provider cost per request, split across the items in that request's prompt | none: it reconciles to the billed total. Input is charged on every request that carries it, so an item shows up multiplied by how long it stayed in the window |
| `prefix` | first request's input; developer and AGENTS.md messages | carry = prefix re-read on every later request at cached price |
| `cache` | cached tokens vs the previous request's input | premium on tokens that were cached and got re-prefilled, by cause: summariser, after compaction, invisible turn, turn boundary, warm-up, same-turn |
| `compaction` | summariser request, context before/after | saving if the summariser had shared the cached prefix |
| `retention` | reasoning tokens per turn (usage), context the turn added | keep-all = R x N x cached; drop-all = S x premium; per-turn optimum; observed where the sidecar recorded the verdict |
| `keepalive` | idle gaps between turns, whether the next request hit | without: every gap > ttl re-prefills the prefix; with: refreshes = min(gap / interval, budget) at cached price, saving the miss when they cover the gap |
| `shrinker` | the marker the shrinker writes into outputs | outputs it would have trimmed (allowlisted command, exit 0, >= 45 lines), on any home |
| `batching` | calls per tool response | each extra call in a response would otherwise be its own round at the same context |
| `failures` | exit codes in tool outputs | responses containing a failure, priced at the session's mean request cost (upper bound) |
| `memories` | `memories/` directory, consolidation threads | - |

Rules that keep the numbers honest:

- Never quote a one-run comparison. Run-to-run spread exceeds the side difference; use `min`
  and `max`, and `$ / tool call` when the sides did different amounts of work.
- `retention.observed` and the shrinker marker come from the fork's sidecar and output text;
  on the baseline they read as empty, which is correct, not missing data.
- The keep-alive leaves no record. Its section is entirely counterfactual, checked against
  what actually happened after long gaps. Measured TTL bounds on Z.ai: hits up to 448 s, a
  miss at 1,023 s.
- Item sizes from rollout text are `len / 4` estimates; only provider usage is billing truth.

## Files

- `ablib.py` — loaders: sessions, requests, tool outputs, sidecar join.
- `measure.py` — the sections above.
- `cost-table.py` — the one-line-per-session dollar table.
- `FEATURES.md`, `REPORT.md` — earlier written reports, kept for history.
