"""Send OpenCode's real second request N times and count how often the model bounds its reads.

Every measurement so far ran a conversation: many turns, sampling noise compounding at each, and a
rep costing cents. This runs one turn - the turn where the window is chosen - reconstructed from the
real run byte for byte, with a known correct answer: `wire-stock-rep1` answered it with thirteen
`read` calls, twelve at `limit: 80` and one at 100.

It splits the question in two:

  20/20 bounded  the input determines the behaviour, so the answer is in the bytes and the
                 simulator's failure to reproduce it is a bug in how it re-serialises turns.
  12/20 bounded  it does not, and OpenCode's 132-of-132 across six runs cannot be luck - something
                 outside the request body differs between the agents.

    python stage0.py --reps 20 [--no-reasoning]
"""

import argparse
import io
import json
import os
import sqlite3
import statistics
import sys

import httpx

import ocreplay as R

HERE = os.path.dirname(os.path.abspath(__file__))
OC = os.path.join(os.path.dirname(HERE), "wire", "oc1")
PRICE = {"fresh": 0.075e-6, "cached": 0.015e-6, "out": 0.25e-6}


def cap(p):
    return io.open(p, encoding="utf-8", errors="replace").read()


def turn_one(session="wire-stock-rep1"):
    """The real first assistant turn and the tool result it received."""
    con = sqlite3.connect(R.snapshot(R.DB))
    sid = None
    for i, d, _t in con.execute("select id, directory, title from session"):
        if session in (d or ""):
            sid = i
            break
    if not sid:
        raise SystemExit(f"no session matching {session!r}")
    msgs = list(con.execute("select id, data from message where session_id=? order by time_created", (sid,)))
    # msgs[0] is the user turn; msgs[1] is the assistant turn that called glob.
    mid = msgs[1][0]
    reasoning, call = "", None
    for (pd,) in con.execute("select data from part where message_id=? order by id", (mid,)):
        p = json.loads(pd)
        if p.get("type") == "reasoning":
            reasoning += p.get("text") or ""
        elif p.get("type") == "tool":
            st = p.get("state") or {}
            call = (p.get("callID"), p.get("tool"), st.get("input") or {}, st.get("output") or "")
    if not call:
        raise SystemExit("no tool call in the first assistant turn")
    return reasoning, call


def build(reps_reasoning=True):
    reasoning, (call_id, tool, args, output) = turn_one()
    assistant = {
        "role": "assistant",
        "content": None,
        "tool_calls": [{
            "id": call_id,
            "type": "function",
            "function": {"name": tool, "arguments": json.dumps(args)},
        }],
    }
    if reps_reasoning and reasoning:
        assistant["reasoning_content"] = reasoning
    return [
        {"role": "system", "content": cap(os.path.join(OC, "msg00_system.txt"))},
        {"role": "user", "content": cap(os.path.join(OC, "msg01_user.txt"))},
        assistant,
        {"role": "tool", "tool_call_id": call_id, "content": output},
    ]


def tools():
    d = os.path.join(OC, "tools")
    return [json.loads(cap(os.path.join(d, n))) for n in sorted(os.listdir(d)) if n.endswith(".json")]


def once(client, url, messages, specs):
    body = {
        "model": "glm-5.3-flash",
        "messages": messages,
        "tools": specs,
        "tool_choice": "auto",
        "stream": True,
        "stream_options": {"include_usage": True},
        "thinking": {"type": "enabled"},
        "reasoning_effort": "high",
        "max_tokens": 32000,
    }
    calls, usage = {}, None
    with client.stream("POST", url, json=body) as resp:
        if resp.status_code != 200:
            raise RuntimeError(f"HTTP {resp.status_code}: {resp.read()[:300]!r}")
        for line in resp.iter_lines():
            line = line.strip()
            if not line.startswith("data:"):
                continue
            payload = line[5:].strip()
            if payload == "[DONE]":
                continue
            try:
                obj = json.loads(payload)
            except ValueError:
                continue
            if obj.get("usage"):
                usage = obj["usage"]
            for choice in obj.get("choices") or []:
                for tc in (choice.get("delta") or {}).get("tool_calls") or []:
                    slot = calls.setdefault(tc.get("index", 0), {"name": "", "args": ""})
                    fn = tc.get("function") or {}
                    if fn.get("name"):
                        slot["name"] = fn["name"]
                    slot["args"] += fn.get("arguments") or ""
    return [calls[k] for k in sorted(calls)], usage


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--reps", type=int, default=20)
    ap.add_argument("--port", type=int, default=8799)
    ap.add_argument("--no-reasoning", action="store_true",
                    help="drop `reasoning_content` from the assistant turn - mutation 2")
    ap.add_argument("--out", default=os.path.join(HERE, "stage0.jsonl"))
    args = ap.parse_args()

    url = f"http://127.0.0.1:{args.port}/api/paas/v4/chat/completions"
    messages, specs = build(not args.no_reasoning), tools()
    label = "no-reasoning" if args.no_reasoning else "faithful"
    print(f"{label}: {len(messages)} messages, {sum(len(m.get('content') or '') for m in messages):,} chars, "
          f"{len(specs)} tools")
    print("known answer from the real run: 13 reads, 12 at limit 80 and one at 100\n")

    client = httpx.Client(timeout=httpx.Timeout(600.0, connect=30.0))
    fresh = cached = out_tok = 0
    rows = []
    for rep in range(1, args.reps + 1):
        calls, usage = once(client, url, messages, specs)
        if usage:
            pt = usage.get("prompt_tokens", 0)
            ct = (usage.get("prompt_tokens_details") or {}).get("cached_tokens", 0)
            fresh += pt - ct
            cached += ct
            out_tok += usage.get("completion_tokens", 0)
        limits, names = [], []
        for c in calls:
            names.append(c["name"])
            if c["name"] == "read":
                try:
                    limits.append(json.loads(c["args"] or "{}").get("limit"))
                except ValueError:
                    limits.append(None)
        reads = len(limits)
        setn = sum(1 for x in limits if x)
        med = statistics.median([x for x in limits if x]) if setn else None
        rows.append({"rep": rep, "label": label, "tools": names, "reads": reads,
                     "limits": limits, "bounded": setn, "median": med})
        print(f"  [{rep:>2}] {reads:>2} reads, {setn}/{reads} bounded, median {med}   {sorted(set(names))}")

    with io.open(args.out, "a", encoding="utf-8") as fh:
        for r in rows:
            fh.write(json.dumps(r) + "\n")

    withreads = [r for r in rows if r["reads"]]
    allb = sum(1 for r in withreads if r["bounded"] == r["reads"])
    every = [x for r in rows for x in r["limits"] if x]
    cost = fresh * PRICE["fresh"] + cached * PRICE["cached"] + out_tok * PRICE["out"]
    print()
    print(f"reps with reads            {len(withreads)}/{len(rows)}")
    print(f"reps where EVERY read bound {allb}/{len(withreads)}")
    print(f"reads bounded overall       {sum(r['bounded'] for r in rows)}/{sum(r['reads'] for r in rows)}")
    if every:
        print(f"limit median {statistics.median(every):.0f}  mean {statistics.mean(every):.0f}  "
              f"[{min(every)}-{max(every)}]")
    print(f"reads per rep: {[r['reads'] for r in rows]}")
    print(f"cost ${cost:.4f}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
