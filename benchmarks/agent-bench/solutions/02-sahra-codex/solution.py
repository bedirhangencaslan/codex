"""SAHRA disaster logistics planning."""

from __future__ import annotations

import argparse
import copy
import json
from collections import defaultdict
from datetime import datetime
from pathlib import Path


def priority(need: dict, now: str) -> int:
    """Return the deterministic priority score for a need."""
    created = datetime.fromisoformat(need["created_at"])
    current = datetime.fromisoformat(now)
    waiting_hours = int((current - created).total_seconds()) // 3600
    return (
        need["severity"] * 100
        + waiting_hours * 2
        + need["population"] // 1000
    )


def allocate(need: dict, warehouses: list[dict]) -> dict:
    """Allocate a need from warehouse stock, updating the stock in place."""
    allocations: list[dict] = []
    unmet: dict[str, int] = {}

    for item in sorted(need["items"]):
        remaining = need["items"][item]

        # Consider warehouses by current stock, then id.  Calculating the key on
        # each pass is important because earlier items can alter stocks.
        candidates = sorted(
            warehouses,
            key=lambda warehouse: (-warehouse["stock"].get(item, 0), warehouse["id"]),
        )

        for warehouse in candidates:
            if remaining <= 0:
                break
            available = warehouse["stock"].get(item, 0)
            if available <= 0:
                continue

            quantity = min(remaining, available)
            warehouse["stock"][item] = available - quantity
            remaining -= quantity
            allocations.append(
                {"warehouse": warehouse["id"], "item": item, "qty": quantity}
            )

        if remaining > 0:
            unmet[item] = remaining

    return {"allocations": allocations, "unmet": unmet}


def load_vehicles(
    allocations: list[dict],
    vehicles: list[dict],
    catalog: dict,
    used: dict,
) -> dict:
    """Load allocations onto vehicles, splitting rows and sharing capacity."""
    loads: dict[str, list[dict]] = {}
    unshipped: list[dict] = []

    by_warehouse: dict[str, list[dict]] = defaultdict(list)
    for allocation in allocations:
        by_warehouse[allocation["warehouse"]].append(allocation)

    vehicle_order = sorted(
        vehicles,
        key=lambda vehicle: (-vehicle["capacity_kg"], vehicle["id"]),
    )

    for warehouse in sorted(by_warehouse):
        rows = sorted(
            by_warehouse[warehouse],
            key=lambda allocation: (
                -allocation["qty"] * catalog[allocation["item"]],
                allocation["item"],
            ),
        )
        warehouse_vehicles = [
            vehicle for vehicle in vehicle_order if vehicle["warehouse"] == warehouse
        ]

        for row in rows:
            item = row["item"]
            unit_kg = catalog[item]
            remaining_qty = row["qty"]

            for vehicle in warehouse_vehicles:
                if remaining_qty <= 0:
                    break
                remaining_kg = vehicle["capacity_kg"] - used.get(vehicle["id"], 0)
                quantity = min(remaining_qty, remaining_kg // unit_kg)
                if quantity <= 0:
                    continue

                kg = quantity * unit_kg
                remaining_qty -= quantity
                used[vehicle["id"]] = used.get(vehicle["id"], 0) + kg
                loads.setdefault(vehicle["id"], []).append(
                    {"item": item, "qty": quantity, "kg": kg}
                )

            if remaining_qty > 0:
                unshipped.append(
                    {"warehouse": warehouse, "item": item, "qty": remaining_qty}
                )

    unshipped.sort(key=lambda row: (row["warehouse"], row["item"]))
    return {"loads": loads, "unshipped": unshipped}


def _load_json(path: Path):
    with path.open(encoding="utf-8") as file:
        return json.load(file)


def plan(fixtures_dir: Path, now: str) -> dict:
    """Build a daily plan using shared stock and vehicle capacity."""
    catalog = _load_json(fixtures_dir / "catalog.json")
    warehouses = _load_json(fixtures_dir / "warehouses.json")
    vehicles = _load_json(fixtures_dir / "vehicles.json")
    needs = _load_json(fixtures_dir / "needs.json")

    ranked = sorted(
        needs,
        key=lambda need: (-priority(need, now), need["id"]),
    )

    # CLI planning must never mutate fixture-loaded originals via the first
    # allocate() call; copy the shared fleet and stock once for all needs.
    warehouses = copy.deepcopy(warehouses)
    used: dict[str, int] = {}
    total_unmet: dict[str, int] = defaultdict(int)
    total_unshipped: dict[str, int] = defaultdict(int)
    for need in ranked:
        result = allocate(need, warehouses)
        for item, quantity in result["unmet"].items():
            total_unmet[item] += quantity

        shipping = load_vehicles(
            result["allocations"], vehicles, catalog, used
        )
        for row in shipping["unshipped"]:
            total_unshipped[row["item"]] += row["qty"]


    return {
        "now": now,
        "ranked": [need["id"] for need in ranked],
        "unmet": {item: total_unmet[item] for item in sorted(total_unmet)},
        "loaded_kg": {
            vehicle_id: vehicle_kg
            for vehicle_id, vehicle_kg in sorted(used.items())
            if vehicle_kg > 0
        },
        "unshipped": {
            item: total_unshipped[item] for item in sorted(total_unshipped)
        },
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="Create SAHRA daily logistics plan")
    parser.add_argument("command", choices=["plan"])
    parser.add_argument("--fixtures", required=True, type=Path)
    parser.add_argument("--now", required=True)
    args = parser.parse_args()
    print(json.dumps(plan(args.fixtures, args.now)))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
