"""Emit a side-by-side document of everything the model receives, from each agent's own wire.

Not from the simulator: both halves are the captured request bodies of real binary runs, so this is
what GLM-5.3-flash was actually sent, byte for byte.

    python mkcontext.py > CONTEXT-KARSILASTIRMA.md
"""

import io
import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
WIRE = os.path.join(os.path.dirname(HERE), "wire")

OC = os.path.join(WIRE, "oc1")     # wire-stock-rep1   : 8 req, 44/44 cites, $0.0086
SUF = os.path.join(WIRE, "suf8")   # wire-sufficefork-rep8 : 11 req, 44/44 cites, $0.0159

out = []
def w(s=""):
    out.append(s)


def read(p):
    return io.open(p, encoding="utf-8", errors="replace").read()


def messages(d):
    rows = []
    for name in sorted(os.listdir(d)):
        if name.startswith("msg") and name.endswith(".txt"):
            role = name.split("_", 1)[1][:-4]
            rows.append((name, role, read(os.path.join(d, name))))
    return rows


def tools(d):
    rows = []
    td = os.path.join(d, "tools")
    for name in sorted(os.listdir(td)):
        if name.endswith(".json"):
            spec = json.loads(read(os.path.join(td, name)))
            fn = spec.get("function", spec)
            rows.append((fn["name"], fn.get("description", ""), fn.get("parameters", {}), len(json.dumps(spec))))
    return rows


oc_msgs, suf_msgs = messages(OC), messages(SUF)
oc_tools, suf_tools = tools(OC), tools(SUF)

w("# Modelin gördüğü her şey: OpenCode vs Suffice")
w()
w("Aynı model (`glm-5.3-flash`), aynı görev, aynı korpus. İki tarafın da **gerçek koşularının**")
w("yakalanmış istek gövdelerinden üretildi — simülatörden değil. Yani modele gerçekten gönderilen")
w("baytlar.")
w()
w("| | OpenCode | Suffice |")
w("|---|---|---|")
w("| kaynak koşu | `wire-stock-rep1` | `wire-sufficefork-rep8` |")
w("| istek | 8 | 11 |")
w("| atıf | 44/44 | 44/44 |")
w("| maliyet | $0.0086 | $0.0159 |")
w()

w("## 1. Mesaj yapısı")
w()
w("| # | OpenCode | | | Suffice | |")
w("|---|---|---|---|---|---|")
w("| | rol | karakter | | rol | karakter |")
n = max(len(oc_msgs), len(suf_msgs))
for i in range(n):
    a = oc_msgs[i] if i < len(oc_msgs) else None
    b = suf_msgs[i] if i < len(suf_msgs) else None
    w("| %d | %s | %s | | %s | %s |" % (
        i,
        a[1] if a else "—", f"{len(a[2]):,}" if a else "—",
        b[1] if b else "—", f"{len(b[2]):,}" if b else "—",
    ))
w("| **toplam** | **%d mesaj** | **%s** | | **%d mesaj** | **%s** |" % (
    len(oc_msgs), f"{sum(len(m[2]) for m in oc_msgs):,}",
    len(suf_msgs), f"{sum(len(m[2]) for m in suf_msgs):,}"))
w()
w("OpenCode her şeyi tek sistem mesajına koyuyor: kendi prompt'u, model kimliği, `<env>` bloğu,")
w("AGENTS.md ve skills listesi — sonra görev. Suffice dörde bölüyor: prompt, skills+permissions,")
w("AGENTS.md'yi **user** mesajı olarak, sonra görev.")
w()

w("## 2. Tool yüzeyi")
w()
w("| OpenCode | spec B | | Suffice | spec B |")
w("|---|---|---|---|---|")
n = max(len(oc_tools), len(suf_tools))
for i in range(n):
    a = oc_tools[i] if i < len(oc_tools) else None
    b = suf_tools[i] if i < len(suf_tools) else None
    w("| `%s` | %s | | `%s` | %s |" % (
        a[0] if a else "—", f"{a[3]:,}" if a else "—",
        b[0] if b else "—", f"{b[3]:,}" if b else "—"))
w("| **%d tool** | **%s** | | **%d tool** | **%s** |" % (
    len(oc_tools), f"{sum(t[3] for t in oc_tools):,}",
    len(suf_tools), f"{sum(t[3] for t in suf_tools):,}"))
w()
oc_names = {t[0] for t in oc_tools}
suf_names = {t[0] for t in suf_tools}
w("Ortak: " + ", ".join("`%s`" % x for x in sorted(oc_names & suf_names)))
w()
w("Yalnız OpenCode'da: " + ", ".join("`%s`" % x for x in sorted(oc_names - suf_names)))
w()
w("Yalnız Suffice'te: " + ", ".join("`%s`" % x for x in sorted(suf_names - oc_names)))
w()

w("## 3. Sabit önek toplamı")
w()
oc_total = sum(len(m[2]) for m in oc_msgs) + sum(t[3] for t in oc_tools)
suf_total = sum(len(m[2]) for m in suf_msgs) + sum(t[3] for t in suf_tools)
w("| | OpenCode | Suffice |")
w("|---|---|---|")
w("| mesajlar | %s | %s |" % (f"{sum(len(m[2]) for m in oc_msgs):,}", f"{sum(len(m[2]) for m in suf_msgs):,}"))
w("| tool şemaları | %s | %s |" % (f"{sum(t[3] for t in oc_tools):,}", f"{sum(t[3] for t in suf_tools):,}"))
w("| **toplam** | **%s** | **%s** |" % (f"{oc_total:,}", f"{suf_total:,}"))
w()
w("Her istekte yeniden gönderilen miktar bu.")
w()

for label, msgs, tls in (("OpenCode", oc_msgs, oc_tools), ("Suffice", suf_msgs, suf_tools)):
    w("---")
    w()
    w("# EK — %s: modele giden metnin tamamı" % label)
    w()
    for name, role, body in msgs:
        w("## `%s` — rol `%s`, %s karakter" % (name, role, f"{len(body):,}"))
        w()
        w("````text")
        w(body.rstrip())
        w("````")
        w()
    w("## %s — tool açıklamaları" % label)
    w()
    for tname, desc, params, size in tls:
        w("### `%s` — %s bayt" % (tname, f"{size:,}"))
        w()
        w("````text")
        w(desc.rstrip() or "(açıklama yok)")
        w("````")
        w()
        props = (params or {}).get("properties") or {}
        if props:
            w("| parametre | tip | açıklama |")
            w("|---|---|---|")
            for k, v in props.items():
                t = v.get("type", "")
                if isinstance(t, list):
                    t = "/".join(t)
                d = (v.get("description") or "").replace("|", "\\|").replace("\n", " ")
                w("| `%s` | %s | %s |" % (k, t, d))
            req = (params or {}).get("required") or []
            if req:
                w()
                w("zorunlu: " + ", ".join("`%s`" % r for r in req))
            w()

sys.stdout.reconfigure(encoding="utf-8")
print("\n".join(out))
