"""Pool every simulator rep and report each arm's rate of the unbounded reading mode.

The measured outcome is bimodal, not continuous: on any given rep the model either bounds its reads
(~3-5 KB a file) or does not (~9-12 KB, the whole file). So the statistic that matters is how often
an arm lands in the unbounded mode, and a mean over reps hides it.
"""

import collections
import glob
import io
import json
import statistics
import sys

UNBOUNDED_BYTES = 8000

rows = []
for path in sys.argv[1:] or glob.glob("*.jsonl"):
    if "relay" in path:
        continue
    for line in io.open(path, encoding="utf-8"):
        line = line.strip()
        if line:
            rows.append(json.loads(line))

by_arm = collections.defaultdict(list)
for r in rows:
    by_arm[(r["arm"].split("#")[0])].append(r)

print(f"{'arm':34} {'reps':>4} {'unbounded':>10} {'B/read med':>11} {'reads':>6} {'cost':>9}")
out = []
for arm, reps in by_arm.items():
    usable = [r for r in reps if r.get("read_calls")]
    if not usable:
        continue
    bpr = [r.get("bytes_per_read", 0) for r in usable if r.get("bytes_per_read")]
    if not bpr:
        continue
    unb = sum(1 for v in bpr if v >= UNBOUNDED_BYTES)
    out.append((unb / len(bpr), arm, len(reps), unb, len(bpr),
                statistics.median(bpr), sum(r["read_calls"] for r in usable),
                sum(r["cost"] for r in reps)))

for rate, arm, nreps, unb, n, med, reads, cost in sorted(out):
    print(f"{arm:34} {nreps:>4} {f'{unb}/{n}':>10} {med:>11.0f} {reads:>6} ${cost:>8.4f}")

print()
print(f"total spent in these files: ${sum(r['cost'] for r in rows):.4f}  ({len(rows)} reps)")
