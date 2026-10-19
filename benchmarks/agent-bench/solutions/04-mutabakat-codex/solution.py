#!/usr/bin/env python3
"""Fatura ve banka ödemelerini mutabık kılan küçük bir araç."""

from __future__ import annotations

import argparse
import csv
import json
import sys
from datetime import date
from decimal import Decimal, InvalidOperation
from itertools import combinations
from pathlib import Path
from typing import Iterable


INVOICE_FIELDS = ("fatura_no", "musteri", "tutar", "para_birimi", "tarih")
PAYMENT_FIELDS = ("tarih", "aciklama", "tutar", "para_birimi")
TOLERANCE = 0.01
RATE_FALLBACK_DAYS = 7
DATE_MATCH_DAYS = 3


def parse_amount(value: str) -> float:
    """Parse a Turkish-formatted amount, raising ValueError if it is invalid."""
    if not isinstance(value, str):
        raise ValueError("tutar metin olmalı")

    cleaned = value.strip()
    if not cleaned:
        raise ValueError("tutar boş")

    # A sign is accepted, but thousands and decimal separators must be used in
    # the Turkish order when either separator is present.
    negative = cleaned.startswith("-")
    if cleaned.startswith(("+", "-")):
        cleaned = cleaned[1:]
    if not cleaned:
        raise ValueError("tutar sayı değil")

    if "," in cleaned:
        whole, _, fraction = cleaned.partition(",")
        if "." in fraction or "," in fraction:
            raise ValueError("tutar biçimi bozuk")
        whole_digits = whole.replace(".", "")
        normalized = f"{whole_digits}.{fraction}" if fraction else whole_digits
    else:
        # Without a comma, a dot can only be a decimal point, not thousands
        # grouping (the specified fixtures use "1.234,56" for grouping).
        normalized = cleaned

    if not normalized or not normalized.replace(".", "", 1).isdigit():
        raise ValueError("tutar sayı değil")

    try:
        result = float(Decimal(normalized))
    except (InvalidOperation, ValueError) as exc:
        raise ValueError("tutar sayı değil") from exc
    return -result if negative else result


def parse_date(value: str) -> date:
    """Parse either DD.MM.YYYY or YYYY-MM-DD."""
    if not isinstance(value, str):
        raise ValueError("tarih metin olmalı")

    text = value.strip()
    try:
        if "." in text:
            day, month, year = text.split(".", 2)
            return date(int(year), int(month), int(day))
        # ISO output is normalized, but the input contract is only the
        # extended YYYY-MM-DD form; reject Python's other accepted variants.
        if len(text) != 10 or text.count("-") != 2:
            raise ValueError("tarih biçimi bozuk")
        year, month, day = text.split("-", 2)
        if not (year.isdigit() and month.isdigit() and day.isdigit()):
            raise ValueError("tarih biçimi bozuk")
        return date(int(year), int(month), int(day))
    except (TypeError, ValueError) as exc:
        raise ValueError("tarih biçimi bozuk") from exc


def iso_date(value: str) -> str:
    return parse_date(value).isoformat()


def _rows_with_errors(path: str | Path, fields: tuple[str, ...]):
    rows: list[dict[str, object]] = []
    errors: list[dict[str, str]] = []

    with open(path, "r", encoding="utf-8-sig", newline="") as handle:
        reader = csv.DictReader(handle, delimiter=";")
        if reader.fieldnames is None:
            return rows, errors

        expected = list(fields)
        missing_columns = [name for name in expected if name not in reader.fieldnames]
        if missing_columns:
            raise ValueError("eksik başlık: " + ", ".join(missing_columns))

        for record in reader:
            line_number = reader.line_num
            values = {
                name: (record.get(name) or "").strip()
                for name in expected
            }
            extra = record.get(None)
            invalid = None

            if any(not values[name] for name in expected):
                invalid = "eksik alan"
            elif extra:
                invalid = "fazla alan"

            if invalid is None:
                try:
                    amount = parse_amount(values["tutar"])
                    row_date = iso_date(values["tarih"])
                except ValueError as exc:
                    invalid = str(exc)
                else:
                    base = {name: values[name] for name in expected
                            if name not in ("tutar", "tarih")}
                    base["tutar"] = amount
                    base["tarih"] = row_date
                    rows.append(base)

            if invalid is not None:
                errors.append({"line": line_number, "reason": invalid})

    return rows, errors


