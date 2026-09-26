import json
import subprocess
import sys
from pathlib import Path


def test_plan_cli(solution_dir: Path, fixtures_dir: Path):
    proc = subprocess.run(
        [sys.executable, str(solution_dir / "solution.py"), "plan",
         "--fixtures", str(fixtures_dir), "--now", "2026-10-01T12:00"],
        capture_output=True, text=True, timeout=60,
    )
    assert proc.returncode == 0, proc.stderr
    out = json.loads(proc.stdout)
    assert out == {
        "now": "2026-10-01T12:00",
        "ranked": ["n1", "n3", "n2"],
        "unmet": {"battaniye": 10, "cadir": 4, "su": 30},
        "loaded_kg": {"v1": 650, "v3": 450, "v4": 310},
        "unshipped": {},
    }
