import importlib
import json
import os
import sys
from pathlib import Path

import pytest


@pytest.fixture(scope="session")
def solution_dir() -> Path:
    sol = os.environ.get("BENCH_SOLUTION")
    if not sol:
        pytest.fail("BENCH_SOLUTION ortam değişkeni ayarlı değil (bench.py kullanın)")
    p = Path(sol).resolve()
    if not (p / "solution.py").exists():
        pytest.fail(f"solution.py bulunamadı: {p}")
    return p


@pytest.fixture(scope="session")
def solution(solution_dir: Path):
    sys.path.insert(0, str(solution_dir))
    try:
        return importlib.import_module("solution")
    finally:
        sys.path.pop(0)


@pytest.fixture(scope="session")
def fixtures_dir() -> Path:
    return Path(__file__).resolve().parent.parent / "fixtures"


@pytest.fixture(scope="session")
def data(fixtures_dir: Path) -> dict:
    out = {}
    for f in fixtures_dir.glob("*.json"):
        out[f.stem] = json.loads(f.read_text())
    return out
