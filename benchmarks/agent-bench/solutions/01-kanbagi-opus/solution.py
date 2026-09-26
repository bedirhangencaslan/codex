"""KANBAĞI — Kan Bağışı Koordinasyon Sistemi (standart kütüphane).

Tüm tarihler ISO YYYY-MM-DD string'idir. Tüm sıralamalar deterministiktir.
"""
from __future__ import annotations

import argparse
import json
import sys
from datetime import date
from pathlib import Path

BLOOD_TYPES = ["O-", "O+", "A-", "A+", "B-", "B+", "AB-", "AB+"]
SHELF_LIFE_DAYS = 42
EXPIRING_WINDOW_DAYS = 7
CRITICAL_THRESHOLD = 2
MALE_INTERVAL = 90
FEMALE_INTERVAL = 120
MIN_AGE = 18
MAX_AGE = 65


def _parse(d: str) -> date:
    return date.fromisoformat(d)


def can_donate(donor_type: str, recipient_type: str) -> bool:
    d_abo, d_rh = donor_type[:-1], donor_type[-1]
    r_abo, r_rh = recipient_type[:-1], recipient_type[-1]
    abo_ok = d_abo == "O" or d_abo == r_abo or r_abo == "AB"
    rh_ok = d_rh == "-" or r_rh == "+"
    return abo_ok and rh_ok


def _is_valid_bag(bag: dict, on: date) -> bool:
    return (on - _parse(bag["collected_on"])).days < SHELF_LIFE_DAYS


def valid_stock(units: list[dict], on_date: str) -> dict[str, int]:
    on = _parse(on_date)
    counts = {bt: 0 for bt in BLOOD_TYPES}
    for bag in units:
        if _is_valid_bag(bag, on):
            counts[bag["blood_type"]] += 1
    return counts


def _age(birth: date, on: date) -> int:
    years = on.year - birth.year
    if (on.month, on.day) < (birth.month, birth.day):
        years -= 1
    return years


def eligible(donor: dict, on_date: str) -> bool:
    on = _parse(on_date)
    age = _age(_parse(donor["birth_date"]), on)
    if age < MIN_AGE or age > MAX_AGE:
        return False
    last = donor.get("last_donation")
    if last is not None:
        interval = MALE_INTERVAL if donor["sex"] == "E" else FEMALE_INTERVAL
        if (on - _parse(last)).days < interval:
            return False
    return True


def match_request(request: dict, units: list[dict], donors: list[dict], on_date: str) -> dict:
    on = _parse(on_date)
    recipient = request["blood_type"]
    units_needed = request["units"]
    urgent = request["urgent"]
    req_city = request["city"]

    # a. Stok seçimi
    candidates = [
        b for b in units
        if _is_valid_bag(b, on) and can_donate(b["blood_type"], recipient)
        and (urgent or b["city"] == req_city)
    ]
    candidates.sort(key=lambda b: (b["collected_on"], b["id"]))
    from_stock = [b["id"] for b in candidates[:units_needed]]

    remaining = units_needed - len(from_stock)

    # b. Bağışçı çağrısı
    donor_calls: list[str] = []
    if remaining > 0:
        pool = [
            d for d in donors
            if eligible(d, on_date) and can_donate(d["blood_type"], recipient)
            and (urgent or d["city"] == req_city)
        ]

        def donor_key(d: dict):
            last = d.get("last_donation")
            city_rank = 0 if d["city"] == req_city else 1
            null_rank = 0 if last is None else 1
            return (city_rank, null_rank, last or "", d["id"])

        pool.sort(key=donor_key)
        donor_calls = [d["id"] for d in pool[:remaining]]

    shortfall = max(0, units_needed - len(from_stock) - len(donor_calls))
    return {"from_stock": from_stock, "donor_calls": donor_calls, "shortfall": shortfall}


def _report(fixtures_dir: str, on_date: str) -> dict:
    base = Path(fixtures_dir)
    stock = json.loads((base / "stock.json").read_text())
    requests = json.loads((base / "requests.json").read_text())

    on = _parse(on_date)
    counts = valid_stock(stock, on_date)
    critical = sorted(bt for bt, n in counts.items() if n < CRITICAL_THRESHOLD)

    expiring = sorted(
        b["id"] for b in stock
        if _is_valid_bag(b, on)
        and (SHELF_LIFE_DAYS - (on - _parse(b["collected_on"])).days) <= EXPIRING_WINDOW_DAYS
    )

    return {
        "date": on_date,
        "stock": counts,
        "critical": critical,
        "expiring_7d": expiring,
        "open_requests": len(requests),
    }


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="solution.py")
    sub = parser.add_subparsers(dest="command", required=True)
    rep = sub.add_parser("report")
    rep.add_argument("--fixtures", required=True)
    rep.add_argument("--date", required=True)
    args = parser.parse_args(argv)

    if args.command == "report":
        print(json.dumps(_report(args.fixtures, args.date), ensure_ascii=False, indent=2))
        return 0
    return 1


if __name__ == "__main__":
    sys.exit(main())
