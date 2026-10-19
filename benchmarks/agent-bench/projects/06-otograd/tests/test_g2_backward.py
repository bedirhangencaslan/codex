import math

import numpy as np


def numeric(f, arrs, i, eps=1e-6):
    """arrs[i] üzerinde merkezi farkla df/darr hesaplar; f: arrs -> float."""
    a = arrs[i]
    g = np.zeros_like(a)
    it = np.nditer(a, flags=["multi_index"])
    while not it.finished:
        ix = it.multi_index
        old = a[ix]
        a[ix] = old + eps
        fp = f(arrs)
        a[ix] = old - eps
        fm = f(arrs)
        a[ix] = old
        g[ix] = (fp - fm) / (2 * eps)
        it.iternext()
    return g


def check(solution, build, shapes, seed):
    """build(tensors) -> skaler Tensor. Analitik ve sayısal gradyanları karşılaştırır."""
    rng = np.random.default_rng(seed)
    arrs = [rng.normal(0.3, 0.8, s) for s in shapes]
    tensors = [solution.Tensor(a.copy(), requires_grad=True) for a in arrs]
    out = build(tensors)
    out.backward()
    for i in range(len(arrs)):
        def f(vals, i=i):
            ts = [solution.Tensor(v.copy(), requires_grad=False) for v in vals]
            return float(build(ts).data)
        num = numeric(f, [a.copy() for a in arrs], i)
        ana = tensors[i].grad
        denom = max(1e-8, float(np.abs(num).max() + np.abs(ana).max()))
        assert float(np.abs(num - ana).max()) / denom < 1e-4, f"tensor {i}"


def test_matmul_relu_sum_chain(solution):
    check(solution, lambda t: ((t[0] @ t[1]).relu() @ t[2]).sum(), [(4, 3), (3, 5), (5, 2)], seed=1)


def test_broadcast_grad_reduces(solution):
    # (3,1) + (1,4) → (3,4); yayınlanan eksenler üzerinden gradyan toplanmalı
    check(solution, lambda t: ((t[0] + t[1]) * t[2]).mean(), [(3, 1), (1, 4), (3, 4)], seed=2)


def test_bias_row_broadcast(solution):
    # (5,3) @ (3,2) + (2,) — tipik katman + bias
    check(solution, lambda t: ((t[0] @ t[1]) + t[2]).relu().sum(), [(5, 3), (3, 2), (2,)], seed=3)


def test_diamond_reuse_accumulates(solution):
    # z = x*y + x → dz/dx = y + 1
    x = solution.Tensor(np.array([2.0, -1.0]), requires_grad=True)
    y = solution.Tensor(np.array([3.0, 5.0]), requires_grad=True)
    z = (x * y + x).sum()
    z.backward()
    assert np.allclose(x.grad, [4.0, 6.0])
    assert np.allclose(y.grad, [2.0, -1.0])


def test_exp_log_composition(solution):
    check(solution, lambda t: ((t[0].exp() + 1.0).log() * t[1]).sum(), [(3, 3), (3, 3)], seed=4)


def test_cross_entropy_logits_grad_and_value(solution):
    rng = np.random.default_rng(5)
    logits_arr = rng.normal(0, 1, (6, 3))
    y = np.array([0, 2, 1, 1, 0, 2])
    t = solution.Tensor(logits_arr.copy(), requires_grad=True)
    loss = solution.cross_entropy_logits(t, y)
    # değer: elle log-softmax
    z = logits_arr - logits_arr.max(axis=1, keepdims=True)
    logp = z - np.log(np.exp(z).sum(axis=1, keepdims=True))
    expected = float(-logp[np.arange(6), y].mean())
    assert abs(float(loss.data) - expected) < 1e-9
    loss.backward()
    probs = np.exp(logp)
    grad = probs.copy()
    grad[np.arange(6), y] -= 1
    grad /= 6
    assert np.allclose(t.grad, grad, atol=1e-6)


def test_net_init_deterministic_and_speced(solution):
    a = solution.Net(2, 4, 3, 3, seed=9)
    b = solution.Net(2, 4, 3, 3, seed=9)
    assert np.array_equal(a.params()[0].data, b.params()[0].data)
    rng = np.random.default_rng(9)
    w1 = rng.normal(0, math.sqrt(2 / 2), (2, 4))
    w2 = rng.normal(0, math.sqrt(2 / 4), (4, 3))
    w3 = rng.normal(0, math.sqrt(2 / 3), (3, 3))
    p = a.params()
    assert np.allclose(p[0].data, w1)
    assert np.allclose(p[2].data, w2)
    assert np.allclose(p[4].data, w3)
