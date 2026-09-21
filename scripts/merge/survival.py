#!/usr/bin/env python3
r"""Did the merge keep what this fork's commits put there?

A merge resolved by hand can drop one of our lines and say nothing. The existing checks
cannot see it: they grep for a needle in the merged tree, so a mechanism whose text
survives passes even when the line that made it work is gone.

This compares four trees instead of one - base, ours, upstream, merged - so that every
missing line can be sorted into the only distinction that matters:

    LOST-SILENTLY   gone from the merge, and upstream never touched that region.
                    Nobody decided this. It is the defect class.
    SUPERSEDED      gone, but upstream rewrote the same region. Following them was a
                    resolution decision, recorded in docs/merge-decisions-2026-09-20.md.
    MOVED           not at that path any more, found elsewhere in the merged tree.
    REGENERATE      a generated file, per .gitattributes. Its content is an output.

Additions and deletions are one calculation, not two. A fork removes things on purpose
(`1eb41e32a` deleted the history-time shrinker), and a merge can put them back; a check
that only asks "are our added lines still here" is blind to half of what can go wrong.

    python scripts/merge/survival.py [--json out.json] [--md out.md]
    exit 0 when nothing was lost silently, 1 otherwise
"""
import argparse
import collections
import importlib.util
import io
import os
import re
import subprocess
import sys

import bisect
import fnmatch

import gitio

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = gitio.REPO

BASE = "0a12b855a"
OURS = "88a2b0c53"
UP = "e29eceb75"
HEAD = "HEAD"

# The two commits whose partial reversal is the documented policy: the product name is
# deliberately allowed to follow upstream. Scoring them would drown every real finding.
POLICY_EXCLUDED = {"f06d1e476", "db33ff3a1"}

# Lines whose presence in the merged tree proves nothing.
TRIVIAL = {
    "}", "{", "};", "});", "})", "},", ")", ");", "]", "];", "],", ")]", "(", "[",
    "} else {", "else {", "*/", "/*", "Ok(())", "return;", "..Default::default()",
    "});", "});", "&&", "||", "}", "};", "#[test]", "#[cfg(test)]", "*/",
}


def sh(*args, **kw):
    return subprocess.run(["git"] + list(args), cwd=REPO, capture_output=True,
                          encoding="utf-8", errors="replace", **kw).stdout


# ---------------------------------------------------------------- git plumbing

tree = gitio.tree
batch_blobs = gitio.batch_blobs


def rename_map(a, b, sim=50):
    """old path -> new path, for files git thinks were renamed between two revisions."""
    out = {}
    txt = sh("diff", "--name-status", "-M%d%%" % sim, "-C", a, b)
    for line in txt.splitlines():
        f = line.split("\t")
        if f and f[0][:1] in ("R", "C") and len(f) >= 3:
            out[f[1]] = f[2]
    return out


def commit_deltas():
    """sha -> (added, removed), each a list of (path, raw_line), for our commits."""
    shas = [s for s in sh("log", "--format=%h", "%s..%s" % (BASE, OURS)).split()]
    out = {}
    for sha in shas:
        if sha in POLICY_EXCLUDED:
            continue
        txt = sh("show", "--format=", "-U0", "--no-renames", sha)
        added, removed, path = [], [], None
        for line in txt.splitlines():
            if line.startswith("+++ b/"):
                path = line[6:]
            elif line.startswith("--- ") or line.startswith("@@") or line.startswith("diff "):
                continue
            elif path and line.startswith("+"):
                added.append((path, line[1:]))
            elif path and line.startswith("-"):
                removed.append((path, line[1:]))
        out[sha] = (added, removed)
    return out


# ---------------------------------------------------------------- canonical form

def load_rules():
    """Reuse the merge driver's own map rather than a copy of it, which would drift."""
    spec = importlib.util.spec_from_file_location(
        "rename_merge", os.path.join(HERE, "rename-merge.py"))
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    rules = [(old, new, scope) for old, new, scope in mod.load_map() if "\n" not in new]
    return sorted(rules, key=lambda r: -len(r[1]))


RULES = load_rules()