def load_invoices(path: str | Path):
    """Load valid invoices and collect invalid physical lines."""
    rows, errors = _rows_with_errors(path, INVOICE_FIELDS)
    for row in rows:
        row["para_birimi"] = str(row["para_birimi"]).upper()
    return rows, errors


def load_payments(path: str | Path):
    """Load payments and assign p1, p2, ... to valid rows in file order."""
    rows, _ = _rows_with_errors(path, PAYMENT_FIELDS)
    for index, row in enumerate(rows, start=1):
        row["id"] = f"p{index}"
        row["para_birimi"] = str(row["para_birimi"]).upper()
        # Keep the API's documented field order stable.
        ordered = {
            "id": row["id"],
            "tarih": row["tarih"],
            "aciklama": row["aciklama"],
            "tutar": row["tutar"],
            "para_birimi": row["para_birimi"],
        }
        row.clear()
        row.update(ordered)
    return rows


def _rate_for(currency: str, on_date: date, rates: dict):
    currency = currency.upper()
    candidates: list[tuple[date, float]] = []
    for rate_date_text, currency_rates in rates.items():
        try:
            rate_date = parse_date(rate_date_text)
        except ValueError:
            continue
        if currency in currency_rates:
            try:
                rate = float(currency_rates[currency])
            except (TypeError, ValueError):
                continue
            candidates.append((rate_date, rate))

    previous = [item for item in candidates if item[0] <= on_date]
    if not previous:
        raise ValueError(f"{currency} için {on_date.isoformat()} kuruna kadar kur yok")

    rate_date, rate = max(previous, key=lambda item: item[0])
    if (on_date - rate_date).days > RATE_FALLBACK_DAYS:
        raise ValueError(
            f"{currency} için {on_date.isoformat()} tarihine uygun kur yok"
        )
    return rate


def to_try(amount: float, currency: str, date_text: str, rates: dict) -> float:
    """Convert an amount to TRY, using at most a seven-day-old prior rate."""
    if currency.upper() == "TRY":
        return round(float(amount), 2)
    on_date = parse_date(date_text)
    rate = _rate_for(currency, on_date, rates)
    return round(float(amount) * rate, 2)


def _amount_difference(
    left_amount: float,
    left_currency: str,
    right_amount: float,
    right_currency: str,
    left_date: str,
    right_date: str,
    rates: dict,
) -> float:
    if left_currency == right_currency:
        return round(abs(float(left_amount) - float(right_amount)), 2)
    return round(
        abs(
            to_try(left_amount, left_currency, left_date, rates)
            - to_try(right_amount, right_currency, right_date, rates)
        ),
        2,
    )


def _within_tolerance(difference: float) -> bool:
    return difference <= TOLERANCE


def _first_word(text: str) -> str:
    words = text.split()
    return words[0].lower() if words else ""


def _fark_try(
    invoice: dict,
    matched_payments: Iterable[dict],
    rates: dict,
) -> float:
    payments = list(matched_payments)
    invoice_amount = float(invoice["tutar"])
    invoice_currency = str(invoice["para_birimi"]).upper()
    payment_currencies = {str(payment["para_birimi"]).upper() for payment in payments}

    if payment_currencies == {invoice_currency}:
        payment_amount = sum(float(payment["tutar"]) for payment in payments)
        difference = abs(invoice_amount - payment_amount)
        return round(to_try(difference, invoice_currency, invoice["tarih"], rates), 2)

    invoice_try = to_try(
        invoice_amount, invoice_currency, invoice["tarih"], rates
    )
    payments_try = sum(
        to_try(
            float(payment["tutar"]),
            payment["para_birimi"],
            payment["tarih"],
            rates,
        )
        for payment in payments
    )
    return round(abs(invoice_try - payments_try), 2)


