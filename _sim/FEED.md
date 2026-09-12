# The rig, validated on both agents — and what it measures

## What it is

`feed.py` replays a **captured request body** N times and compares the answers with the one the
agent really received. Nothing is rebuilt: the body is the bytes that agent sent, and the known
answer is read out of the *next* captured body, which carries the reply as its new assistant
message. That works identically for either agent.

Both agents' decision point is body **003** — the request that follows the file listing, where the
model first calls `read` and therefore first chooses a window.

## Validation

| | OpenCode `wire-stock-rep11` | Suffice `wire-sufficefork-rep22` |
|---|---|---|
| what the agent really got at body 003 | 3 reads, **0/3 bounded** | 12 reads, **0/12 bounded** |
| rig: reads per response | 3–12, median **4** | 5–13, median **12** |
| rig: reps reproducing an all-unbounded response | **7/20** | **4/20** |
| rig: reads bounded overall | 49/92 — **53%** | 97/180 — **54%** |
| the real whole run's bounded rate | 25/44 — **57%** | 26/53 — **49%** |

Both real answers sit inside the distribution the rig produces, and the rig's bounded rate matches
each run's own rate to within a few points. **The isolated setup measures what the binaries do.**

(The real runs' *median* limit differs from the rig's because a run's later requests carry a far
larger context — `rep22` reached 2000 on 17 of its 26 bounded reads, all of them after body 003.
The rig replays one request, so it reports that request's behaviour, which is what it is for.)

## The comparison, at the identical decision point, 20 samples each

| | OpenCode | Suffice | ratio |
|---|---|---|---|
| responses that bound every read | 8/18 | 6/16 | — |
| **reads bounded** | 49/92 — 53% | 97/180 — **54%** | **same** |
| **reads per response, median** | **4** | **12** | **3×** |
| **`limit` median** | **80** | **140** | 1.75× |
| `limit` mean | 92 | 173 | 1.9× |
| `limit` range | 60–150 | 49–719 | |
| **lines pulled per response** | **~368** | **~2,076** | **5.6×** |

## What this says

**It is not that the fork fails to bound its reads.** The rate is identical — 53% against 54%. That
hypothesis, which drove the `read` spec work and the prompt edits, is dead.

The fork is more expensive for two reasons that multiply:

1. **It asks for three times as many files per turn** — median 12 reads against 4.
2. **It asks for a window about twice as wide** — median 80 against 140, mean 92 against 173.

Together that is **5.6× the lines in a single response**, and the lines stay in the window for every
request afterwards. Nothing else in the measurement needs to be invoked to explain the bill.

Note which of these is the *larger* factor: the batch size, not the window. Every fix attempted so
far — the byte-ceiling clause, moving the guidance into `read`'s description, stripping the prompt's
reading bullets — acted on the window. None of them touched how many files the model asks for at
once, and that is the 3× term.

## Where the batch size could come from

Untested, listed so the next rig knows what to mutate at body 003:

- `read`'s description tells the model to call it in parallel, and the fork's version adds "several
  `read` calls in one response run together and each answers on its own" where OpenCode's stops at
  "Call this tool in parallel when you know there are multiple files you want to read."
- `instructions_template` has its own parallel-calls bullet; OpenCode's prompt has one too.
- `parallel_tool_calls: true` is sent by the fork and not by OpenCode.
- The file listing that body 003 answers differs: the fork's `glob` returned 4,181 characters
  against OpenCode's 3,916, sorted rather than in walk order.

## Aside: the transports differ far more than expected

Captured headers, same relay, both forwarded upstream:

```
OpenCode  User-Agent: opencode/1.18.29 ai-sdk/provider-utils/4.0.23 runtime/bun/1.3.14
          x-session-id, x-session-affinity

Suffice   user-agent: codex_exec/0.0.0 (Windows 10.0.19045; x86_64)
          originator: codex_exec
          x-codex-beta-features: remote_compaction_v2
          x-codex-window-id, x-client-request-id, session-id, thread-id
          x-codex-turn-metadata: {installation_id, session_id, thread_id, agent_name, turn_id,
                                  window_id, window_number, context_window_id, request_kind,
                                  root_turn_id, thread_source, sandbox, sandbox_mode,
                                  auto_review_enabled, ...}
```

Headers were already shown not to move OpenCode's behaviour when added, so this is recorded rather
than suspected. It is also a privacy note: `x-codex-turn-metadata` ships installation and session
identifiers to whatever endpoint the model provider happens to be.

## Cost

$0.0065 for 20 OpenCode replays, $0.0098 for 20 Suffice replays. A conversation arm was six to
fifteen cents for one sample.
