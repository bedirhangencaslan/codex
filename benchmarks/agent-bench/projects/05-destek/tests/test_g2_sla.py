HOL = ["2026-10-29"]  # Perşembe


def test_business_minutes_same_day(solution):
    assert solution.business_minutes_between("2026-10-26T10:00", "2026-10-26T14:00", HOL) == 240
    # mesai dışı kısımlar sayılmaz
    assert solution.business_minutes_between("2026-10-26T08:00", "2026-10-26T10:00", HOL) == 60
    assert solution.business_minutes_between("2026-10-26T17:30", "2026-10-26T19:00", HOL) == 30


def test_business_minutes_skips_weekend_and_holiday(solution):
    # Cuma 17:00 → Pazartesi 10:00: 60 + 60
    assert solution.business_minutes_between("2026-10-23T17:00", "2026-10-26T10:00", HOL) == 120
    # Çarşamba 17:00 → Cuma 10:00 (Perşembe tatil): 60 + 60
    assert solution.business_minutes_between("2026-10-28T17:00", "2026-10-30T10:00", HOL) == 120


def test_add_business_minutes(solution):
    assert solution.add_business_minutes("2026-10-26T10:00", 30, HOL) == "2026-10-26T10:30"
    # gün taşması: Pzt 11:00 + 480 → Salı 10:00
    assert solution.add_business_minutes("2026-10-26T11:00", 480, HOL) == "2026-10-27T10:00"
    # mesai dışı başlangıç ileri yuvarlanır: Cumartesi → Pazartesi 09:00
    assert solution.add_business_minutes("2026-10-24T12:00", 60, HOL) == "2026-10-26T10:00"
    # tam 18:00'e denk gelen bitiş ertesi güne taşmaz
    assert solution.add_business_minutes("2026-10-26T14:00", 240, HOL) == "2026-10-26T18:00"
    # tatil atlanır: Çrş 17:30 + 60 → Cuma 09:30
    assert solution.add_business_minutes("2026-10-28T17:30", 60, HOL) == "2026-10-30T09:30"


def test_deadline_across_holiday(solution):
    # t3 senaryosu: Pzt 12:30 + 1440 iş dk → Cuma 09:30 (Perşembe tatil)
    assert solution.add_business_minutes("2026-10-26T12:30", 1440, HOL) == "2026-10-30T09:30"
