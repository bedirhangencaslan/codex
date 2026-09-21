#!/usr/bin/env python3
r"""The half of the rename a merge driver cannot do. Run after every merge.

A merge driver only sees files that needed merging. A file upstream added arrives whole,
without a merge, so nothing folds or restores it - and if it names one of the identifiers
this fork renamed, the tree ends up with the declaration saying one thing and the new file
saying another. That does not conflict and does not warn; it fails at the compiler, after
everything else looks finished.

Found that way: after a trial merge, `ToolError` was consistent but three of upstream's new
test files still said `Product::Codex` while the enum declared `Suffice`.

So this sweeps the whole tree with the same map the driver uses, and only with the entries
that carry function. The product name is left alone - it is allowed to follow upstream, which
is what keeps it out of the next merge's conflicts.

    python scripts/merge/post-merge.py            # report only
    python scripts/merge/post-merge.py --apply
"""
import io
import os
import subprocess
import sys
import importlib.util

HERE = os.path.dirname(os.path.abspath(__file__))
spec = importlib.util.spec_from_file_location("rm", os.path.join(HERE, "rename-merge.py"))
rm = importlib.util.module_from_spec(spec)
spec.loader.exec_module(rm)

REPO = subprocess.run(["git", "rev-parse", "--show-toplevel"], cwd=HERE,
                      capture_output=True, encoding="utf-8").stdout.strip()
APPLY = "--apply" in sys.argv

# `scripts/merge` excludes itself: the map's left-hand sides are the very strings being
# replaced, so a sweep over it rewrites the rules into `SUFFICE_HOME => SUFFICE_HOME`.
SKIP_DIRS = {".git", "node_modules", "target", "__pycache__", "_labs"}
SKIP_PREFIX = ("scripts/merge/",)
SKIP_EXT = {".zst", ".png", ".jpg", ".gif", ".ico", ".woff", ".woff2", ".snap"}

files = subprocess.run(["git", "ls-files"], cwd=REPO, capture_output=True,
                       encoding="utf-8").stdout.splitlines()

changed = {}
for rel in files:
    if any(part in SKIP_DIRS for part in rel.split("/")) or rel.startswith(SKIP_PREFIX):
        continue
    if os.path.splitext(rel)[1] in SKIP_EXT:
        continue
    path = os.path.join(REPO, rel.replace("/", os.sep))
    try:
        with io.open(path, "rb") as fh:
            before = fh.read()
    except (OSError, IOError):
        continue
    if b"odex" not in before and b"ODEX" not in before:
        continue
    after = rm.restore(before, rel)
    if after != before:
        n = sum(1 for a, b in zip(before.splitlines(), after.splitlines()) if a != b)
        changed[rel] = n
        if APPLY:
            with io.open(path, "wb") as fh:
                fh.write(after)

if not changed:
    print("degisecek bir sey yok - agac zaten tutarli.")
    sys.exit(0)

print("%d dosyada %d satir%s" % (len(changed), sum(changed.values()),
                                 " duzeltildi" if APPLY else " duzeltilecek"))
for rel, n in sorted(changed.items(), key=lambda kv: -kv[1])[:40]:
    print("   %-72s %d" % (rel[:70], n))
if not APPLY:
    print("\n(rapor - uygulamak icin --apply)")
