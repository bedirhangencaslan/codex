"""MUTABAKAT — Fatura ↔ Banka Ekstresi Mutabakatı."""

import argparse
import csv
import json
import sys
from datetime import date, timedelta
from pathlib import Path


# ---------------------------------------------------------------- formats ---

def parse_amount(s):
    """TR yazımı: binlik ayırıcı '.', ondalık ','.  -> float"""
    if s is None:
        raise ValueError("tutar boş")
    text = str(s).strip()
    if not text:
        raise ValueError("tutar boş")
    text = text.replace(" ", "").replace("\u00a0", "")
    has_comma = "," in text
    has_dot = "." in text
    if has_comma and has_dot:
        # nokta binlik, virgül ondalık
        text = text.replace(".", "").replace(",", ".")
    elif has_comma:
        text = text.replace(".", "").replace(",", ".")
    elif has_dot:
        # yalnız nokta: 3'lü gruplar binlik ayırıcıdır, aksi halde ondalık
        parts = text.split(".")
        if len(parts) > 2 or (len(parts[-1]) == 3 and len(parts) == 2
                              and len(parts[0]) in (1, 2, 3)):
            text = text.replace(".", "")
    value = float(text)
    if value != value or value in (float("inf"), float("-inf")):
        raise ValueError(f"geçersiz tutar: {s!r}")
    return value


def _parse_date(s):
    """GG.AA.YYYY veya YYYY-AA-GG -> date, ya da None."""
    if not s:
        return None
    text = str(s).strip()
    if not text:
        return None
    try:
        if "." in text:
            d, m, y = text.split(".")
            if len(d) <= 2 and len(m) <= 2 and len(y) == 4:
                return date(int(y), int(m), int(d))
            return None
        if "-" in text:
            y, m, d = text.split("-")
            if len(y) == 4 and len(m) <= 2 and len(d) <= 2:
                return date(int(y), int(m), int(d))
            return None
    except (ValueError, TypeError):
        return None
    return None


def _norm_date(s):
    d = _parse_date(s)
    if d is None:
        raise ValueError(f"bozuk tarih: {s!r}")
    return d.isoformat()


# ---------------------------------------------------------------- loading ---

def load_invoices(path):
    """-> (rows, errors); rows: fatura_no, musti... (SPEC alan adlarıyla)."""
    rows, errors = [], []
    with open(path, newline="", encoding="utf-8-sig") as fh:
        reader = csv.reader(fh, delimiter=";")
        for lineno, raw in enumerate(reader, start=1):
            if lineno == 1:
                continue  # başlık
            if not raw or all(not c.strip() for c in raw):
                errors.append({"line": lineno, "reason": "boş satır"})
                continue
            if len(raw) < 5:
                errors.append({"line": lineno,
                               "reason": f"eksik alan ({len(raw)}/5)"})
                continue
            fatura_no, musteri, tutar_s, para, tarih_s = (c.strip() for c in raw[:5])
            try:
                tutar = parse_amount(tutar_s)
            except ValueError as exc:
                errors.append({"line": lineno, "reason": f"tutar: {exc}"})
                continue
            try:
                tarih = _norm_date(tarih_s)
            except ValueError as exc:
                errors.append({"line": lineno, "reason": str(exc)})
                continue
            if not fatura_no:
                errors.append({"line": lineno, "reason": "fatura_no boş"})
                continue
            rows.append({
                "fatura_no": fatura_no,
                "musteri": musteri,
                "tutar": tutar,
                "para_birimi": para,
                "tarih": tarih,
            })
    return rows, errors


def load_payments(path):
    """-> rows; her veri satırına dosya sırasına göre p1, p2, … verilir."""
    rows = []
    with open(path, newline="", encoding="utf-8-sig") as fh:
        reader = csv.reader(fh, delimiter=";")
        for lineno, raw in enumerate(reader, start=1):
            if lineno == 1:
                continue  # başlık
            if not raw or all(not c.strip() for c in raw):
                continue
            if len(raw) < 4:
                continue
            tarih_s, aciklama, tutar_s, para = (c.strip() for c in raw[:4])
            try:
                tutar = parse_amount(tutar_s)
            except ValueError:
                continue
            try:
                tarih = _norm_date(tarih_s)
            except ValueError:
                continue
            rows.append({
                "id": f"p{len(rows) + 1}",
                "tarih": tarih,
                "aciklama": aciklama,
                "tutar": tutar,
                "para_birimi": para,
            })
    return rows


# -------------------------------------------------------------- currency ----

def to_try(amount, currency, date_s, rates):
    """1 birim *amount* -> TRY. TRY ise aynen; değilse gün kuru, yoksa en
    yakın önceki gün (≤7 gün geriye), o da yoksa ValueError. Sonuç 2 hane."""
    currency = (currency or "").strip().upper()
    if currency == "TRY":
        return round(amount, 2)
    target = _parse_date(date_s)
    if target is None:
        raise ValueError(f"bozuk tarih: {date_s!r}")
    candidates = []
    for day_s, day_rates in rates.items():
        day = _parse_date(day_s)
        if day is None or day > target:
            continue
        if currency not in day_rates:
            continue
        gap = (target - day).days
        if gap > 7:
            continue
        candidates.append((gap, day.isoformat(), day_rates[currency]))
    if not candidates:
        raise ValueError(f"kur yok: {currency} @ {date_s}")
    candidates.sort(key=lambda c: (c[0], c[1]))
    rate = candidates[0][2]
    return round(amount * float(rate), 2)


