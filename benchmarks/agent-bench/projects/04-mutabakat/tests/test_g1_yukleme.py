def test_parse_amount(solution):
    assert solution.parse_amount("12.500,00") == 12500.0
    assert solution.parse_amount("1.850,50") == 1850.5
    assert solution.parse_amount("730,25") == 730.25


def test_load_invoices_rows_and_errors(solution, fixtures_dir):
    rows, errors = solution.load_invoices(str(fixtures_dir / "invoices.csv"))
    assert [r["fatura_no"] for r in rows] == [
        "F-2026-001", "F-2026-002", "F-2026-003", "F-2026-004", "F-2026-006", "F-2026-008",
    ]
    f1 = rows[0]
    assert f1 == {"fatura_no": "F-2026-001", "musteri": "Yılmaz İnşaat",
                  "tutar": 12500.0, "para_birimi": "TRY", "tarih": "2026-01-05"}
    # ISO tarih varyantı da normalize edilir
    assert rows[3]["tarih"] == "2026-01-12"
    assert sorted(e["line"] for e in errors) == [6, 8]


def test_load_payments_ids_in_file_order(solution, fixtures_dir):
    rows = solution.load_payments(str(fixtures_dir / "bank.csv"))
    assert [r["id"] for r in rows] == ["p1", "p2", "p3", "p4", "p5", "p6", "p7"]
    assert rows[0]["tarih"] == "2026-01-06"
    assert rows[1]["tutar"] == 1850.49
    assert rows[6]["para_birimi"] == "USD"
