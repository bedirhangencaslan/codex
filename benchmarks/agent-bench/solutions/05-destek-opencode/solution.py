"""DESTEK — Kurumsal destek talep (ticket) sistemi çekirdeği.

Önceliklendirme, yönlendirme/atama, iş-saati temelli SLA, olay
yeniden-oynatma ile yaşam döngüsü, eskalasyon, denetim izi ve metrik raporu.
Yalnızca standart kütüphane kullanılır; tüm zaman damgaları ISO
``YYYY-MM-DDTHH:MM`` (yerel, tz'siz) biçimindedir.
"""

from __future__ import annotations

import argparse
import json
import sys
from datetime import date, datetime, time, timedelta
from pathlib import Path

TIME_FMT = "%Y-%m-%dT%H:%M"
OPENING = time(9, 0)
CLOSING = time(18, 0)
P1_KEYWORDS = ("çöktü", "kesinti", "veri kaybı", "güvenlik")
P2_KEYWORDS = ("hata", "çalışmıyor", "yavaş")
OPEN_STATUSES = frozenset({"new", "open", "pending"})
TERMINAL_STATUSES = frozenset({"resolved", "closed"})
ALLOWED_TRANSITIONS = frozenset(
    {
        ("open", "pending"),
        ("pending", "open"),
        ("open", "resolved"),
        ("pending", "resolved"),
        ("resolved", "closed"),
    }
)
REOPEN_WINDOW = timedelta(days=14)
ONE_MINUTE = timedelta(minutes=1)


def _parse(value: str) -> datetime:
    return datetime.strptime(value, TIME_FMT)


def _fmt(value: datetime) -> str:
    return value.strftime(TIME_FMT)


def _is_workday(day: date, holidays) -> bool:
    return day.weekday() < 5 and day.isoformat() not in set(holidays)


def _day_bounds(day: date) -> tuple[datetime, datetime]:
    return datetime.combine(day, OPENING), datetime.combine(day, CLOSING)


def _next_opening(dt: datetime, holidays) -> datetime:
    day = dt.date()
    while True:
        if _is_workday(day, holidays):
            opening, _ = _day_bounds(day)
            if opening > dt:
                return opening
        day += timedelta(days=1)


def _roll_to_opening(dt: datetime, holidays) -> datetime:
    if not _is_workday(dt.date(), holidays):
        return _next_opening(dt, holidays)
    opening, closing = _day_bounds(dt.date())
    if dt < opening:
        return opening
    if dt >= closing:
        return _next_opening(dt, holidays)
    return dt


def business_minutes_between(a: str, b: str, holidays) -> int:
    start, end = _parse(a), _parse(b)
    if end <= start:
        return 0
    total = 0
    day = start.date()
    while day <= end.date():
        if _is_workday(day, holidays):
            opening, closing = _day_bounds(day)
            lo = max(start, opening)
            hi = min(end, closing)
            if hi > lo:
                total += (hi - lo) // ONE_MINUTE
        day += timedelta(days=1)
    return int(total)


def add_business_minutes(start: str, minutes: int, holidays) -> str:
    dt = _roll_to_opening(_parse(start), holidays)
    remaining = int(minutes)
    while remaining > 0:
        _, closing = _day_bounds(dt.date())
        available = (closing - dt) // ONE_MINUTE
        if remaining <= available:
            dt += timedelta(minutes=remaining)
            remaining = 0
        else:
            remaining -= available
            dt = _next_opening(closing, holidays)
    return _fmt(dt)


def _bump(priority: str) -> str:
    level = int(priority[1:])
    return "P1" if level <= 1 else "P%d" % (level - 1)


def derive_priority(ticket: dict) -> str:
    priority = ticket.get("priority")
    if priority:
        return priority
    text = ((ticket.get("subject") or "") + " " + (ticket.get("description") or "")).lower()
    if any(word in text for word in P1_KEYWORDS):
        priority = "P1"
    elif any(word in text for word in P2_KEYWORDS):
        priority = "P2"
    else:
        priority = "P3"
    if ticket.get("channel") == "telefon":
        priority = _bump(priority)
    if ticket.get("plan") == "kurumsal":
        priority = _bump(priority)
    return priority


def route(ticket: dict, agents, teams, open_counts):
    category = ticket.get("category")
    team = teams.get(category)
    if team is None:
        return None

    def load_key(agent):
        return open_counts.get(agent["id"], 0), agent["id"]

    def has_capacity(agent):
        return open_counts.get(agent["id"], 0) < agent.get("max_open", 0)

    skilled = [
        agent
        for agent in agents
        if agent.get("team") == team
        and category in (agent.get("skills") or [])
        and has_capacity(agent)
    ]
    if skilled:
        return sorted(skilled, key=load_key)[0]["id"]
    fallback = [
        agent for agent in agents if agent.get("team") == team and has_capacity(agent)
    ]
    if fallback:
        return sorted(fallback, key=load_key)[0]["id"]
    return None


def assign_all(tickets, agents, teams):
    open_counts: dict = {}
    assignments: dict = {}
    ordered = sorted(tickets, key=lambda t: (t.get("created_at") or "", t.get("id") or ""))
    for ticket in ordered:
        agent_id = route(ticket, agents, teams, open_counts)
        assignments[ticket["id"]] = agent_id
        if agent_id is not None:
            open_counts[agent_id] = open_counts.get(agent_id, 0) + 1
    return assignments


