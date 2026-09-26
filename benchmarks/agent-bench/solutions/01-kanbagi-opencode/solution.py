"""KANBAĞI — Kan Bağışı Koordinasyon Sistemi (yalnız standart kütüphane)."""

import argparse
import json
import sys
from datetime import date, timedelta
from pathlib import Path

BLOOD_TYPES = ["O-", "O+", "A-", "A+", "B-", "B+", "AB-", "AB+"]
SHELF_LIFE_DAYS = 42
MIN_AGE, MAX_AGE = 18, 65
INTERVAL_DAYS = {"E": 90, "K": 120}


def _parse(s: str) -> date:
    return date.fromisoformat(s)


def can_donate(donor_type: str, recipient_type: str) -> bool:
    d_abo, d_rh = donor_type[:-1], donor_type[-1]
    r_abo, r_rh = recipient_type[:-1], recipient_type[-1]
    abo_ok = d_abo == "O" or d_abo == r_abo or r_abo == "AB"
    rh_ok = d_rh == "-" or r_rh == "+"
    return abo_ok and rh_ok


def _bag_valid(unit: dict, on_date: date) -> bool:
    return (on_date - _parse(unit["collected_on"])).days < SHELF_LIFE_DAYS


def valid_stock(units: list[dict], on_date: str) -> dict[str, int]:
    on = _parse(on_date)
    counts = {t: 0 for t in BLOOD_TYPES}
    for u in units:
        if _bag_valid(u, on):
            counts[u["blood_type"]] += 1
    return counts


def _age_on(birth_date: str, on_date: date) -> int:
    b = _parse(birth_date)
    age = on_date.year - b.year
    if (on_date.month, on_date.day) < (b.month, b.day):
        age -= 1
    return age


def eligible(donor: dict, on_date: str) -> bool:
    on = _parse(on_date)
    age = _age_on(donor["birth_date"], on)
    if not (MIN_AGE <= age <= MAX_AGE):
        return False
    last = donor.get("last_donation")
    if last is None:
        return True
    gap = (on - _parse(last)).days
    return gap >= INTERVAL_DAYS[donor["sex"]]


def match_request(request: dict, units: list[dict], donors: list[dict], on_date: str) -> dict:
    on = _parse(on_date)
    need = int(request["units"])
    req_type = request["blood_type"]
    req_city = request["city"]
    urgent = request["urgent"]

    candidates = [
        u for u in units
        if _bag_valid(u, on) and can_donate(u["blood_type"], req_type)
        and (urgent or u["city"] == req_city)
    ]
    candidates.sort(key=lambda u: (u["collected_on"], u["id"]))
    from_stock = [u["id"] for u in candidates[:need]]
    remaining = need - len(from_stock)

    donor_calls: list[str] = []
    if remaining > 0:
        pool = [
            d for d in donors
            if eligible(d, on_date) and can_donate(d["blood_type"], req_type)
            and (urgent or d["city"] == req_city)
        ]
        pool.sort(key=lambda d: (
            0 if urgent and d["city"] == req_city else 1,
            0 if d["last_donation"] is None else 1,
            d["last_donation"] or "",
            d["id"],
        ))
        donor_calls = [d["id"] for d in pool[:remaining]]

    shortfall = max(0, need - len(from_stock) - len(donor_calls))
    return {"from_stock": from_stock, "donor_calls": donor_calls, "shortfall": shortfall}


def build_report(fixtures_dir: Path, on_date: str) -> dict:
    def load(name: str) -> list:
        return json.loads((fixtures_dir / name).read_text(encoding="utf-8"))

    stock_units = load("stock.json")
    donors = load("donors.json")
    requests = load("requests.json")

    counts = valid_stock(stock_units, on_date)
    critical = sorted(t for t in BLOOD_TYPES if counts[t] < 2)

    on = _parse(on_date)
    expiring = sorted(
        u["id"] for u in stock_units
        if _bag_valid(u, on) and 0 < SHELF_LIFE_DAYS - (on - _parse(u["collected_on"])).days <= 7
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

    report = build_report(Path(args.fixtures), args.date)
    print(json.dumps(report, ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