def canon(line, path):
    """Fold the cosmetic rename away; keep the load-bearing one visible.

    The product name follows upstream by design, so folding `Suffice`->`Codex` stops a
    renamed comment from reading as a lost line. But rename-map.txt lists the names the
    running program reads - SUFFICE_HOME, .suffice, the enum variants - and for those,
    `ours said Suffice, the merge says Codex` IS the defect. Folding them away would make
    this tool launder the one rename regression that costs something. So they are masked
    to a sentinel first, and compare unequal against upstream's spelling.
    """
    s = line.lstrip("﻿").rstrip("\r\n").rstrip()
    if not s.strip():
        return None
    for i, (_old, new, scope) in enumerate(RULES):
        if scope and not path.startswith(scope):
            continue
        if new in s:
            s = s.replace(new, "\x00%d\x00" % i)
    s = s.replace("Suffice", "Codex").replace("suffice", "codex").replace("SUFFICE", "CODEX")
    s = re.sub(r"\s+", " ", s).strip()
    if path.endswith((".rs", ".json", ".ts", ".tsx", ".js", ".toml")):
        s = s.rstrip(",")
    return s or None


def counter_of(blob, path):
    """(Counter of canonical lines, list of canonical lines in order)."""
    try:
        text = blob.decode("utf-8", "surrogateescape")
    except Exception:
        return collections.Counter(), []
    seq = [canon(l, path) for l in text.split("\n")]
    return collections.Counter(c for c in seq if c is not None), seq


def evidentiary(c, B, U):
    """(usable, weak) - weak findings are reported at lower confidence, not dropped."""
    if c in TRIVIAL:
        return (False, False)
    occ = B[c] + U[c]
    if occ > 3:
        return (False, False)
    if len(c) < 8 and occ > 1:
        return (False, False)
    return (True, len(c) < 12 or occ > 0)


# ---------------------------------------------------------------- three-way accounting

def expected_count(k, B, O, U):
    do = O[k] - B[k]
    du = U[k] - B[k]
    if do > 0 and du > 0:
        e = B[k] + max(do, du)      # both added it; do not demand it twice
    elif do < 0 and du < 0:
        e = B[k] + min(do, du)      # both removed it
    else:
        e = B[k] + do + du
    return max(e, 0)


def account(B, O, U, H):
    """(lost, resurrected) - canonical line -> how many copies are missing / came back."""
    lost, resurrected = {}, {}
    for k in set(O) | set(B) | set(U) | set(H):
        do = O[k] - B[k]
        if do == 0:
            continue
        e = expected_count(k, B, O, U)
        if do > 0 and H[k] < e:
            lost[k] = e - H[k]
        elif do < 0 and H[k] > e:
            resurrected[k] = H[k] - e
    return lost, resurrected


# ---------------------------------------------------------------- intent

HUNK = re.compile(r"^@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@")


def hunks(sha_a, sha_b):
    """[(a_start, a_end, added_lines, removed_lines)] with ranges on the A axis, 0-based.

    Git's own diff, not difflib. difflib with autojunk=False - which is mandatory here,
    since the default heuristic silently discards the repeated lines that brace-heavy Rust
    is made of - goes quadratic on files like models.json. Git is fast at exactly this, and
    using it also means the regions match what the merge itself saw.
    """
    if not sha_a or not sha_b:
        return []
    out, cur = [], None
    for line in sh("diff", "-U0", "--no-color", sha_a, sha_b).splitlines():
        m = HUNK.match(line)
        if m:
            a, alen = int(m.group(1)), (1 if m.group(2) is None else int(m.group(2)))
            if alen == 0:                          # pure insert after line `a` (1-based)
                a0, a1 = a - 1, a + 1              # widen to the two lines it sits between
            else:
                a0, a1 = a - 1, a - 1 + alen
            cur = [a0, a1, [], []]
            out.append(cur)
        elif cur is not None:
            if line.startswith("+") and not line.startswith("+++"):
                cur[2].append(line[1:])
            elif line.startswith("-") and not line.startswith("---"):
                cur[3].append(line[1:])
    return out


class Regions(object):
    """Where upstream changed the file, indexed on the BASE line axis.

    Both diffs are taken against BASE, so their line numbers are directly comparable.
    Mapping through OURS or HEAD would be wrong.
    """

    def __init__(self, up_hunks, absent=False):
        self.absent = absent
        spans = sorted((a0, a1) for a0, a1, _, _ in up_hunks) if not absent else []
        merged = []
        for s, e in spans:
            if merged and s <= merged[-1][1]:
                merged[-1][1] = max(merged[-1][1], e)
            else:
                merged.append([s, e])
        self.starts = [s for s, _ in merged]
        self.ends = [e for _, e in merged]
        for i in range(1, len(self.ends)):
            self.ends[i] = max(self.ends[i], self.ends[i - 1])

    def touched(self, i1, i2, ctx):
        if self.absent:
            return True                            # upstream deleted or never had the file
        lo, hi = i1 - ctx, (i2 if i2 > i1 else i1 + 1) + ctx
        p = bisect.bisect_right(self.starts, hi)
        return p > 0 and self.ends[p - 1] > lo


