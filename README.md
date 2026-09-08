# Suffice — measurement archive

Cost measurements for **Suffice**, a fork of OpenAI Codex CLI running GLM 5.3 Flash through Z.ai.
The fork itself lives on the `suffice` branch of
[bedirhangencaslan/codex](https://github.com/bedirhangencaslan/codex); this repository holds the
evidence behind it and a patch-series copy of the same work.

## `suffice-patches/`

The 71 commits that make up Suffice, as `git format-patch` output on top of upstream
`0a12b855a0b21068108a8a3b311d492712737e0f`. Applying them reproduces the fork without cloning it:

```bash
git clone https://github.com/openai/codex.git && cd codex
git checkout 0a12b855a0b21068108a8a3b311d492712737e0f
git am --3way --empty=keep /path/to/suffice-patches/*.patch
```

`--empty=keep` is required: the first patch is an empty commit and `git am` stops on it otherwise.
Verified — applying the series reproduces tree `6d08f6d1a1d177bff213e619978c8de88e673ecb`, which is
byte-for-byte the fork it was cut from.

Kept because a patch series survives a force-push, a rebase, or a lost branch; the branch is the
working copy, this is the backup.

## `measurement/`

The harness and every run behind the numbers.

| | |
|---|---|
| `relay.py` | A metering proxy in front of Z.ai. No agent's self-reported cost is comparable — each prices from its own table and two report nothing — so this reads the provider's own `usage` and normalises the two request fields that would otherwise decide the result (`stream_options.include_usage`, `reasoning_effort`). |
| `compare.sh`, `compare-par.sh` | Run one task across Suffice, baseline Codex, Cline and OpenCode, sequentially or all at once, each behind its own relay. |
| `run-read.sh` | Repeat one task on the fork alone, archiving every artifact of every run. |
| `read-report.py`, `compare-report.py`, `ledger.py`, `measure.py` | Pricing and mechanism reports. `ledger.py` is the only breakdown that reconciles to the billed total. |
| `logs-*/` | One directory per experiment: relay log (`*.jsonl`), stdout, rollout (`*.rollout.jsonl`, the only place a tool call's arguments are recorded) and the deliverable. |
| `MEASURE-*.md` | Written-up snapshots. |
| `backup-*/` | Working-tree state that was reverted rather than committed, kept recoverable. |

### The corpus

`measurement/seed-rs/` holds what the `wire` task actually reads: `codex-api/` (44 `.rs` files,
545,777 bytes, 15,096 lines) and the `AGENTS.md` that every request carries. It is a snapshot, not a
checkout of any one commit, and the numbers only compare against each other because every run read
these exact bytes.

To run a comparison, copy it to a working directory the agent can write in; `compare-par.sh` does
that per run. The other fixtures that lived beside it (`par-*`, `base-git`, `fork-*`, the rest of
`seed-rs`) were ~700 MB of Codex-tree copies that nothing reads, and are not included.

### Reading the logs

`read-report.py logs-<tag>` prints requests, compactions, peak window, fresh/cached split and cost
for every run in a directory, plus the read-call shape. Prices are the Z.ai promo card:
`$0.075 / $0.015 / $0.25` per M fresh / cached / output.

**Never quote a single run.** Run-to-run variance on the same configuration exceeded every effect
measured here; the archive keeps the spread so a claim can be checked against it.
