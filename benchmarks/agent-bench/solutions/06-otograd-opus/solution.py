"""OTOGRAD — reverse-mode automatic differentiation engine (numpy only).

A micro autograd engine (PyTorch-core idea). The neural net, loss and training
are built entirely on top of this engine.
"""

import json
import math

import numpy as np


def _unbroadcast(grad, shape):
    """Reduce `grad` to `shape` by summing over broadcasted axes."""
    grad = np.asarray(grad)
    # Sum over leading extra dimensions.
    while grad.ndim > len(shape):
        grad = grad.sum(axis=0)
    # Sum over axes that were broadcast (size 1 in the original shape).
    for axis, dim in enumerate(shape):
        if dim == 1 and grad.shape[axis] != 1:
            grad = grad.sum(axis=axis, keepdims=True)
    return grad.reshape(shape)


class Tensor:
    def __init__(self, data, requires_grad=False, _children=(), _backward=None):
        self.data = np.asarray(data, dtype=np.float64)
        self.requires_grad = bool(requires_grad)
        self.grad = None
        self._prev = set(_children)
        # _backward propagates this node's grad into its children's grads.
        self._backward = _backward if _backward is not None else (lambda: None)

    def __repr__(self):
        return f"Tensor(shape={self.data.shape}, requires_grad={self.requires_grad})"

    # --- helpers ---------------------------------------------------------
    @staticmethod
    def _ensure(x):
        return x if isinstance(x, Tensor) else Tensor(x)

    def _accum(self, g):
        """Accumulate gradient g (already reduced to self.shape) into self.grad."""
        if not self.requires_grad:
            return
        if self.grad is None:
            self.grad = np.array(g, dtype=np.float64)
        else:
            self.grad = self.grad + g

    # --- operations ------------------------------------------------------
    def __add__(self, other):
        other = self._ensure(other)
        out = Tensor(self.data + other.data,
                     requires_grad=self.requires_grad or other.requires_grad,
                     _children=(self, other))

        def _backward():
            g = out.grad
            self._accum(_unbroadcast(g, self.data.shape))
            other._accum(_unbroadcast(g, other.data.shape))

        out._backward = _backward
        return out

    def __radd__(self, other):
        return self.__add__(other)

    def __sub__(self, other):
        other = self._ensure(other)
        out = Tensor(self.data - other.data,
                     requires_grad=self.requires_grad or other.requires_grad,
                     _children=(self, other))

        def _backward():
            g = out.grad
            self._accum(_unbroadcast(g, self.data.shape))
            other._accum(_unbroadcast(-g, other.data.shape))

        out._backward = _backward
        return out

    def __rsub__(self, other):
        return self._ensure(other).__sub__(self)

    def __mul__(self, other):
        other = self._ensure(other)
        out = Tensor(self.data * other.data,
                     requires_grad=self.requires_grad or other.requires_grad,
                     _children=(self, other))

        def _backward():
            g = out.grad
            self._accum(_unbroadcast(g * other.data, self.data.shape))
            other._accum(_unbroadcast(g * self.data, other.data.shape))

        out._backward = _backward
        return out

    def __rmul__(self, other):
        return self.__mul__(other)

    def __neg__(self):
        return self.__mul__(-1.0)

    def __matmul__(self, other):
        other = self._ensure(other)
        out = Tensor(self.data @ other.data,
                     requires_grad=self.requires_grad or other.requires_grad,
                     _children=(self, other))

        def _backward():
            g = out.grad
            self._accum(g @ other.data.T)
            other._accum(self.data.T @ g)

        out._backward = _backward
        return out

    def relu(self):
        out = Tensor(np.maximum(self.data, 0.0),
                     requires_grad=self.requires_grad,
                     _children=(self,))

        def _backward():
            self._accum((self.data > 0.0) * out.grad)

        out._backward = _backward
        return out

    def exp(self):
        e = np.exp(self.data)
        out = Tensor(e, requires_grad=self.requires_grad, _children=(self,))

        def _backward():
            self._accum(e * out.grad)

        out._backward = _backward
        return out

    def log(self):
        out = Tensor(np.log(self.data),
                     requires_grad=self.requires_grad, _children=(self,))

        def _backward():
            self._accum(out.grad / self.data)

        out._backward = _backward
        return out

    def sum(self):
        out = Tensor(self.data.sum(),
                     requires_grad=self.requires_grad, _children=(self,))

        def _backward():
            self._accum(np.ones_like(self.data) * out.grad)

        out._backward = _backward
        return out

    def mean(self):
        n = self.data.size
        out = Tensor(self.data.mean(),
                     requires_grad=self.requires_grad, _children=(self,))

        def _backward():
            self._accum(np.ones_like(self.data) * (out.grad / n))

        out._backward = _backward
        return out

    # --- backward --------------------------------------------------------
    def backward(self):
        assert self.data.size == 1, "backward yalnız skaler üzerinde çağrılır"

        topo = []
        visited = set()

        def build(v):
            if id(v) in visited:
                return
            visited.add(id(v))
            for child in v._prev:
                build(child)
            topo.append(v)

        build(self)

        # Reset grads on all nodes in this graph.
        for v in topo:
            v.grad = None

        self.grad = np.ones_like(self.data)
        for v in reversed(topo):
            if v.grad is not None:
                v._backward()


