import json
import subprocess
import sys
from pathlib import Path


def test_report_cli(solution_dir: Path, fixtures_dir: Path):
    proc = subprocess.run(
        [sys.executable, str(solution_dir / "solution.py"), "report",
         "--fixtures", str(fixtures_dir), "--date", "2026-10-01"],
        capture_output=True, text=True, timeout=60,
    )
    assert proc.returncode == 0, proc.stderr
    out = json.loads(proc.stdout)
    assert out == {
        "date": "2026-10-01",
        "stock": {"O-": 2, "O+": 1, "A-": 0, "A+": 2, "B-": 0, "B+": 1, "AB-": 0, "AB+": 0},
        "critical": sorted(["A-", "B-", "AB-", "AB+", "O+", "B+"]),
        "expiring_7d": ["u4"],
        "open_requests": 3,
    }