def match(invoices: list[dict], payments: list[dict], rates: dict) -> dict:
    """Match invoices to at most one payment each, using the specified rules."""
    available = {str(payment["id"]): payment for payment in payments}
    payment_order = [str(payment["id"]) for payment in payments]
    matched: list[dict[str, object]] = []
    used_invoice_numbers: set[str] = set()

    for invoice in invoices:
        invoice_number = str(invoice["fatura_no"])
        invoice_currency = str(invoice["para_birimi"]).upper()
        selected_ids: list[str] | None = None

        # Rule 1: exact reference and amount.
        for payment_id in payment_order:
            payment = available.get(payment_id)
            if payment is None:
                continue
            if invoice_number not in str(payment["aciklama"]):
                continue
            difference = _amount_difference(
                invoice["tutar"],
                invoice_currency,
                payment["tutar"],
                payment["para_birimi"],
                invoice["tarih"],
                payment["tarih"],
                rates,
            )
            if _within_tolerance(difference):
                selected_ids = [payment_id]
                break

        # Rule 2: amount, nearby date, and the customer's first word.
        if selected_ids is None:
            customer_word = _first_word(str(invoice["musteri"]))
            invoice_day = parse_date(invoice["tarih"])
            for payment_id in payment_order:
                payment = available.get(payment_id)
                if payment is None:
                    continue
                if customer_word not in str(payment["aciklama"]).lower():
                    continue
                difference = _amount_difference(
                    invoice["tutar"],
                    invoice_currency,
                    payment["tutar"],
                    payment["para_birimi"],
                    invoice["tarih"],
                    payment["tarih"],
                    rates,
                )
                if not _within_tolerance(difference):
                    continue
                if abs((parse_date(payment["tarih"]) - invoice_day).days) > DATE_MATCH_DAYS:
                    continue
                selected_ids = [payment_id]
                break

        # Rule 3: a two-payment split, each containing the invoice reference.
        if selected_ids is None:
            candidates = [
                payment_id
                for payment_id in payment_order
                if payment_id in available
                and invoice_number in str(available[payment_id]["aciklama"])
            ]
            for first_id, second_id in combinations(candidates, 2):
                first = available[first_id]
                second = available[second_id]
                if first["para_birimi"] == second["para_birimi"]:
                    first_value = float(first["tutar"])
                    second_value = float(second["tutar"])
                    difference = _amount_difference(
                        invoice["tutar"],
                        invoice_currency,
                        first_value + second_value,
                        first["para_birimi"],
                        invoice["tarih"],
                        first["tarih"],
                        rates,
                    )
                else:
                    payment_try = sum(
                        to_try(
                            float(payment["tutar"]),
                            payment["para_birimi"],
                            payment["tarih"],
                            rates,
                        )
                        for payment in (first, second)
                    )
                    difference = _amount_difference(
                        invoice["tutar"],
                        invoice_currency,
                        payment_try,
                        "TRY",
                        invoice["tarih"],
                        invoice["tarih"],
                        rates,
                    )
                if _within_tolerance(difference):
                    selected_ids = [first_id, second_id]
                    break

        if selected_ids is None:
            continue

        selected_payments = [available.pop(payment_id) for payment_id in selected_ids]
        matched.append(
            {
                "fatura_no": invoice_number,
                "payment_ids": selected_ids,
                "fark_try": _fark_try(invoice, selected_payments, rates),
            }
        )
        used_invoice_numbers.add(invoice_number)

    unmatched_invoices = [
        str(invoice["fatura_no"]) for invoice in invoices
        if str(invoice["fatura_no"]) not in used_invoice_numbers
    ]
    return {
        "matched": matched,
        "unmatched_invoices": unmatched_invoices,
        "unmatched_payments": list(available.keys()),
    }


def _report(fixtures_dir: str | Path) -> dict:
    fixtures = Path(fixtures_dir)
    invoices, errors = load_invoices(fixtures / "invoices.csv")
    payments = load_payments(fixtures / "bank.csv")
    with open(fixtures / "rates.json", "r", encoding="utf-8") as handle:
        rates = json.load(handle)

    result = match(invoices, payments, rates)
    total_difference = round(
        sum(float(item["fark_try"]) for item in result["matched"]), 2
    )
    return {
        "matched": len(result["matched"]),
        "unmatched_invoices": result["unmatched_invoices"],
        "unmatched_payments": result["unmatched_payments"],
        "toplam_fark_try": total_difference,
        "hatali_satirlar": len(errors),
    }


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)
    report_parser = subparsers.add_parser("report", help="mutabakat özeti üret")
    report_parser.add_argument("--fixtures", required=True)
    args = parser.parse_args(argv)

    if args.command == "report":
        print(json.dumps(_report(args.fixtures), ensure_ascii=False))
        return 0
    return 2


if __name__ == "__main__":
    sys.exit(main())
