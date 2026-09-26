import numpy as np


def T(solution, data, rg=True):
    return solution.Tensor(np.array(data, dtype=float), requires_grad=rg)


def test_add_mul_broadcast_values(solution):
    a = T(solution, [[1.0, 2.0], [3.0, 4.0]])
    b = T(solution, [10.0, 20.0])
    assert np.allclose((a + b).data, [[11, 22], [13, 24]])
    assert np.allclose((a * b).data, [[10, 40], [30, 80]])
    assert np.allclose((a - b).data, [[-9, -18], [-7, -16]])
    assert np.allclose((a * 2.0).data, [[2, 4], [6, 8]])


def test_matmul_and_relu(solution):
    a = T(solution, [[1.0, -1.0]])
    w = T(solution, [[2.0, 0.0], [0.0, 3.0]])
    out = (a @ w).relu()
    assert np.allclose(out.data, [[2.0, 0.0]])


def test_exp_log_sum_mean(solution):
    x = T(solution, [1.0, np.e])
    assert np.allclose(x.log().data, [0.0, 1.0])
    assert np.allclose(T(solution, [0.0, 1.0]).exp().data, [1.0, np.e])
    m = T(solution, [[1.0, 2.0], [3.0, 4.0]])
    assert float(m.sum().data) == 10.0
    assert float(m.mean().data) == 2.5


def test_requires_grad_propagates(solution):
    a = T(solution, [1.0, 2.0], rg=True)
    b = T(solution, [3.0, 4.0], rg=False)
    out = (a * b).sum()
    out.backward()
    assert a.grad is not None and np.allclose(a.grad, [3.0, 4.0])
    assert b.grad is None
