#!/usr/bin/env python3
"""MUTABAKAT — Fatura <-> Banka ekstresi mutabakatı (yalnız standart kütüphane)."""

import csv
import datetime as dt
import json
import os
import sys


# --------------------------------------------------------------------------- #
# Ayrıştırma yardımcıları
# --------------------------------------------------------------------------- #
def parse_amount(s: str) -> float:
    """TR yazımı tutarı float'a çevirir: '12.500,00' -> 12500.0."""
    if s is None:
        raise ValueError("bos tutar")
    t = s.strip()
    if not t:
        raise ValueError("bos tutar")
    # binlik ayracı nokta, ondalik ayraci virgul
    t = t.replace(".", "").replace(",", ".")
    return float(t)


def _parse_date(s: str) -> str:
    """GG.AA.YYYY veya YYYY-AA-GG -> ISO YYYY-MM-DD. Hatada ValueError."""
    if s is None:
        raise ValueError("bos tarih")
    t = s.strip()
    if not t:
        raise ValueError("bos tarih")
    for fmt in ("%d.%m.%Y", "%Y-%m-%d"):
        try:
            return dt.datetime.strptime(t, fmt).date().isoformat()
        except ValueError:
            continue
    raise ValueError(f"bozuk tarih: {s!r}")


# --------------------------------------------------------------------------- #
# Yükleme
# --------------------------------------------------------------------------- #
def load_invoices(path):
    """Faturaları yükler. (rows, errors) döner; bozuk satir cökertmez."""
    rows = []
    errors = []
    with open(path, newline="", encoding="utf-8") as fh:
        reader = csv.reader(fh, delimiter=";")
        for line_no, fields in enumerate(reader, start=1):
            if line_no == 1:
                continue  # başlık
            if not fields or (len(fields) == 1 and not fields[0].strip()):
                continue  # tamamen boş satır: atla
            if len(fields) < 5:
                errors.append({"line": line_no, "reason": "eksik alan"})
                continue
            fatura_no, musteri, tutar_s, para_birimi, tarih_s = (
                fields[0], fields[1], fields[2], fields[3], fields[4]
            )
            try:
                tutar = parse_amount(tutar_s)
            except ValueError:
                errors.append({"line": line_no, "reason": f"parse edilemeyen tutar: {tutar_s!r}"})
                continue
            try:
                tarih = _parse_date(tarih_s)
            except ValueError as exc:
                errors.append({"line": line_no, "reason": str(exc)})
                continue
            if not fatura_no.strip():
                errors.append({"line": line_no, "reason": "eksik fatura_no"})
                continue
            rows.append({
                "fatura_no": fatura_no.strip(),
                "musteri": musteri.strip(),
                "tutar": tutar,
                "para_birimi": para_birimi.strip(),
                "tarih": tarih,
            })
    return rows, errors


def load_payments(path):
    """Banka ekstresini yükler. Geçerli satirlara dosya sirasiyla p1, p2, ... verilir."""
    rows = []
    with open(path, newline="", encoding="utf-8") as fh:
        reader = csv.reader(fh, delimiter=";")
        idx = 0
        for line_no, fields in enumerate(reader, start=1):
            if line_no == 1:
                continue  # başlık
            if not fields or (len(fields) == 1 and not fields[0].strip()):
                continue
            if len(fields) < 4:
                continue  # bozuk satir: atla
            tarih_s, aciklama, tutar_s, para_birimi = (
                fields[0], fields[1], fields[2], fields[3]
            )
            try:
                tutar = parse_amount(tutar_s)
                tarih = _parse_date(tarih_s)
            except ValueError:
                continue
            idx += 1
            rows.append({
                "id": f"p{idx}",
                "tarih": tarih,
                "aciklama": aciklama.strip(),
                "tutar": tutar,
                "para_birimi": para_birimi.strip(),
            })
    return rows


# --------------------------------------------------------------------------- #
# Kur çevrimi
# --------------------------------------------------------------------------- #
def to_try(amount: float, currency: str, date: str, rates: dict) -> float:
    """Tutari TRY'ye çevirir. Ayni gün kuru, yoksa <=7 gün geriye. round(x, 2)."""
    if currency == "TRY":
        return round(amount, 2)
    d = dt.date.fromisoformat(date)
    for back in range(0, 8):  # 0..7 gün geriye
        key = (d - dt.timedelta(days=back)).isoformat()
        day = rates.get(key)
        if day is not None and currency in day:
            return round(amount * day[currency], 2)
    raise ValueError(f"kur bulunamadi: {currency} @ {date} (<=7 gün geriye)")


# --------------------------------------------------------------------------- #
# Eşleştirme
# --------------------------------------------------------------------------- #
def _date_diff(a: str, b: str) -> int:
    return abs((dt.date.fromisoformat(a) - dt.date.fromisoformat(b)).days)


