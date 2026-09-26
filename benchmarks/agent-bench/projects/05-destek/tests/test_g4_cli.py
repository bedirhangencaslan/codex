import json
import subprocess
import sys
from pathlib import Path


def test_report_cli(solution_dir: Path, fixtures_dir: Path):
    proc = subprocess.run(
        [sys.executable, str(solution_dir / "solution.py"), "report",
         "--fixtures", str(fixtures_dir), "--now", "2026-10-30T12:00"],
        capture_output=True, text=True, timeout=60,
    )
    assert proc.returncode == 0, proc.stderr
    out = json.loads(proc.stdout)
    assert out == {
        "open_by_team": {"destek": 2, "muhendislik": 3},
        "first_response": {"met": 1, "breached": 3},
        "escalated": ["t1", "t4"],
        "reopened": ["t1"],
        "unassigned": ["t6"],
        "agent_open": {"a1": 2, "a2": 1, "a5": 1},
    }
