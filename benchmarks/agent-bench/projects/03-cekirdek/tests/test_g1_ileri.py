import math

import numpy as np


def test_relu(solution):
    x = np.array([[-2.0, 0.0, 3.5]])
    assert np.array_equal(solution.relu(x), np.array([[0.0, 0.0, 3.5]]))


def test_softmax_rows_and_stability(solution):
    s = solution.softmax(np.array([[0.0, 0.0], [1000.0, 1000.0]]))
    assert np.allclose(s, [[0.5, 0.5], [0.5, 0.5]])
    s2 = solution.softmax(np.array([[1.0, 2.0, 3.0]]))
    assert abs(float(s2.sum()) - 1.0) < 1e-9


def test_cross_entropy_value(solution):
    probs = np.array([[0.9, 0.1], [0.2, 0.8]])
    y = np.array([0, 1])
    expected = -(math.log(0.9) + math.log(0.8)) / 2
    assert abs(solution.cross_entropy(probs, y) - expected) < 1e-9


def test_forward_with_manual_weights(solution):
    mlp = solution.MLP(2, 2, 2, seed=0)
    mlp.W1[:] = np.eye(2)
    mlp.b1[:] = 0.0
    mlp.W2[:] = np.eye(2)
    mlp.b2[:] = 0.0
    probs = mlp.forward(np.array([[1.0, -1.0]]))
    e = math.e
    assert np.allclose(probs, [[e / (e + 1), 1 / (e + 1)]], atol=1e-9)


def test_init_is_deterministic_and_speced(solution):
    a = solution.MLP(3, 5, 2, seed=7)
    b = solution.MLP(3, 5, 2, seed=7)
    assert np.array_equal(a.W1, b.W1) and np.array_equal(a.W2, b.W2)
    rng = np.random.default_rng(7)
    w1 = rng.normal(0, math.sqrt(2 / 3), (3, 5))
    w2 = rng.normal(0, math.sqrt(2 / 5), (5, 2))
    assert np.allclose(a.W1, w1) and np.allclose(a.W2, w2)
    assert np.array_equal(a.b1, np.zeros(5)) and np.array_equal(a.b2, np.zeros(2))
