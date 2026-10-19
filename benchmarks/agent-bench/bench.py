#!/usr/bin/env python3
"""Koşucu: her (proje, ajan) hücresi için goal dosyalarını ayrı ayrı pytest ile
çalıştırır, goal başına geçen/toplam sayar, tabloyu basar ve results.json yazar."""

import argparse
import json
import os
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PROJECTS = ["01-kanbagi", "02-sahra", "03-cekirdek", "04-mutabakat", "05-destek", "06-otograd"]
AGENTS = ["opencode", "suffice", "codex", "opus"]
SUMMARY_RE = re.compile(r"(?:(\d+) passed)?(?:.*?(\d+) failed)?(?:.*?(\d+) error)?")


def run_goal(project: str, solution_dir: Path, goal_file: Path) -> tuple[int, int]:
    """Bir goal dosyasını çalıştırır; (geçen, toplam) döner."""
    env = os.environ.copy()
    env["BENCH_SOLUTION"] = str(solution_dir)
    proc = subprocess.run(
        [sys.executable, "-m", "pytest", "-q", "--no-header", "-p", "no:cacheprovider", str(goal_file)],
        capture_output=True,
        text=True,
        env=env,
        cwd=ROOT,
        timeout=300,
    )
    out = proc.stdout + proc.stderr
    passed = failed = errors = 0
    m = re.search(r"(\d+) passed", out)
    if m:
        passed = int(m.group(1))
    m = re.search(r"(\d+) failed", out)
    if m:
        failed = int(m.group(1))
    m = re.search(r"(\d+) error", out)
    if m:
        errors = int(m.group(1))
    total = passed + failed + errors
    if total == 0:
        # toplama/başlatma hatası: dosyadaki test sayısını "hepsi kaldı" say
        collect = subprocess.run(
            [sys.executable, "-m", "pytest", "--collect-only", "-q", "-p", "no:cacheprovider", str(goal_file)],
            capture_output=True,
            text=True,
            cwd=ROOT,
            timeout=120,
        )
        m = re.search(r"(\d+) tests? collected", collect.stdout)
        total = int(m.group(1)) if m else 0
    return passed, total


def run_cell(project: str, agent: str) -> dict:
    solution_dir = ROOT / "solutions" / f"{project}-{agent}"
    tests_dir = ROOT / "projects" / project / "tests"
    goals = sorted(tests_dir.glob("test_g*.py"))
    cell = {"project": project, "agent": agent, "goals": {}, "passed": 0, "total": 0}
    if not (solution_dir / "solution.py").exists():
        for g in goals:
            cell["goals"][g.stem] = {"passed": 0, "total": None, "note": "solution.py yok"}
        return cell
    for g in goals:
        passed, total = run_goal(project, solution_dir, g)
        cell["goals"][g.stem] = {"passed": passed, "total": total}
        cell["passed"] += passed
        cell["total"] += total
    return cell


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--project", choices=PROJECTS)
    ap.add_argument("--solution", choices=AGENTS)
    ap.add_argument("--all", action="store_true")
    args = ap.parse_args()

    cells = []
    if args.all:
        pairs = [(p, a) for p in PROJECTS for a in AGENTS]
    elif args.project and args.solution:
        pairs = [(args.project, args.solution)]
    else:
        ap.error("--all veya (--project ve --solution) verin")
        return

    for project, agent in pairs:
        cell = run_cell(project, agent)
        cells.append(cell)
        goals = "  ".join(
            f"{k.split('_')[1]}:{v['passed']}/{v['total'] if v['total'] is not None else '?'}"
            for k, v in sorted(cell["goals"].items())
        )
        print(f"{project:14} {agent:9} {cell['passed']:3}/{cell['total'] or 0:<3}  {goals}")

    (ROOT / "results.json").write_text(json.dumps(cells, indent=2, ensure_ascii=False))
    print(f"\nresults.json yazıldı ({len(cells)} hücre)")


if __name__ == "__main__":
    main()
