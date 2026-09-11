# What makes the fork read whole files: the three `*_goal` tool descriptions

## How this was found

Five real runs of the fork (8, 16, 18, 12, 11 requests; $0.0073–$0.0301) failed to isolate anything,
because a binary moves every variable at once and costs six to twelve requests before it says a
word. The simulator in this directory replaces both binaries with the only part that matters — the
loop that talks to the model — and assembles each arm's prefix from the **captured wire bytes** of
real runs (`_labs/wire/opencode/`, `_labs/wire/suf/`). An arm is byte-identical to the agent it
imitates except in the one place under test.

Each arm stops the moment the model asks for its first `read`, because the number under test — the
window it chooses — is decided there. That is 2 requests instead of 12, and with the prefix cached
across arms a rep costs **$0.0006**.

## The result

Every arm below shares the same system prompt, the same four-message layout, the same AGENTS.md, the
same task, the same `read` output envelope and the same request parameters. **Only the tool list
differs.**

| arm | tool surface | `limit` set | median window |
|---|---|---|---|
| `0-oc` ×4 | OpenCode's 10 | **16/16** | **100** [80–120] |
| `3-oc+suftools` ×4 | Suffice's 12 | 19/32 | **240** [150–2000] |
| `3i-suftools-minus-goals-only` ×3 | Suffice's 12 **minus `create_goal`, `get_goal`, `update_goal`** | **33/33** | **80** [60–120] |
| `3g-suftools-minus-goals` ×3 | …minus those and `update_plan` | 27/27 | 140 [100–150] |
| `3f-suftools-bash-for-exec` ×3 | Suffice's, with OpenCode's `bash` for `exec_command` | **3/16** | — (worse) |

Read the second row's `19/32` carefully: in one rep of four the model set **no `limit` at all** on
any of thirteen calls, which takes the documented default of **2000 lines per file**. That is the
shape `wire-sufficefork-rep4` showed for real — 51 reads, median 500, 479 KB of source into context,
peak 111,090, the first compaction ever recorded on this task.

Dropping three tool specs takes the window from 240 to 80 and the "did it bound the read at all"
rate from 19/32 to 33/33. **80 is exactly what OpenCode's model settles at**, across five real runs
and two different system prompts.

## Things this rules out

- **The system prompt.** Rung 1 put Suffice's `instructions_template` in OpenCode's slot and the
  window stayed in band. Six real OpenCode runs — three on its own prompt, three on Suffice's — all
  landed at 70–80 with 132/132 calls carrying an explicit window.
- **The `read` spec.** `3a` gave OpenCode's tool surface Suffice's `read` spec: still 60. `3b` did
  the reverse: still unbounded. The description the model reads for `read` is not what decides this.
- **`task` and `todowrite`.** `3e` removed both from OpenCode's surface and it still settled at 80;
  `3d` added both to Suffice's and it still asked for 200.
- **`bash` vs `exec_command`.** Substituting OpenCode's 5,969-byte `bash` spec made it *worse*
  (3/16 calls bounded). The anti-shell paragraph moves *which* tool gets used, not *how much* is
  read — a distinction four binary runs could not separate.

## Why the goal tools do it

Their text is saturated with budget language that has nothing to do with the context window:

> `create_goal` — "Set **token_budget** only when an explicit **token budget** is requested."
>
> `get_goal` — "including status, **budgets, token and elapsed-time usage, and remaining token
> budget**."
>
> `update_goal` — "When marking a **budgeted goal** achieved … report the final **token usage** to
> the user." / "Do not mark a goal complete merely because its **budget is nearly exhausted**."

A model reading that has been told, in the tool surface, that a budget system exists, tracks its
usage, and will tell it when the budget is nearly exhausted. It then stops rationing its own reads.

This sits exactly on top of the earlier finding from the binary runs: the phrase that *made* the
fork ration in `rep1`/`rep3` was `exec_command`'s `max_output_tokens` — *"Output token budget.
Defaults to 10000 tokens"* — which the model misread as its context budget (*"we have token budget
19k only"*) and rationed against. Both findings are the same mechanism: **this model sizes its reads
against whatever the tool surface tells it about a token budget, and the fork's surface tells it
things that are not about the context window.**

## What to do

`create_goal`'s own description says *"Create a goal only when explicitly requested by the user or
system/developer instructions; do not infer goals from ordinary tasks."* No goal was created, or any
goal tool called, in any of the five real runs. So on an ordinary task these three specs are 3,087
bytes of dead weight on the fixed prefix that also cost 3× the reading window.

Gate them off when no goal exists and none was requested. If they must stay visible, strip the
budget accounting from their descriptions — it is addressed to a feature the model is not using.

## Cost

The whole investigation, 26 arms: **$0.037**. One inconclusive binary run cost $0.030.

## Caveats

- Arms are 2-4 requests, so this measures the *opening* window, not a whole task. It is the right
  measurement — 44 files are read with that window — but it does not observe compaction or the
  write step.
- 3-4 reps per arm. Enough to separate 16/16 from 19/32 and 80 from 240; not enough to rank 80
  against 140.
- The simulator's tools are re-implementations (`tools.py`), matched to each agent's own source for
  caps and wording but not byte-identical to either runtime.
