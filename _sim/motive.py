"""Why does the fork go and measure the files, when OpenCode just reads them?

Both agents reach the same situation: the file listing is in hand and nothing has been read yet.

    OpenCode, body 003   system + task + glob call + glob result      -> answers with `read`
    Suffice,  body 002   system + skills + AGENTS + task + prose
                         + glob call + glob result                    -> answers with `exec_command`

and what that `exec_command` fetches is every file's line count, which the previous experiment
showed is enough on its own to take a model from 88% of reads bounded to 0%.

So the question is no longer how wide the window is. It is which tool gets called at that moment,
and what in the request decides it. The metric here is the tool the model reaches for, not the
`limit` it writes.

    python motive.py --list
    python motive.py --reps 12 --variant suf002-base --variant suf002+ocprompt
"""

import argparse
import copy
import io
import json
import os
import sys

import httpx

import feed
import swap

HERE = os.path.dirname(os.path.abspath(__file__))
MEASURING = {"exec_command", "bash"}


def suf002():
    return feed.load(os.path.join(swap.SUF_DIR, "002.json"))


def oc003():
    return feed.load(os.path.join(swap.OC_DIR, "003.json"))


# --------------------------------------------------------------------------- variants


def m_ocprompt(b, oc):
    prompt, _a, _s = swap.cut_oc_system(oc["messages"][0]["content"])
    b["messages"][0]["content"] = prompt
    return b


def m_ocread(b, oc):
    return swap.replace_tool(b, swap.tool_named(oc, "read"))


def m_ocbash(b, oc):
    """OpenCode's `bash` in place of `exec_command` - 5,969 bytes of it, against our 3,662.

    Both forbid file work through the shell. Ours says "File search: use `glob` (NOT
    `Get-ChildItem`...)" and the model called `Get-ChildItem -Recurse ... Measure-Object -Line`
    anyway, so the question is whether the longer wording holds where ours did not.
    """
    b["tools"] = [t for t in b["tools"] if t.get("function", t)["name"] != "exec_command"]
    b["tools"].append(swap.tool_named(oc, "bash"))
    return b


def m_noshell(b, oc):
    """No shell at all. If it still finds a way to measure, the motive is not the tool."""
    b["tools"] = [t for t in b["tools"] if t.get("function", t)["name"] not in MEASURING]
    return b


def m_noskills(b, oc):
    b["messages"] = [m for i, m in enumerate(b["messages"]) if i != 1]
    return b


def m_noagents(b, oc):
    b["messages"] = [m for m in b["messages"]
                     if not (m.get("role") == "user" and "AGENTS.md instructions for" in (m.get("content") or ""))]
    return b


def m_octools(b, oc):
    b["tools"] = copy.deepcopy(oc["tools"])
    return b


def o_sufexec(b, suf):
    """The reverse: give OpenCode our `exec_command` beside its own tools."""
    b["tools"] = [t for t in b["tools"] if t.get("function", t)["name"] != "bash"]
    b["tools"].append(swap.tool_named(suf, "exec_command"))
    return b


def o_sufprompt(b, suf):
    _p, agents, skills = swap.cut_oc_system(b["messages"][0]["content"])
    b["messages"][0]["content"] = "\n\n".join(x for x in (suf["messages"][0]["content"], agents, skills) if x)
    return b


def m_ocprompt_ocbash(b, oc):
    """Both leading candidates at once.

    Separately they move the measuring rate 4/12 -> 1/12 and 4/12 -> 2/12, neither significant on
    its own. If they compose, together they should land near OpenCode's 0/12; if they do not, the
    motive is in neither.
    """
    return m_ocbash(m_ocprompt(b, oc), oc)


def o_sufprompt_sufexec(b, suf):
    """The reverse pair - our prompt and our shell spec together, inside OpenCode's request."""
    return o_sufexec(o_sufprompt(b, suf), suf)


VARIANTS = {
    "suf002-base":      (None, "suf"),
    "suf002+ocboth":    (m_ocprompt_ocbash, "suf"),
    "oc003+sufboth":    (o_sufprompt_sufexec, "oc"),
    "suf002+ocprompt":  (m_ocprompt, "suf"),
    "suf002+ocread":    (m_ocread, "suf"),
    "suf002+ocbash":    (m_ocbash, "suf"),
    "suf002-noshell":   (m_noshell, "suf"),
    "suf002-skills":    (m_noskills, "suf"),
    "suf002-agents":    (m_noagents, "suf"),
    "suf002+octools":   (m_octools, "suf"),
    "oc003-base":       (None, "oc"),
    "oc003+sufexec":    (o_sufexec, "oc"),
    "oc003+sufprompt":  (o_sufprompt, "oc"),
}


def build(name):
    fn, host = VARIANTS[name]
    if host == "suf":
        a, b = suf002(), oc003()
    else:
        a, b = oc003(), feed.load(os.path.join(swap.SUF_DIR, "002.json"))
    return fn(a, b) if fn else a


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--variant", action="append")
    ap.add_argument("--reps", type=int, default=12)
    ap.add_argument("--port", type=int, default=8799)
    ap.add_argument("--list", action="store_true")
    ap.add_argument("--out", default=os.path.join(HERE, "motive.jsonl"))
    args = ap.parse_args()

    if args.list or not args.variant:
        for k in VARIANTS:
            b = build(k)
            print("  %-18s msgs=%-3d tools=%-3d %s" % (
                k, len(b["messages"]), len(b.get("tools") or []),
                [t.get("function", t)["name"] for t in b.get("tools") or []]))
        return 0

    for n in args.variant:
        if n not in VARIANTS:
            print("unknown variant %r" % n)
            return 2

    url = "http://127.0.0.1:%d/api/paas/v4/chat/completions" % args.port
    client = httpx.Client(timeout=httpx.Timeout(600.0, connect=30.0))
    built = {n: build(n) for n in args.variant}
    state = {n: [] for n in args.variant}

    for rep in range(1, args.reps + 1):
        for n in args.variant:
            # One upstream timeout used to take a 96-request batch with it. A dropped request is
            # recorded and skipped: the arms are interleaved, so a loss lands on one rep of one
            # variant rather than on whatever was running at the time.
            try:
                calls, _usage = feed.send(client, url, built[n], None)
            except Exception as exc:  # noqa: BLE001 - a transport hiccup is not a result
                print("  rep %2d  %-18s DROPPED (%s)" % (rep, n, type(exc).__name__), flush=True)
                continue
            names = [c[0] for c in calls]
            state[n].append(names)
            print("  rep %2d  %-18s %s" % (rep, n, names[:6]), flush=True)

    print()
    print("%-18s %10s %10s %10s   %s" % ("variant", "measured", "read", "other", "first call"))
    rows = []
    for n in args.variant:
        reps = state[n]
        meas = sum(1 for r in reps if any(x in MEASURING for x in r))
        read = sum(1 for r in reps if "read" in r and not any(x in MEASURING for x in r))
        other = len(reps) - meas - read
        firsts = {}
        for r in reps:
            firsts[r[0] if r else "-"] = firsts.get(r[0] if r else "-", 0) + 1
        rows.append({"variant": n, "reps": len(reps), "measured": meas, "read": read, "other": other,
                     "firsts": firsts})
        print("%-18s %10s %10s %10s   %s" % (
            n, "%d/%d" % (meas, len(reps)), "%d/%d" % (read, len(reps)), other,
            ", ".join("%s:%d" % kv for kv in sorted(firsts.items(), key=lambda kv: -kv[1]))))

    with io.open(args.out, "a", encoding="utf-8") as fh:
        for r in rows:
            fh.write(json.dumps(r) + "\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
