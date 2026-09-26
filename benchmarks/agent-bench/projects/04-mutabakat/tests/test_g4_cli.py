import json
import subprocess
import sys
from pathlib import Path


def test_report_cli(solution_dir: Path, fixtures_dir: Path):
    proc = subprocess.run(
        [sys.executable, str(solution_dir / "solution.py"), "report",
         "--fixtures", str(fixtures_dir)],
        capture_output=True, text=True, timeout=60,
    )
    assert proc.returncode == 0, proc.stderr
    out = json.loads(proc.stdout)
    assert out == {
        "matched": 5,
        "unmatched_invoices": ["F-2026-008"],
        "unmatched_payments": ["p6"],
        "toplam_fark_try": 0.01,
        "hatali_satirlar": 2,
    }
