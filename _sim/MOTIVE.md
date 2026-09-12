# Why the fork measures the files first

The previous experiment found what costs the money: before reading, the fork runs
`Get-ChildItem -Recurse … Measure-Object -Line` and receives every file's exact length. Handed that
same listing, OpenCode goes from **57/65 reads bounded to 0/74**, and from 1,528 lines a response to
**10,571** — landing on the fork's own 12,266. Knowing how long a file is, the model stops asking
for a window.

This asks the question one step back: why does the fork take that step when OpenCode does not?

## The decision point

Both agents reach the same situation — file listing in hand, nothing read yet:

```
OpenCode  body 003   system + task + glob call + glob result                -> read
Suffice   body 002   system + skills + AGENTS + task + prose + glob + result -> exec_command
```

Replaying each 12 times, round-robin, and counting which tool the model reaches for:

| variant | went measuring | read directly |
|---|---|---|
| `suf002-base` | **4/12 — 33%** | 8/12 |
| `suf002+ocprompt` | **1/12** | 11/12 |
| `suf002+ocbash` | 2/12 | 10/12 |
| `suf002-noshell` | 0/12 *(no shell to call)* | 12/12 |
| `suf002-skills` | 5/12 | 7/12 |
| `suf002-agents` | 4/12 | 8/12 |
| **`oc003-base`** | **0/12** | 11/12 |
| `oc003+sufprompt` | **0/12** | 12/12 |

## What it establishes

**The behaviour is real and one-sided.** The fork reaches for the shell a third of the time in this
situation; OpenCode did not once in twelve. That is the difference, and it is not about how wide a
window is — it is about whether an extra step happens at all.

**It is intermittent, which explains the erratic runs.** A third of the time it measures, and when it
does the response costs about seven times as much. That is the shape of the fork's recorded spread —
$0.0073 to $0.0301 on configurations that differ by nothing.

**The prompt is the leading candidate, and it is not proven.** OpenCode's prompt in place of the
fork's takes it from 4/12 to 1/12. But the reverse fails: the fork's prompt given to OpenCode
produces 0/12, no measuring at all. A cause should work both ways, and this one does not.

**The skills block and AGENTS.md are cleared.** 5/12 and 4/12 against a base of 4/12.

**`exec_command`'s own description is a weaker candidate than expected.** Substituting OpenCode's
`bash` spec — 5,969 bytes against our 3,662, both forbidding file work through the shell — moves it
4/12 to 2/12. Our own wording already says *"File search: use `glob` (NOT `Get-ChildItem`...)"* and
the model called `Get-ChildItem -Recurse … Measure-Object -Line` anyway. Neither text stops it
reliably.

At n = 12, 4/12 against 1/12 is p ≈ 0.16 and 4/12 against 0/12 is p ≈ 0.05. Only the last is worth
anything, and it is the one comparing the two agents rather than isolating a cause inside one.

## What the model says it is doing

Its own reasoning on the real run, verbatim:

> 44 files. Need read every .rs. Could read in batches parallel. Need citation file:line. Must
> accurately characterize. We can use read whole files but **risk token. Many potentially huge.** We
> can inspect heads and grep key structures. "Read every .rs file" must actually read every. We can
> read perhaps full? **Need know line counts.** Use wc via rg? command read-only. Get lines counts.

Two beliefs drive it: that it must read every file completely, and that some might be too big to
afford. It resolves the tension by measuring first — and the measurement then removes the reason to
limit anything.

## What to test next

- **prompt and `bash` together**, both directions, at higher n. Separately they give 4→1 and 4→2;
  the question is whether they compose.
- The sentence in `instructions_template` that produces "must actually read every". OpenCode's
  prompt has no equivalent, and the fork's model quotes the task back at itself as an obligation.
- Whether `read`'s documented 2000-line default is what makes "many potentially huge" a worry worth
  a round trip.

## Cost

$0.021 for 96 single-turn replays across eight variants.