# ------------------------------------------------------------- matching -----

def _in_tolerance(diff):
    return round(abs(diff), 2) <= 0.01


def _first_word_lower(name):
    words = (name or "").split()
    return words[0].lower() if words else ""


def _amount_equal(inv, pay, rates):
    """Tutar toleransı: aynı para birimiyse orijinal tutarlar, değilse TRY."""
    if inv["para_birimi"].upper() == pay["para_birimi"].upper():
        return _in_tolerance(inv["tutar"] - pay["tutar"])
    try:
        return _in_tolerance(to_try(inv["tutar"], inv["para_birimi"],
                                    inv["tarih"], rates)
                             - to_try(pay["tutar"], pay["para_birimi"],
                                      pay["tarih"], rates))
    except ValueError:
        return False


def _sum_equal(inv, pays, rates):
    total = sum(p["tutar"] for p in pays)
    currencies = {inv["para_birimi"].upper()} | {
        p["para_birimi"].upper() for p in pays}
    if len(currencies) == 1:
        return _in_tolerance(inv["tutar"] - total)
    try:
        inv_try = to_try(inv["tutar"], inv["para_birimi"], inv["tarih"], rates)
        pay_try = sum(to_try(p["tutar"], p["para_birimi"], p["tarih"], rates)
                      for p in pays)
        return _in_tolerance(inv_try - pay_try)
    except ValueError:
        return False


def _fark_try(inv, pays, rates):
    currencies = {inv["para_birimi"].upper()} | {
        p["para_birimi"].upper() for p in pays}
    if len(currencies) == 1:
        diff = abs(inv["tutar"] - sum(p["tutar"] for p in pays))
        return to_try(diff, inv["para_birimi"], inv["tarih"], rates)
    inv_try = to_try(inv["tutar"], inv["para_birimi"], inv["tarih"], rates)
    pay_try = sum(to_try(p["tutar"], p["para_birimi"], p["tarih"], rates)
                  for p in pays)
    return round(abs(inv_try - pay_try), 2)


def match(invoices, payments, rates):
    used = set()
    matched = []
    for inv in invoices:
        no = inv["fatura_no"]
        chosen = None

        # Kural 1 — kesin: açıklamada fatura_no VE tutar toleransta.
        for pay in payments:
            if pay["id"] in used:
                continue
            if no in pay["aciklama"] and _amount_equal(inv, pay, rates):
                chosen = [pay]
                break

        # Kural 2 — tutar + tarih(≤3 gün) + müşteri ilk kelimesi açıklamada.
        if chosen is None:
            word = _first_word_lower(inv.get("musteri", ""))
            if word:
                for pay in payments:
                    if pay["id"] in used:
                        continue
                    if not _amount_equal(inv, pay, rates):
                        continue
                    pd = _parse_date(pay["tarih"])
                    idate = _parse_date(inv["tarih"])
                    if pd is None or idate is None or abs((pd - idate).days) > 3:
                        continue
                    if word not in pay["aciklama"].lower():
                        continue
                    chosen = [pay]
                    break

        # Kural 3 — bölünmüş: açıklamada fatura_no geçen tam iki ödemenin
        # toplamı toleransta.
        if chosen is None:
            cands = [p for p in payments
                     if p["id"] not in used and no in p["aciklama"]]
            if len(cands) == 2 and _sum_equal(inv, cands, rates):
                chosen = cands  # zaten id sırasıyla

        if chosen is None:
            continue
        for pay in chosen:
            used.add(pay["id"])
        matched.append({
            "fatura_no": no,
            "payment_ids": [pay["id"] for pay in chosen],
            "fark_try": _fark_try(inv, chosen, rates),
        })

    matched_ids = {pid for m in matched for pid in m["payment_ids"]}
    return {
        "matched": matched,
        "unmatched_invoices": [inv["fatura_no"] for inv in invoices
                               if inv["fatura_no"] not in
                               {m["fatura_no"] for m in matched}],
        "unmatched_payments": [p["id"] for p in payments
                               if p["id"] not in matched_ids],
    }


# ------------------------------------------------------------------ CLI -----

def report(fixtures_dir):
    fixtures = Path(fixtures_dir)
    invoices, errors = load_invoices(fixtures / "invoices.csv")
    payments = load_payments(fixtures / "bank.csv")
    rates = json.loads((fixtures / "rates.json").read_text(encoding="utf-8"))
    result = match(invoices, payments, rates)
    toplam = round(sum(m["fark_try"] for m in result["matched"]), 2)
    return {
        "matched": len(result["matched"]),
        "unmatched_invoices": result["unmatched_invoices"],
        "unmatched_payments": result["unmatched_payments"],
        "toplam_fark_try": toplam,
        "hatali_satirlar": len(errors),
    }


def main(argv=None):
    parser = argparse.ArgumentParser(prog="mutabakat")
    sub = parser.add_subparsers(dest="command", required=True)
    rep = sub.add_parser("report")
    rep.add_argument("--fixtures", required=True)
    args = parser.parse_args(argv)
    print(json.dumps(report(args.fixtures)))
    return 0


if __name__ == "__main__":
    sys.exit(main())
