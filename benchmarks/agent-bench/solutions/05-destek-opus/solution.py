"""DESTEK — Kurumsal Destek Talep (Ticket) Sistemi.

Standart kütüphane ile tam yardım masası çekirdeği: önceliklendirme,
yönlendirme/atama, iş-saati temelli SLA, olay yeniden-oynatma, eskalasyon,
denetim izi ve metrik raporu.
"""

from __future__ import annotations

import argparse
import json
import sys
from datetime import datetime, time, timedelta
from pathlib import Path

FMT = "%Y-%m-%dT%H:%M"
PRIORITIES = ["P1", "P2", "P3", "P4"]

WORK_OPEN = time(9, 0)
WORK_CLOSE = time(18, 0)


# --------------------------------------------------------------------------- #
# zaman yardımcıları
# --------------------------------------------------------------------------- #

def _parse(s: str) -> datetime:
    return datetime.strptime(s, FMT)


def _fmt(dt: datetime) -> str:
    return dt.strftime(FMT)


def _is_business_day(d, holidays) -> bool:
    return d.weekday() < 5 and d.isoformat() not in holidays


def _next_business_open(cur: datetime, holidays) -> datetime:
    d = cur.date() + timedelta(days=1)
    while not _is_business_day(d, holidays):
        d += timedelta(days=1)
    return datetime.combine(d, WORK_OPEN)


def _roll_forward(cur: datetime, holidays) -> datetime:
    """Mesai dışı bir anı bir sonraki mesai açılışına yuvarlar."""
    while not (
        _is_business_day(cur.date(), holidays)
        and WORK_OPEN <= cur.time() < WORK_CLOSE
    ):
        if _is_business_day(cur.date(), holidays) and cur.time() < WORK_OPEN:
            cur = datetime.combine(cur.date(), WORK_OPEN)
        else:
            cur = _next_business_open(cur, holidays)
    return cur


