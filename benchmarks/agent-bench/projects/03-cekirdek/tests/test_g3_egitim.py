import json
import time

import numpy as np


def load(fixtures_dir, name):
    d = json.loads((fixtures_dir / name).read_text())
    return np.array(d["X"]), np.array(d["y"])


def test_training_reaches_097_within_60s(solution, fixtures_dir):
    X, y = load(fixtures_dir, "train.json")
    mlp = solution.MLP(2, 16, 2, seed=0)
    t0 = time.monotonic()
    mlp.fit(X, y, epochs=400, lr=1.0)
    assert time.monotonic() - t0 < 60
    acc = float((mlp.predict(X) == y).mean())
    assert acc >= 0.97, f"train acc {acc}"


def test_loss_decreases(solution, fixtures_dir):
    X, y = load(fixtures_dir, "train.json")
    mlp = solution.MLP(2, 16, 2, seed=0)
    before = mlp.loss(X, y)
    mlp.fit(X, y, epochs=50, lr=1.0)
    after = mlp.loss(X, y)
    assert after < before * 0.5
