TYPES = ["O-", "O+", "A-", "A+", "B-", "B+", "AB-", "AB+"]


def expected_can_donate(donor: str, recipient: str) -> bool:
    """Bağımsız kâhin: ABO + Rh kuralından türetilir, çözümden kopyalanmaz."""
    d_abo, d_rh = donor[:-1], donor[-1]
    r_abo, r_rh = recipient[:-1], recipient[-1]
    abo_ok = d_abo == "O" or d_abo == r_abo or r_abo == "AB"
    rh_ok = d_rh == "-" or r_rh == "+"
    return abo_ok and rh_ok


def test_can_donate_all_64(solution):
    for d in TYPES:
        for r in TYPES:
            assert solution.can_donate(d, r) == expected_can_donate(d, r), f"{d} -> {r}"


def test_valid_stock_counts(solution, data):
    counts = solution.valid_stock(data["stock"], "2026-10-01")
    assert counts == {
        "O-": 2, "O+": 1, "A-": 0, "A+": 2,
        "B-": 0, "B+": 1, "AB-": 0, "AB+": 0,
    }


def test_valid_stock_42nd_day_is_expired(solution):
    units = [
        {"id": "x1", "blood_type": "O+", "collected_on": "2026-08-20", "city": "A"},  # 42 gün → geçersiz
        {"id": "x2", "blood_type": "O+", "collected_on": "2026-08-21", "city": "A"},  # 41 gün → geçerli
    ]
    counts = solution.valid_stock(units, "2026-10-01")
    assert counts["O+"] == 1


def test_valid_stock_has_all_eight_keys(solution):
    counts = solution.valid_stock([], "2026-10-01")
    assert sorted(counts) == sorted(TYPES)
    assert all(v == 0 for v in counts.values())
