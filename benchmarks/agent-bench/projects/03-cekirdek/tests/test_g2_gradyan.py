import numpy as np


def numerical_grad(mlp, X, y, param: np.ndarray, eps=1e-5) -> np.ndarray:
    num = np.zeros_like(param)
    it = np.nditer(param, flags=["multi_index"])
    while not it.finished:
        ix = it.multi_index
        old = param[ix]
        param[ix] = old + eps
        lp = mlp.loss(X, y)
        param[ix] = old - eps
        lm = mlp.loss(X, y)
        param[ix] = old
        num[ix] = (lp - lm) / (2 * eps)
        it.iternext()
    return num


def rel_err(a, b):
    denom = max(1e-8, float(np.abs(a).max() + np.abs(b).max()))
    return float(np.abs(a - b).max()) / denom


def test_gradients_match_numerical(solution):
    rng = np.random.default_rng(3)
    X = rng.normal(0, 1, (6, 3))
    y = np.array([0, 1, 2, 1, 0, 2])
    mlp = solution.MLP(3, 5, 3, seed=1)
    grads = mlp.grads(X, y)
    for name in ["W1", "b1", "W2", "b2"]:
        analytic = np.asarray(grads[name], dtype=float)
        numeric = numerical_grad(mlp, X, y, getattr(mlp, name))
        assert rel_err(analytic, numeric) < 1e-4, name


def test_step_updates_in_place(solution):
    mlp = solution.MLP(2, 4, 2, seed=0)
    before = mlp.W1.copy()
    g = {"W1": np.ones_like(mlp.W1), "b1": np.zeros_like(mlp.b1),
         "W2": np.zeros_like(mlp.W2), "b2": np.zeros_like(mlp.b2)}
    mlp.step(g, lr=0.1)
    assert np.allclose(mlp.W1, before - 0.1)
