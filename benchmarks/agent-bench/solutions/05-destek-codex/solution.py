#!/usr/bin/env python3
"""Kurumsal destek talebi çekirdeği (yalnız standart kütüphane)."""

from __future__ import annotations

import argparse
import json
from datetime import date, datetime, time, timedelta
from pathlib import Path
from typing import Any, Iterable


OPENING = time(9, 0)
CLOSING = time(18, 0)
PRIORITIES = ("P1", "P2", "P3", "P4")
P1_KEYWORDS = ("çöktü", "kesinti", "veri kaybı", "güvenlik")
P2_KEYWORDS = ("hata", "çalışmıyor", "yavaş")
VALID_TRANSITIONS = {
    "open": {"pending", "resolved"},
    "pending": {"open", "resolved"},
    "resolved": {"closed"},
}


def _parse_moment(value: str) -> datetime:
    return datetime.strptime(value, "%Y-%m-%dT%H:%M")


def _format_moment(value: datetime) -> str:
    return value.strftime("%Y-%m-%dT%H:%M")


def derive_priority(ticket: dict[str, Any]) -> str:
    """Apply explicit priority or derive it from keywords, channel, and plan."""
    priority = ticket.get("priority")
    if priority is not None:
        return priority

    haystack = f'{ticket.get("subject", "")} {ticket.get("description", "")}'.lower()
    if any(keyword in haystack for keyword in P1_KEYWORDS):
        level = 0
    elif any(keyword in haystack for keyword in P2_KEYWORDS):
        level = 1
    else:
        level = 2

    if ticket.get("channel") == "telefon":
        level -= 1
    if ticket.get("plan") == "kurumsal":
        level -= 1
    level = max(0, min(len(PRIORITIES) - 1, level))
    return PRIORITIES[level]


def route(
    ticket: dict[str, Any],
    agents: list[dict[str, Any]],
    teams: dict[str, str],
    open_counts: dict[str, int],
) -> str | None:
    """Choose the lowest-loaded, lowest-id capable agent with capacity."""
    category = ticket["category"]
    team = teams.get(category)
    if team is None:
        return None

    def has_capacity(agent_id: str, agent: dict[str, Any]) -> bool:
        return open_counts.get(agent_id, 0) < agent.get("max_open", 0)

    skilled = sorted(
        (
            (open_counts.get(agent["id"], 0), agent["id"])
            for agent in agents
            if agent.get("team") == team
            and category in agent.get("skills", [])
            and has_capacity(agent["id"], agent)
        ),
    )
    if skilled:
        return skilled[0][1]

    fallback = sorted(
        (
            (open_counts.get(agent["id"], 0), agent["id"])
            for agent in agents
            if agent.get("team") == team and has_capacity(agent["id"], agent)
        ),
    )
    return fallback[0][1] if fallback else None


def assign_all(
    tickets: Iterable[dict[str, Any]],
    agents: list[dict[str, Any]],
    teams: dict[str, str],
) -> dict[str, str | None]:
    """Assign tickets in chronological order, updating live open counts."""
    counts = {agent["id"]: 0 for agent in agents}
    result: dict[str, str | None] = {}
    ordered = sorted(tickets, key=lambda ticket: (ticket["created_at"], ticket["id"]))
    for ticket in ordered:
        agent_id = route(ticket, agents, teams, counts)
        result[ticket["id"]] = agent_id
        if agent_id is not None:
            counts[agent_id] += 1
    return result


def _is_workday(day: date, holidays: set[str]) -> bool:
    return day.weekday() < 5 and day.isoformat() not in holidays


