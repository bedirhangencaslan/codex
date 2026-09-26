ON = "2026-10-01"


def donor(**kw):
    base = {"id": "t", "name": "T", "blood_type": "O+", "sex": "E",
            "birth_date": "1990-01-01", "last_donation": None, "city": "Ankara"}
    base.update(kw)
    return base


def test_fixture_donors(solution, data):
    expected = {"d1": False, "d2": True, "d3": False, "d4": True,
                "d5": False, "d6": False, "d7": True}
    got = {d["id"]: solution.eligible(d, ON) for d in data["donors"]}
    assert got == expected


def test_interval_boundaries_male(solution):
    # erkek: tam 90 gün uygun, 89 gün değil (2026-10-01'den geriye)
    assert solution.eligible(donor(sex="E", last_donation="2026-07-03"), ON) is True   # 90
    assert solution.eligible(donor(sex="E", last_donation="2026-07-04"), ON) is False  # 89


def test_interval_boundaries_female(solution):
    assert solution.eligible(donor(sex="K", last_donation="2026-06-03"), ON) is True   # 120
    assert solution.eligible(donor(sex="K", last_donation="2026-06-04"), ON) is False  # 119


def test_age_boundaries(solution):
    assert solution.eligible(donor(birth_date="2008-10-01"), ON) is True    # bugün 18
    assert solution.eligible(donor(birth_date="2008-10-02"), ON) is False   # yarın 18
    assert solution.eligible(donor(birth_date="1961-10-01"), ON) is True    # bugün 65
    assert solution.eligible(donor(birth_date="1960-09-30"), ON) is False   # 66