def business_minutes_between(a: str, b: str, holidays) -> int:
    start = _parse(a)
    end = _parse(b)
    if end <= start:
        return 0
    hol = set(holidays)
    total = 0
    day = start.date()
    while day <= end.date():
        if _is_business_day(day, hol):
            day_open = datetime.combine(day, WORK_OPEN)
            day_close = datetime.combine(day, WORK_CLOSE)
            s = max(start, day_open)
            e = min(end, day_close)
            if e > s:
                total += int((e - s).total_seconds() // 60)
        day += timedelta(days=1)
    return total


def add_business_minutes(start: str, minutes: int, holidays) -> str:
    hol = set(holidays)
    cur = _roll_forward(_parse(start), hol)
    remaining = minutes
    while True:
        close = datetime.combine(cur.date(), WORK_CLOSE)
        avail = int((close - cur).total_seconds() // 60)
        if remaining <= avail:
            return _fmt(cur + timedelta(minutes=remaining))
        remaining -= avail
        cur = _next_business_open(cur, hol)


# --------------------------------------------------------------------------- #
# önceliklendirme
# --------------------------------------------------------------------------- #

def _bump(priority: str) -> str:
    idx = PRIORITIES.index(priority)
    return PRIORITIES[max(0, idx - 1)]


def derive_priority(ticket) -> str:
    if ticket.get("priority") is not None:
        return ticket["priority"]
    text = (ticket.get("subject", "") + " " + ticket.get("description", "")).lower()
    p1 = ["çöktü", "kesinti", "veri kaybı", "güvenlik"]
    p2 = ["hata", "çalışmıyor", "yavaş"]
    if any(k in text for k in p1):
        p = "P1"
    elif any(k in text for k in p2):
        p = "P2"
    else:
        p = "P3"
    if ticket.get("channel") == "telefon":
        p = _bump(p)
    if ticket.get("plan") == "kurumsal":
        p = _bump(p)
    return p


# --------------------------------------------------------------------------- #
# yönlendirme ve atama
# --------------------------------------------------------------------------- #

def route(ticket, agents, teams, open_counts):
    team = teams.get(ticket["category"])
    team_agents = [a for a in agents if a["team"] == team]

    def has_cap(a):
        return open_counts.get(a["id"], 0) < a["max_open"]

    skilled = [
        a for a in team_agents
        if ticket["category"] in a.get("skills", []) and has_cap(a)
    ]
    pool = skilled if skilled else [a for a in team_agents if has_cap(a)]
    if not pool:
        return None
    pool.sort(key=lambda a: (open_counts.get(a["id"], 0), a["id"]))
    return pool[0]["id"]


def assign_all(tickets, agents, teams):
    counts = {}
    result = {}
    for t in sorted(tickets, key=lambda t: (t["created_at"], t["id"])):
        aid = route(t, agents, teams, counts)
        result[t["id"]] = aid
        if aid is not None:
            counts[aid] = counts.get(aid, 0) + 1
    return result


# --------------------------------------------------------------------------- #
# yaşam döngüsü — olay yeniden-oynatma
# --------------------------------------------------------------------------- #

_ALLOWED = {
    ("open", "pending"),
    ("pending", "open"),
    ("open", "resolved"),
    ("pending", "resolved"),
    ("resolved", "closed"),
}


def replay(tickets, events, agents, teams, sla, holidays):
    assignments = assign_all(tickets, agents, teams)
    state = {}
    order = []
    for t in tickets:
        p = derive_priority(t)
        state[t["id"]] = {
            "status": "new",
            "assignee": assignments.get(t["id"]),
            "priority": p,
            "first_response_at": None,
            "resolved_at": None,
            "reopen_count": 0,
            "escalated": False,
            "tags": list(t.get("tags", [])),
            "first_response_deadline": add_business_minutes(
                t["created_at"], sla[p]["first_response"], holidays
            ),
            "resolve_deadline": add_business_minutes(
                t["created_at"], sla[p]["resolve"], holidays
            ),
        }
        order.append(t["id"])

    audit = []

    for e in sorted(events, key=lambda e: e["ts"]):
        ts = e["ts"]

        # olay uygulanmadan ÖNCE eskalasyon kontrolü (tüm biletler, id sırası)
        for tid in order:
            st = state[tid]
            if (
                st["status"] not in ("resolved", "closed")
                and not st["escalated"]
                and st["resolve_deadline"] < ts
            ):
                st["priority"] = _bump(st["priority"])
                st["escalated"] = True
                audit.append({"ts": ts, "ticket_id": tid, "action": "escalated"})

        tid = e["ticket_id"]
        st = state[tid]
        typ = e["type"]

        if typ == "agent_reply":
            if st["first_response_at"] is None:
                st["first_response_at"] = ts
            if st["status"] == "new":
                st["status"] = "open"

        elif typ == "customer_reply":
            if st["status"] == "pending":
                st["status"] = "open"
            elif st["status"] == "resolved":
                if st["resolved_at"] is not None and (
                    _parse(ts) - _parse(st["resolved_at"]) <= timedelta(days=14)
                ):
                    st["status"] = "open"
                    st["reopen_count"] += 1
                    st["resolved_at"] = None

        elif typ == "status":
            to = e["to"]
            if (st["status"], to) in _ALLOWED:
                st["status"] = to
                if to == "resolved":
                    st["resolved_at"] = ts
            else:
                audit.append(
                    {"ts": ts, "ticket_id": tid, "error": "invalid_transition"}
                )

        elif typ == "tag":
            v = e["value"]
            if v not in st["tags"]:
                st["tags"].append(v)

    state["_audit"] = audit
    return state


# --------------------------------------------------------------------------- #
# rapor
# --------------------------------------------------------------------------- #

_OPEN_STATUSES = {"open", "pending", "new"}


def build_report(state, tickets, teams):
    by_id = {t["id"]: t for t in tickets}
    open_by_team = {}
    agent_open = {}
    first_met = 0
    first_breached = 0
    escalated = []
    reopened = []
    unassigned = []

    for tid, st in state.items():
        if tid == "_audit":
            continue
        t = by_id[tid]
        if st["status"] in _OPEN_STATUSES:
            team = teams.get(t["category"])
            open_by_team[team] = open_by_team.get(team, 0) + 1
            if st["assignee"] is not None:
                agent_open[st["assignee"]] = agent_open.get(st["assignee"], 0) + 1
        if st["first_response_at"] is not None:
            if st["first_response_at"] <= st["first_response_deadline"]:
                first_met += 1
            else:
                first_breached += 1
        if st["escalated"]:
            escalated.append(tid)
        if st["reopen_count"] > 0:
            reopened.append(tid)
        if st["assignee"] is None:
            unassigned.append(tid)

    return {
        "open_by_team": {k: open_by_team[k] for k in sorted(open_by_team)},
        "first_response": {"met": first_met, "breached": first_breached},
        "escalated": sorted(escalated),
        "reopened": sorted(reopened),
        "unassigned": sorted(unassigned),
        "agent_open": {k: agent_open[k] for k in sorted(agent_open)},
    }


# --------------------------------------------------------------------------- #
# CLI
# --------------------------------------------------------------------------- #

def _load_fixtures(fixtures_dir):
    base = Path(fixtures_dir)
    out = {}
    for name in ("agents", "teams", "tickets", "sla", "holidays", "events"):
        path = base / f"{name}.json"
        out[name] = json.loads(path.read_text())
    return out


def main(argv=None):
    parser = argparse.ArgumentParser(prog="solution.py")
    sub = parser.add_subparsers(dest="command", required=True)
    rep = sub.add_parser("report")
    rep.add_argument("--fixtures", required=True)
    rep.add_argument("--now", required=True)
    args = parser.parse_args(argv)

    if args.command == "report":
        data = _load_fixtures(args.fixtures)
        state = replay(
            data["tickets"], data["events"], data["agents"],
            data["teams"], data["sla"], data["holidays"],
        )
        report = build_report(state, data["tickets"], data["teams"])
        print(json.dumps(report))
        return 0
    return 1


if __name__ == "__main__":
    sys.exit(main())
