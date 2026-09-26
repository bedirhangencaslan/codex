import json

import numpy as np


def load(fixtures_dir, name):
    d = json.loads((fixtures_dir / name).read_text())
    return np.array(d["X"]), np.array(d["y"])


def trained(solution, fixtures_dir):
    X, y = load(fixtures_dir, "train.json")
    mlp = solution.MLP(2, 16, 2, seed=0)
    mlp.fit(X, y, epochs=400, lr=1.0)
    return mlp


def test_generalizes_to_test_set(solution, fixtures_dir):
    mlp = trained(solution, fixtures_dir)
    Xte, yte = load(fixtures_dir, "test.json")
    acc = float((mlp.predict(Xte) == yte).mean())
    assert acc >= 0.90, f"test acc {acc}"


def test_parameter_budget(solution):
    mlp = solution.MLP(2, 16, 2, seed=0)
    n = mlp.W1.size + mlp.b1.size + mlp.W2.size + mlp.b2.size
    assert n < 2000


def test_save_load_roundtrip(solution, fixtures_dir, tmp_path):
    mlp = trained(solution, fixtures_dir)
    Xte, _ = load(fixtures_dir, "test.json")
    path = tmp_path / "model.json"
    mlp.save(str(path))
    data = json.loads(path.read_text())
    assert set(data) == {"W1", "b1", "W2", "b2"}
    restored = solution.load_mlp(str(path))
    assert np.array_equal(mlp.predict(Xte), restored.predict(Xte))
