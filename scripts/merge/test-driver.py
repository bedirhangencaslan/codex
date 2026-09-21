#!/usr/bin/env python3
r"""Replay the 2026-09-20 merge through the driver and report what it would have done.

That merge is the only honest test available: its 256 conflicts are real, and the committed
result is a decision a person made for each one. Three outcomes are interesting.

    temiz + ayni     the driver resolves it and lands on what was committed. Saved work.
    temiz + farkli   it resolves it differently. Worth reading - it may be a policy
                     difference (the product name follows upstream) or a defect.
    catismali        it leaves markers. A real disagreement; the driver is not supposed
                     to decide those and does not pretend to.

    python scripts/merge/test-driver.py [--diff]
"""
import io
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import importlib.util

spec = importlib.util.spec_from_file_location("rm", os.path.join(HERE, "rename-merge.py"))
rm = importlib.util.module_from_spec(spec)
spec.loader.exec_module(rm)

REPO = os.path.abspath(os.path.join(HERE, "..", ".."))
BASE, OURS, THEIRS, RESULT = "0a12b855a", "88a2b0c53", "e29eceb75", "658ddfd66"


def git(*a):
    return subprocess.run(["git"] + list(a), cwd=REPO, capture_output=True).stdout


def blob(rev, path):
    r = subprocess.run(["git", "show", "%s:%s" % (rev, path)], cwd=REPO,
                       capture_output=True)
    return r.stdout if r.returncode == 0 else None


conflicted = []
out = git("merge-tree", "--write-tree", "--name-only", OURS, THEIRS).decode("utf-8", "replace")
for line in out.splitlines()[1:]:
    if not line.strip():
        break
    conflicted.append(line.strip())

same = diff = still = skipped = 0
diffs = []
for p in conflicted:
    b, o, t, r = blob(BASE, p), blob(OURS, p), blob(THEIRS, p), blob(RESULT, p)
    if b is None or o is None or t is None:
        skipped += 1          # add/add or delete; the driver never sees these
        continue
    merged, clean = rm.merge(b, o, t, "7", p)
    if not clean:
        still += 1
    elif r is not None and merged == r:
        same += 1
    else:
        diff += 1
        diffs.append(p)

total = same + diff + still
print("2026-09-20 merge'inin %d catismasi, surucuden gecirildi" % len(conflicted))
print("  surucunun gordugu (uc tarafi da olan)  : %d" % total)
print("  atlanan (ekle/ekle veya silme)         : %d" % skipped)
print()
print("  temiz cozdu, commit'le AYNI            : %d" % same)
print("  temiz cozdu, commit'ten FARKLI         : %d" % diff)
print("  catismali biraktigi (gercek karar)     : %d" % still)
if total:
    print()
    print("  elle dokunulmasi gerekmeyecek olan     : %d / %d  (%%%d)"
          % (same, total, 100 * same // total))

if "--diff" in sys.argv and diffs:
    print("\n--- farkli cozdukleri ---")
    for p in diffs[:40]:
        print("   %s" % p)
