"""OTOGRAD — minimal reverse-mode autograd engine (numpy only).

Everything (network, loss, training) is built on top of the Tensor graph;
no hand-coded layer gradients.
"""

import json
import math

import numpy as np


def _unbroadcast(grad, shape):
    """Reduce `grad` back to `shape` by summing over broadcast axes."""
    if grad.shape == tuple(shape):
        return grad
    # Sum over prepended axes (result gained extra leading dims).
    while grad.ndim > len(shape):
        grad = grad.sum(axis=0)
    # Sum over axes that were broadcast from size 1.
    for i, s in enumerate(shape):
        if s == 1 and grad.shape[i] != 1:
            grad = grad.sum(axis=i, keepdims=True)
    return grad.reshape(shape)


class Tensor:
    def __init__(self, data, requires_grad=False):
        self.data = np.asarray(data, dtype=np.float64)
        self.requires_grad = bool(requires_grad)
        self.grad = None
        self._backward = lambda: None
        self._prev = ()

    # ------------------------------------------------------------------
    # graph helpers
    # ------------------------------------------------------------------
    def _accumulate(self, grad):
        if self.grad is None:
            self.grad = grad.copy()
        else:
            self.grad = self.grad + grad

    def _register(self, parents, backward_fn):
        self._prev = tuple(parents)
        self._backward = backward_fn
        self.requires_grad = any(p.requires_grad for p in self._prev)
        return self

    def backward(self):
        assert self.data.size == 1, "backward yalnızca skaler Tensor üzerinde çağrılabilir"
        # Iterative topological sort (DFS postorder).
        topo = []
        visited = set()
        stack = [(self, False)]
        while stack:
            node, processed = stack.pop()
            if processed:
                topo.append(node)
                continue
            if id(node) in visited:
                continue
            visited.add(id(node))
            stack.append((node, True))
            for p in node._prev:
                if id(p) not in visited:
                    stack.append((p, False))
        self._accumulate(np.ones_like(self.data))
        for node in reversed(topo):
            if node.requires_grad:
                node._backward()

    # ------------------------------------------------------------------
    # operations
    # ------------------------------------------------------------------
    @staticmethod
    def _coerce(other):
        return other if isinstance(other, Tensor) else Tensor(other)

    def __add__(self, other):
        other = Tensor._coerce(other)
        out = Tensor(self.data + other.data)

        def _backward():
            if self.requires_grad:
                self._accumulate(_unbroadcast(out.grad, self.data.shape))
            if other.requires_grad:
                other._accumulate(_unbroadcast(out.grad, other.data.shape))

        return out._register((self, other), _backward)

    def __radd__(self, other):
        return self.__add__(other)

    def __sub__(self, other):
        other = Tensor._coerce(other)
        out = Tensor(self.data - other.data)

        def _backward():
            if self.requires_grad:
                self._accumulate(_unbroadcast(out.grad, self.data.shape))
            if other.requires_grad:
                other._accumulate(_unbroadcast(-out.grad, other.data.shape))

        return out._register((self, other), _backward)

    def __rsub__(self, other):
        return Tensor._coerce(other).__sub__(self)

    def __mul__(self, other):
        other = Tensor._coerce(other)
        out = Tensor(self.data * other.data)

        def _backward():
            if self.requires_grad:
                self._accumulate(_unbroadcast(out.grad * other.data, self.data.shape))
            if other.requires_grad:
                other._accumulate(_unbroadcast(out.grad * self.data, other.data.shape))

        return out._register((self, other), _backward)

    def __rmul__(self, other):
        return self.__mul__(other)

    def __matmul__(self, other):
        other = Tensor._coerce(other)
        out = Tensor(self.data @ other.data)

        def _backward():
            g = out.grad
            if self.requires_grad:
                self._accumulate(_unbroadcast(g @ np.swapaxes(other.data, -1, -2), self.data.shape))
            if other.requires_grad:
                other._accumulate(_unbroadcast(np.swapaxes(self.data, -1, -2) @ g, other.data.shape))

        return out._register((self, other), _backward)

    def relu(self):
        out = Tensor(np.maximum(self.data, 0.0))

        def _backward():
            if self.requires_grad:
                self._accumulate(out.grad * (self.data > 0))

        return out._register((self,), _backward)

    def exp(self):
        out = Tensor(np.exp(self.data))

        def _backward():
            if self.requires_grad:
                self._accumulate(out.grad * out.data)

        return out._register((self,), _backward)

    def log(self):
        out = Tensor(np.log(self.data))

        def _backward():
            if self.requires_grad:
                self._accumulate(out.grad / self.data)

        return out._register((self,), _backward)

    def sum(self):
        out = Tensor(self.data.sum())

        def _backward():
            if self.requires_grad:
                self._accumulate(np.ones_like(self.data) * out.grad)

        return out._register((self,), _backward)

    def mean(self):
        out = Tensor(self.data.mean())

        def _backward():
            if self.requires_grad:
                self._accumulate(np.ones_like(self.data) * (out.grad / self.data.size))

        return out._register((self,), _backward)

    def __repr__(self):
        return f"Tensor({self.data!r}, requires_grad={self.requires_grad})"


