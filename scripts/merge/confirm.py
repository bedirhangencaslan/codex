#!/usr/bin/env python3
r"""Positive confirmation: are our lines in the merged tree, commit by commit?

`survival.py` reports discrepancies, so "no finding" there means "nothing went wrong that
the method can see". That is a weaker claim than "this commit's work is demonstrably
present", and reading the first as the second overstates the result.

This asks the other question directly. For each commit, of the lines it added that our
branch tip still had, how many are in the merged tree? Generated files are skipped
(their content is an output), as are trivial lines, and so are lines a later commit of
ours replaced - we are not owed those.

A commit can land in "nothing to confirm" honestly: fourteen of the prompt-tweak commits
rewrite the same one-line template inside models.json, so only the last one's version
survives to our branch tip and the earlier ones have nothing of their own left to check.

    python scripts/merge/confirm.py [--detail <sha> ...]
"""
import argparse
import collections
import importlib.util
import os
import sys

import gitio

HERE = os.path.dirname(os.path.abspath(__file__))

_spec = importlib.util.spec_from_file_location("survival", os.path.join(HERE, "survival.py"))
sv = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(sv)


def build():
    t_ours, t_head = gitio.tree(sv.OURS), gitio.tree(sv.HEAD)
    gen = sv.generated_patterns()
    deltas = sv.commit_deltas()

    paths = {p for _s, (a, _r) in deltas.items() for p, _l in a}
    paths = {p for p in paths if p in t_ours and not sv.is_generated(p, gen)}

    blobs = gitio.batch_blobs([t_ours[p] for p in paths]
                              + [t_head[p] for p in paths if p in t_head])
    ours_c, head_c = {}, {}
    for p in paths:
        ours_c[p] = sv.counter_of(blobs.get(t_ours[p], b""), p)[0]
        head_c[p] = (sv.counter_of(blobs.get(t_head.get(p), b""), p)[0]
                     if p in t_head else collections.Counter())

    rows = {}
    for sha, (added, _removed) in deltas.items():
        total, present, miss = 0, 0, collections.defaultdict(list)
        for p, raw in added:
            if p not in paths:
                continue
            c = sv.canon(raw, p)
            if not c or c in sv.TRIVIAL or ours_c[p][c] == 0:
                continue                     # trivial, or we replaced it ourselves later
            total += 1
            if head_c[p][c] > 0:
                present += 1
            else:
                miss[p].append(raw.strip())
        rows[sha] = (total, present, miss)
    return rows


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--detail", nargs="*", default=[])
    args = ap.parse_args()

    rows = build()
    full = [s for s, (t, p, _) in rows.items() if t and t == p]
    part = [s for s, (t, p, _) in rows.items() if t and p < t]
    none = [s for s, (t, _p, _) in rows.items() if not t]
    tot = sum(t for t, _p, _m in rows.values())
    pre = sum(p for _t, p, _m in rows.values())

    print("=" * 70)
    print("  POZITIF DOGRULAMA  (%s -> %s)" % (sv.OURS, sv.HEAD))
    print("=" * 70)
    print("  butun kanit satirlari dogrulanan : %d commit" % len(full))
    print("  kismen dogrulanan                : %d commit" % len(part))
    print("  dogrulanacak satiri olmayan      : %d commit" % len(none))
    print("")
    print("  kanit satiri                     : %d" % tot)
    print("  HEAD'de bulunan                  : %d  (%%%.2f)"
          % (pre, 100.0 * pre / max(tot, 1)))
    print("")
    for sha in sorted(part, key=lambda s: rows[s][1] - rows[s][0]):
        t, p, miss = rows[sha]
        print("  %s  %4d/%-4d  %s" % (sha, p, t, ", ".join(sorted(miss))[:56]))

    for sha in args.detail:
        t, p, miss = rows.get(sha, (0, 0, {}))
        print("")
        print("  --- %s  %d/%d ---" % (sha, p, t))
        for path, lines in sorted(miss.items()):
            print("    %s" % path)
            for line in lines:
                print("        %s" % line[:100])
    return 0


if __name__ == "__main__":
    sys.exit(main())
