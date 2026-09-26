#!/usr/bin/env python3
"""Suffice koşumundan sonra ~/.suffice/analytics altındaki EN YENİ sidecar'ı
özetler: taze/önbellekli/çıktı token, istek sayısı, Z.ai fiyatıyla maliyet."""

import json
import sys
from pathlib import Path

FRESH, CACHED, OUT = 0.15, 0.03, 0.50  # USD / 1M token (Z.ai glm-5.3-flash, docs.z.ai güncel)

home = Path.home() / ".suffice" / "analytics"
files = sorted(home.glob("*.jsonl"), key=lambda p: p.stat().st_mtime, reverse=True)
if not files:
    sys.exit("sidecar yok — [features] request_stats = true mu?")
f = files[0]
lines = f.read_text().splitlines()
header = json.loads(lines[0])
inp = cach = out = reqs = 0
for line in lines[1:]:
    try:
        r = json.loads(line)
    except json.JSONDecodeError:
        continue
    reqs += 1
    inp += r.get("input_tokens", 0)
    cach += r.get("cached_input_tokens", 0)
    out += r.get("output_tokens", 0)
fresh = max(0, inp - cach)
cost = (fresh * FRESH + cach * CACHED + out * OUT) / 1e6
print(json.dumps({
    "sidecar": f.name,
    "model": header.get("model"),
    "provider": header.get("provider"),
    "requests": reqs,
    "fresh_tokens": fresh,
    "cached_tokens": cach,
    "output_tokens": out,
    "cost_usd": round(cost, 4),
}, ensure_ascii=False, indent=2))
