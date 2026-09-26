from __future__ import annotations

import argparse
import csv
import json
import os
import re
import sys
from datetime import date as _date


def parse_amount(s: str) -> float:
    t = str(s).strip().replace(" ", "").replace("\u00a0", "")
    if not t:
        raise ValueError("bos tutar")
    if "," in t:
        t = t.replace(".", "").replace(",", ".")
    elif t.count(".") > 1 or re.fullmatch(r"\d{1,3}(?:\.\d{3})+", t):
        t = t.replace(".", "")
    return float(t)


def _norm_date(value) -> str:
    t = str(value or "").strip()
    if not t:
        raise ValueError("bos tarih")
    parts = re.split(r"[.\-/]", t)
    if len(parts) != 3 or any(not p for p in parts):
        raise ValueError("bozuk tarih")
    a, b, c = parts
    try:
        if len(a) == 4:
            d = _date(int(a), int(b), int(c))
        elif len(c) == 4:
            d = _date(int(c), int(b), int(a))
        else:
            raise ValueError
    except ValueError:
        raise ValueError("bozuk tarih") from None
    return d.isoformat()


def _to_date(value) -> _date | None:
    try:
        return _date.fromisoformat(_norm_date(value))
    except ValueError:
        return None


def to_try(amount: float, currency: str, date: str, rates: dict) -> float:
    cur = str(currency or "").strip().upper()
    amt = float(amount)
    if cur == "TRY":
        return round(amt, 2)
    target = _to_date(date)
    if target is None:
        raise ValueError(f"bozuk tarih: {date!r}")
    best = None
    for key, table in (rates or {}).items():
        kd = _to_date(key)
        if kd is None or kd > target:
            continue
        rate = (table or {}).get(cur)
        if rate is None:
            continue
        delta = (target - kd).days
        if best is None or delta < best[0]:
            best = (delta, float(rate))
    if best is None or best[0] > 7:
        raise ValueError(f"{cur} icin {date} ve 7 gun oncesinde kur bulunamadi")
    return round(amt * best[1], 2)


def load_invoices(path) -> tuple[list[dict], list[dict]]:
    rows: list[dict] = []
    errors: list[dict] = []
    with open(path, newline="", encoding="utf-8-sig") as fh:
        reader = csv.reader(fh, delimiter=";")
        for lineno, row in enumerate(reader, start=1):
            if lineno == 1:
                continue
            if not row or all(not c.strip() for c in row):
                continue
            try:
                if len(row) != 5:
                    raise ValueError("eksik alan" if len(row) < 5 else "fazla alan")
                fatura_no, musteri, tutar_s, cur, tarih = (c.strip() for c in row)
                if not (fatura_no and musteri and cur and tarih):
                    raise ValueError("eksik alan")
                rows.append({
                    "fatura_no": fatura_no,
                    "musteri": musteri,
                    "tutar": parse_amount(tutar_s),
                    "para_birimi": cur.upper(),
                    "tarih": _norm_date(tarih),
                })
            except ValueError as exc:
                errors.append({"line": lineno, "reason": str(exc)})
    return rows, errors


def load_payments(path) -> list[dict]:
    rows: list[dict] = []
    with open(path, newline="", encoding="utf-8-sig") as fh:
        reader = csv.reader(fh, delimiter=";")
        for lineno, row in enumerate(reader, start=1):
            if lineno == 1:
                continue
            if not row or all(not c.strip() for c in row):
                continue
            try:
                if len(row) != 4:
                    raise ValueError("eksik alan" if len(row) < 4 else "fazla alan")
                tarih, aciklama, tutar_s, cur = (c.strip() for c in row)
                if not (tarih and aciklama and cur):
                    raise ValueError("eksik alan")
                rows.append({
                    "id": f"p{len(rows) + 1}",
                    "tarih": _norm_date(tarih),
                    "aciklama": aciklama,
                    "tutar": parse_amount(tutar_s),
                    "para_birimi": cur.upper(),
                })
            except ValueError:
                continue
    return rows


def _currencies(pays: list[dict]) -> set[str]:
    return {str(p["para_birimi"]).strip().upper() for p in pays}


