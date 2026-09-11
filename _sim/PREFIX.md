# Why the fix bought 14% and not 50%

`ANSWER.md` identified the byte-ceiling clause in `read`'s description, removed it, and the binary
got 14% cheaper instead of the 50% claimed. This is why, and what the rest of the gap is.

## The arm that should have been run first

The clause was isolated against **OpenCode's** prefix, where it was the only fork-shaped thing
present — and there, removing it worked 3/3. The arm that says what it buys on the **fork's** stack
was never run. Run now:

| | B/read per rep | mean | mean cost |
|---|---|---|---|
| `f-suf` — the fork whole | 9,706 · 6,502 | 8,104 | $0.0150 |
| `f-suf-nobytes` — same, clause removed | 5,185 · 9,144 · 6,918 | 7,082 | $0.0122 |

**19% in the simulator, 14% in the binary.** The two agree. The fix is real and its ceiling is
about a sixth of the bill — the claim of a half came from carrying a single-piece result onto the
whole stack, which is an error and not a measurement.

## What the rest of it is: the prompt and the layout, together

The fork's prefix alone — with OpenCode's **entire** tool surface, its `read` spec and its
envelope — is as expensive as the fork whole. Splitting that prefix, everything else byte-identical:

| system[0] | rest of the prefix | B/read per rep | mean |
|---|---|---|---|
| OpenCode's prompt | OpenCode's layout | 2,840 · 2,566 · 3,264 · 2,160 | 2,708 |
| **the fork's prompt** | OpenCode's layout | 3,601 · 2,906 | 3,253 |
| OpenCode's prompt | **the fork's layout** | 2,828 · 4,219 · 3,349 | 3,465 |
| **the fork's prompt** | **the fork's layout** | **8,317 · 8,980 · 10,922** | **9,406** |

**2.71×, and the two groups do not overlap** — worst clean rep 4,219, best dirty rep 8,317.

Neither piece does it alone. Both were measured clean on their own, twice and three times over. It
is the pair.

### What "the fork's layout" is

Four messages where OpenCode sends two:

```
system[0]  the prompt                         16,665 chars
system[1]  skills instructions + permissions   5,591
user[0]    AGENTS.md + <environment_context>  23,951
user[1]    the task                              519
```

OpenCode puts all of that in one system message and then the task. So in the fork's shape the
reading guidance sits about 30 KB and one role boundary upstream of the task it applies to.

OpenCode's prompt survives that arrangement (3,465) because **OpenCode's prompt contains no window
guidance at all** — its reading discipline lives in the `read` tool description, which sits next to
the decision no matter how the messages are arranged. The fork's guidance is in
`instructions_template` lines 81-82, and that is what gets buried.

### Flattening helps, but not all the way

The fork's own content in OpenCode's shape — one system message carrying prompt, skills block and
AGENTS.md, then the task — same 46,577 characters:

| | B/read per rep | clean reps |
|---|---|---|
| four messages (`f-prefix-only`) | 8,317 · 8,980 · 10,922 | **0/3** |
| one message (`f-prefix-agents-inline`) | 10,257 · 2,980 · 3,718 | **2/3** |

So the role boundary and the distance are most of it, and not all of it.

## What to do, in order of evidence

1. **Move the window guidance out of the prompt and into `read`'s description.** This is OpenCode's
   design and the reason its prompt is layout-proof. It is also the cheapest change: the sentence
   already exists, at `instructions_template` lines 81-82.
2. **Put AGENTS.md in the system message** rather than a user turn. 2/3 clean against 0/3, on the
   fork's own text.
3. The clause removal, already applied, holds its 14-19%.

## Caveats

- Three reps an arm, on an outcome that is bimodal. The prompt/layout separation is clean (no
  overlap, 3 v 3); the flattening result is not (2/3).
- Eight steps, read phase only.
- The simulator's absolute costs run below the binary's; the ratios are what carry, and on the one
  case where both were measured — the clause removal — they agreed to within five points.
