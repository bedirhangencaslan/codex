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
git am /path/to/suffice-patches/*.patch
```

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

Corpus fixtures (`seed-rs`, `par-*`, `base-git`, `fork-*`) are copies of the Codex tree and are not
included; they are regenerable and were ~700 MB.

### Reading the logs

`read-report.py logs-<tag>` prints requests, compactions, peak window, fresh/cached split and cost
for every run in a directory, plus the read-call shape. Prices are the Z.ai promo card:
`$0.075 / $0.015 / $0.25` per M fresh / cached / output.

**Never quote a single run.** Run-to-run variance on the same configuration exceeded every effect
measured here; the archive keeps the spread so a claim can be checked against it.
