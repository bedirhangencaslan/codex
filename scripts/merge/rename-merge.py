#!/usr/bin/env python3
r"""Merge driver that stops this fork's rename from causing conflicts.

Git calls this with the three sides of one file. Instead of merging our text against
upstream's directly - where every `Suffice` meets a `Codex` and conflicts - it speaks our side
back in upstream's words first, merges, and then restores only the renames that carry function.

    1. fold ours:   Suffice -> Codex, suffice -> codex, SUFFICE -> CODEX
    2. merge base / folded-ours / theirs the ordinary way
    3. restore the pairs in rename-map.txt, which are the names the running program reads

The product name is left as upstream writes it. That is the point: it is what made roughly
nine of every ten conflicts in the 2026-09-20 merge, it costs nothing to give up, and giving
it up is what makes those conflicts stop happening.

A real disagreement - both sides changing the same logic - still conflicts, with markers, as
it should. This driver removes the noise, not the decisions.

    usage: rename-merge.py %O %A %B %L %P
           %O base   %A ours (and the file to write)   %B theirs   %L marker size   %P path

Exit 0 means merged cleanly; non-zero means conflict markers were written, and git reports it.
"""
import io
import os
import re
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
MAP = os.path.join(HERE, "rename-map.txt")
CASES = [("Suffice", "Codex"), ("suffice", "codex"), ("SUFFICE", "CODEX")]


def read(path):
    with io.open(path, "rb") as fh:
        return fh.read()


def write(path, data):
    with io.open(path, "wb") as fh:
        fh.write(data)


def load_map():
    """(pattern, replacement, path-prefix) triples, longest first."""
    rules = []
    if not os.path.exists(MAP):
        return rules
    for line in io.open(MAP, encoding="utf-8"):
        line = line.rstrip("\r\n")
        # A comment is `#` followed by space or end of line. Not just `#`: a rule may have
        # to start with one, because `#[serde(alias = "CODEX")]` is the anchor that makes a
        # bare `Codex,` identifiable.
        stripped = line.strip()
        if not stripped or stripped == "#" or stripped.startswith("# ") or "=>" not in stripped:
            continue
        line = stripped
        body, _, scope = line.partition("|")
        old, _, new = body.partition("=>")
        old, new, scope = old.strip(), new.strip(), scope.strip()
        # `\n` so a rule can anchor on the line above. An enum's declaration is just
        # `Codex,`, which is too generic to replace on its own even inside one file;
        # with the attribute above it, it is exactly one place.
        old, new = old.replace("\\n", "\n"), new.replace("\\n", "\n")
        if old and new:
            rules.append((old, new, scope or None))
    return sorted(rules, key=lambda r: -len(r[0]))


def fold(text):
    for new, old in CASES:
        text = text.replace(new.encode(), old.encode())
    return text


def restore(text, path):
    for old, new, scope in load_map():
        if scope and not path.replace(os.sep, "/").startswith(scope):
            continue
        if old.startswith("."):
            # A path segment, not any occurrence: `com.openai.codex` is a bundle id
            # registered with the OS, not our config directory, and rewriting it would be
            # a real breakage wearing a rename's clothes.
            pat = re.compile(rb"(?<![A-Za-z0-9_])" + re.escape(old.encode())
                             + rb"(?=[/\\\"'\s)\],]|$)")
            text = pat.sub(new.encode(), text)
        elif re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", old):
            # A bare word: match it whole, so `Codex` never fires inside `AsyncCodex`.
            text = re.sub(rb"\b" + re.escape(old.encode()) + rb"\b(?=[^A-Za-z0-9_]|$)",
                          new.encode(), text)
        else:
            text = text.replace(old.encode(), new.encode())
    return text


def merge(base, ours, theirs, marker, path):
    """Returns (merged bytes, clean?). Pure function - writes nothing."""
    import tempfile

    folded = fold(ours)
    if folded == ours and fold(theirs) == theirs:
        # Nothing of the rename here; let git's own merge decide, unchanged.
        pass
    tmp = tempfile.mkdtemp(prefix="rename-merge-")
    try:
        po = os.path.join(tmp, "ours")
        pb = os.path.join(tmp, "base")
        pt = os.path.join(tmp, "theirs")
        write(po, folded)
        write(pb, base)
        write(pt, theirs)
        cmd = ["git", "merge-file", "-L", "ours", "-L", "base", "-L", "theirs"]
        if marker:
            cmd.append("--marker-size=%s" % marker)
        cmd += [po, pb, pt]
        rc = subprocess.run(cmd, capture_output=True).returncode
        merged = read(po)
    finally:
        for f in ("ours", "base", "theirs"):
            try:
                os.remove(os.path.join(tmp, f))
            except OSError:
                pass
        try:
            os.rmdir(tmp)
        except OSError:
            pass
    return restore(merged, path), rc == 0


def main(argv):
    if len(argv) < 4:
        sys.stderr.write(__doc__)
        return 2
    o, a, b = argv[0], argv[1], argv[2]
    marker = argv[3] if len(argv) > 3 else None
    path = argv[4] if len(argv) > 4 else a
    merged, clean = merge(read(o), read(a), read(b), marker, path)
    write(a, merged)
    return 0 if clean else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
