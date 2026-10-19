"""DESTEK — kurumsal destek talep (ticket) sistemi çekirdeği.

Yalnızca standart kütüphane. Tüm zaman damgaları ISO "YYYY-MM-DDTHH:MM"
biçimindedir (yerel, tz'siz). Tüm sıralamalar deterministiktir.
"""

from __future__ import annotations

import argparse
import json
import sys
from datetime import date, datetime, timedelta, time as dtime
from pathlib import Path

DAY = timedelta(days=1)
REOPEN_WINDOW = timedelta(days=14)
WORK_START = dtime(9, 0)
WORK_END = dtime(18, 0)

# Düşükten yükseğe; "yükseltme" = dizinde bir sola.
PRIORITY_LADDER = ["P4", "P3", "P2", "P1"]

P1_WORDS = ("çöktü", "kesinti", "veri kaybı", "güvenlik")
P2_WORDS = ("hata", "çalışmıyor", "yavaş")

ALLOWED_TRANSITIONS = {
    ("open", "pending"),
    ("pending", "open"),
    ("open", "resolved"),
    ("pending", "resolved"),
    ("resolved", "closed"),
}

OPEN_STATUSES = ("new", "open", "pending")


# --------------------------------------------------------------------------- #
# Yardımcılar
# --------------------------------------------------------------------------- #

def parse_ts(s: str) -> datetime:
    return datetime.strptime(s, "%Y-%m-%dT%H:%M")


def bump_priority(priority: str) -> str:
    """Önceliği bir seviye yükseltir; P1 tavandır."""
    try:
        idx = PRIORITY_LADDER.index(priority)
    except ValueError:
        return "P1"
    return PRIORITY_LADDER[min(idx + 1, len(PRIORITY_LADDER) - 1)]


def is_business_day(d: date, holidays) -> bool:
    return d.weekday() < 5 and d.isoformat() not in set(holidays)


