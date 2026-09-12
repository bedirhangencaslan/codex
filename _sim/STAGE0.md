# The request does not determine the behaviour

Four days of work moved prompts, tool descriptions, message layouts and request parameters. This
test says none of that could have worked, and why.

## The test

OpenCode's **real second request** on `wire-stock-rep1`, rebuilt from its own session database: the
same 32,481-character system message, the same task, the same ten tool specs, the same first
assistant turn (its reasoning and its `glob` call, with the original `callID`), and the same `glob`
result it actually received — 3,872 characters in the original walk order.

That request has a known answer. The real run replied to it with **13 `read` calls, twelve at
`limit: 80` and one at 100**, and across its six recorded runs OpenCode bounded **132 of 132** reads.

Sent to the same model, same relay, same parameters, **20 times**:

| | |
|---|---|
| reps where every read was bounded | **5 / 18** |
| reads bounded overall | **53 / 108 — 49%** |
| `read` calls per rep | 0, 0, 3, 3, 4, 4, 4, 4, 5, 5, 8, 8, 8, 8, 10, 11, 11, 12 |
| limit when set | median **80**, mean 78, range 60–120 |
| two reps called `task` and read nothing | |

The window it chooses when it chooses one is exactly right — median 80, OpenCode's own number. **It
just doesn't choose one half the time.**

## What that rules out

If a read is bounded with probability 0.49, six real runs bounding 132 of 132 has probability
0.49¹³² ≈ 10⁻⁴⁰.

So OpenCode's discipline **is not a property of the bytes it sends**. No edit to the prompt, the
tool descriptions, the message layout or the request parameters can explain a difference that
survives holding all of them identical. Every arm measured before this one was searching a space
that does not contain the answer — which is why nothing separated, why `n = 3` never resolved
anything, and why two findings had to be retracted.

## Stage 1: the rebuild was right, and it changes nothing

`relay_all.py` dumps every request body instead of only the first. One OpenCode run
(`wire-stock-rep11`, 25 requests, 44/44 citations) gives the real turn-two request to diff against
the rebuild above.

The rebuild was structurally correct. Same seven parameters — `max_tokens` 32000, `tool_choice`
auto, `stream`, `stream_options`, `thinking`, `reasoning_effort` high, `model`. The ten tool specs
**byte for byte identical**. `reasoning_content` carried as a sibling field on the assistant turn,
exactly as guessed. The tool result a plain string with `tool_call_id`.

One field differed:

```
assistant.content    real ""   (empty string)
                     mine null
```

Sent 20 more times with `content: ""`:

| | `null` | `""` |
|---|---|---|
| reps where every read was bounded | 5 / 18 | **4 / 17** |
| reads bounded overall | 53 / 108 (49%) | **35 / 86 (41%)** |

No difference. Pooled, **9 of 35 responses** bound every read — 26%.

### The statistic, stated correctly

The unit is not a read, it is a *response*: within one reply the model almost always bounds all of
its reads or none of them (5/5, 8/8, 13/13 against 0/5, 0/11). OpenCode's six runs produced roughly
twenty read-carrying responses and bounded every read in all of them. At 26% a response, twenty in
a row is 0.26²⁰ ≈ 10⁻¹².

So it is settled, and no longer by inference: **a request byte-identical to OpenCode's, with
identical parameters and identical tools, does not produce OpenCode's behaviour.** The cause is not
in the request.

### What is left

The transport. `relay.py` forwards the client's own headers upstream, substituting only
`Authorization` — so OpenCode's headers reach Z.ai and this script's do not. OpenCode calls through
`@ai-sdk/openai-compatible`; the rebuild calls through `httpx`. Nothing else about the exchange has
been compared, and headers are now captured alongside each body for exactly this.

## What remains

Two possibilities, and they are distinguished by the same cheap measurement.

**1. The reconstruction is wrong where it could not be copied.** The system message, the task and
the tool specs came from a captured wire body, but the *assistant* turn and the *tool result* turn
were rebuilt by hand — `content: null`, `reasoning_content` as a sibling field, the tool output as a
raw string. OpenCode's provider may serialise any of those differently, and those three messages are
the entire difference between request one, which is captured, and request two, which is not.

**2. Something outside the request body differs** — a parameter OpenCode sends only on later
requests, a different endpoint, a retry the agent performs and does not record.

Both are answered by capturing **every** request body from a real OpenCode run rather than only the
first, and diffing request two against this reconstruction. That is a change to `relay.py` and one
run: no new measurement technique, no new spend beyond the run itself.

## Method note

One turn, one decision, a known correct answer, half a cent a rep. Every previous arm ran a whole
conversation — many turns, sampling noise compounding at each, six to fifteen cents a rep — which is
why 18 reps here cost less than two reps there and say more.

Cost of this test: **$0.0072** for 20 requests.
