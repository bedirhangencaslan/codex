"""Transplant one block between the two agents' decision-point requests, both directions.

The comparison is settled: at body 003 the fork pulls 5.6x the lines OpenCode does, from three times
the reads per response and twice the window. What is not settled is which part of the request causes
it.

So: take OpenCode's body 003 and put one of the fork's blocks in the corresponding place - if the
lines go up, that block is implicated. Then take the fork's body 003 and put OpenCode's version of
the same block in - if the lines go down, it is confirmed in both directions. A block that only
moves one way is a suspect; a block that moves both is the answer.

Everything replays a captured body, so the baseline for each direction is that agent's own real
request, already validated in FEED.md against what the binary actually received.

    python swap.py --list
    python swap.py --variant oc+sufread --reps 12
"""

import argparse
import copy
import io
import json
import os
import statistics
import sys

import httpx

import feed

HERE = os.path.dirname(os.path.abspath(__file__))
LOGS = os.path.join(os.path.dirname(HERE), "logs")
OC_DIR = os.path.join(LOGS, "wire-stock-rep11", "relay.jsonl.bodies")
SUF_DIR = os.path.join(LOGS, "wire-sufficefork-rep22", "relay.jsonl.bodies")
OC_ROOT = r"C:\Users\azsxd\codex\_labs\work\wire-stock-rep11"
SUF_ROOT = r"C:\Users\azsxd\codex\_labs\work\wire-sufficefork-rep22"

PRICE = {"fresh": 0.075e-6, "cached": 0.015e-6, "out": 0.25e-6}

MARK_AGENTS = "Instructions from: "
MARK_SKILLS = "Skills provide specialized instructions"


def retarget(text, frm, to):
    return text.replace(frm, to).replace(frm.replace("\\", "/"), to.replace("\\", "/"))


def oc_body():
    return feed.load(os.path.join(OC_DIR, "003.json"))


def suf_body():
    return feed.load(os.path.join(SUF_DIR, "003.json"))


def cut_oc_system(text):
    """OpenCode's single system message -> (prompt+env, agents_md, skills)."""
    i_a = text.find(MARK_AGENTS)
    i_s = text.find(MARK_SKILLS, max(i_a, 0))
    if i_a < 0:
        return text, "", ""
    end = i_s if i_s > i_a else len(text)
    return text[:i_a], text[i_a:end], (text[i_s:] if i_s > i_a else "")


def tool_named(body, name):
    for t in body.get("tools") or []:
        if t.get("function", t).get("name") == name:
            return copy.deepcopy(t)
    return None


def replace_tool(body, spec):
    name = spec.get("function", spec)["name"]
    out = []
    done = False
    for t in body.get("tools") or []:
        if t.get("function", t)["name"] == name:
            out.append(copy.deepcopy(spec))
            done = True
        else:
            out.append(t)
    if not done:
        out.append(copy.deepcopy(spec))
    body["tools"] = out
    return body


def last_tool_message(body):
    for m in reversed(body["messages"]):
        if m.get("role") == "tool":
            return m
    return None


def first_glob_result(body):
    """The tool message answering the `glob` call - the file listing the model is looking at."""
    ids = {tc["id"] for m in body["messages"] if m.get("role") == "assistant"
           for tc in (m.get("tool_calls") or []) if tc["function"]["name"] == "glob"}
    for m in body["messages"]:
        if m.get("role") == "tool" and m.get("tool_call_id") in ids:
            return m
    return None


# --------------------------------------------------------------------------- variants


def v_oc_suftools(oc, suf):
    oc["tools"] = copy.deepcopy(suf["tools"])
    return oc


def v_oc_sufread(oc, suf):
    return replace_tool(oc, tool_named(suf, "read"))


def v_oc_parallel(oc, suf):
    oc["parallel_tool_calls"] = True
    return oc


def v_oc_sufprompt(oc, suf):
    _p, agents, skills = cut_oc_system(oc["messages"][0]["content"])
    oc["messages"][0]["content"] = "\n\n".join(x for x in (suf["messages"][0]["content"], agents, skills) if x)
    return oc


def v_oc_sufskills(oc, suf):
    oc["messages"].insert(1, copy.deepcopy(suf["messages"][1]))
    return oc


def v_oc_sufglob(oc, suf):
    m = first_glob_result(oc)
    src = first_glob_result(suf)
    if m and src:
        m["content"] = retarget(src["content"], SUF_ROOT, OC_ROOT)
    return oc


def v_suf_octools(suf, oc):
    suf["tools"] = copy.deepcopy(oc["tools"])
    return suf


def v_suf_ocread(suf, oc):
    return replace_tool(suf, tool_named(oc, "read"))


def v_suf_noparallel(suf, oc):
    suf.pop("parallel_tool_calls", None)
    return suf


def v_suf_ocprompt(suf, oc):
    prompt, _a, _s = cut_oc_system(oc["messages"][0]["content"])
    suf["messages"][0]["content"] = prompt
    return suf


def v_suf_noskills(suf, oc):
    suf["messages"] = [m for i, m in enumerate(suf["messages"]) if i != 1]
    return suf


def v_suf_ocglob(suf, oc):
    m = first_glob_result(suf)
    src = first_glob_result(oc)
    if m and src:
        m["content"] = retarget(src["content"], OC_ROOT, SUF_ROOT)
    return suf


