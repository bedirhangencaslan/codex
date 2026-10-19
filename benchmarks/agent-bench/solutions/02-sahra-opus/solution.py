#!/usr/bin/env python3
"""SAHRA — Afet Yardım Lojistiği. Standart kütüphane; deterministik sıralama."""
from __future__ import annotations

import argparse
import json
import math
import sys
from datetime import datetime
from pathlib import Path

_FMT = "%Y-%m-%dT%H:%M"


def _parse(ts: str) -> datetime:
    return datetime.strptime(ts, _FMT)


def priority(need: dict, now: str) -> int:
    created = _parse(need["created_at"])
    now_dt = _parse(now)
    waiting_hours = math.floor((now_dt - created).total_seconds() / 3600)
    return (
        int(need["severity"]) * 100
        + waiting_hours * 2
        + int(need["population"]) // 1000
    )


def allocate(need: dict, warehouses: list[dict]) -> dict:
    allocations: list[dict] = []
    unmet: dict[str, int] = {}

    for item in sorted(need["items"]):
        remaining = int(need["items"][item])
        while remaining > 0:
            # o kalemden stoğu > 0 olan ambarlar arasında en çok stoklu,
            # eşitlikte id küçük olan.
            best = None
            for w in warehouses:
                stock = w.get("stock", {}).get(item, 0)
                if stock <= 0:
                    continue
                if best is None or stock > best[1] or (
                    stock == best[1] and w["id"] < best[0]["id"]
                ):
                    best = (w, stock)
            if best is None:
                break
            w, stock = best
            take = min(remaining, stock)
            w["stock"][item] -= take
            allocations.append({"warehouse": w["id"], "item": item, "qty": take})
            remaining -= take
        if remaining > 0:
            unmet[item] = remaining

    return {"allocations": allocations, "unmet": unmet}


def load_vehicles(
    allocations: list[dict],
    vehicles: list[dict],
    catalog: dict,
    used: dict,
) -> dict:
    loads: dict[str, list[dict]] = {}
    unshipped: list[dict] = []

    # ambar bazında grupla
    by_wh: dict[str, list[dict]] = {}
    for a in allocations:
        by_wh.setdefault(a["warehouse"], []).append(a)

    # ambar başına sabit araç sırası: kapasite azalan, eşitlikte id artan
    vehicles_by_wh: dict[str, list[dict]] = {}
    for v in vehicles:
        vehicles_by_wh.setdefault(v["warehouse"], []).append(v)
    for wh, vs in vehicles_by_wh.items():
        vs.sort(key=lambda v: (-v["capacity_kg"], v["id"]))

    for wh in by_wh:
        rows = by_wh[wh]
        # satırlar toplam ağırlık azalan, eşitlikte item alfabetik
        rows_sorted = sorted(
            rows,
            key=lambda r: (-(r["qty"] * catalog[r["item"]]), r["item"]),
        )
        wh_vehicles = vehicles_by_wh.get(wh, [])
        for row in rows_sorted:
            item = row["item"]
            unit = catalog[item]
            remaining = row["qty"]
            for v in wh_vehicles:
                if remaining <= 0:
                    break
                vid = v["id"]
                free = v["capacity_kg"] - used.get(vid, 0)
                if free <= 0 or unit <= 0:
                    continue
                fit = min(remaining, free // unit)
                if fit <= 0:
                    continue
                kg = fit * unit
                used[vid] = used.get(vid, 0) + kg
                loads.setdefault(vid, []).append(
                    {"item": item, "qty": fit, "kg": kg}
                )
                remaining -= fit
            if remaining > 0:
                unshipped.append(
                    {"warehouse": wh, "item": item, "qty": remaining}
                )

    unshipped.sort(key=lambda u: (u["warehouse"], u["item"]))
    return {"loads": loads, "unshipped": unshipped}


def _plan(fixtures: Path, now: str) -> dict:
    catalog = json.loads((fixtures / "catalog.json").read_text())
    warehouses = json.loads((fixtures / "warehouses.json").read_text())
    vehicles = json.loads((fixtures / "vehicles.json").read_text())
    needs = json.loads((fixtures / "needs.json").read_text())

    ranked = sorted(needs, key=lambda n: (-priority(n, now), n["id"]))

    unmet_total: dict[str, int] = {}
    unshipped_total: dict[str, int] = {}
    used: dict[str, int] = {}

    for need in ranked:
        alloc = allocate(need, warehouses)
        for item, qty in alloc["unmet"].items():
            if qty > 0:
                unmet_total[item] = unmet_total.get(item, 0) + qty
        loaded = load_vehicles(alloc["allocations"], vehicles, catalog, used)
        for u in loaded["unshipped"]:
            if u["qty"] > 0:
                unshipped_total[u["item"]] = (
                    unshipped_total.get(u["item"], 0) + u["qty"]
                )

    loaded_kg = {vid: kg for vid, kg in used.items() if kg > 0}

    return {
        "now": now,
        "ranked": [n["id"] for n in ranked],
        "unmet": {k: unmet_total[k] for k in sorted(unmet_total) if unmet_total[k] > 0},
        "loaded_kg": {vid: loaded_kg[vid] for vid in loaded_kg},
        "unshipped": {
            k: unshipped_total[k] for k in sorted(unshipped_total) if unshipped_total[k] > 0
        },
    }


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="solution.py")
    sub = parser.add_subparsers(dest="cmd", required=True)
    p = sub.add_parser("plan")
    p.add_argument("--fixtures", required=True)
    p.add_argument("--now", required=True)
    args = parser.parse_args(argv)

    if args.cmd == "plan":
        result = _plan(Path(args.fixtures), args.now)
        print(json.dumps(result))
        return 0
    return 1


if __name__ == "__main__":
    sys.exit(main())