def _overlap_minutes(day_start: datetime, start: datetime, end: datetime) -> int:
    window_start = datetime.combine(day_start.date(), OPENING)
    window_end = datetime.combine(day_start.date(), CLOSING)
    return max(0, int((min(end, window_end) - max(start, window_start)).total_seconds() // 60))


def business_minutes_between(a: str, b: str, holidays: Iterable[str]) -> int:
    """Count minutes in the intersection of [a, b] and business hours."""
    start = _parse_moment(a)
    end = _parse_moment(b)
    if end < start:
        raise ValueError("business_minutes_between requires a <= b")

    holiday_dates = set(holidays)
    total = 0
    day = start.date()
    last_day = end.date()
    while day <= last_day:
        if _is_workday(day, holiday_dates):
            total += _overlap_minutes(datetime.combine(day, OPENING), start, end)
        day += timedelta(days=1)
    return total


def _roll_to_opening(start: datetime, holidays: set[str]) -> datetime:
    day = start.date()
    if not _is_workday(day, holidays) or start.time() < OPENING:
        while not _is_workday(day, holidays):
            day += timedelta(days=1)
        return datetime.combine(day, OPENING)
    if start.time() >= CLOSING:
        day += timedelta(days=1)
        while not _is_workday(day, holidays):
            day += timedelta(days=1)
        return datetime.combine(day, OPENING)
    return start


def add_business_minutes(start: str, minutes: int, holidays: Iterable[str]) -> str:
    """Add business minutes, without rolling an exact 18:00 finish forward."""
    if minutes < 0:
        raise ValueError("minutes must be non-negative")

    holiday_dates = set(holidays)
    current = _parse_moment(start)
    if minutes == 0:
        if current.time() < OPENING or current.time() >= CLOSING:
            return _format_moment(_roll_to_opening(current, holiday_dates))
        return _format_moment(current)

    current = _roll_to_opening(current, holiday_dates)
    remaining = minutes
    while True:
        closing = datetime.combine(current.date(), CLOSING)
        available = int((closing - current).total_seconds() // 60)
        if remaining <= available:
            return _format_moment(current + timedelta(minutes=remaining))

        remaining -= available
        day = current.date() + timedelta(days=1)
        while not _is_workday(day, holiday_dates):
            day += timedelta(days=1)
        current = datetime.combine(day, OPENING)


def _initial_state(
    tickets: Iterable[dict[str, Any]],
    agents: list[dict[str, Any]],
    teams: dict[str, str],
    sla: dict[str, dict[str, int]],
    holidays: Iterable[str],
) -> tuple[dict[str, dict[str, Any]], dict[str, str | None]]:
    assignments = assign_all(tickets, agents, teams)
    state: dict[str, dict[str, Any]] = {}
    for ticket in tickets:
        priority = derive_priority(ticket)
        durations = sla[priority]
        state[ticket["id"]] = {
            "status": "new",
            "assignee": assignments[ticket["id"]],
            "priority": priority,
            "first_response_at": None,
            "resolved_at": None,
            "reopen_count": 0,
            "escalated": False,
            "tags": sorted(set(ticket.get("tags", []))),
            "first_response_deadline": add_business_minutes(
                ticket["created_at"], durations["first_response"], holidays
            ),
            "resolve_deadline": add_business_minutes(
                ticket["created_at"], durations["resolve"], holidays
            ),
        }
    return state, assignments


def replay(
    tickets: Iterable[dict[str, Any]],
    events: Iterable[dict[str, Any]],
    agents: list[dict[str, Any]],
    teams: dict[str, str],
    sla: dict[str, dict[str, int]],
    holidays: Iterable[str],
) -> dict[str, Any]:
    """Replay the event stream, including pre-event deadline escalation."""
    ticket_list = list(tickets)
    state, _ = _initial_state(ticket_list, agents, teams, sla, holidays)
    audit: list[dict[str, str]] = []

    for event in sorted(events, key=lambda item: (item["ts"], item.get("ticket_id", ""))):
        event_ts = event["ts"]
        for ticket_id in sorted(state):
            ticket_state = state[ticket_id]
            if (
                not ticket_state["escalated"]
                and ticket_state["status"] not in {"resolved", "closed"}
                and ticket_state["resolve_deadline"] < event_ts
            ):
                old_index = PRIORITIES.index(ticket_state["priority"])
                ticket_state["priority"] = PRIORITIES[max(0, old_index - 1)]
                ticket_state["escalated"] = True
                audit.append({"ts": event_ts, "ticket_id": ticket_id, "action": "escalated"})

        ticket_id = event.get("ticket_id")
        if ticket_id not in state:
            continue
        ticket_state = state[ticket_id]
        event_type = event.get("type")

        if event_type == "agent_reply":
            if ticket_state["first_response_at"] is None:
                ticket_state["first_response_at"] = event_ts
            if ticket_state["status"] == "new":
                ticket_state["status"] = "open"

        elif event_type == "customer_reply":
            if ticket_state["status"] == "pending":
                ticket_state["status"] = "open"
            elif (
                ticket_state["status"] == "resolved"
                and ticket_state["resolved_at"] is not None
                and _parse_moment(event_ts) - _parse_moment(ticket_state["resolved_at"])
                <= timedelta(days=14)
            ):
                ticket_state["status"] = "open"
                ticket_state["reopen_count"] += 1
                ticket_state["resolved_at"] = None

        elif event_type == "status":
            target = event.get("to")
            if target in VALID_TRANSITIONS.get(ticket_state["status"], set()):
                ticket_state["status"] = target
                if target == "resolved":
                    ticket_state["resolved_at"] = event_ts
            else:
                audit.append(
                    {
                        "ts": event_ts,
                        "ticket_id": ticket_id,
                        "error": "invalid_transition",
                    }
                )

        elif event_type == "tag":
            value = event.get("value")
            if value not in ticket_state["tags"]:
                ticket_state["tags"].append(value)
                ticket_state["tags"].sort()

    state["_audit"] = audit
    return state


def _team_for_ticket(ticket_by_id: dict[str, dict[str, Any]], teams: dict[str, str], ticket_id: str) -> str:
    return teams.get(ticket_by_id[ticket_id]["category"], "")


def build_report(state: dict[str, Any], tickets: list[dict[str, Any]], teams: dict[str, str]) -> dict[str, Any]:
    """Summarize replay state for the report CLI."""
    ticket_by_id = {ticket["id"]: ticket for ticket in tickets}
    open_by_team: dict[str, int] = {}
    agent_open: dict[str, int] = {}
    first_response = {"met": 0, "breached": 0}

    for ticket_id in sorted(state):
        if ticket_id == "_audit":
            continue
        ticket_state = state[ticket_id]
        is_open = ticket_state["status"] in {"new", "open", "pending"}
        if is_open:
            team = _team_for_ticket(ticket_by_id, teams, ticket_id)
            open_by_team[team] = open_by_team.get(team, 0) + 1
            assignee = ticket_state["assignee"]
            if assignee is not None:
                agent_open[assignee] = agent_open.get(assignee, 0) + 1

        responded_at = ticket_state["first_response_at"]
        if responded_at is not None:
            if responded_at <= ticket_state["first_response_deadline"]:
                first_response["met"] += 1
            else:
                first_response["breached"] += 1

    return {
        "open_by_team": {team: count for team, count in sorted(open_by_team.items()) if count > 0},
        "first_response": first_response,
        "escalated": sorted(tid for tid, item in state.items() if tid != "_audit" and item["escalated"]),
        "reopened": sorted(tid for tid, item in state.items() if tid != "_audit" and item["reopen_count"] > 0),
        "unassigned": sorted(tid for tid, item in state.items() if tid != "_audit" and item["assignee"] is None),
        "agent_open": {agent: count for agent, count in sorted(agent_open.items()) if count > 0},
    }


def _load_fixtures(directory: Path) -> dict[str, Any]:
    return {
        path.stem: json.loads(path.read_text(encoding="utf-8"))
        for path in directory.glob("*.json")
    }


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(dest="command", required=True)
    report_parser = subparsers.add_parser("report")
    report_parser.add_argument("--fixtures", required=True)
    report_parser.add_argument("--now", required=True)
    args = parser.parse_args(argv)

    if args.command == "report":
        data = _load_fixtures(Path(args.fixtures))
        state = replay(
            data["tickets"],
            data["events"],
            data["agents"],
            data["teams"],
            data["sla"],
            data["holidays"],
        )
        print(json.dumps(build_report(state, data["tickets"], data["teams"])))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