def _minutes_between(t1: dtime, t2: dtime) -> int:
    delta = (datetime.combine(date.min, t2) - datetime.combine(date.min, t1))
    return int(delta.total_seconds() // 60)


# --------------------------------------------------------------------------- #
# 1. Önceliklendirme
# --------------------------------------------------------------------------- #

def derive_priority(ticket: dict) -> str:
    priority = ticket.get("priority")
    if priority:
        return priority

    text = (ticket.get("subject", "") + " " + ticket.get("description", "")).lower()
    if any(w in text for w in P1_WORDS):
        base = "P1"
    elif any(w in text for w in P2_WORDS):
        base = "P2"
    else:
        base = "P3"

    if ticket.get("channel") == "telefon":
        base = bump_priority(base)
    if ticket.get("plan") == "kurumsal":
        base = bump_priority(base)
    return base


# --------------------------------------------------------------------------- #
# 2. Yönlendirme ve atama
# --------------------------------------------------------------------------- #

def _sorted_candidates(candidates, open_counts):
    return sorted(candidates, key=lambda a: (open_counts.get(a["id"], 0), a["id"]))


def route(ticket: dict, agents, teams, open_counts):
    team = teams.get(ticket.get("category"))
    if team is None:
        return None

    def has_capacity(a):
        return open_counts.get(a["id"], 0) < a["max_open"]

    same_team = [a for a in agents if a.get("team") == team and has_capacity(a)]
    skilled = [a for a in same_team if ticket.get("category") in a.get("skills", [])]

    for pool in (_sorted_candidates(skilled, open_counts),
                 _sorted_candidates(same_team, open_counts)):
        if pool:
            return pool[0]["id"]
    return None


def assign_all(tickets, agents, teams) -> dict:
    open_counts = {a["id"]: 0 for a in agents}
    out = {}
    for t in sorted(tickets, key=lambda t: (t["created_at"], t["id"])):
        agent_id = route(t, agents, teams, open_counts)
        out[t["id"]] = agent_id
        if agent_id is not None:
            open_counts[agent_id] += 1
    return out


# --------------------------------------------------------------------------- #
# 3. İş saati ve SLA
# --------------------------------------------------------------------------- #

def business_minutes_between(a: str, b: str, holidays) -> int:
    da = parse_ts(a)
    db = parse_ts(b)
    total = 0
    d = da.date()
    end = db.date()
    while d <= end:
        if is_business_day(d, holidays):
            lo = WORK_START
            hi = WORK_END
            if d == da.date():
                lo = max(lo, da.time())
            if d == db.date():
                hi = min(hi, db.time())
            if hi > lo:
                total += _minutes_between(lo, hi)
        d += DAY
    return total


def add_business_minutes(start: str, minutes: int, holidays) -> str:
    cur = parse_ts(start)
    d = cur.date()
    t = cur.time()

    # Mesai dışı başlangıç bir sonraki mesai açılışına yuvarlanır.
    if not (is_business_day(d, holidays) and WORK_START <= t < WORK_END):
        if is_business_day(d, holidays) and t < WORK_START:
            t = WORK_START
        else:
            d += DAY
            while not is_business_day(d, holidays):
                d += DAY
            t = WORK_START

    remaining = int(minutes)
    while True:
        available = _minutes_between(t, WORK_END)
        if remaining <= available:
            result = datetime.combine(d, t) + timedelta(minutes=remaining)
            return result.strftime("%Y-%m-%dT%H:%M")
        remaining -= available
        d += DAY
        while not is_business_day(d, holidays):
            d += DAY
        t = WORK_START


def _deadlines(ticket: dict, sla, holidays):
    priority = derive_priority(ticket)
    targets = sla.get(priority, {})
    created = ticket["created_at"]
    first = add_business_minutes(created, targets.get("first_response", 0), holidays)
    resolve = add_business_minutes(created, targets.get("resolve", 0), holidays)
    return first, resolve


# --------------------------------------------------------------------------- #
# 4. Yaşam döngüsü — olay yeniden-oynatma
# --------------------------------------------------------------------------- #

def replay(tickets, events, agents, teams, sla, holidays) -> dict:
    assignments = assign_all(tickets, agents, teams)
    state = {}
    for t in tickets:
        first_dl, resolve_dl = _deadlines(t, sla, holidays)
        state[t["id"]] = {
            "status": "new",
            "assignee": assignments.get(t["id"]),
            "priority": derive_priority(t),
            "first_response_at": None,
            "resolved_at": None,
            "reopen_count": 0,
            "escalated": False,
            "tags": list(t.get("tags", [])),
            "first_response_deadline": first_dl,
            "resolve_deadline": resolve_dl,
        }

    audit = []

    for ev in sorted(events, key=lambda e: e["ts"]):
        ts = ev["ts"]

        # Eskalasyon: her olay uygulanmadan ÖNCE tüm biletler için bakılır.
        for tid in sorted(state):
            st = state[tid]
            if (st["status"] not in ("resolved", "closed")
                    and not st["escalated"]
                    and st["resolve_deadline"] < ts):
                st["priority"] = bump_priority(st["priority"])
                st["escalated"] = True
                audit.append({"ts": ts, "ticket_id": tid, "action": "escalated"})

        st = state.get(ev.get("ticket_id"))
        if st is None:
            continue
        etype = ev.get("type")

        if etype == "agent_reply":
            if st["first_response_at"] is None:
                st["first_response_at"] = ts
            if st["status"] == "new":
                st["status"] = "open"

        elif etype == "customer_reply":
            if st["status"] == "pending":
                st["status"] = "open"
            elif (st["status"] == "resolved" and st["resolved_at"] is not None
                    and parse_ts(ts) - parse_ts(st["resolved_at"]) <= REOPEN_WINDOW):
                st["status"] = "open"
                st["reopen_count"] += 1
                st["resolved_at"] = None

        elif etype == "status":
            to = ev.get("to")
            if (st["status"], to) in ALLOWED_TRANSITIONS:
                st["status"] = to
                if to == "resolved":
                    st["resolved_at"] = ts
            else:
                audit.append({"ts": ts, "ticket_id": ev["ticket_id"],
                              "error": "invalid_transition"})

        elif etype == "tag":
            value = ev.get("value")
            if value is not None and value not in st["tags"]:
                st["tags"].append(value)

    for st in state.values():
        st["tags"] = sorted(st["tags"])

    state["_audit"] = audit
    return state


# --------------------------------------------------------------------------- #
# 5. Rapor CLI
# --------------------------------------------------------------------------- #

def build_report(state, tickets, teams) -> dict:
    categories = {t["id"]: t.get("category") for t in tickets}
    ticket_ids = [k for k in state if k != "_audit"]

    open_by_team = {}
    agent_open = {}
    for tid in ticket_ids:
        st = state[tid]
        if st["status"] in OPEN_STATUSES:
            team = teams.get(categories.get(tid))
            open_by_team[team] = open_by_team.get(team, 0) + 1
            if st["assignee"] is not None:
                agent_open[st["assignee"]] = agent_open.get(st["assignee"], 0) + 1

    met = breached = 0
    for tid in ticket_ids:
        st = state[tid]
        if st["first_response_at"] is not None:
            if parse_ts(st["first_response_at"]) <= parse_ts(st["first_response_deadline"]):
                met += 1
            else:
                breached += 1

    return {
        "open_by_team": {k: v for k, v in sorted(open_by_team.items()) if v > 0},
        "first_response": {"met": met, "breached": breached},
        "escalated": sorted(tid for tid in ticket_ids if state[tid]["escalated"]),
        "reopened": sorted(tid for tid in ticket_ids if state[tid]["reopen_count"] > 0),
        "unassigned": sorted(tid for tid in ticket_ids if state[tid]["assignee"] is None),
        "agent_open": {k: v for k, v in sorted(agent_open.items()) if v > 0},
    }


def load_fixtures(fixtures_dir: str) -> dict:
    data = {}
    for path in sorted(Path(fixtures_dir).glob("*.json")):
        data[path.stem] = json.loads(path.read_text(encoding="utf-8"))
    return data


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(prog="solution.py")
    sub = parser.add_subparsers(dest="command", required=True)
    rep = sub.add_parser("report", help="replay sonucundan özet rapor üret")
    rep.add_argument("--fixtures", required=True)
    rep.add_argument("--now", required=True, help="rapor anı (bilgi amaçlı)")
    args = parser.parse_args(argv)

    if args.command == "report":
        data = load_fixtures(args.fixtures)
        state = replay(data["tickets"], data["events"], data["agents"],
                       data["teams"], data["sla"], data["holidays"])
        report = build_report(state, data["tickets"], data["teams"])
        print(json.dumps(report))
    return 0


if __name__ == "__main__":
    sys.exit(main())