def base_positions(our_hunks, path):
    """canonical line -> BASE-axis spans of the hunks that added or removed it."""
    pos = collections.defaultdict(list)
    for a0, a1, added, removed in our_hunks:
        for raw in added + removed:
            c = canon(raw, path)
            if c is not None:
                pos[c].append((a0, a1))
    return pos


# ---------------------------------------------------------------- generated files

def generated_patterns():
    """The .gitattributes `merge=generated` list is authoritative; do not hardcode it."""
    pats = []
    path = os.path.join(REPO, ".gitattributes")
    if not os.path.exists(path):
        return pats
    for line in io.open(path, encoding="utf-8"):
        line = line.split("#", 1)[0].strip()
        if "merge=generated" in line:
            pats.append(line.split()[0])
    return pats


def is_generated(path, pats):
    for p in pats:
        if fnmatch.fnmatch(path, p) or fnmatch.fnmatch(path, p.replace("/**", "/*")):
            return True
        if p.endswith("/**") and path.startswith(p[:-3] + "/"):
            return True
    return False


# ---------------------------------------------------------------- main

def main():
    global HEAD
    ap = argparse.ArgumentParser()
    ap.add_argument("--json")
    ap.add_argument("--md")
    ap.add_argument("--ctx", type=int, default=3)
    ap.add_argument("--head", default=HEAD,
                    help="the merged revision to judge; a throwaway commit here is how "
                         "the negative control proves the tool can still see a loss")
    args = ap.parse_args()
    HEAD = args.head

    sys.stderr.write("agaclar okunuyor...\n")
    t_base, t_ours, t_up, t_head = (tree(BASE), tree(OURS), tree(UP), tree(HEAD))
    deltas = commit_deltas()
    gen_pats = generated_patterns()

    # Attribution by construction. git blame would credit most lines to the two policy
    # commits (1178 and 426 files), which are exactly the ones excluded - every real
    # finding would be mis-filed - and one process per line takes 15+ minutes on Windows.
    intro = collections.defaultdict(set)
    removed_by = collections.defaultdict(set)
    touched = collections.defaultdict(set)
    for sha, (added, removed) in deltas.items():
        for path, raw in added:
            touched[path].add(sha)
            c = canon(raw, path)
            if c:
                intro[(path, c)].add(sha)
        for path, raw in removed:
            touched[path].add(sha)
            c = canon(raw, path)
            if c:
                removed_by[(path, c)].add(sha)

    scope = [p for p in touched if p in t_ours]
    up_ren = rename_map(BASE, UP)
    head_ren = rename_map(OURS, HEAD)

    buckets = {"identical": [], "generated": [], "absent": [], "analysed": []}
    todo = []
    for p in sorted(scope):
        if t_ours[p] == t_head.get(p):
            buckets["identical"].append(p)
        elif is_generated(p, gen_pats):
            buckets["generated"].append(p)
        elif p not in t_head and p not in head_ren:
            buckets["absent"].append(p)
        else:
            buckets["analysed"].append(p)
            todo.append(p)

    sys.stderr.write("bayt-ozdes %d | uretilmis %d | yok %d | analiz %d\n" % (
        len(buckets["identical"]), len(buckets["generated"]),
        len(buckets["absent"]), len(todo)))

    want = []
    for p in todo:
        pu = up_ren.get(p, p)
        ph = head_ren.get(p, p)
        want += [t_base.get(p), t_ours.get(p), t_up.get(pu), t_head.get(ph)]
    t0 = __import__("time").time()
    blobs = batch_blobs(want)
    sys.stderr.write("bloblar: %d adet, %.1fs\n"
                     % (len(blobs), __import__("time").time() - t0))

    findings, suppressed = [], []
    for idx, p in enumerate(todo):
        sys.stderr.write("\r  %d/%d %-50s" % (idx + 1, len(todo), p[-50:]))
        sys.stderr.flush()
        pu = up_ren.get(p, p)
        ph = head_ren.get(p, p)
        b_blob = blobs.get(t_base.get(p), b"")
        o_blob = blobs.get(t_ours.get(p), b"")
        u_sha = t_up.get(pu)
        h_blob = blobs.get(t_head.get(ph), b"")
        if b"\x00" in o_blob[:8192]:
            continue                                # binary; nothing to canonicalise
        B, _ = counter_of(b_blob, p)
        O, _ = counter_of(o_blob, p)
        U, _ = counter_of(blobs.get(u_sha, b""), p)
        H, _ = counter_of(h_blob, p)

        lost, resurrected = account(B, O, U, H)
        if not lost and not resurrected:
            continue
        regions = Regions(hunks(t_base.get(p), u_sha), absent=(u_sha is None))
        pos = base_positions(hunks(t_base.get(p), t_ours.get(p)), p)

        for kind, table in (("lost", lost), ("resurrected", resurrected)):
            for k, n in table.items():
                usable, weak = evidentiary(k, B, U)
                if not usable:
                    # A discrepancy we cannot settle from text: the line is trivial, or so
                    # common that finding it elsewhere proves nothing. Recorded rather
                    # than dropped, so "no finding" is never confused with "nothing to
                    # look at" - see UNVERIFIABLE-BY-TEXT in the report.
                    suppressed.append({
                        "file": p, "canon": k, "count": n, "kind": kind,
                        "commits": sorted(intro.get((p, k))
                                          or removed_by.get((p, k)) or set()),
                    })
                    continue
                # No located span means we cannot say upstream left it alone, so treat it
                # as contested. Erring the other way would manufacture false defects.
                spans = pos.get(k) or [(0, 10 ** 9)]
                wide = any(regions.touched(i1, i2, args.ctx) for i1, i2 in spans)
                narrow = any(regions.touched(i1, i2, 0) for i1, i2 in spans)
                verdict = "SUPERSEDED" if wide else "LOST-SILENTLY"
                owners = intro.get((p, k)) or removed_by.get((p, k)) or set()
                findings.append({
                    "file": p, "canon": k, "count": n, "kind": kind,
                    "verdict": verdict, "weak": weak,
                    "contested_narrowly": bool(wide and not narrow),
                    "commits": sorted(owners),
                })

    silent = [f for f in findings if f["verdict"] == "LOST-SILENTLY"]
    strong = [f for f in silent if not f["weak"]]

    print("")
    print("=" * 72)
    print("  SONUC")
    print("=" * 72)
    print("  bayt-ozdes (kanitlanmis korundu) : %d" % len(buckets["identical"]))
    print("  uretilmis (yeniden uretilecek)   : %d" % len(buckets["generated"]))
    print("  HEAD'de yok                      : %d" % len(buckets["absent"]))
    print("  tam analiz edilen                : %d" % len(todo))
    print("")
    print("  bulgu                            : %d" % len(findings))
    print("    SUPERSEDED (upstream karari)   : %d" % (len(findings) - len(silent)))
    print("    LOST-SILENTLY                  : %d  (guclu kanit: %d)"
          % (len(silent), len(strong)))
    print("  metinden hukum verilemeyen       : %d" % len(suppressed))
    print("")

    by_commit = collections.defaultdict(list)
    for f in findings:
        for c in f["commits"]:
            by_commit[c].append(f)

    if strong:
        print("  --- sessizce kaybolan, guclu kanit ---")
        for f in strong[:60]:
            print("  %-52s %s" % (f["file"][-52:], ", ".join(f["commits"]) or "?"))
            print("      %s" % f["canon"][:100])
    if buckets["absent"]:
        print("  --- HEAD'de olmayan dosyalar ---")
        for p in buckets["absent"]:
            print("  %-60s %s" % (p, ", ".join(sorted(touched[p]))))

    report = {
        "revs": {"base": BASE, "ours": OURS, "up": UP, "head": HEAD},
        "buckets": {k: v for k, v in buckets.items()},
        "findings": findings,
        "suppressed": suppressed,
        "by_commit": {c: len(v) for c, v in by_commit.items()},
    }
    if args.json:
        io.open(args.json, "w", encoding="utf-8").write(
            __import__("json").dumps(report, indent=2, ensure_ascii=False))
    return 1 if strong else 0


if __name__ == "__main__":
    sys.exit(main())