# ---------------------------------------------------------------------------
# Loss
# ---------------------------------------------------------------------------
def cross_entropy_logits(logits, y):
    """Numerically-stable softmax cross-entropy over rows, built on Tensor ops.

    logits: Tensor (N, C); y: np.ndarray (N,) of class indices. Returns scalar.
    """
    y = np.asarray(y)
    N = logits.data.shape[0]
    C = logits.data.shape[1]

    # max per row (constant, for numerical stability) — no grad needed.
    m = logits.data.max(axis=1, keepdims=True)
    shifted = logits - Tensor(m)               # (N, C)
    exp = shifted.exp()                         # (N, C)
    denom = exp.sum()                           # not used directly; need per-row
    # per-row logsumexp: sum over columns. Implement via matmul with ones.
    ones_c = Tensor(np.ones((C, 1)))
    sum_exp = exp @ ones_c                      # (N, 1)
    logsumexp = sum_exp.log()                   # (N, 1)
    logp = shifted - logsumexp                  # (N, C) broadcast

    # Select the log-prob of the true class via a one-hot mask.
    onehot = np.zeros((N, C))
    onehot[np.arange(N), y] = 1.0
    picked = (logp * Tensor(onehot)).sum()     # sum of correct-class logprobs
    loss = picked * (-1.0 / N)
    return loss


# ---------------------------------------------------------------------------
# Network + optimizer
# ---------------------------------------------------------------------------
class Net:
    def __init__(self, n_in, h1, h2, n_out, seed=0, _params=None):
        self.dims = (n_in, h1, h2, n_out)
        if _params is not None:
            (W1, b1, W2, b2, W3, b3) = _params
            self.W1 = Tensor(W1, requires_grad=True)
            self.b1 = Tensor(b1, requires_grad=True)
            self.W2 = Tensor(W2, requires_grad=True)
            self.b2 = Tensor(b2, requires_grad=True)
            self.W3 = Tensor(W3, requires_grad=True)
            self.b3 = Tensor(b3, requires_grad=True)
        else:
            rng = np.random.default_rng(seed)
            self.W1 = Tensor(rng.normal(0, math.sqrt(2 / n_in), (n_in, h1)), requires_grad=True)
            self.W2 = Tensor(rng.normal(0, math.sqrt(2 / h1), (h1, h2)), requires_grad=True)
            self.W3 = Tensor(rng.normal(0, math.sqrt(2 / h2), (h2, n_out)), requires_grad=True)
            self.b1 = Tensor(np.zeros(h1), requires_grad=True)
            self.b2 = Tensor(np.zeros(h2), requires_grad=True)
            self.b3 = Tensor(np.zeros(n_out), requires_grad=True)

    def params(self):
        return [self.W1, self.b1, self.W2, self.b2, self.W3, self.b3]

    def forward(self, X):
        X = Tensor(np.asarray(X, dtype=np.float64))
        h1 = (X @ self.W1 + self.b1).relu()
        h2 = (h1 @ self.W2 + self.b2).relu()
        logits = h2 @ self.W3 + self.b3
        return logits

    def predict(self, X):
        return np.argmax(self.forward(X).data, axis=1)

    def fit(self, X, y, epochs, lr, momentum):
        X = np.asarray(X, dtype=np.float64)
        y = np.asarray(y)
        opt = SGD(self.params(), lr=lr, momentum=momentum)
        for _ in range(epochs):
            opt.zero_grad()
            logits = self.forward(X)
            loss = cross_entropy_logits(logits, y)
            loss.backward()
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


def load_net(path):
    with open(path) as f:
        d = json.load(f)
    params = [np.array(d[k], dtype=np.float64)
              for k in ("W1", "b1", "W2", "b2", "W3", "b3")]
    W1, b1, W2, b2, W3, b3 = params
    n_in, h1 = W1.shape
    h2 = W2.shape[1]
    n_out = W3.shape[1]
    return Net(n_in, h1, h2, n_out, _params=params)


class SGD:
    def __init__(self, params, lr, momentum=0.0):
        self.params = list(params)
        self.lr = lr
        self.momentum = momentum
        self.v = [np.zeros_like(p.data) for p in self.params]

    def zero_grad(self):
        for p in self.params:
            p.grad = None

    def step(self):
        for i, p in enumerate(self.params):
            g = p.grad
            if g is None:
                g = np.zeros_like(p.data)
            self.v[i] = self.momentum * self.v[i] + g
            p.data = p.data - self.lr * self.v[i]
