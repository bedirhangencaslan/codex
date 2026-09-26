"""OTOGRAD — reverse-mode autograd engine built on numpy only.

A micro autograd core (PyTorch-style dynamic graph): every operation returns a
new Tensor that records its producing inputs and a backward closure. Gradients
flow in reverse topological order, accumulate on reuse (diamonds), and are
reduced over broadcast axes. The network, loss and training loop are built
entirely on top of this engine.
"""

import json
import math

import numpy as np

__all__ = ["Tensor", "SGD", "Net", "load_net", "cross_entropy_logits"]


def _unbroadcast(g, shape):
    """Reduce an upstream gradient to the original ``shape``.

    Axes that were broadcast get their gradient contributions summed, both for
    prepended dimensions and for size-1 dimensions that were expanded.
    """
    shape = tuple(shape)
    if g.shape == shape:
        return g
    if g.ndim > len(shape):
        g = g.sum(axis=tuple(range(g.ndim - len(shape))))
    axes = tuple(i for i, s in enumerate(shape) if s == 1 and g.shape[i] != 1)
    if axes:
        g = g.sum(axis=axes, keepdims=True)
    return np.reshape(g, shape)


def _accumulate(t, g):
    """Add gradient ``g`` into ``t.grad`` (creating it on first write)."""
    g = _unbroadcast(np.asarray(g, dtype=np.float64), t.data.shape)
    if t.grad is None:
        t.grad = g
    else:
        t.grad = t.grad + g


def _matmul_grad(a, b, g):
    """Gradients of ``np.matmul(a, b)`` w.r.t. ``a`` and ``b``."""
    if a.ndim == 1 and b.ndim == 1:
        return g * b, g * a
    a2 = a if a.ndim > 1 else a.reshape(1, -1)
    b2 = b if b.ndim > 1 else b.reshape(-1, 1)
    g2 = g
    if a.ndim == 1:
        g2 = np.expand_dims(g2, -2)
    if b.ndim == 1:
        g2 = np.expand_dims(g2, -1)
    ga = np.matmul(g2, np.swapaxes(b2, -1, -2))
    gb = np.matmul(np.swapaxes(a2, -1, -2), g2)
    ga = _unbroadcast(ga, a2.shape)
    gb = _unbroadcast(gb, b2.shape)
    if a.ndim == 1:
        ga = np.reshape(ga, a.shape)
    if b.ndim == 1:
        gb = np.reshape(gb, b.shape)
    return ga, gb


def _ensure(x):
    return x if isinstance(x, Tensor) else Tensor(x)


def _finish(out, backward, *inputs):
    out._prev = tuple(t for t in inputs if t.requires_grad)
    out.requires_grad = bool(out._prev)
    out._backward = backward
    return out


class Tensor:
    """Numpy-backed value node in a dynamically built autograd graph."""

    def __init__(self, data, requires_grad=False):
        if isinstance(data, Tensor):
            data = data.data
        self.data = np.array(data, dtype=np.float64)
        self.requires_grad = bool(requires_grad)
        self.grad = None
        self._prev = ()
        self._backward = None

    @staticmethod
    def _wrap(arr):
        t = Tensor.__new__(Tensor)
        t.data = np.asarray(arr, dtype=np.float64)
        t.requires_grad = False
        t.grad = None
        t._prev = ()
        t._backward = None
        return t

    # ------------------------------------------------------------------ ops
    def __add__(self, other):
        a, b = self, _ensure(other)
        out = Tensor._wrap(a.data + b.data)

        def backward():
            if a.requires_grad:
                _accumulate(a, out.grad)
            if b.requires_grad:
                _accumulate(b, out.grad)

        return _finish(out, backward, a, b)

    def __sub__(self, other):
        a, b = self, _ensure(other)
        out = Tensor._wrap(a.data - b.data)

        def backward():
            if a.requires_grad:
                _accumulate(a, out.grad)
            if b.requires_grad:
                _accumulate(b, -out.grad)

        return _finish(out, backward, a, b)

    def __mul__(self, other):
        a, b = self, _ensure(other)
        out = Tensor._wrap(a.data * b.data)

        def backward():
            if a.requires_grad:
                _accumulate(a, out.grad * b.data)
            if b.requires_grad:
                _accumulate(b, out.grad * a.data)

        return _finish(out, backward, a, b)

    def __truediv__(self, other):
        a, b = self, _ensure(other)
        out = Tensor._wrap(a.data / b.data)

        def backward():
            if a.requires_grad:
                _accumulate(a, out.grad / b.data)
            if b.requires_grad:
                _accumulate(b, -out.grad * a.data / (b.data * b.data))

        return _finish(out, backward, a, b)

    def __matmul__(self, other):
        a, b = self, _ensure(other)
        out = Tensor._wrap(np.matmul(a.data, b.data))

        def backward():
            ga, gb = _matmul_grad(a.data, b.data, out.grad)
            if a.requires_grad:
                _accumulate(a, ga)
            if b.requires_grad:
                _accumulate(b, gb)

        return _finish(out, backward, a, b)

    def __neg__(self):
        return self * -1.0

    __radd__ = __add__
    __rmul__ = __mul__

    def __rsub__(self, other):
        return _ensure(other) - self

    def __rtruediv__(self, other):
        return _ensure(other) / self

    def __rmatmul__(self, other):
        return _ensure(other) @ self

    def relu(self):
        a = self
        out = Tensor._wrap(np.maximum(a.data, 0.0))

        def backward():
            if a.requires_grad:
                _accumulate(a, out.grad * (a.data > 0.0))

        return _finish(out, backward, a)

    def exp(self):
        a = self
        out = Tensor._wrap(np.exp(a.data))

        def backward():
            if a.requires_grad:
                _accumulate(a, out.grad * out.data)

        return _finish(out, backward, a)

    def log(self):
        a = self
        out = Tensor._wrap(np.log(a.data))

        def backward():
            if a.requires_grad:
                _accumulate(a, out.grad / a.data)

        return _finish(out, backward, a)

    def sum(self):
        a = self
        out = Tensor._wrap(a.data.sum())

        def backward():
            if a.requires_grad:
                _accumulate(a, np.broadcast_to(out.grad, a.data.shape))

        return _finish(out, backward, a)

    def mean(self):
        a = self
        out = Tensor._wrap(a.data.mean())

        def backward():
            if a.requires_grad:
                _accumulate(a, np.broadcast_to(out.grad, a.data.shape) / a.data.size)

        return _finish(out, backward, a)

    # ------------------------------------------------------------- backward
    def backward(self):
        if self.data.size != 1:
            raise RuntimeError("backward() yalniz skaler Tensor uzerinde cagrilabilir")
        topo = []
        visited = set()
        stack = [(self, False)]
        while stack:
            node, done = stack.pop()
            if done:
                topo.append(node)
                continue
            if node in visited:
                continue
            visited.add(node)
            stack.append((node, True))
            for p in node._prev:
                if p not in visited:
                    stack.append((p, False))
        self.grad = np.ones_like(self.data)
        for node in reversed(topo):
            if node._backward is not None:
                node._backward()

    def zero_grad(self):
        self.grad = None

    def __repr__(self):
        return f"Tensor(data={self.data!r}, requires_grad={self.requires_grad})"


