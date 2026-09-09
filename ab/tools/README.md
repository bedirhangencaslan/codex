# Reading a recorded run

Nothing here talks to a provider. Each script takes the artifacts a run already left in
`ab/logs-<tag>/` and turns them into the numbers the measurements were argued from.

    pip install -r ../requirements.txt      # httpx is only needed to *run* experiments; tiktoken to read them

| script | what it answers | call |
|---|---|---|
| `shape.py` | requests, peak prompt, compactions, cost, and the read shape: how many calls carried a `limit`, the median window, files vs slices, re-reads | `python tools/shape.py logs-ask ask-rep1` |
| `trace.py` | the run as a decision log: every reasoning block followed by the tool calls it produced | `python tools/trace.py logs-ask/ask-rep1.rollout.jsonl` |
| `vw.py` | whether the deliverable is real: does every `file:line` citation resolve to a line that exists | `python tools/vw.py <corpus-dir> logs-ask/ask-rep1.WIRE.md` |
| `dump_sf2.py` | a readable transcript of a Codex/Suffice run | `python tools/dump_sf2.py <rollout> out.md <jsonl> "title"` |
| `dump_oc2.py` | the same for an OpenCode session, read from its own sqlite store | `python tools/dump_oc2.py <slug> out.md <jsonl>` |
| `dump_small.py` | the short form: thinking and tool calls only, outputs as sizes | `python tools/dump_small.py <rollout> out.txt <jsonl> "title"` |

Two things a fresh checkout does not have, and what to do about them:

- **The corpus.** `vw.py` resolves citations against the files they name, and the seed
  (`ab/seed-rs`, 77 MB) is not in the repository because it is a copy of this workspace. Point the
  first argument at `codex-rs` in this checkout instead: the `wire` task reads
  `codex-api/src`, which is here.
- **OpenCode's sessions.** `dump_oc2.py` reads `%TMP%/oc_copy.db`, a copy of
  `~/.local/share/opencode/opencode.db`. Only the machine that ran OpenCode has it; the
  transcripts already rendered from it are in `ab/transcript-opencode.*`.

`shape.py` and `trace.py` need nothing but the two files each run archived, so they work anywhere.
