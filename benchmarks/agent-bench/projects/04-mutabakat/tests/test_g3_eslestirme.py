import json


def loaded(solution, fixtures_dir):
    invoices, _ = solution.load_invoices(str(fixtures_dir / "invoices.csv"))
    payments = solution.load_payments(str(fixtures_dir / "bank.csv"))
    rates = json.loads((fixtures_dir / "rates.json").read_text())
    return invoices, payments, rates


def test_full_match_output(solution, fixtures_dir):
    invoices, payments, rates = loaded(solution, fixtures_dir)
    out = solution.match(invoices, payments, rates)
    assert out["matched"] == [
        {"fatura_no": "F-2026-001", "payment_ids": ["p1"], "fark_try": 0.0},
        {"fatura_no": "F-2026-002", "payment_ids": ["p2"], "fark_try": 0.01},
        {"fatura_no": "F-2026-003", "payment_ids": ["p3"], "fark_try": 0.0},
        {"fatura_no": "F-2026-004", "payment_ids": ["p4", "p5"], "fark_try": 0.0},
        {"fatura_no": "F-2026-006", "payment_ids": ["p7"], "fark_try": 0.0},
    ]
    assert out["unmatched_invoices"] == ["F-2026-008"]
    assert out["unmatched_payments"] == ["p6"]


def test_each_payment_used_once(solution, fixtures_dir):
    invoices, payments, rates = loaded(solution, fixtures_dir)
    out = solution.match(invoices, payments, rates)
    used = [p for m in out["matched"] for p in m["payment_ids"]]
    assert len(used) == len(set(used))


def test_rule2_needs_customer_word(solution, fixtures_dir):
    _, _, rates = loaded(solution, fixtures_dir)
    invoices = [{"fatura_no": "F-X", "musteri": "Demir Gıda", "tutar": 100.0,
                 "para_birimi": "TRY", "tarih": "2026-01-10"}]
    payments = [{"id": "p1", "tarih": "2026-01-11", "aciklama": "EFT BASKA FIRMA",
                 "tutar": 100.0, "para_birimi": "TRY"}]
    out = solution.match(invoices, payments, rates)
    assert out["matched"] == []
    assert out["unmatched_invoices"] == ["F-X"]


def test_rule2_date_window(solution, fixtures_dir):
    _, _, rates = loaded(solution, fixtures_dir)
    inv = [{"fatura_no": "F-X", "musteri": "Demir Gıda", "tutar": 100.0,
            "para_birimi": "TRY", "tarih": "2026-01-10"}]
    pay_far = [{"id": "p1", "tarih": "2026-01-14", "aciklama": "EFT DEMIR GIDA",
                "tutar": 100.0, "para_birimi": "TRY"}]  # 4 gün → olmaz
    assert solution.match(inv, pay_far, rates)["matched"] == []
    pay_ok = [dict(pay_far[0], tarih="2026-01-13")]      # 3 gün → olur
    assert solution.match(inv, pay_ok, rates)["matched"][0]["payment_ids"] == ["p1"]


def test_split_requires_exactly_two(solution, fixtures_dir):
    _, _, rates = loaded(solution, fixtures_dir)
    inv = [{"fatura_no": "F-Y", "musteri": "Kaya", "tutar": 300.0,
            "para_birimi": "TRY", "tarih": "2026-01-10"}]
    pays = [
        {"id": "p1", "tarih": "2026-01-10", "aciklama": "F-Y taksit", "tutar": 100.0, "para_birimi": "TRY"},
        {"id": "p2", "tarih": "2026-01-11", "aciklama": "F-Y taksit", "tutar": 100.0, "para_birimi": "TRY"},
        {"id": "p3", "tarih": "2026-01-12", "aciklama": "F-Y taksit", "tutar": 100.0, "para_birimi": "TRY"},
    ]
    # üç ödemenin ikilisi 200 ≠ 300; tam-iki kuralı → eşleşme yok
    assert solution.match(inv, pays, rates)["matched"] == []
