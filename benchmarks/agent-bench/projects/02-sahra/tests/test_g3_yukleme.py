CATALOG = {"su": 10, "battaniye": 3, "cadir": 25}


def test_weight_desc_and_vehicle_order(solution):
    vehicles = [
        {"id": "v1", "warehouse": "w1", "capacity_kg": 800},
        {"id": "v2", "warehouse": "w1", "capacity_kg": 300},
    ]
    allocations = [
        {"warehouse": "w1", "item": "battaniye", "qty": 40},  # 120 kg
        {"warehouse": "w1", "item": "su", "qty": 40},          # 400 kg → önce bu
    ]
    used = {}
    out = solution.load_vehicles(allocations, vehicles, CATALOG, used)
    assert out["loads"] == {
        "v1": [
            {"item": "su", "qty": 40, "kg": 400},
            {"item": "battaniye", "qty": 40, "kg": 120},
        ]
    }
    assert out["unshipped"] == []
    assert used == {"v1": 520}


def test_split_across_vehicles_and_unshipped(solution):
    vehicles = [
        {"id": "v1", "warehouse": "w1", "capacity_kg": 100},
        {"id": "v2", "warehouse": "w1", "capacity_kg": 60},
    ]
    allocations = [{"warehouse": "w1", "item": "cadir", "qty": 7}]  # 175 kg
    used = {}
    out = solution.load_vehicles(allocations, vehicles, CATALOG, used)
    # v1: floor(100/25)=4 adet; v2: floor(60/25)=2 adet; kalan 1 → unshipped
    assert out["loads"] == {
        "v1": [{"item": "cadir", "qty": 4, "kg": 100}],
        "v2": [{"item": "cadir", "qty": 2, "kg": 50}],
    }
    assert out["unshipped"] == [{"warehouse": "w1", "item": "cadir", "qty": 1}]
    assert used == {"v1": 100, "v2": 50}


def test_shared_used_across_calls(solution):
    vehicles = [{"id": "v1", "warehouse": "w1", "capacity_kg": 100}]
    used = {}
    solution.load_vehicles([{"warehouse": "w1", "item": "su", "qty": 8}], vehicles, CATALOG, used)  # 80 kg
    out = solution.load_vehicles([{"warehouse": "w1", "item": "su", "qty": 3}], vehicles, CATALOG, used)
    # kalan 20 kg → 2 adet sığar, 1 adet unshipped
    assert out["loads"] == {"v1": [{"item": "su", "qty": 2, "kg": 20}]}
    assert out["unshipped"] == [{"warehouse": "w1", "item": "su", "qty": 1}]
    assert used == {"v1": 100}


def test_wrong_warehouse_vehicles_unused(solution):
    vehicles = [{"id": "vX", "warehouse": "OTHER", "capacity_kg": 999}]
    out = solution.load_vehicles([{"warehouse": "w1", "item": "su", "qty": 1}], vehicles, CATALOG, {})
    assert out["loads"] == {}
    assert out["unshipped"] == [{"warehouse": "w1", "item": "su", "qty": 1}]