VARIANTS = {
    # host = OpenCode's real body 003; one of the fork's blocks put in its place. Lines going UP
    # implicates that block.
    "oc-base":        (None, "oc"),
    "oc+suftools":    (v_oc_suftools, "oc"),
    "oc+sufread":     (v_oc_sufread, "oc"),
    "oc+parallel":    (v_oc_parallel, "oc"),
    "oc+sufprompt":   (v_oc_sufprompt, "oc"),
    "oc+sufskills":   (v_oc_sufskills, "oc"),
    "oc+sufglob":     (v_oc_sufglob, "oc"),
    # host = the fork's real body 003; OpenCode's version of the same block. Lines going DOWN
    # confirms it.
    "suf-base":       (None, "suf"),
    "suf+octools":    (v_suf_octools, "suf"),
    "suf+ocread":     (v_suf_ocread, "suf"),
    "suf-parallel":   (v_suf_noparallel, "suf"),
    "suf+ocprompt":   (v_suf_ocprompt, "suf"),
    "suf-skills":     (v_suf_noskills, "suf"),
    "suf+ocglob":     (v_suf_ocglob, "suf"),
}


def build(name):
    fn, host = VARIANTS[name]
    a, b = (oc_body(), suf_body()) if host == "oc" else (suf_body(), oc_body())
    return fn(a, b) if fn else a


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--variant", action="append")
    ap.add_argument("--reps", type=int, default=12)
    ap.add_argument("--port", type=int, default=8799)
    ap.add_argument("--list", action="store_true")
    ap.add_argument("--out", default=os.path.join(HERE, "swap.jsonl"))
    args = ap.parse_args()

    if args.list or not args.variant:
        for k, (fn, host) in VARIANTS.items():
            body = build(k)
            print("  %-16s host=%-4s msgs=%-3d tools=%-3d chars=%s  parallel=%s" % (
                k, host, len(body["messages"]), len(body.get("tools") or []),
                f"{sum(len(m.get('content') or '') for m in body['messages']):,}",
                body.get("parallel_tool_calls", "-")))
        return 0

    url = "http://127.0.0.1:%d/api/paas/v4/chat/completions" % args.port
    client = httpx.Client(timeout=httpx.Timeout(600.0, connect=30.0))

    for name in args.variant:
        if name not in VARIANTS:
            print("unknown variant %r" % name)
            return 2

    # Round-robin, not variant-by-variant. Run in blocks and anything that drifts during the
    # session - and something did drift, badly enough to move OpenCode's own cost 3x between two
    # days - lands entirely on whichever variant happened to be running. Interleaving spreads it
    # across all of them instead.
    built = {n: build(n) for n in args.variant}
    state = {n: {"fresh": 0, "cached": 0, "out": 0, "per_rep": []} for n in args.variant}
    for rep in range(1, args.reps + 1):
        for name in args.variant:
            calls, usage = feed.send(client, url, built[name], None)
            s = state[name]
            if usage:
                pt = usage.get("prompt_tokens", 0)
                ct = (usage.get("prompt_tokens_details") or {}).get("cached_tokens", 0)
                s["fresh"] += pt - ct
                s["cached"] += ct
                s["out"] += usage.get("completion_tokens", 0)
            lim = feed.read_limits(calls)
            s["per_rep"].append(lim)
            print("  rep %2d  %-16s %2d reads, %d bounded, %s" %
                  (rep, name, len(lim), sum(1 for x in lim if x), [x for x in lim][:8]))

    summary = []
    for name in args.variant:
        s = state[name]
        per_rep = s["per_rep"]
        fresh, cached, out_tok = s["fresh"], s["cached"], s["out"]
        reads = [len(l) for l in per_rep]
        every = [x for l in per_rep for x in l if x]
        allr = [x for l in per_rep for x in l]
        med_reads = statistics.median(reads) if reads else 0
        med_lim = statistics.median(every) if every else 0
        # The cost proxy: what one response drags into the window. An unbounded read takes the
        # documented default, which is what makes it expensive, so it is counted at that.
        default = 2000
        lines = sum((x if x else default) for x in allr) / max(1, len(per_rep))
        cost = fresh * PRICE["fresh"] + cached * PRICE["cached"] + out_tok * PRICE["out"]
        row = {"variant": name, "reps": args.reps, "reads_median": med_reads,
               "limit_median": med_lim, "bounded": "%d/%d" % (len(every), len(allr)),
               "lines_per_response": round(lines), "cost": round(cost, 4)}
        summary.append(row)
        print("  -> reads/resp median %s, limit median %s, bounded %s, lines/response %s, $%.4f" %
              (med_reads, med_lim, row["bounded"], row["lines_per_response"], cost))
        with io.open(args.out, "a", encoding="utf-8") as fh:
            fh.write(json.dumps({**row, "per_rep": per_rep}) + "\n")
        print()

    print("%-16s %7s %7s %9s %12s %9s" % ("variant", "reads", "limit", "bounded", "lines/resp", "cost"))
    for r in summary:
        print("%-16s %7s %7s %9s %12s %9.4f" %
              (r["variant"], r["reads_median"], r["limit_median"], r["bounded"],
               r["lines_per_response"], r["cost"]))
    return 0


if __name__ == "__main__":
    sys.exit(main())
