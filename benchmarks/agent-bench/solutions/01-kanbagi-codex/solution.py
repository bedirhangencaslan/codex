#!/usr/bin/env python3
"""KANBAĞI blood donation coordination core."""

from __future__ import annotations

import argparse
import json
from datetime import date, timedelta
from pathlib import Path
from typing import Any


BLOOD_TYPES = ("O-", "O+", "A-", "A+", "B-", "B+", "AB-", "AB+")
SHELF_LIFE_DAYS = 42


def _parse_date(value: str) -> date:
    return date.fromisoformat(value)


def can_donate(donor_type: str, recipient_type: str) -> bool:
    """Return whether a blood unit from ``donor_type`` suits ``recipient_type``."""
    donor_abo, donor_rh = donor_type[:-1], donor_type[-1]
    recipient_abo, recipient_rh = recipient_type[:-1], recipient_type[-1]
    abo_ok = donor_abo == "O" or donor_abo == recipient_abo or recipient_abo == "AB"
    rh_ok = donor_rh == "-" or recipient_rh == "+"
    return abo_ok and rh_ok


def valid_stock(units: list[dict], on_date: str) -> dict[str, int]:
    """Count non-expired units by blood type, including all eight types."""
    current = _parse_date(on_date)
    counts = dict.fromkeys(BLOOD_TYPES, 0)
    for unit in units:
        collected = _parse_date(unit["collected_on"])
        if (current - collected).days < SHELF_LIFE_DAYS:
            blood_type = unit["blood_type"]
            if blood_type in counts:
                counts[blood_type] += 1
    return counts


def eligible(donor: dict, on_date: str) -> bool:
    """Check age and same-sex donation-interval rules on ``on_date``."""
    current = _parse_date(on_date)
    born = _parse_date(donor["birth_date"])

    # Age is one lower until the birthday has occurred.
    age = current.year - born.year - (
        (current.month, current.day) < (born.month, born.day)
    )
    if not 18 <= age <= 65:
        return False

    last_donation = donor.get("last_donation")
    if last_donation is None:
        return True

    interval = (current - _parse_date(last_donation)).days
    required_interval = 90 if donor["sex"] == "E" else 120
    return interval >= required_interval


def match_request(
    request: dict,
    units: list[dict],
    donors: list[dict],
    on_date: str,
) -> dict:
    """Allocate valid stock FIFO, then call eligible compatible donors."""
    recipient_type = request["blood_type"]
    needed = int(request["units"])
    urgent = bool(request["urgent"])
    request_city = request["city"]

    stock_candidates = [
        unit
        for unit in units
        if can_donate(unit["blood_type"], recipient_type)
        and (urgent or unit["city"] == request_city)
        and (_parse_date(on_date) - _parse_date(unit["collected_on"])).days
        < SHELF_LIFE_DAYS
    ]
    stock_candidates.sort(key=lambda unit: (unit["collected_on"], unit["id"]))
    from_stock = [unit["id"] for unit in stock_candidates[:needed]]
    remaining = needed - len(from_stock)

    donor_candidates = [
        donor
        for donor in donors
        if eligible(donor, on_date)
        and can_donate(donor["blood_type"], recipient_type)
        and (urgent or donor["city"] == request_city)
    ]

    if urgent:
        donor_candidates.sort(
            key=lambda donor: (
                donor["city"] != request_city,
                donor.get("last_donation") is not None,
                donor["last_donation"] or "",
                donor["id"],
            )
        )
    else:
        donor_candidates.sort(
            key=lambda donor: (
                donor.get("last_donation") is not None,
                donor["last_donation"] or "",
                donor["id"],
            )
        )

    donor_calls = [donor["id"] for donor in donor_candidates[:remaining]]
    shortfall = max(0, remaining - len(donor_calls))
    return {"from_stock": from_stock, "donor_calls": donor_calls, "shortfall": shortfall}


def report(fixtures_dir: str | Path, on_date: str) -> dict[str, Any]:
    """Build the command-line report from a fixtures directory."""
    root = Path(fixtures_dir)
    with (root / "stock.json").open(encoding="utf-8") as file:
        units = json.load(file)
    with (root / "requests.json").open(encoding="utf-8") as file:
        requests = json.load(file)

    stock = valid_stock(units, on_date)
    current = _parse_date(on_date)
    expiring = []
    for unit in units:
        collected = _parse_date(unit["collected_on"])
        age = (current - collected).days
        if 0 <= age < SHELF_LIFE_DAYS and (SHELF_LIFE_DAYS - age) <= 7:
            expiring.append(unit["id"])

    return {
        "date": on_date,
        "stock": stock,
        "critical": sorted(blood for blood, count in stock.items() if count < 2),
        "expiring_7d": sorted(expiring),
        "open_requests": len(requests),
    }


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="KANBAĞI coordination core")
    subparsers = parser.add_subparsers(dest="command", required=True)
    report_parser = subparsers.add_parser("report", help="write a JSON report")
    report_parser.add_argument("--fixtures", required=True)
    report_parser.add_argument("--date", required=True, dest="on_date")
    return parser


def main() -> None:
    args = _build_parser().parse_args()
    if args.command == "report":
        print(json.dumps(report(args.fixtures, args.on_date), ensure_ascii=False))


if __name__ == "__main__":
    main()
