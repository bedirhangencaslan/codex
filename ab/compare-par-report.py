"""Report for the parallel run, with compaction detected from the wire.

Compaction is visible without reading any agent's internals: the prompt grows request by request,
then drops sharply when the agent replaces history with a summary. That sawtooth is the same shape
whatever the tool, so one detector covers all of them - including Cline, whose compaction settings
are not readable from disk.

    python compare-par-report.py [--drop 0.30] [--list-price]
"""

import argparse
import glob
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from ablib import CACHED, FRESH, OUTPUT, table  # noqa: E402

AB = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.join(AB, "logs5")
ORDER = ["suffice", "basefix", "base", "cline", "opencode"]
LABEL = {
    "suffice": "Suffice (fork)",
    "basefix": "Codex + wire fix",
    "base": "Codex (upstream)",
    "cline": "Cline",
    "opencode": "OpenCode",
}


def load(agent):
    p = os.path.join(OUT, agent + ".jsonl")
    if not os.path.exists(p):
        return []
    rows = []
    for line in open(p, encoding="utf-8"):
        line = line.strip()
        if line:
            try:
                rows.append(json.loads(line))
            except ValueError:
                pass
    return rows


def results():
    out = {}
    p = os.path.join(OUT, "results.tsv")
    if not os.path.exists(p):
        return out
    with open(p, encoding="utf-8") as fh:
        next(fh, None)
        for line in fh:
            f = line.rstrip("\n").split("\t")
            if len(f) >= 4:
                out[f[0]] = {"exit": int(f[2]), "secs": int(f[3])}
    return out


def compactions(rows, drop):
    """Requests whose prompt fell by more than `drop` of the previous one."""
    hits = []
    prev = 0
    for i, r in enumerate(rows):
        pt = r.get("prompt_tokens", 0) or 0
        if prev and pt < prev * (1 - drop):
            hits.append({"at": i, "from": prev, "to": pt})
        prev = max(prev, 0) if pt == 0 else pt
    return hits


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--drop", type=float, default=0.30)
    ap.add_argument("--list-price", action="store_true")
    args = ap.parse_args()
    mult = 2.0 if args.list_price else 1.0
    res = results()

    print("# One task, five agents, all running at the same time\n")
    print("Task `wire`: read every .rs file under codex-api/src (44 files, 624 KB) and write WIRE.md.")
    print("30-minute cap each, one relay per agent on its own port, reasoning normalised to `high`.\n")

    rows = []
    detail = []
    for a in ORDER:
        recs = load(a)
        if not recs and a not in res:
            continue
        ok = [r for r in recs if "error" not in r]
        p = sum(r.get("prompt_tokens", 0) or 0 for r in ok)
        c = sum(r.get("cached_tokens", 0) or 0 for r in ok)
        o = sum(r.get("completion_tokens", 0) or 0 for r in ok)
        fresh = max(0, p - c)
        cost = (fresh * FRESH + c * CACHED + o * OUTPUT) * mult / 1e6
        peak = max((r.get("prompt_tokens", 0) or 0) for r in ok) if ok else 0
        comp = compactions(ok, args.drop)
        bad = [r for r in recs if r.get("status") not in (200, None) or "error" in r]
        r_ = res.get(a, {})
        rows.append([
            LABEL.get(a, a),
            str(len(recs)),
            f"{peak:,}",
            str(len(comp)),
            f"{fresh:,}", f"{c:,}", f"{o:,}",
            f"**${cost:.4f}**",
            f"{r_.get('secs', 0) / 60:.1f}",
            "timeout" if r_.get("exit") == 124 else str(r_.get("exit", "-")),
            str(len(bad)) if bad else "-",
        ])
        detail.append((a, ok, comp))

    print(table(
        ["agent", "req", "peak prompt", "compactions", "fresh in", "cached in", "output",
         "cost", "min", "exit", "bad"],
        rows,
    ))

    print("\n## Where each agent compacted\n")
    for a, ok, comp in detail:
        if not comp:
            print(f"- **{LABEL.get(a, a)}**: never compacted "
                  f"(peak {max((r.get('prompt_tokens', 0) or 0) for r in ok) if ok else 0:,}).")
            continue
        parts = ", ".join(f"req {h['at']}: {h['from']:,} -> {h['to']:,}" for h in comp)
        print(f"- **{LABEL.get(a, a)}**: {len(comp)} - {parts}")

    print("\n## Prompt size per request\n")
    for a, ok, _ in detail:
        seq = [r.get("prompt_tokens", 0) or 0 for r in ok]
        print(f"- `{a}`: {seq}")


if __name__ == "__main__":
    sys.exit(main())
