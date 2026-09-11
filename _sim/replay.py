"""The tool outputs a real run actually received, keyed by (tool, arguments).

`tools.py` re-implements each tool, and a re-implementation is a difference like any other. The
first one found: OpenCode's real `glob` answered 3,872 characters in filesystem order, while the
re-implementation answered 4,137 in alphabetical order - and that single result is the *only* input
separating turn one from turn two, the turn where the model chooses its window.

OpenCode stores every tool result in its session database, so an arm can be an exact replay for any
call the real run made, and fall back to the local implementation only for calls it did not.
"""

import json
import os
import shutil
import sqlite3
import tempfile

DB = os.path.expanduser(r"~\.local\share\opencode\opencode.db")


def _snapshot(db):
    tmp = tempfile.mkdtemp(prefix="ocdb-")
    for suffix in ("", "-wal", "-shm"):
        src = db + suffix
        if os.path.exists(src):
            shutil.copy2(src, os.path.join(tmp, os.path.basename(db) + suffix))
    return os.path.join(tmp, os.path.basename(db))


def _key(tool, args, retarget_from, retarget_to):
    """Arguments normalised so a replayed call matches whatever path the simulator is using."""
    norm = {}
    for k, v in sorted((args or {}).items()):
        if isinstance(v, str) and retarget_from:
            v = v.replace(retarget_from, retarget_to).replace(
                retarget_from.replace("\\", "/"), retarget_to.replace("\\", "/")
            )
        norm[k] = v
    return tool, json.dumps(norm, sort_keys=True)


class Store:
    def __init__(self, session_match, retarget_to, db=DB):
        self.rows = {}
        self.root = None
        con = sqlite3.connect(_snapshot(db))
        sid = None
        for i, d, _t in con.execute("select id, directory, title from session"):
            if session_match in (d or ""):
                sid, self.root = i, (d or "").replace("/", "\\")
                break
        if not sid:
            raise SystemExit(f"no opencode session matching {session_match!r}")
        for (data,) in con.execute("select data from part where session_id=?", (sid,)):
            j = json.loads(data)
            if j.get("type") != "tool":
                continue
            state = j.get("state") or {}
            out = state.get("output")
            if out is None:
                continue
            k = _key(j.get("tool"), state.get("input"), self.root, retarget_to)
            self.rows.setdefault(k, []).append(
                out.replace(self.root, retarget_to).replace(
                    self.root.replace("\\", "/"), retarget_to.replace("\\", "/")
                )
            )
        self.retarget_to = retarget_to
        self.hits = 0
        self.misses = 0

    def get(self, tool, args, glob_order=None):
        k = _key(tool, args, self.root, self.retarget_to)
        seq = self.rows.get(k)
        if not seq:
            self.misses += 1
            return None
        self.hits += 1
        out = seq[0] if len(seq) == 1 else seq.pop(0)
        if tool == "glob" and glob_order == "sorted":
            # Same files, same byte count, alphabetical instead of walk order. OpenCode streams
            # `rg`'s walk order untouched; this fork's `search.rs:176` sorts, for determinism. That
            # is the only difference this arm makes, and it is the one the re-implementation made
            # by accident.
            lines = [l for l in out.splitlines() if l.strip()]
            out = "\n".join(sorted(lines))
        return out
