import pytest

RATES = {"2026-01-10": {"EUR": 37.0, "USD": 34.0},
         "2026-01-15": {"EUR": 37.5, "USD": 34.3}}


def test_try_passthrough(solution):
    assert solution.to_try(950.0, "TRY", "2026-01-01", RATES) == 950.0


def test_direct_rate(solution):
    assert solution.to_try(100.0, "EUR", "2026-01-10", RATES) == 3700.0


def test_falls_back_to_previous_day_within_7(solution):
    # 2026-01-14 → en yakın önceki kur 2026-01-10 (4 gün geri)
    assert solution.to_try(100.0, "USD", "2026-01-14", RATES) == 3400.0
    # tam 7 gün geri de kabul: 2026-01-17 → 2026-01-10? hayır, 15 var (2 gün) → 3750
    assert solution.to_try(100.0, "EUR", "2026-01-17", RATES) == 3750.0
    assert solution.to_try(100.0, "EUR", "2026-01-22", RATES) == 3750.0  # 7 gün


def test_raises_beyond_seven_days(solution):
    with pytest.raises(ValueError):
        solution.to_try(100.0, "EUR", "2026-01-23", RATES)  # 8 gün
    with pytest.raises(ValueError):
        solution.to_try(100.0, "EUR", "2026-01-09", RATES)  # geçmişte kur yok


def test_rounds_to_two(solution):
    rates = {"2026-01-10": {"USD": 33.333}}
    assert solution.to_try(10.0, "USD", "2026-01-10", rates) == 333.33
