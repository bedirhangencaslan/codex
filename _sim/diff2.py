"""Exact diff: the real turn-two request against the one `stage0.py` rebuilds."""

import difflib
import io
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import stage0

REAL = sys.argv[1] if len(sys.argv) > 1 else \
    r"C:\Users\azsxd\codex\_labs\logs\wire-stock-rep11\relay.jsonl.bodies\003.json"

real = json.loads(io.open(REAL, encoding="utf-8", errors="replace").read())
mine = stage0.build(True)

rm, mm = real["messages"], mine

print("messages: real=%d mine=%d" % (len(rm), len(mm)))
print()
for i in range(max(len(rm), len(mm))):
    a = rm[i] if i < len(rm) else {}
    b = mm[i] if i < len(mm) else {}
    ka, kb = sorted(a), sorted(b)
    same_keys = ka == kb
    print("[%d] role real=%s mine=%s" % (i, a.get("role"), b.get("role")))
    print("    keys  real=%s" % ka)
    print("          mine=%s%s" % (kb, "" if same_keys else "   <== DIFFERENT"))
    for k in sorted(set(ka) | set(kb)):
        va, vb = a.get(k), b.get(k)
        if va == vb:
            continue
        sa = va if isinstance(va, str) else json.dumps(va, ensure_ascii=False, sort_keys=True)
        sb = vb if isinstance(vb, str) else json.dumps(vb, ensure_ascii=False, sort_keys=True)
        if sa is None:
            sa = "<absent>"
        if sb is None:
            sb = "<absent>"
        print("    %s: real %r (%s, %d)" % (k, type(va).__name__, "len %d" % len(sa) if sa else "empty", len(sa)))
        print("    %s: mine %r (%s, %d)" % (" " * len(k), type(vb).__name__, "len %d" % len(sb) if sb else "empty", len(sb)))
        if isinstance(va, str) and isinstance(vb, str) and sa != sb:
            n = 0
            for line in difflib.unified_diff(vb.splitlines(), va.splitlines(),
                                             "mine", "real", lineterm="", n=0):
                if line.startswith(("---", "+++", "@@")):
                    continue
                print("        %s" % line[:220])
                n += 1
                if n >= 10:
                    print("        … (%d more differing lines)" % 0)
                    break
            if not n:
                print("        (identical line-wise; differs in trailing whitespace or line endings)")
                print("        real repr tail: %r" % va[-60:])
                print("        mine repr tail: %r" % vb[-60:])
    print()

print("=== parameters ===")
mine_params = {
    "model": "glm-5.3-flash", "tool_choice": "auto", "stream": True,
    "stream_options": {"include_usage": True}, "thinking": {"type": "enabled"},
    "reasoning_effort": "high", "max_tokens": 32000,
}
rp = {k: v for k, v in real.items() if k not in ("messages", "tools")}
for k in sorted(set(rp) | set(mine_params)):
    a, b = rp.get(k, "<absent>"), mine_params.get(k, "<absent>")
    flag = "" if a == b else "   <== DIFFERENT"
    print("  %-18s real=%-34s mine=%s%s" % (k, json.dumps(a), json.dumps(b), flag))

print()
print("=== tools ===")
rt = [t.get("function", t).get("name") for t in real.get("tools") or []]
mt = [t.get("function", t).get("name") for t in stage0.tools()]
print("  real:", rt)
print("  mine:", mt)
print("  identical:", json.dumps(real.get("tools"), sort_keys=True) == json.dumps(stage0.tools(), sort_keys=True))
