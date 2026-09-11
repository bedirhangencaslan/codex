# `_sim` — a conversation simulator for isolating one difference at a time

Scratch-branch tooling, not part of the product. It exists because comparing this fork against
OpenCode by running both binaries moves every variable at once — prompt, tool schemas, tool output
envelopes, message layout, request parameters — and costs six to twelve requests before it says
anything. Five such runs left the cost gap unexplained; 26 arms here found the cause for $0.037.

## What it is

`sim.py` replaces both binaries with the only part that matters: the loop that talks to the model.
Each arm's prefix is assembled from the **captured wire bytes** of real runs, under `wire/`, rather
than rebuilt — so an arm is byte-identical to the agent it imitates except in the one place under
test. `tools.py` implements `glob`, `grep`, `read` and the shell once, and renders `read` through
whichever envelope the arm asks for.

Each arm stops at the model's first `read` call, because the number under test — the window it
chooses — is decided there. Two requests instead of twelve, and with the prefix cached across arms
a rep costs about $0.0006.

## Running it

Start the metering relay, then run arms:

```powershell
$env:ZAI_API_KEY = '<key>'
python ..\..\measurement\measurement\relay.py --port 8799 --label sim --out sim.jsonl

python sim.py --arm 0-oc --arm 3-oc+suftools --reps 4
python sim.py --arm 3i-suftools-minus-goals-only --reps 3
```

`ARMS` in `sim.py` is the ladder: rung 0 is a faithful OpenCode, rung 5 a faithful fork, and the
lettered rungs between them move one thing each. Adding a rung means adding a dict, not writing
code — `read_from`, `read_desc_edit`, `extra_tools` and `drop_tools` cover swapping a single tool
spec, editing one sentence of a description, and adding or removing tools.

## What it found

`RESULTS.md`. Short version: the reading window is set by the **tool surface**, not the prompt, and
the three `*_goal` specs are what makes this model read whole files. Dropping them moves the median
window from 240 lines to 80 and the "did it bound the read at all" rate from 19/32 to 33/33 —
OpenCode's own numbers.

## Caveats

`wire/` captures carry absolute paths from the machines they were taken on; `sim.py` rewrites them
to `work/` at load time. The tools are re-implementations matched to each agent's source for caps
and wording, not byte-identical to either runtime. Arms are 2-4 requests, so this measures the
opening window and not a whole task.
