"""Report for the four-way agent comparison.

Reads the relay's per-request logs and the orchestrator's result lines, and prices every agent
with one rate card. Prices come from ablib so they are defined in exactly one place.

    python compare-report.py [--list-price] [--task patch]
"""

import argparse
import glob
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from ablib import CACHED, FRESH, OUTPUT, table  # noqa: E402

AB = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.join(AB, "logs4")
ORDER = ["suffice", "basefix", "base", "cline", "opencode"]
LABEL = {
    "suffice": "Suffice (fork)",
    "base": "Codex (upstream, no editor)",
    "basefix": "Codex + wire fix",
    "cline": "Cline",
    "opencode": "OpenCode",
}


def load_runs(task):
    rows = {}
    path = os.path.join(OUT, "results.tsv")
    if not os.path.exists(path):
        return rows
    with open(path, encoding="utf-8") as fh:
        next(fh, None)
        for line in fh:
            parts = line.rstrip("\n").split("\t")
            if len(parts) < 6:
                continue
            agent, rep, t, code, done, secs = parts[:6]
            if task and t != task:
                continue
            rows[(agent, int(rep))] = {
                "exit": int(code),
                "completed": done == "yes",
                "secs": int(secs),
            }
    return rows


def load_relay():
    per = {}
    for path in sorted(glob.glob(os.path.join(OUT, "*-rep*.jsonl"))):
        label = os.path.basename(path)[: -len(".jsonl")]
        agent, rep = label.rsplit("-rep", 1)
        recs = []
        with open(path, encoding="utf-8") as fh:
            for line in fh:
                line = line.strip()
                if line:
                    try:
                        recs.append(json.loads(line))
                    except ValueError:
                        pass
        per[(agent, int(rep))] = recs
    return per


def totals(recs):
    p = sum(r.get("prompt_tokens", 0) or 0 for r in recs)
    c = sum(r.get("cached_tokens", 0) or 0 for r in recs)
    o = sum(r.get("completion_tokens", 0) or 0 for r in recs)
    return {
        "requests": len(recs),
        "input": p,
        "cached": c,
        "fresh": max(0, p - c),
        "output": o,
        "no_usage": sum(1 for r in recs if not r.get("prompt_tokens") and "error" not in r),
        "errors": sum(1 for r in recs if "error" in r),
    }


def cost(t, mult):
    return (t["fresh"] * FRESH + t["cached"] * CACHED + t["output"] * OUTPUT) * mult / 1e6


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--task", default="patch")
    ap.add_argument("--list-price", action="store_true", help="double the promo rates")
    args = ap.parse_args()
    mult = 2.0 if args.list_price else 1.0
    card = "list" if args.list_price else "promo (ends 2026-09-09)"

    runs = load_runs(args.task)
    relay = load_relay()
    keys = sorted(set(relay) | set(runs), key=lambda k: (ORDER.index(k[0]) if k[0] in ORDER else 9, k[1]))

    print(f"# Four agents, one task (`{args.task}`), one model (glm-5.3-flash via Z.ai)\n")
    print(f"Rate card: **{card}** — fresh {FRESH}, cached {CACHED}, output {OUTPUT} per M tokens.")
    print("Reasoning depth was normalised to `high` for all four by the relay.\n")

    print("## Per run\n")
    rows = []
    for k in keys:
        agent, rep = k
        t = totals(relay.get(k, []))
        r = runs.get(k, {})
        hit = f"{100.0 * t['cached'] / t['input']:.1f}%" if t["input"] else "-"
        rows.append([
            LABEL.get(agent, agent), str(rep),
            "yes" if r.get("completed") else "no",
            "yes" if r.get("exit") == 124 else "no",
            str(t["requests"]), f"{t['fresh']:,}", f"{t['cached']:,}", hit, f"{t['output']:,}",
            f"**${cost(t, mult):.4f}**",
            f"{r.get('secs', 0) / 60:.1f}",
        ])
    print(table(
        ["agent", "rep", "done", "timeout", "req", "fresh in", "cached in", "hit", "output",
         "cost", "min"],
        rows,
    ))

    print("\n## Per agent\n")
    rows = []
    for agent in ORDER:
        ks = [k for k in keys if k[0] == agent]
        if not ks:
            continue
        ts = [totals(relay.get(k, [])) for k in ks]
        cs = [cost(t, mult) for t in ts]
        done = sum(1 for k in ks if runs.get(k, {}).get("completed"))
        mins = [runs.get(k, {}).get("secs", 0) / 60 for k in ks]
        per_done = (sum(cs) / done) if done else None
        rows.append([
            LABEL.get(agent, agent),
            f"{done}/{len(ks)}",
            f"{sum(t['requests'] for t in ts) / len(ts):.0f}",
            f"{sum(t['fresh'] for t in ts) / len(ts):,.0f}",
            f"{sum(t['cached'] for t in ts) / len(ts):,.0f}",
            f"{sum(t['output'] for t in ts) / len(ts):,.0f}",
            f"${min(cs):.4f}-${max(cs):.4f}",
            (f"**${per_done:.4f}**" if per_done is not None else "**n/a**"),
            f"{min(mins):.1f}-{max(mins):.1f}",
        ])
    print(table(
        ["agent", "completed", "req (mean)", "fresh in", "cached in", "output",
         "cost min-max", "$ / completed run", "min"],
        rows,
    ))

    # Anything that would make a number dishonest gets said out loud rather than averaged away.
    notes = []
    for k in keys:
        t = totals(relay.get(k, []))
        if t["errors"]:
            notes.append(f"- `{k[0]}` rep {k[1]}: {t['errors']} relay error(s); its cost is a floor.")
        if t["no_usage"]:
            notes.append(
                f"- `{k[0]}` rep {k[1]}: {t['no_usage']} request(s) returned no usage; undercounted."
            )
        if not relay.get(k):
            notes.append(f"- `{k[0]}` rep {k[1]}: no relay records at all — it did not use the relay.")
    effort = {}
    for k, recs in relay.items():
        for r in recs:
            effort.setdefault(k[0], set()).add(str(r.get("sent_reasoning_effort")))
    if effort:
        notes.append(
            "- reasoning_effort each agent asked for before normalisation: "
            + ", ".join(f"`{a}`={sorted(v)}" for a, v in sorted(effort.items()))
            + " (`None` means the agent sent nothing, which on GLM means max)."
        )
    if notes:
        print("\n## Caveats\n")
        print("\n".join(notes))


if __name__ == "__main__":
    sys.exit(main())