def _amounts_match(inv, pays, rates) -> bool:
    """Fatura ile ödeme(ler) toplami toleransta mi? round(|fark|, 2) <= 0.01."""
    pay_curs = {p["para_birimi"] for p in pays}
    if len(pay_curs) == 1 and inv["para_birimi"] in pay_curs:
        diff = abs(inv["tutar"] - sum(p["tutar"] for p in pays))
    else:
        inv_try = to_try(inv["tutar"], inv["para_birimi"], inv["tarih"], rates)
        pay_try = sum(to_try(p["tutar"], p["para_birimi"], p["tarih"], rates) for p in pays)
        diff = abs(inv_try - pay_try)
    return round(diff, 2) <= 0.01


def _fark_try(inv, pays, rates) -> float:
    pay_curs = {p["para_birimi"] for p in pays}
    if len(pay_curs) == 1 and inv["para_birimi"] in pay_curs:
        diff = abs(inv["tutar"] - sum(p["tutar"] for p in pays))
        return to_try(diff, inv["para_birimi"], inv["tarih"], rates)
    inv_try = to_try(inv["tutar"], inv["para_birimi"], inv["tarih"], rates)
    pay_try = sum(to_try(p["tutar"], p["para_birimi"], p["tarih"], rates) for p in pays)
    return round(abs(inv_try - pay_try), 2)


def match(invoices, payments, rates) -> dict:
    used = set()
    matched = []

    for inv in invoices:
        fatura_no = inv["fatura_no"]
        aday = None

        # Kural 1 — Kesin: açiklama fatura_no içeriyor VE tutar toleransta.
        for p in payments:
            if p["id"] in used:
                continue
            if fatura_no in p["aciklama"] and _amounts_match(inv, [p], rates):
                aday = [p["id"]]
                break

        # Kural 2 — Tutar + tarih + müşteri.
        if aday is None:
            musteri_first = inv["musteri"].split()
            first_word = musteri_first[0].lower() if musteri_first else ""
            for p in payments:
                if p["id"] in used:
                    continue
                if not _amounts_match(inv, [p], rates):
                    continue
                if _date_diff(inv["tarih"], p["tarih"]) > 3:
                    continue
                if first_word and first_word in p["aciklama"].lower():
                    aday = [p["id"]]
                    break

        # Kural 3 — Bölünmüş: fatura_no geçen ödemelerden tam iki tanesinin toplami.
        if aday is None:
            cands = [p for p in payments
                     if p["id"] not in used and fatura_no in p["aciklama"]]
            found = None
            for i in range(len(cands)):
                for j in range(i + 1, len(cands)):
                    if _amounts_match(inv, [cands[i], cands[j]], rates):
                        found = [cands[i]["id"], cands[j]["id"]]
                        break
                if found:
                    break
            aday = found

        if aday:
            for pid in aday:
                used.add(pid)
            pays = [p for p in payments if p["id"] in aday]
            matched.append({
                "fatura_no": fatura_no,
                "payment_ids": aday,
                "fark_try": _fark_try(inv, pays, rates),
            })

    matched_faturas = {m["fatura_no"] for m in matched}
    unmatched_invoices = [inv["fatura_no"] for inv in invoices
                          if inv["fatura_no"] not in matched_faturas]
    unmatched_payments = [p["id"] for p in payments if p["id"] not in used]

    return {
        "matched": matched,
        "unmatched_invoices": unmatched_invoices,
        "unmatched_payments": unmatched_payments,
    }


# --------------------------------------------------------------------------- #
# CLI
# --------------------------------------------------------------------------- #
def _report(fixtures_dir: str) -> dict:
    invoices, errors = load_invoices(os.path.join(fixtures_dir, "invoices.csv"))
    payments = load_payments(os.path.join(fixtures_dir, "bank.csv"))
    with open(os.path.join(fixtures_dir, "rates.json"), encoding="utf-8") as fh:
        rates = json.load(fh)

    out = match(invoices, payments, rates)
    toplam = round(sum(m["fark_try"] for m in out["matched"]), 2)
    return {
        "matched": len(out["matched"]),
        "unmatched_invoices": out["unmatched_invoices"],
        "unmatched_payments": out["unmatched_payments"],
        "toplam_fark_try": toplam,
        "hatali_satirlar": len(errors),
    }


def main(argv) -> int:
    if len(argv) >= 1 and argv[0] == "report":
        fixtures_dir = None
        i = 1
        while i < len(argv):
            if argv[i] == "--fixtures" and i + 1 < len(argv):
                fixtures_dir = argv[i + 1]
                i += 2
            else:
                i += 1
        if not fixtures_dir:
            print("kullanım: solution.py report --fixtures <DIR>", file=sys.stderr)
            return 2
        print(json.dumps(_report(fixtures_dir)))
        return 0
    print("kullanım: solution.py report --fixtures <DIR>", file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
