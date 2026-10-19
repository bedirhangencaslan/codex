ON = "2026-10-01"


def by_id(data, key, ident):
    return next(x for x in data[key] if x["id"] == ident)


def test_r1_city_bound_fifo_and_donor_call(solution, data):
    r1 = by_id(data, "requests", "r1")
    out = solution.match_request(r1, data["stock"], data["donors"], ON)
    # acil değil → yalnız Ankara stoğu; A+ alıcıya uyumlu geçerli Ankara torbaları:
    # u2 (A+, 09-05), u3 O+ (09-25), u7 O- (09-27); u1 SKT'li. FIFO → u2, u3, u7 = 3 torba, tam.
    assert out == {"from_stock": ["u2", "u3", "u7"], "donor_calls": [], "shortfall": 0}


def test_r2_urgent_crosses_cities(solution, data):
    r2 = by_id(data, "requests", "r2")
    out = solution.match_request(r2, data["stock"], data["donors"], ON)
    # acil → tüm şehirler; O- alıcıya yalnız O- verilebilir: u4 (08-25), u7 (09-27) FIFO
    assert out == {"from_stock": ["u4", "u7"], "donor_calls": [], "shortfall": 0}


def test_r3_fifo_within_city(solution, data):
    r3 = by_id(data, "requests", "r3")
    out = solution.match_request(r3, data["stock"], data["donors"], ON)
    # İstanbul'da AB+ alıcıya uyumlu geçerli torbalar: u4 (O-, 08-25) ve u6 (A+, 09-28);
    # FIFO → u4 önce, 1 ünite yeter.
    assert out == {"from_stock": ["u4"], "donor_calls": [], "shortfall": 0}


def test_donor_call_ordering_and_shortfall(solution, data):
    req = {"id": "rx", "blood_type": "AB+", "units": 4, "urgent": False,
           "city": "İstanbul", "created_at": "2026-09-30"}
    out = solution.match_request(req, data["stock"], data["donors"], ON)
    # stok: u4, u6; kalan 2 → İstanbul'daki uygun bağışçılar: d4 (AB+, null) —
    # d6 bağış aralığı nedeniyle uygun değil → çağrı [d4], shortfall 1
    assert out["from_stock"] == ["u4", "u6"]
    assert out["donor_calls"] == ["d4"]
    assert out["shortfall"] == 1


def test_urgent_donor_call_city_priority(solution, data):
    req = {"id": "ry", "blood_type": "AB+", "units": 6, "urgent": True,
           "city": "Ankara", "created_at": "2026-09-30"}
    out = solution.match_request(req, data["stock"], data["donors"], ON)
    # acil → stok tüm şehirler, AB+ alıcıya uyumlu geçerli torbalar FIFO:
    # u4 (O-, 08-25), u2 (A+, 09-05), u5 (B+, 09-20), u3 (O+, 09-25), u7 (O-, 09-27), u6 (A+, 09-28) → 6 torba
    assert out["from_stock"] == ["u4", "u2", "u5", "u3", "u7", "u6"]
    assert out["donor_calls"] == []
    assert out["shortfall"] == 0


def test_determinism(solution, data):
    r1 = by_id(data, "requests", "r1")
    a = solution.match_request(r1, data["stock"], data["donors"], ON)
    b = solution.match_request(r1, data["stock"], data["donors"], ON)
    assert a == b
