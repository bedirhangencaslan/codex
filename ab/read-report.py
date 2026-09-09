"""Price a set of repeated single-agent runs and read the mechanism behind the price.

Cost follows request count almost exactly, and request count has a spread wider than any effect
worth looking for - the recorded read runs ranged 12 to 23 for the same deliverable. So this prints
the median and the full spread rather than a mean, and alongside them the read-call shape, whose
variance is far lower: how many files per call, how many calls carried a window, and how many
single-path calls carried an offset. That last one is the window-chasing signature the per-file
range was added to remove.

    python read-report.py logs-read [logs-base ...]
"""

import glob
import json
import os
import statistics
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from ablib import CACHED, FRESH, OUTPUT, table  # noqa: E402


def relay_rows(path):
    rows = []
    with open(path, encoding="utf-8") as handle:
        for line in handle:
            line = line.strip()
            if line:
                try:
                    rows.append(json.loads(line))
                except ValueError:
                    pass
    return rows


def price(rows):
    ok = [r for r in rows if r.get("status") == 200]
    prompt = sum(r.get("prompt_tokens", 0) or 0 for r in ok)
    cached = sum(r.get("cached_tokens", 0) or 0 for r in ok)
    output = sum(r.get("completion_tokens", 0) or 0 for r in ok)
    fresh = max(0, prompt - cached)
    return {
        "requests": len(ok),
        "bad": len(rows) - len(ok),
        "fresh": fresh,
        "cached": cached,
        "output": output,
        "peak": max((r.get("prompt_tokens", 0) or 0) for r in ok) if ok else 0,
        "compactions": compactions(ok),
        "cost": (fresh * FRESH + cached * CACHED + output * OUTPUT) / 1e6,
    }


def compactions(rows, drop=0.30):
    hits, previous = 0, 0
    for row in rows:
        tokens = row.get("prompt_tokens", 0) or 0
        if previous and tokens < previous * (1 - drop):
            hits += 1
        if tokens:
            previous = tokens
    return hits


def read_calls(rollout):
    """Per-call `paths` length, whether the call carried a window, whether it was a lone offset."""
    calls = []
    if not os.path.exists(rollout):
        return calls
    with open(rollout, encoding="utf-8") as handle:
        for line in handle:
            try:
                record = json.loads(line)
            except ValueError:
                continue
            payload = record.get("payload") or {}
            if payload.get("type") != "function_call" or payload.get("name") != "read":
                continue
            try:
                args = json.loads(payload.get("arguments") or "{}")
            except ValueError:
                args = {}
            entries = args.get("paths") or args.get("files") or []
            if isinstance(entries, (str, dict)):
                entries = [entries]
            windows = sum(
                1 for e in entries if isinstance(e, dict) and ("offset" in e or "limit" in e)
            )
            broadcast = args.get("offset") is not None or args.get("limit") is not None
            calls.append(
                {
                    "paths": len(entries),
                    "windows": windows or (len(entries) if broadcast else 0),
                    "lone_offset": len(entries) <= 1 and args.get("offset") is not None,
                }
            )
    return calls


def summarise(directory):
    rows = []
    for relay in sorted(glob.glob(os.path.join(directory, "*.jsonl"))):
        if relay.endswith(".rollout.jsonl"):
            continue
        stats = price(relay_rows(relay))
        stats["name"] = os.path.basename(relay)[: -len(".jsonl")]
        stats["calls"] = read_calls(relay[: -len(".jsonl")] + ".rollout.jsonl")
        rows.append(stats)
    return rows


def main(directories):
    for directory in directories:
        runs = summarise(directory)
        if not runs:
            print(f"# {directory}: no runs found")
            continue

        print(f"\n# {directory} - {len(runs)} run(s)\n")
        print(
            table(
                ["run", "req", "compact", "peak", "fresh in", "cached in", "output", "cost", "bad"],
                [
                    [
                        run["name"],
                        str(run["requests"]),
                        str(run["compactions"]),
                        f"{run['peak']:,}",
                        f"{run['fresh']:,}",
                        f"{run['cached']:,}",
                        f"{run['output']:,}",
                        f"**${run['cost']:.4f}**",
                        str(run["bad"]) if run["bad"] else "-",
                    ]
                    for run in runs
                ],
            )
        )

        costs = sorted(run["cost"] for run in runs)
        requests = sorted(run["requests"] for run in runs)
        print(
            f"\n**cost** median ${statistics.median(costs):.4f}, "
            f"spread ${costs[0]:.4f}-${costs[-1]:.4f}"
        )
        print(
            f"**requests** median {statistics.median(requests):.0f}, "
            f"spread {requests[0]}-{requests[-1]}"
        )

        print("\n## read calls\n")
        for run in runs:
            calls = run["calls"]
            if not calls:
                print(f"- `{run['name']}`: no rollout archived")
                continue
            sizes = [call["paths"] for call in calls]
            print(
                f"- `{run['name']}`: {sizes} - {sum(sizes)} files in {len(calls)} calls, "
                f"{sum(1 for c in calls if c['windows'])} carried a window, "
                f"**{sum(1 for c in calls if c['lone_offset'])} lone-offset calls**"
            )


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:] or ["logs-read"]))
