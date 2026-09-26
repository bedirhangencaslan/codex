"""SAHRA — afet yardım lojistiği (yalnız standart kütüphane).

Kurallar SPEC.md'de birebir uygulanır:
  - priority:      severity*100 + bekleme_saati*2 + population//1000
  - allocate:      kalemler alfabetik; her adımda o kalemde stoğu en çok olan
                   ambar (eşitlikte küçük id); stok yerinde düşülür; karşılanamayan
                   miktar unmet'e (>0) yazılır.
  - load_vehicles: satırlar ambar bazında, toplam ağırlık azalan (eşitlikte item
                   alfabetik); araçlar orijinal kapasite azalan (eşitlikte id artan);
                   satırlar adet bazında bölünebilir; kalan unshipped'e yazılır;
                   `used` yerinde güncellenir (paylaşımlı filo).
  - plan (CLI):    ihtiyaçlar öncelik azalan (eşitlikte id artan), paylaşımlı stok
                   ve kapasiteyle sırayla işlenir.
"""

from __future__ import annotations

import argparse
import json
from datetime import datetime
from pathlib import Path


def _parse_ts(ts: str) -> datetime:
    return datetime.strptime(ts, "%Y-%m-%dT%H:%M")


def priority(need: dict, now: str) -> int:
    """Öncelik skoru; bekleme saati taban (floor)."""
    delta = _parse_ts(now) - _parse_ts(need["created_at"])
    # timedelta normalizasyonu zaten taban yapar: days negatifse saniyeler 0..86399.
    waiting_hours = delta.days * 24 + delta.seconds // 3600
    return need["severity"] * 100 + waiting_hours * 2 + need["population"] // 1000


def allocate(need: dict, warehouses: list[dict]) -> dict:
    """İhtiyacı ambar stoklarından karşıla; `warehouses` yerinde güncellenir."""
    allocations: list[dict] = []
    unmet: dict[str, int] = {}

    for item in sorted(need.get("items", {})):
        remaining = need["items"][item]
        if remaining <= 0:
            continue
        while remaining > 0:
            candidates = [w for w in warehouses if w["stock"].get(item, 0) > 0]
            if not candidates:
                break
            best = min(candidates, key=lambda w: (-w["stock"][item], w["id"]))
            take = min(remaining, best["stock"][item])
            allocations.append({"warehouse": best["id"], "item": item, "qty": take})
            best["stock"][item] = best["stock"].get(item, 0) - take
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
    """Tahsis satırları ambar bazında araçlara yüklenir; `used` yerinde güncellenir."""
    by_warehouse: dict[str, list[dict]] = {}
    for row in allocations:
        by_warehouse.setdefault(row["warehouse"], []).append(row)

    loads: dict[str, list[dict]] = {}
    unshipped: list[dict] = []

    for warehouse in sorted(by_warehouse):
        rows = sorted(
            by_warehouse[warehouse],
            key=lambda r: (-r["qty"] * catalog.get(r["item"], 0), r["item"]),
        )
        warehouse_vehicles = sorted(
            (v for v in vehicles if v["warehouse"] == warehouse),
            key=lambda v: (-v["capacity_kg"], v["id"]),
        )

        for row in rows:
            item = row["item"]
            remaining_qty = row["qty"]
            unit_kg = catalog.get(item, 0)
            for vehicle in warehouse_vehicles:
                if remaining_qty <= 0:
                    break
                vehicle_id = vehicle["id"]
                remaining_capacity = vehicle["capacity_kg"] - used.get(vehicle_id, 0)
                if unit_kg > 0:
                    fits = remaining_capacity // unit_kg
                else:
                    fits = remaining_qty
                take = min(remaining_qty, fits)
                if take <= 0:
                    continue
                kg = take * unit_kg
                used[vehicle_id] = used.get(vehicle_id, 0) + kg
                loads.setdefault(vehicle_id, []).append(
                    {"item": item, "qty": take, "kg": kg}
                )
                remaining_qty -= take
            if remaining_qty > 0:
                unshipped.append(
                    {"warehouse": warehouse, "item": item, "qty": remaining_qty}
                )

    unshipped.sort(key=lambda r: (r["warehouse"], r["item"]))
    return {"loads": loads, "unshipped": unshipped}


def _plan(fixtures_dir: Path, now: str) -> dict:
    def load_json(name: str):
        return json.loads((fixtures_dir / name).read_text(encoding="utf-8"))

    catalog = load_json("catalog.json")
    warehouses = load_json("warehouses.json")
    vehicles = load_json("vehicles.json")
    needs = load_json("needs.json")

    ranked = sorted(needs, key=lambda n: (-priority(n, now), n["id"]))

    used: dict[str, int] = {}
    unmet_total: dict[str, int] = {}
    loaded_kg: dict[str, int] = {}
    unshipped_total: dict[str, int] = {}

    for need in ranked:
        outcome = allocate(need, warehouses)
        for item, qty in outcome["unmet"].items():
            unmet_total[item] = unmet_total.get(item, 0) + qty
        result = load_vehicles(outcome["allocations"], vehicles, catalog, used)
        for vehicle_id, rows in result["loads"].items():
            loaded_kg[vehicle_id] = (
                loaded_kg.get(vehicle_id, 0) + sum(r["kg"] for r in rows)
            )
        for row in result["unshipped"]:
            unshipped_total[row["item"]] = (
                unshipped_total.get(row["item"], 0) + row["qty"]
            )

    return {
        "now": now,
        "ranked": [n["id"] for n in ranked],
        "unmet": {k: v for k, v in sorted(unmet_total.items()) if v > 0},
        "loaded_kg": {k: v for k, v in sorted(loaded_kg.items()) if v > 0},
        "unshipped": {k: v for k, v in sorted(unshipped_total.items()) if v > 0},
    }


def main(argv: list[str] | None = None) -> None:
    parser = argparse.ArgumentParser(prog="sahra")
    subparsers = parser.add_subparsers(dest="command", required=True)
    plan_parser = subparsers.add_parser("plan", help="Günlük sevkiyat planı")
    plan_parser.add_argument("--fixtures", required=True)
    plan_parser.add_argument("--now", required=True)
    args = parser.parse_args(argv)

    if args.command == "plan":
        print(json.dumps(_plan(Path(args.fixtures), args.now), ensure_ascii=False))


if __name__ == "__main__":
    main()
