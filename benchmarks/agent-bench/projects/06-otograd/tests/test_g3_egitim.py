import json
import time

import numpy as np


def load(fixtures_dir, name):
    d = json.loads((fixtures_dir / name).read_text())
    return np.array(d["X"]), np.array(d["y"])


def test_training_reaches_095_within_120s(solution, fixtures_dir):
    X, y = load(fixtures_dir, "train.json")
    net = solution.Net(2, 32, 16, 3, seed=0)
    t0 = time.monotonic()
    net.fit(X, y, epochs=1200, lr=0.3, momentum=0.9)
    assert time.monotonic() - t0 < 120
    acc = float((net.predict(X) == y).mean())
    assert acc >= 0.95, f"train acc {acc}"


def test_loss_decreases_with_momentum(solution, fixtures_dir):
    X, y = load(fixtures_dir, "train.json")
    net = solution.Net(2, 32, 16, 3, seed=0)
    before = float(solution.cross_entropy_logits(net.forward(X), y).data)
    net.fit(X, y, epochs=200, lr=0.3, momentum=0.9)
    after = float(solution.cross_entropy_logits(net.forward(X), y).data)
    assert after < before * 0.5


def test_same_seed_same_result(solution, fixtures_dir):
    X, y = load(fixtures_dir, "train.json")
    a = solution.Net(2, 32, 16, 3, seed=0)
    b = solution.Net(2, 32, 16, 3, seed=0)
    a.fit(X, y, epochs=40, lr=0.3, momentum=0.9)
    b.fit(X, y, epochs=40, lr=0.3, momentum=0.9)
    assert np.array_equal(a.params()[0].data, b.params()[0].data)
    assert np.array_equal(a.predict(X), b.predict(X))