def cross_entropy_logits(logits: Tensor, y: np.ndarray) -> Tensor:
    """Numerically stable mean cross-entropy over classes (fused primitive)."""
    x = logits.data
    n = x.shape[0]
    z = x - x.max(axis=1, keepdims=True)
    logp = z - np.log(np.exp(z).sum(axis=1, keepdims=True))
    y = np.asarray(y).astype(np.int64)
    loss_val = -logp[np.arange(n), y].mean()
    out = Tensor(loss_val)

    def _backward():
        if logits.requires_grad:
            probs = np.exp(logp)
            grad = probs.copy()
            grad[np.arange(n), y] -= 1.0
            grad /= n
            logits._accumulate(grad)

    return out._register((logits,), _backward)


class SGD:
    def __init__(self, params, lr, momentum):
        self.params = list(params)
        self.lr = float(lr)
        self.momentum = float(momentum)
        self.velocities = [np.zeros_like(p.data) for p in self.params]

    def zero_grad(self):
        for p in self.params:
            p.grad = None

    def step(self):
        for p, v in zip(self.params, self.velocities):
            v *= self.momentum
            v += p.grad
            p.data -= self.lr * v


class Net:
    def __init__(self, n_in, h1, h2, n_out, seed=None):
        self.n_in, self.h1, self.h2, self.n_out = n_in, h1, h2, n_out
        rng = np.random.default_rng(seed)
        self.W1 = Tensor(rng.normal(0, math.sqrt(2 / n_in), (n_in, h1)), requires_grad=True)
        self.W2 = Tensor(rng.normal(0, math.sqrt(2 / h1), (h1, h2)), requires_grad=True)
        self.W3 = Tensor(rng.normal(0, math.sqrt(2 / h2), (h2, n_out)), requires_grad=True)
        self.b1 = Tensor(np.zeros(h1), requires_grad=True)
        self.b2 = Tensor(np.zeros(h2), requires_grad=True)
        self.b3 = Tensor(np.zeros(n_out), requires_grad=True)

    def params(self):
        return [self.W1, self.b1, self.W2, self.b2, self.W3, self.b3]

    def forward(self, X: np.ndarray) -> Tensor:
        X = Tensor(X)
        h = (X @ self.W1 + self.b1).relu()
        h = (h @ self.W2 + self.b2).relu()
        return h @ self.W3 + self.b3

    def predict(self, X) -> np.ndarray:
        return np.argmax(self.forward(X).data, axis=1)

    def fit(self, X, y, epochs, lr, momentum):
        y = np.asarray(y).astype(np.int64)
        opt = SGD(self.params(), lr, momentum)
        for _ in range(int(epochs)):
            opt.zero_grad()
            logits = self.forward(X)
            loss = cross_entropy_logits(logits, y)
            loss.backward()
            # Global-norm gradient clipping: keeps momentum SGD stable on
            # full-batch spiral without touching the engine's gradients.
            total_sq = sum(float((p.grad ** 2).sum()) for p in self.params() if p.grad is not None)
            total_norm = total_sq ** 0.5
            if total_norm > 1.0:
                scale = 1.0 / total_norm
                for p in self.params():
                    if p.grad is not None:
                        p.grad = p.grad * scale
            opt.step()

    def save(self, path):
        payload = {
            "W1": self.W1.data.tolist(),
            "b1": self.b1.data.tolist(),
            "W2": self.W2.data.tolist(),
            "b2": self.b2.data.tolist(),
            "W3": self.W3.data.tolist(),
            "b3": self.b3.data.tolist(),
        }
        with open(path, "w") as f:
            json.dump(payload, f)


def load_net(path) -> Net:
    with open(path) as f:
        payload = json.load(f)
    W1 = np.array(payload["W1"], dtype=np.float64)
    W2 = np.array(payload["W2"], dtype=np.float64)
    W3 = np.array(payload["W3"], dtype=np.float64)
    net = Net(W1.shape[0], W1.shape[1], W2.shape[1], W3.shape[1])
    net.W1.data = W1
    net.b1.data = np.array(payload["b1"], dtype=np.float64)
    net.W2.data = W2
    net.b2.data = np.array(payload["b2"], dtype=np.float64)
    net.W3.data = W3
    net.b3.data = np.array(payload["b3"], dtype=np.float64)
    return net
