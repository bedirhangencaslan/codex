import json

import numpy as np


def load(fixtures_dir, name):
    d = json.loads((fixtures_dir / name).read_text())
    return np.array(d["X"]), np.array(d["y"])


def trained(solution, fixtures_dir):
    X, y = load(fixtures_dir, "train.json")
    net = solution.Net(2, 32, 16, 3, seed=0)
    net.fit(X, y, epochs=1200, lr=0.3, momentum=0.9)
    return net


def test_generalizes(solution, fixtures_dir):
    net = trained(solution, fixtures_dir)
    Xte, yte = load(fixtures_dir, "test.json")
    acc = float((net.predict(Xte) == yte).mean())
    assert acc >= 0.88, f"test acc {acc}"


def test_parameter_budget(solution):
    net = solution.Net(2, 32, 16, 3, seed=0)
    n = sum(p.data.size for p in net.params())
    assert n < 5000


def test_save_load_roundtrip(solution, fixtures_dir, tmp_path):
    net = trained(solution, fixtures_dir)
    Xte, _ = load(fixtures_dir, "test.json")
    path = tmp_path / "net.json"
    net.save(str(path))
    payload = json.loads(path.read_text())
    assert set(payload) == {"W1", "b1", "W2", "b2", "W3", "b3"}
    restored = solution.load_net(str(path))
    assert np.array_equal(net.predict(Xte), restored.predict(Xte))
