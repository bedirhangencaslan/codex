NOW = "2026-10-01T12:00"


def test_fixture_priorities(solution, data):
    got = {n["id"]: solution.priority(n, NOW) for n in data["needs"]}
    # n1: 5*100 + 10*2 + 12000//1000 = 532
    # n2: 4*100 + 24*2 + 30000//1000 = 478
    # n3: 5*100 + 3*2 + 4000//1000 = 510
    assert got == {"n1": 532, "n2": 478, "n3": 510}


def test_waiting_hours_floors(solution):
    need = {"id": "x", "city": "A", "items": {}, "population": 0,
            "severity": 1, "created_at": "2026-10-01T10:30"}
    # 1.5 saat → floor 1 → 100 + 2 + 0 = 102
    assert solution.priority(need, NOW) == 102


def test_population_floors(solution):
    need = {"id": "x", "city": "A", "items": {}, "population": 1999,
            "severity": 1, "created_at": NOW}
    assert solution.priority(need, NOW) == 101
