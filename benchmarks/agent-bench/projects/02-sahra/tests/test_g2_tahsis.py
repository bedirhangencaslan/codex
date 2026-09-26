import copy


def test_n1_allocation_and_mutation(solution, data):
    warehouses = copy.deepcopy(data["warehouses"])
    n1 = next(n for n in data["needs"] if n["id"] == "n1")
    out = solution.allocate(n1, warehouses)
    assert out["allocations"] == [
        {"warehouse": "w1", "item": "battaniye", "qty": 40},
        {"warehouse": "w3", "item": "cadir", "qty": 10},
        {"warehouse": "w2", "item": "cadir", "qty": 2},
        {"warehouse": "w1", "item": "su", "qty": 40},
        {"warehouse": "w2", "item": "su", "qty": 10},
    ]
    assert out["unmet"] == {}
    stocks = {w["id"]: w["stock"] for w in warehouses}
    assert stocks["w1"]["battaniye"] == 10
    assert stocks["w1"]["su"] == 0
    assert stocks["w2"]["su"] == 20
    assert stocks["w2"]["cadir"] == 4
    assert stocks["w3"]["cadir"] == 0


def test_partial_and_unmet(solution):
    warehouses = [
        {"id": "wa", "city": "X", "stock": {"su": 5}},
        {"id": "wb", "city": "Y", "stock": {"su": 3}},
    ]
    need = {"id": "n", "city": "X", "items": {"su": 12, "cadir": 2},
            "population": 0, "severity": 1, "created_at": "2026-10-01T00:00"}
    out = solution.allocate(need, warehouses)
    assert out["allocations"] == [
        {"warehouse": "wa", "item": "su", "qty": 5},
        {"warehouse": "wb", "item": "su", "qty": 3},
    ]
    assert out["unmet"] == {"cadir": 2, "su": 4}


def test_tie_breaks_on_warehouse_id(solution):
    warehouses = [
        {"id": "wb", "city": "Y", "stock": {"su": 7}},
        {"id": "wa", "city": "X", "stock": {"su": 7}},
    ]
    need = {"id": "n", "city": "X", "items": {"su": 7},
            "population": 0, "severity": 1, "created_at": "2026-10-01T00:00"}
    out = solution.allocate(need, warehouses)
    assert out["allocations"] == [{"warehouse": "wa", "item": "su", "qty": 7}]
