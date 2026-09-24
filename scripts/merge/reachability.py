#!/usr/bin/env python3
r"""Find a mechanism that survived as text but lost everyone who called it.

Every existing check in this repo asks whether a name is present. None asks whether
anything still reaches it, so a merge that keeps `fn write_spill` and drops its only call
site reports success. The compiler will not complain either: dead private code is a
warning at most, and a `pub` item not even that.

The trigger is a delta, not an absolute. "Zero callers" on its own means nothing - plenty
of things are legitimately called only from tests, and upstream renaming a symbol drops
its callers to zero while the mechanism is fine. What is suspicious is losing the callers
it used to have:

    ORPHANED   defined at HEAD, >=1 external reference at OURS, 0 at HEAD
    WEAKENED   external references fell but did not reach zero

References are counted on comment- and string-stripped code (rustscan), outside test
files and `#[cfg(test)]` items, and outside the symbol's own defining file.

This tier REVIEWS, it does not FAIL: trait dispatch, macros and serde field names all
reference a symbol without a textual call site.

    python scripts/merge/reachability.py
"""
import collections
import os
import re
import subprocess
import sys

import gitio
import rustscan

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = gitio.REPO

BASE, OURS, HEAD = "0a12b855a", "88a2b0c53", "HEAD"
POLICY_EXCLUDED = {"f06d1e476", "db33ff3a1"}

DEF = re.compile(
    r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:default\s+)?(?:async\s+)?(?:unsafe\s+)?"
    r"(?:extern\s+\"[^\"]+\"\s+)?"
    r"(fn|struct|enum|trait|type|union|static|const)\s+([A-Za-z_][A-Za-z0-9_]*)")

TEST_PATH = re.compile(r"(^|/)(tests?)/|_tests?\.rs$|(^|/)(benches|examples)/")


def sh(*args):
    return subprocess.run(["git"] + list(args), cwd=REPO, capture_output=True,
                          encoding="utf-8", errors="replace").stdout


def introduced():
    """(symbol, defining path) pairs our commits added, excluding test files."""
    out = {}
    for sha in sh("log", "--format=%h", "%s..%s" % (BASE, OURS)).split():
        # `%h` grows with the repository; see the same check in survival.py.
        if any(sha.startswith(p) for p in POLICY_EXCLUDED):
            continue
        path = None
        for line in sh("show", "--format=", "-U0", "--no-renames", sha).splitlines():
            if line.startswith("+++ b/"):
                path = line[6:]
            elif path and line.startswith("+") and path.endswith(".rs"):
                if TEST_PATH.search(path):
                    continue
                m = DEF.match(line[1:])
                if m:
                    out.setdefault(m.group(2), path)
    return out


def blob(rev, path):
    r = subprocess.run(["git", "show", "%s:%s" % (rev, path)], cwd=REPO,
                       capture_output=True)
    return None if r.returncode else r.stdout.decode("utf-8", "replace")


def counts(rev, symbols, sources):
    """symbol -> number of code references outside its own file and outside tests.

    One alternation over all the names, run against comment- and string-stripped code.
    Testing 530 names against every line separately, or spawning `git show` per file,
    both turn this into minutes.
    """
    hits = collections.defaultdict(int)
    rx = re.compile(r"\b(%s)\b" % "|".join(re.escape(s) for s in sorted(symbols)))
    for path, src in sources.items():
        if TEST_PATH.search(path):
            continue
        if not rx.search(src):                  # cheap reject before the expensive scan
            continue
        lines = rustscan.scan(src)
        spans = rustscan.test_spans(src, lines)
        for no, line in lines:
            if not line.strip():
                continue
            if any(a <= no <= b for a, b in spans):
                continue
            for m in rx.finditer(line):
                s = m.group(1)
                if symbols.get(s) != path:
                    hits[s] += 1
    return hits


def defined_in(src, sym):
    return bool(re.search(
        r"(fn|struct|enum|trait|type|union|static|const)\s+%s\b" % re.escape(sym), src))


def main():
    sys.stderr.write("kaynaklar okunuyor...\n")
    src_ours = gitio.files_at(OURS, ".rs")
    src_head = gitio.files_at(HEAD, ".rs")

    sys.stderr.write("semboller cikariliyor...\n")
    syms = introduced()
    # Keep only those whose definition still stands at OURS; a symbol we introduced and
    # later removed ourselves is not the merge's doing.
    alive = {s: p for s, p in syms.items() if defined_in(src_ours.get(p, ""), s)}
    sys.stderr.write("izlenecek sembol: %d\n" % len(alive))

    sys.stderr.write("OURS sayiliyor...\n")
    a = counts(OURS, alive, src_ours)
    sys.stderr.write("HEAD sayiliyor...\n")
    b = counts(HEAD, alive, src_head)

    orphaned, weakened = [], []
    for s, p in sorted(alive.items()):
        na, nb = a.get(s, 0), b.get(s, 0)
        if not defined_in(src_head.get(p, ""), s):
            continue                    # a lost definition is survival.py's finding, not this one
        if na >= 1 and nb == 0:
            orphaned.append((s, p, na, nb))
        elif nb < na:
            weakened.append((s, p, na, nb))

    print("")
    print("=" * 74)
    print("  ULASILABILIRLIK  (inceleme katmani - hukum degil)")
    print("=" * 74)
    print("  izlenen sembol: %d" % len(alive))
    print("")
    print("  YETIM (tanim duruyor, cagiran kalmadi): %d" % len(orphaned))
    for s, p, na, nb in orphaned:
        print("    %-42s %s  (%d -> %d)" % (s, p[-34:], na, nb))
    print("")
    print("  ZAYIFLAMIS (cagiran sayisi dustu): %d" % len(weakened))
    for s, p, na, nb in weakened[:40]:
        print("    %-42s %s  (%d -> %d)" % (s, p[-34:], na, nb))
    return 0


if __name__ == "__main__":
    sys.exit(main())
