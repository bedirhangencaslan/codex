"""KANBAĞI — Kan Bağışı Koordinasyon Sistemi (çekirdek).

Yalnızca Python standart kütüphanesi. Tüm tarihler ISO ``YYYY-MM-DD``.
"""

from __future__ import annotations

import argparse
import json
import sys
from datetime import date

BLOOD_TYPES = ["O-", "O+", "A-", "A+", "B-", "B+", "AB-", "AB+"]

SHELF_LIFE_DAYS = 42
MIN_AGE = 18
MAX_AGE = 65
MALE_INTERVAL_DAYS = 90
FEMALE_INTERVAL_DAYS = 120
CRITICAL_THRESHOLD = 2
EXPIRING_WINDOW_DAYS = 7


def _parse(iso: str) -> date:
    return date.fromisoformat(iso)


def can_donate(donor_type: str, recipient_type: str) -> bool:
    """Bağışçı → alıcı ABO+Rh uyumluluğu."""
    d_abo, d_rh = donor_type[:-1], donor_type[-1]
    r_abo, r_rh = recipient_type[:-1], recipient_type[-1]
    abo_ok = d_abo == "O" or d_abo == r_abo or r_abo == "AB"
    rh_ok = d_rh == "-" or r_rh == "+"
    return abo_ok and rh_ok


def valid_stock(units: list[dict], on_date: str) -> dict[str, int]:
    """``on_date`` itibarıyla geçerli (SKT'si geçmemiş) torba sayıları.

    SKT = ``collected_on + 42 gün``; 42. gün dahil değildir.
    Dönen sözlük 8 grubun tamamını anahtar olarak içerir.
    """
    on = _parse(on_date)
    counts = {t: 0 for t in BLOOD_TYPES}
    for unit in units:
        collected = _parse(unit["collected_on"])
        if (on - collected).days < SHELF_LIFE_DAYS:
            counts[unit["blood_type"]] += 1
    return counts


def eligible(donor: dict, on_date: str) -> bool:
    """``on_date`` itibarıyla bağış uygunluğu (kural 3)."""
    on = _parse(on_date)
    birth = _parse(donor["birth_date"])
    # Yaş: doğum günü henüz geçmediyse bir eksik sayılır.
    age = on.year - birth.year - ((on.month, on.day) < (birth.month, birth.day))
    if not (MIN_AGE <= age <= MAX_AGE):
        return False
    last = donor.get("last_donation")
    if last is None:
        return True
    gap = (on - _parse(last)).days
    required = MALE_INTERVAL_DAYS if donor["sex"] == "E" else FEMALE_INTERVAL_DAYS
    return gap >= required


def match_request(
    request: dict, units: list[dict], donors: list[dict], on_date: str
) -> dict:
    """Talep karşılama: stok (FIFO) sonra bağışçı çağrısı (kural 4)."""
    need = request["units"]
    rtype = request["blood_type"]
    city = request["city"]
    urgent = request["urgent"]
    on = _parse(on_date)

    candidates = [
        u for u in units
        if can_donate(u["blood_type"], rtype)
        and (on - _parse(u["collected_on"])).days < SHELF_LIFE_DAYS
    ]
    if not urgent:
        candidates = [u for u in candidates if u["city"] == city]
    candidates.sort(key=lambda u: (u["collected_on"], u["id"]))
    from_stock = [u["id"] for u in candidates[:need]]
    remaining = need - len(from_stock)

    donor_calls: list[str] = []
    if remaining > 0:
        pool = [
            d for d in donors
            if eligible(d, on_date) and can_donate(d["blood_type"], rtype)
        ]
        def donor_key(d: dict):
            d.get("last_donation") is not None,
            d.get("last_donation") or "",
            d["id"],

        if urgent:
            local = sorted((d for d in pool if d["city"] == city), key=donor_key)
            others = sorted((d for d in pool if d["city"] != city), key=donor_key)
            ordered = local + others
        else:
            ordered = sorted((d for d in pool if d["city"] == city), key=donor_key)
        donor_calls = [d["id"] for d in ordered[:remaining]]

    shortfall = max(0, need - len(from_stock) - len(donor_calls))
    return {
        "from_stock": from_stock,
        "donor_calls": donor_calls,
        "shortfall": shortfall,
    }


def _report(fixtures_dir: str, on_date: str) -> dict:
    with open(f"{fixtures_dir}/stock.json", encoding="utf-8") as f:
        stock = json.load(f)
    with open(f"{fixtures_dir}/requests.json", encoding="utf-8") as f:
        requests = json.load(f)

    counts = valid_stock(stock, on_date)
    on = _parse(on_date)
    expiring = sorted(
        u["id"] for u in stock
        if 0 < SHELF_LIFE_DAYS - (on - _parse(u["collected_on"])).days
        <= EXPIRING_WINDOW_DAYS
    )
    return {
        "date": on_date,
        "stock": counts,
        "critical": sorted(t for t in BLOOD_TYPES if counts[t] < CRITICAL_THRESHOLD),
        "expiring_7d": expiring,
        "open_requests": len(requests),
    }


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="kanbagi")
    sub = parser.add_subparsers(dest="command", required=True)
    rep = sub.add_parser("report")
    rep.add_argument("--fixtures", required=True)
    rep.add_argument("--date", required=True)
    args = parser.parse_args(argv)

    if args.command == "report":
        print(json.dumps(_report(args.fixtures, args.date)))
    return 0


if __name__ == "__main__":
    sys.exit(main())