def replay(tickets, events, agents, teams, sla, holidays):
    assignments = assign_all(tickets, agents, teams)
    state: dict = {}
    for ticket in tickets:
        priority = derive_priority(ticket)
        state[ticket["id"]] = {
            "status": "new",
            "assignee": assignments.get(ticket["id"]),
            "priority": priority,
            "first_response_at": None,
            "resolved_at": None,
            "reopen_count": 0,
            "escalated": False,
            "tags": sorted(set(ticket.get("tags") or [])),
            "first_response_deadline": add_business_minutes(
                ticket["created_at"], sla[priority]["first_response"], holidays
            ),
            "resolve_deadline": add_business_minutes(
                ticket["created_at"], sla[priority]["resolve"], holidays
            ),
        }

    audit: list = []
    for event in sorted(events, key=lambda e: e.get("ts") or ""):
        ts = event["ts"]
        for ticket_id in sorted(state):
            current = state[ticket_id]
            if current["status"] in TERMINAL_STATUSES or current["escalated"]:
                continue
            if _parse(current["resolve_deadline"]) < _parse(ts):
                current["priority"] = _bump(current["priority"])
                current["escalated"] = True
                audit.append({"ts": ts, "ticket_id": ticket_id, "action": "escalated"})

        ticket_id = event.get("ticket_id")
        if ticket_id not in state:
            continue
        current = state[ticket_id]
        etype = event.get("type")
        if etype == "agent_reply":
            if current["first_response_at"] is None:
                current["first_response_at"] = ts
            if current["status"] == "new":
                current["status"] = "open"
        elif etype == "customer_reply":
            if current["status"] == "pending":
                current["status"] = "open"
            elif current["status"] == "resolved":
                resolved_at = current["resolved_at"]
                if (
                    resolved_at is not None
                    and _parse(ts) - _parse(resolved_at) <= REOPEN_WINDOW
                ):
                    current["status"] = "open"
                    current["reopen_count"] += 1
                    current["resolved_at"] = None
        elif etype == "status":
            target = event.get("to")
            if (current["status"], target) in ALLOWED_TRANSITIONS:
                current["status"] = target
                if target == "resolved":
                    current["resolved_at"] = ts
            else:
                audit.append(
                    {"ts": ts, "ticket_id": ticket_id, "error": "invalid_transition"}
                )
        elif etype == "tag":
            value = event.get("value")
            if value is not None and value not in current["tags"]:
                current["tags"].append(value)
                current["tags"].sort()

    state["_audit"] = audit
    return state


def build_report(state, tickets, teams):
    categories = {t["id"]: t.get("category") for t in tickets}
    open_by_team: dict = {}
    agent_open: dict = {}
    met = 0
    breached = 0
    escalated: list = []
    reopened: list = []
    unassigned: list = []
    ticket_ids = sorted(k for k in state if k != "_audit")
    for ticket_id in ticket_ids:
        current = state[ticket_id]
        if current["status"] in OPEN_STATUSES:
            team = teams.get(categories.get(ticket_id))
            if team is not None:
                open_by_team[team] = open_by_team.get(team, 0) + 1
            if current["assignee"] is not None:
                agent_open[current["assignee"]] = agent_open.get(current["assignee"], 0) + 1
        if current["first_response_at"] is not None:
            if _parse(current["first_response_at"]) <= _parse(
                current["first_response_deadline"]
            ):
                met += 1
            else:
                breached += 1
        if current["escalated"]:
            escalated.append(ticket_id)
        if current["reopen_count"] > 0:
            reopened.append(ticket_id)
        if current["assignee"] is None:
            unassigned.append(ticket_id)
    return {
        "open_by_team": {team: open_by_team[team] for team in sorted(open_by_team)},
        "first_response": {"met": met, "breached": breached},
        "escalated": escalated,
        "reopened": reopened,
        "unassigned": unassigned,
        "agent_open": {agent: agent_open[agent] for agent in sorted(agent_open)},
    }


def load_fixtures(fixtures_dir) -> dict:
    data: dict = {}
    for path in sorted(Path(fixtures_dir).glob("*.json")):
        data[path.stem] = json.loads(path.read_text(encoding="utf-8"))
    return data


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(
        prog="solution.py", description="DESTEK kurumsal destek talep sistemi"
    )
    subparsers = parser.add_subparsers(dest="command", required=True)
    report_parser = subparsers.add_parser(
        "report", help="yeniden-oynatma sonucundan metrik raporu"
    )
    report_parser.add_argument("--fixtures", required=True, help="fixture JSON dizini")
    report_parser.add_argument(
        "--now", required=True, help="rapor zamanı (YYYY-MM-DDTHH:MM)"
    )
    args = parser.parse_args(argv)
    _parse(args.now)
    data = load_fixtures(args.fixtures)
    tickets = data.get("tickets", [])
    teams = data.get("teams", {})
    state = replay(
        tickets,
        data.get("events", []),
        data.get("agents", []),
        teams,
        data.get("sla", {}),
        data.get("holidays", []),
    )
    print(json.dumps(build_report(state, tickets, teams)))
    return 0


if __name__ == "__main__":
    sys.exit(main())
