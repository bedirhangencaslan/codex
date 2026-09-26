"""SAHRA - afet yardim lojistigi (yalnizca standut kutuphane).

Kurallar SPEC.md'de tanimlidir; tum siralamalar deterministiktir.
"""

import argparse
import json
from datetime import datetime
from pathlib import Path

DT_FORMAT = "%Y-%m-%dT%H:%M"


def _parse_dt(value: str) -> datetime:
    return datetime.strptime(value, DT_FORMAT)


def priority(need: dict, now: str) -> int:
    """priority = severity*100 + bekleme_saati*2 + population // 1000."""
    delta = _parse_dt(now) - _parse_dt(need["created_at"])
    waiting_hours = delta.days * 24 + delta.seconds // 3600
    return need["severity"] * 100 + waiting_hours * 2 + need["population"] // 1000


def allocate(need: dict, warehouses: list[dict]) -> dict:
    """Ihtiyac kalemlerini alfabetik sirayla, en cok stoklu ambardan tahsis eder.

    ``warehouses`` listesi yerinde guncellenir (stok dusulur).
    """
    allocations: list[dict] = []
    unmet: dict[str, int] = {}
    for item in sorted(need.get("items", {})):
        remaining = need["items"][item]
        if remaining <= 0:
            continue
        order = sorted(
            warehouses,
            key=lambda w: (-w["stock"].get(item, 0), w["id"]),
        )
        for wh in order:
            if remaining <= 0:
                break
            avail = wh["stock"].get(item, 0)
            if avail <= 0:
                continue
            take = min(avail, remaining)
            wh["stock"][item] = avail - take
            remaining -= take
            allocations.append({"warehouse": wh["id"], "item": item, "qty": take})
        if remaining > 0:
            unmet[item] = remaining
    return {"allocations": allocations, "unmet": unmet}


def load_vehicles(
    allocations: list[dict], vehicles: list[dict], catalog: dict, used: dict
) -> dict:
    """Tahsis satirlarini ambar bazinda gruplayip araclarla doldurur.

    ``used`` (arac -> kullanilmis kg) yerinde guncellenir; ardışık çağrılar
    aynı filoyu paylaşır.
    """
    loads: dict[str, list[dict]] = {}
    unshipped: list[dict] = []

    by_warehouse: dict[str, list[dict]] = {}
    for row in allocations:
        by_warehouse.setdefault(row["warehouse"], []).append(row)

    for warehouse, rows in by_warehouse.items():
        rows.sort(key=lambda r: (-catalog[r["item"]] * r["qty"], r["item"]))
        fleet = sorted(
            (v for v in vehicles if v["warehouse"] == warehouse),
            key=lambda v: (-v["capacity_kg"], v["id"]),
        )
        for row in rows:
            item = row["item"]
            unit_kg = catalog[item]
            remaining_qty = row["qty"]
            for vehicle in fleet:
                if remaining_qty <= 0:
                    break
                free_kg = vehicle["capacity_kg"] - used.get(vehicle["id"], 0)
                fits = free_kg // unit_kg
                take = min(remaining_qty, fits)
                if take <= 0:
                    continue
                kg = take * unit_kg
                used[vehicle["id"]] = used.get(vehicle["id"], 0) + kg
                remaining_qty -= take
                loads.setdefault(vehicle["id"], []).append(
                    {"item": item, "qty": take, "kg": kg}
                )
            if remaining_qty > 0:
                unshipped.append(
                    {"warehouse": warehouse, "item": item, "qty": remaining_qty}
                )

    unshipped.sort(key=lambda r: (r["warehouse"], r["item"]))
    return {"loads": loads, "unshipped": unshipped}


def _plan(fixtures_dir: str, now: str) -> dict:
    base = Path(fixtures_dir)
    catalog = json.loads((base / "catalog.json").read_text(encoding="utf-8"))
    warehouses = json.loads((base / "warehouses.json").read_text(encoding="utf-8"))
    vehicles = json.loads((base / "vehicles.json").read_text(encoding="utf-8"))
    needs = json.loads((base / "needs.json").read_text(encoding="utf-8"))

    ranked = sorted(needs, key=lambda n: (-priority(n, now), n["id"]))

    used: dict[str, int] = {}
    unmet_total: dict[str, int] = {}
    unshipped_total: dict[str, int] = {}
    for need in ranked:
        result = allocate(need, warehouses)
        for item, qty in result["unmet"].items():
            unmet_total[item] = unmet_total.get(item, 0) + qty
        shipped = load_vehicles(result["allocations"], vehicles, catalog, used)
        for row in shipped["unshipped"]:
            unshipped_total[row["item"]] = unshipped_total.get(row["item"], 0) + row["qty"]

    return {
        "now": now,
        "ranked": [n["id"] for n in ranked],
        "unmet": {k: v for k, v in sorted(unmet_total.items()) if v > 0},
        "loaded_kg": {k: v for k, v in sorted(used.items()) if v > 0},
        "unshipped": {k: v for k, v in sorted(unshipped_total.items()) if v > 0},
    }


def main() -> None:
    parser = argparse.ArgumentParser(prog="sahra")
    subparsers = parser.add_subparsers(dest="command", required=True)
    plan_parser = subparsers.add_parser("plan", help="gunluk sevkiyat plani")
    plan_parser.add_argument("--fixtures", required=True)
    plan_parser.add_argument("--now", required=True)
    args = parser.parse_args()
    if args.command == "plan":
        print(json.dumps(_plan(args.fixtures, args.now), sort_keys=True))


if __name__ == "__main__":
    main()
