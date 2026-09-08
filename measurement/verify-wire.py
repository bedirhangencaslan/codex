"""Check whether each agent's WIRE.md citations point at lines that exist.

All five agents produced a WIRE.md with 44 file:line citations, so completion alone does not
separate them. A citation that names a real file and a line inside it is evidence the agent read
the file; one that overshoots the file's length, or names a file that is not there, is not.
"""

import os
import re
import sys

BSL = chr(92)
PAT = re.compile(r"([A-Za-z0-9_./" + BSL + r"-]+\.rs)[:\s]+(?:line\s*)?(\d+)")
AGENTS = ("suffice", "basefix", "base", "cline", "opencode")


_INDEX = {}


def index_for(root):
    """basename -> first matching path, so a citation may name a file without its directory."""
    if root not in _INDEX:
        found = {}
        for dirpath, _dirs, files in os.walk(os.path.join(root, "codex-api")):
            for f in files:
                found.setdefault(f, os.path.join(dirpath, f))
        _INDEX[root] = found
    return _INDEX[root]


def resolve(root, rel):
    rel = rel.replace(BSL, "/").lstrip("./")
    for cand in (
        os.path.join(root, rel),
        os.path.join(root, "codex-api", rel),
    ):
        if os.path.exists(cand):
            return cand
    # Agents cite `chat.rs:41` as often as `codex-api/src/requests/chat.rs:41`; a bare basename is
    # still a real citation, and failing it would penalise a format choice rather than the work.
    return index_for(root).get(os.path.basename(rel))


def main():
    for a in AGENTS:
        root = "par-" + a
        f = os.path.join(root, "WIRE.md")
        if not os.path.exists(f):
            print(f"  {a:9s} WIRE.md yok")
            continue
        txt = open(f, encoding="utf-8", errors="replace").read()
        ok = over = missing = 0
        for m in PAT.finditer(txt):
            p = resolve(root, m.group(1))
            if p is None:
                missing += 1
                continue
            n = sum(1 for _ in open(p, encoding="utf-8", errors="replace"))
            if 1 <= int(m.group(2)) <= n:
                ok += 1
            else:
                over += 1
        tot = ok + over + missing
        rate = f"{100.0 * ok / tot:.0f}%" if tot else "-"
        print(f"  {a:9s} atıf={tot:3d}  geçerli={ok:3d} ({rate})  "
              f"satır dosyayı aşıyor={over:3d}  dosya yok={missing:3d}")


if __name__ == "__main__":
    sys.exit(main())
