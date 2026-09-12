"""Replay a captured request body N times and compare the answers with the one the agent really got.

No reconstruction anywhere. The body is the bytes that agent sent; the known answer is read out of
the *next* captured body, which carries the response as its new assistant message. So both halves
come from the same capture, for either agent, with nothing rebuilt by hand.

This is the validation the rig needed: if replaying a real request reproduces the distribution of
`limit` values the real run got, the isolated setup is measuring the same thing the binary does and
can be trusted to bisect. If it does not, it cannot.

    python feed.py <bodies-dir> --index 3 --reps 20 --label opencode
"""

import argparse
import io
import json
import os
import statistics
import sys

import httpx

PRICE = {"fresh": 0.075e-6, "cached": 0.015e-6, "out": 0.25e-6}


def load(p):
    return json.loads(io.open(p, encoding="utf-8", errors="replace").read())


def bodies(d):
    return sorted(f for f in os.listdir(d) if f.endswith(".json") and "headers" not in f)


def known_answer(d, files, i):
    """The tool calls the agent really received for body `i`, read from body `i+1`.

    Request i+1 is request i plus the reply, so the messages it has that request i does not are
    exactly that reply.
    """
    if i + 1 >= len(files):
        return None
    a, b = load(os.path.join(d, files[i])), load(os.path.join(d, files[i + 1]))
    n = len(a.get("messages") or [])
    new = (b.get("messages") or [])[n:]
    calls = []
    for m in new:
        if m.get("role") != "assistant":
            continue
        for tc in m.get("tool_calls") or []:
            fn = tc.get("function") or {}
            try:
                args = json.loads(fn.get("arguments") or "{}")
            except ValueError:
                args = {}
            calls.append((fn.get("name"), args))
    return calls


def read_limits(calls):
    return [a.get("limit") for n, a in calls if n == "read"]


def send(client, url, body, headers):
    calls, usage = {}, None
    with client.stream("POST", url, json=body, headers=headers or {}) as resp:
        if resp.status_code != 200:
            raise RuntimeError("HTTP %s: %r" % (resp.status_code, resp.read()[:300]))
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
            for ch in obj.get("choices") or []:
                for tc in (ch.get("delta") or {}).get("tool_calls") or []:
                    slot = calls.setdefault(tc.get("index", 0), {"name": "", "args": ""})
                    fn = tc.get("function") or {}
                    if fn.get("name"):
                        slot["name"] = fn["name"]
                    slot["args"] += fn.get("arguments") or ""
    out = []
    for k in sorted(calls):
        c = calls[k]
        try:
            a = json.loads(c["args"] or "{}")
        except ValueError:
            a = {}
        out.append((c["name"], a))
    return out, usage


def describe(limits, reads):
    setn = sum(1 for x in limits if x)
    med = statistics.median([x for x in limits if x]) if setn else None
    return "%2d reads, %d/%d bounded, median %s" % (reads, setn, reads, med)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("dir")
    ap.add_argument("--index", type=int, default=3, help="1-based body number to replay")
    ap.add_argument("--reps", type=int, default=20)
    ap.add_argument("--port", type=int, default=8799)
    ap.add_argument("--label", default="")
    ap.add_argument("--headers", default="", metavar="FILE",
                    help="send the headers captured beside the body")
    ap.add_argument("--out", default=os.path.join(os.path.dirname(os.path.abspath(__file__)), "feed.jsonl"))
    args = ap.parse_args()

    files = bodies(args.dir)
    i = args.index - 1
    if not 0 <= i < len(files):
        raise SystemExit("index out of range; %d bodies" % len(files))

    body = load(os.path.join(args.dir, files[i]))
    label = args.label or os.path.basename(os.path.dirname(args.dir.rstrip("\\/")))
    truth = known_answer(args.dir, files, i)

    hdrs = None
    if args.headers:
        hdrs = {k: v for k, v in load(args.headers).items()
                if k.lower() not in ("host", "content-length", "connection", "accept-encoding",
                                     "authorization", "content-type")}

    print("=== %s, body %s ===" % (label, files[i]))
    print("messages %d, tools %d, %s chars" % (
        len(body.get("messages") or []), len(body.get("tools") or []),
        f"{sum(len(m.get('content') or '') for m in body.get('messages') or []):,}"))
    if truth is not None:
        tl = read_limits(truth)
        print("the real answer: %s   tools=%s" % (
            describe(tl, len(tl)), sorted({n for n, _ in truth})))
        if tl:
            print("                 limits %s" % tl)
    if hdrs:
        print("headers: %s" % json.dumps(hdrs))
    print()

    url = "http://127.0.0.1:%d/api/paas/v4/chat/completions" % args.port
    client = httpx.Client(timeout=httpx.Timeout(600.0, connect=30.0))
    fresh = cached = out_tok = 0
    rows = []
    for rep in range(1, args.reps + 1):
        calls, usage = send(client, url, body, hdrs)
        if usage:
            pt = usage.get("prompt_tokens", 0)
            ct = (usage.get("prompt_tokens_details") or {}).get("cached_tokens", 0)
            fresh += pt - ct
            cached += ct
            out_tok += usage.get("completion_tokens", 0)
        lim = read_limits(calls)
        rows.append({"label": label, "body": files[i], "rep": rep,
                     "tools": sorted({n for n, _ in calls}), "limits": lim})
        print("  [%2d] %s   %s" % (rep, describe(lim, len(lim)), sorted({n for n, _ in calls})))

    with io.open(args.out, "a", encoding="utf-8") as fh:
        for r in rows:
            fh.write(json.dumps(r) + "\n")

    every = [x for r in rows for x in r["limits"] if x]
    allr = [x for r in rows for x in r["limits"]]
    withreads = [r for r in rows if r["limits"]]
    full = sum(1 for r in withreads if all(r["limits"]))
    print()
    print("replayed %d times" % len(rows))
    print("  responses with reads       %d/%d" % (len(withreads), len(rows)))
    print("  responses fully bounded    %d/%d" % (full, len(withreads) or 1))
    print("  reads bounded              %d/%d" % (len(every), len(allr) or 1))
    if every:
        print("  limit median %.0f  mean %.0f  [%d-%d]" %
              (statistics.median(every), statistics.mean(every), min(every), max(every)))
    print("  reads per rep %s" % [len(r["limits"]) for r in rows])
    print("  cost $%.4f" % (fresh * PRICE["fresh"] + cached * PRICE["cached"] + out_tok * PRICE["out"]))
    return 0


if __name__ == "__main__":
    sys.exit(main())
