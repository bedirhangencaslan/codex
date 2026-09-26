def ticket(**kw):
    base = {"id": "tx", "subject": "", "description": "", "customer": "C",
            "plan": "standart", "channel": "portal", "category": "teknik",
            "priority": None, "created_at": "2026-10-26T10:00", "tags": []}
    base.update(kw)
    return base


def test_fixture_priorities(solution, data):
    got = {t["id"]: solution.derive_priority(t) for t in data["tickets"]}
    assert got == {"t1": "P1", "t2": "P2", "t3": "P3", "t4": "P1", "t5": "P4", "t6": "P3"}


def test_keyword_tiers_and_bumps(solution):
    assert solution.derive_priority(ticket(subject="Veri kaybı yaşıyoruz")) == "P1"
    assert solution.derive_priority(ticket(description="rapor hata veriyor")) == "P2"
    assert solution.derive_priority(ticket(subject="soru")) == "P3"
    # telefon + kurumsal: P3 → P2 → P1
    assert solution.derive_priority(ticket(subject="soru", channel="telefon", plan="kurumsal")) == "P1"
    # tavan: P1 üstüne çıkılmaz
    assert solution.derive_priority(ticket(subject="kesinti", channel="telefon", plan="kurumsal")) == "P1"
    # açık öncelik türetmeyi ezer
    assert solution.derive_priority(ticket(subject="çöktü", priority="P4")) == "P4"


def test_route_prefers_skill_then_load_then_id(solution, data):
    agents, teams = data["agents"], data["teams"]
    t = ticket(category="teknik")
    assert solution.route(t, agents, teams, {"a1": 0, "a2": 0}) == "a1"
    assert solution.route(t, agents, teams, {"a1": 1, "a2": 0}) == "a2"
    # kapasite dolunca beceri eşleşmesi olmayan takım arkadaşına düşmez (muhendislikte yok);
    # finans'ta beceri yoksa takım içi fallback: a3 dolu → a4 (skills boş ama takımda)
    tf = ticket(category="fatura")
    assert solution.route(tf, agents, teams, {"a3": 2, "a4": 0}) == "a4"
    # herkes doluysa None
    th = ticket(category="hesap")
    assert solution.route(th, agents, teams, {"a5": 1}) is None


def test_assign_all_sequential_loads(solution, data):
    out = solution.assign_all(data["tickets"], data["agents"], data["teams"])
    assert out == {"t1": "a1", "t2": "a3", "t3": "a5", "t4": "a2", "t5": "a1", "t6": None}
