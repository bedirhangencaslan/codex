"""Map every captured request body, and diff the real second request against the rebuild.

Only the first request had ever been captured, and the first request is the only one with no
assistant or tool message in it - so how each agent re-serialises a growing conversation had never
been looked at. This prints the shape of each body and then compares the real turn-two request with
the one `stage0.py` rebuilt by hand.
"""

import io
import json
import os
import sys

D = sys.argv[1] if len(sys.argv) > 1 else r"C:\Users\azsxd\codex\_labs\logs\wire-stock-rep11\relay.jsonl.bodies"


def load(p):
    return json.loads(io.open(p, encoding="utf-8", errors="replace").read())


def shape(m):
    """A message's structural signature: role, which keys it carries, how its content is typed."""
    role = m.get("role")
    keys = sorted(k for k in m if k != "role")
    c = m.get("content")
    ctype = ("null" if c is None else
             "str" if isinstance(c, str) else
             "list[%s]" % ",".join(sorted({p.get("type", "?") for p in c if isinstance(p, dict)}))
             if isinstance(c, list) else type(c).__name__)
    n = len(c) if isinstance(c, (str, list)) else 0
    extra = ""
    if m.get("tool_calls"):
        extra = " tool_calls=%d[%s]" % (len(m["tool_calls"]),
                                        ",".join(tc.get("function", {}).get("name", "?") for tc in m["tool_calls"]))
    return "%-9s keys=%-38s content=%-14s %6d%s" % (role, ",".join(keys), ctype, n, extra)


files = sorted(f for f in os.listdir(D) if f.endswith(".json"))
print("=== bodies: %d ===" % len(files))
for f in files:
    b = load(os.path.join(D, f))
    msgs = b.get("messages") or []
    params = {k: v for k, v in b.items() if k not in ("messages", "tools")}
    print("%s  msgs=%-3d tools=%-3s params=%s" % (
        f, len(msgs), len(b.get("tools") or []) or "-", json.dumps(params, sort_keys=True)[:150]))

print()
print("=== per-message shape of the first conversation body that has an assistant turn ===")
target = None
for f in files:
    b = load(os.path.join(D, f))
    msgs = b.get("messages") or []
    if any(m.get("role") == "tool" for m in msgs):
        target = (f, b)
        break
if not target:
    print("none found")
    sys.exit(0)

fname, body = target
print("file:", fname)
for i, m in enumerate(body["messages"]):
    print("  [%d] %s" % (i, shape(m)))

print()
print("=== the real turn-two request, message by message ===")
for i, m in enumerate(body["messages"]):
    print("  [%d] role=%s" % (i, m.get("role")))
    for k, v in m.items():
        if k == "role":
            continue
        s = v if isinstance(v, str) else json.dumps(v, ensure_ascii=False)
        print("       %-16s %s" % (k + ":", (s[:300] + ("…" if len(s) > 300 else "")).replace("\n", "\\n")))

print()
print("=== parameters on that request ===")
for k, v in sorted(body.items()):
    if k in ("messages", "tools"):
        continue
    print("  %-22s %s" % (k, json.dumps(v)[:200]))
print("  %-22s %s" % ("tools", [t.get("function", t).get("name") for t in body.get("tools") or []]))