def cross_entropy_logits(logits: Tensor, y) -> Tensor:
    """Mean cross-entropy of ``logits`` (N,C Tensor) for integer labels ``y``.

    Numerically stable via the log-sum-exp shift; wired into the graph with
    the standard softmax-gradient backward (probs - onehot) / N.
    """
    if not isinstance(logits, Tensor):
        logits = Tensor(logits)
    y_idx = np.asarray(y).astype(np.int64).reshape(-1)
    x = logits.data
    n = x.shape[0]
    idx = np.arange(n)
    z = x - x.max(axis=1, keepdims=True)
    s = np.exp(z).sum(axis=1, keepdims=True)
    logp = z - np.log(s)
    out = Tensor._wrap(-logp[idx, y_idx].mean())
    out.requires_grad = logits.requires_grad
    out._prev = (logits,) if logits.requires_grad else ()

    def backward():
        if logits.requires_grad:
            probs = np.exp(z) / s
            g = probs
            g[idx, y_idx] -= 1.0
            g /= n
            _accumulate(logits, out.grad * g)

    out._backward = backward
    return out


class SGD:
    """Momentum SGD: v = momentum*v + grad; p.data -= lr*v (v starts at 0)."""

    def __init__(self, params, lr, momentum):
        self.params = list(params)
        self.lr = float(lr)
        self.momentum = float(momentum)
        self._v = [np.zeros_like(p.data) for p in self.params]

    def zero_grad(self):
        for p in self.params:
            p.grad = None

    def step(self):
        for p, v in zip(self.params, self._v):
            if p.grad is None:
                continue
            v *= self.momentum
            v += p.grad
            p.data -= self.lr * v


class Net:
    """MLP 2 hidden layers; forward/backward entirely via Tensor ops."""

    def __init__(self, n_in, h1, h2, n_out, seed=0):
        rng = np.random.default_rng(seed)
        self.W1 = Tensor(rng.normal(0, math.sqrt(2 / n_in), (n_in, h1)), requires_grad=True)
        self.b1 = Tensor(np.zeros(h1), requires_grad=True)
        self.W2 = Tensor(rng.normal(0, math.sqrt(2 / h1), (h1, h2)), requires_grad=True)
        self.b2 = Tensor(np.zeros(h2), requires_grad=True)
        self.W3 = Tensor(rng.normal(0, math.sqrt(2 / h2), (h2, n_out)), requires_grad=True)
        self.b3 = Tensor(np.zeros(n_out), requires_grad=True)

    def params(self):
        return [self.W1, self.b1, self.W2, self.b2, self.W3, self.b3]

    def forward(self, X) -> Tensor:
        x = Tensor(np.asarray(X, dtype=np.float64))
        h = (x @ self.W1 + self.b1).relu()
        h = (h @ self.W2 + self.b2).relu()
        return h @ self.W3 + self.b3

    def predict(self, X) -> np.ndarray:
        return np.argmax(self.forward(X).data, axis=1)

    def fit(self, X, y, epochs, lr, momentum):
        X = np.asarray(X, dtype=np.float64)
        opt = SGD(self.params(), lr=lr, momentum=momentum)
        for _ in range(int(epochs)):
            opt.zero_grad()
            loss = cross_entropy_logits(self.forward(X), y)
            loss.backward()
            opt.step()

    def save(self, path):
        payload = {k: getattr(self, k).data.tolist() for k in ("W1", "b1", "W2", "b2", "W3", "b3")}
        with open(path, "w") as f:
            json.dump(payload, f)


def load_net(path) -> Net:
    with open(path) as f:
        d = json.load(f)
    W1 = np.asarray(d["W1"], dtype=np.float64)
    W2 = np.asarray(d["W2"], dtype=np.float64)
    W3 = np.asarray(d["W3"], dtype=np.float64)
    net = Net(W1.shape[0], W1.shape[1], W2.shape[1], W3.shape[1], seed=0)
    for name in ("W1", "b1", "W2", "b2", "W3", "b3"):
        getattr(net, name).data = np.asarray(d[name], dtype=np.float64)
    return net