def _amount_diff(inv: dict, pays: list[dict], rates: dict) -> float | None:
    inv_cur = str(inv["para_birimi"]).strip().upper()
    try:
        if _currencies(pays) == {inv_cur}:
            total = sum(float(p["tutar"]) for p in pays)
            return abs(float(inv["tutar"]) - total)
        inv_try = to_try(inv["tutar"], inv_cur, inv["tarih"], rates)
        pay_try = sum(
            to_try(p["tutar"], p["para_birimi"], p["tarih"], rates) for p in pays
        )
        return abs(inv_try - pay_try)
    except ValueError:
        return None


def _within_tolerance(inv: dict, pays: list[dict], rates: dict) -> bool:
    diff = _amount_diff(inv, pays, rates)
    return diff is not None and round(diff, 2) <= 0.01


def _fark_try(inv: dict, pays: list[dict], rates: dict) -> float:
    inv_cur = str(inv["para_birimi"]).strip().upper()
    if _currencies(pays) == {inv_cur}:
        diff = abs(float(inv["tutar"]) - sum(float(p["tutar"]) for p in pays))
        return round(to_try(diff, inv_cur, inv["tarih"], rates), 2)
    inv_try = to_try(inv["tutar"], inv_cur, inv["tarih"], rates)
    pay_try = sum(
        to_try(p["tutar"], p["para_birimi"], p["tarih"], rates) for p in pays
    )
    return round(abs(inv_try - pay_try), 2)


def _match_invoice(inv: dict, avail: list[dict], rates: dict) -> list[dict] | None:
    no = str(inv["fatura_no"])
    inv_d = _to_date(inv.get("tarih"))
    words = str(inv.get("musteri", "")).split()
    first_word = words[0].lower() if words else ""

    for p in avail:
        if no and no in str(p.get("aciklama", "")) and _within_tolerance(inv, [p], rates):
            return [p]

    for p in avail:
        if inv_d is None or not _within_tolerance(inv, [p], rates):
            continue
        pay_d = _to_date(p.get("tarih"))
        if pay_d is None or abs((pay_d - inv_d).days) > 3:
            continue
        if first_word and first_word in str(p.get("aciklama", "")).lower():
            return [p]

    cands = [p for p in avail if no and no in str(p.get("aciklama", ""))]
    for i in range(len(cands)):
        for j in range(i + 1, len(cands)):
            if _within_tolerance(inv, [cands[i], cands[j]], rates):
                return [cands[i], cands[j]]

    return None


def match(invoices, payments, rates) -> dict:
    rates = rates or {}
    used: set[str] = set()
    matched: list[dict] = []
    matched_nos: set[str] = set()
    for inv in invoices:
        avail = [p for p in payments if p["id"] not in used]
        pick = _match_invoice(inv, avail, rates)
        if pick:
            ids = [p["id"] for p in pick]
            matched.append({
                "fatura_no": inv["fatura_no"],
                "payment_ids": ids,
                "fark_try": _fark_try(inv, pick, rates),
            })
            used.update(ids)
            matched_nos.add(inv["fatura_no"])
    return {
        "matched": matched,
        "unmatched_invoices": [
            inv["fatura_no"] for inv in invoices if inv["fatura_no"] not in matched_nos
        ],
        "unmatched_payments": [
            p["id"] for p in payments if p["id"] not in used
        ],
    }


def _cmd_report(fixtures_dir: str) -> None:
    invoices, errors = load_invoices(os.path.join(fixtures_dir, "invoices.csv"))
    payments = load_payments(os.path.join(fixtures_dir, "bank.csv"))
    rates: dict = {}
    try:
        with open(os.path.join(fixtures_dir, "rates.json"), encoding="utf-8-sig") as fh:
            rates = json.load(fh)
    except OSError:
        rates = {}
    out = match(invoices, payments, rates)
    report = {
        "matched": len(out["matched"]),
        "unmatched_invoices": out["unmatched_invoices"],
        "unmatched_payments": out["unmatched_payments"],
        "toplam_fark_try": round(sum(m["fark_try"] for m in out["matched"]), 2),
        "hatali_satirlar": len(errors),
    }
    print(json.dumps(report, ensure_ascii=False))


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(prog="solution.py")
    sub = parser.add_subparsers(dest="command", required=True)
    rep = sub.add_parser("report")
    rep.add_argument("--fixtures", required=True)
    args = parser.parse_args(argv)
    if args.command == "report":
        _cmd_report(args.fixtures)
    return 0


if __name__ == "__main__":
    sys.exit(main())
