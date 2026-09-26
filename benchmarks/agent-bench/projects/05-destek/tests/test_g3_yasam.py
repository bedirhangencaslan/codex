import pytest


@pytest.fixture(scope="module")
def state(solution, data):
    return solution.replay(data["tickets"], data["events"], data["agents"],
                           data["teams"], data["sla"], data["holidays"])


def test_t1_escalated_then_reopened(state):
    t1 = state["t1"]
    assert t1["status"] == "open"            # resolved → reopen; closed geçişi geçersizdi
    assert t1["escalated"] is True           # 14:30 olayından önce, son teslim 14:00 aşılmıştı
    assert t1["reopen_count"] == 1
    assert t1["resolved_at"] is None         # yeniden açılınca sıfırlanır
    assert t1["first_response_at"] == "2026-10-26T10:10"
    assert t1["tags"] == ["kesinti"]


def test_t2_full_lifecycle(state):
    t2 = state["t2"]
    assert t2["status"] == "closed"
    assert t2["resolved_at"] == "2026-10-26T16:30"
    assert t2["reopen_count"] == 0
    assert t2["escalated"] is False


def test_invalid_transitions_ignored_and_audited(state):
    assert state["t3"]["status"] == "new"    # new→resolved geçersiz
    assert state["t6"]["status"] == "new"
    errors = [(a["ts"], a["ticket_id"]) for a in state["_audit"] if a.get("error") == "invalid_transition"]
    assert errors == [
        ("2026-10-26T16:00", "t3"),
        ("2026-10-26T17:30", "t1"),
        ("2026-10-27T12:00", "t6"),
    ]


def test_escalations_in_audit_order(state):
    esc = [(a["ts"], a["ticket_id"]) for a in state["_audit"] if a.get("action") == "escalated"]
    # t4, 27T10:00'daki olayın (t2 closed) ön-kontrolünde eskale olur — son teslimi 26T18:00'di
    assert esc == [("2026-10-26T14:30", "t1"), ("2026-10-27T10:00", "t4")]


def test_t4_escalation_keeps_deadlines(state):
    t4 = state["t4"]
    assert t4["escalated"] is True
    assert t4["priority"] == "P1"
    assert t4["resolve_deadline"] == "2026-10-26T18:00"   # eskalasyon son teslimi değiştirmez
    assert t4["first_response_at"] == "2026-10-27T11:00"


def test_deadline_equality_does_not_escalate(state):
    # t3'ün resolve_deadline'ı 2026-10-30T09:30; son olay tam 09:30'da geldi (< değil) → eskalasyon yok
    assert state["t3"]["resolve_deadline"] == "2026-10-30T09:30"
    assert state["t3"]["escalated"] is False


def test_assignments_carried_into_state(state):
    assert {k: state[k]["assignee"] for k in ["t1", "t4", "t5", "t6"]} == {
        "t1": "a1", "t4": "a2", "t5": "a1", "t6": None,
    }
