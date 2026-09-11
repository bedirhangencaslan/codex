"""What OpenCode's own session store recorded for a real run, turn by turn.

The simulator's OpenCode baseline goes unbounded on about one read batch in three; the real agent,
across six recorded runs, never did - 132 of 132 reads carried an explicit window. So the simulator
is missing something the real loop has. This prints the real conversation's shape so the two can be
compared without spending a request.
"""

import json
import os
import shutil
import sqlite3
import sys
import tempfile

DB = os.path.expanduser(r"~\.local\share\opencode\opencode.db")


def snapshot(db):
    tmp = tempfile.mkdtemp(prefix="ocdb-")
    for suffix in ("", "-wal", "-shm"):
        src = db + suffix
        if os.path.exists(src):
            shutil.copy2(src, os.path.join(tmp, os.path.basename(db) + suffix))
    return os.path.join(tmp, os.path.basename(db))


def main():
    want = sys.argv[1] if len(sys.argv) > 1 else "wire-stock-rep1"
    con = sqlite3.connect(snapshot(DB))

    names = [r[0] for r in con.execute("select name from sqlite_master where type='table'")]
    print("tables:", names)

    sid = None
    for i, d, _t in con.execute("select id, directory, title from session"):
        if want in (d or ""):
            sid = i
            print("session:", i, d)
            break
    if not sid:
        print("no session matching", want)
        return 1

    msgs = list(
        con.execute(
            "select id, data from message where session_id=? order by time_created",
            (sid,),
        )
    )
    print("messages:", len(msgs))
    for mid, mdata in msgs:
        role = (json.loads(mdata) or {}).get("role", "?")
        parts = [json.loads(d) for (d,) in con.execute("select data from part where message_id=?", (mid,))]
        summary = []
        for p in parts:
            kind = p.get("type")
            if kind == "tool":
                state = p.get("state") or {}
                summary.append("tool:%s(%s)" % (p.get("tool"), json.dumps(state.get("input") or {})[:120]))
            elif kind == "text":
                summary.append("text:%d" % len(p.get("text") or ""))
            elif kind == "reasoning":
                summary.append("reasoning:%d" % len(p.get("text") or ""))
            else:
                summary.append(kind or "?")
        print(f"  {role:9} parts={len(parts)}")
        for s in summary:
            print("      ", s[:150])
    return 0


if __name__ == "__main__":
    sys.exit(main())
