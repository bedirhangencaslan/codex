"""A small reverse-mode autograd engine built on NumPy."""

from __future__ import annotations

import json
import math

import numpy as np


class Tensor:
    """A value in a dynamically-built expression graph."""

    def __init__(self, data, requires_grad: bool = False):
        self.data = np.asarray(data, dtype=np.float64)
        self.grad: np.ndarray | None = None
        self.requires_grad = bool(requires_grad)
        self._parents: tuple[Tensor, ...] = ()
        self._backward = None

    def _set_graph(self, parents, backward):
        self._parents = tuple(parents)
        self._backward = backward

    def _accumulate(self, value):
        if self.grad is None:
            self.grad = np.array(value, dtype=np.float64, copy=True)
        else:
            self.grad += value

    @staticmethod
    def _as_tensor(value):
        return value if isinstance(value, Tensor) else Tensor(value)

    @staticmethod
    def _reduce_gradient(gradient: np.ndarray, shape: tuple[int, ...]) -> np.ndarray:
        gradient = np.asarray(gradient, dtype=np.float64)
        while gradient.ndim > len(shape):
            gradient = gradient.sum(axis=0)
        for axis, size in enumerate(shape):
            if size == 1 and gradient.shape[axis] != 1:
                gradient = gradient.sum(axis=axis, keepdims=True)
        return gradient.reshape(shape)

    def __add__(self, other):
        other = self._as_tensor(other)
        out_shape = np.broadcast_shapes(self.data.shape, other.data.shape)
        out = Tensor(np.empty(out_shape, dtype=np.float64),
                     self.requires_grad or other.requires_grad)
        out.data[...] = self.data + other.data

        def backward():
            if self.requires_grad:
                self._accumulate(self._reduce_gradient(out.grad, self.data.shape))
            if other.requires_grad:
                other._accumulate(self._reduce_gradient(out.grad, other.data.shape))

        out._set_graph((self, other), backward)
        return out

    def __radd__(self, other):
        return self._as_tensor(other).__add__(self)

    def __sub__(self, other):
        other = self._as_tensor(other)
        out_shape = np.broadcast_shapes(self.data.shape, other.data.shape)
        out = Tensor(np.empty(out_shape, dtype=np.float64),
                     self.requires_grad or other.requires_grad)
        out.data[...] = self.data - other.data

        def backward():
            if self.requires_grad:
                self._accumulate(self._reduce_gradient(out.grad, self.data.shape))
            if other.requires_grad:
                other._accumulate(-self._reduce_gradient(out.grad, other.data.shape))

        out._set_graph((self, other), backward)
        return out

    def __rsub__(self, other):
        return self._as_tensor(other).__sub__(self)

    def __mul__(self, other):
        other = self._as_tensor(other)
        out_shape = np.broadcast_shapes(self.data.shape, other.data.shape)
        out = Tensor(np.empty(out_shape, dtype=np.float64),
                     self.requires_grad or other.requires_grad)
        out.data[...] = self.data * other.data

        def backward():
            if self.requires_grad:
                local = other.data * out.grad
                self._accumulate(self._reduce_gradient(local, self.data.shape))
            if other.requires_grad:
                local = self.data * out.grad
                other._accumulate(self._reduce_gradient(local, other.data.shape))

        out._set_graph((self, other), backward)
        return out

    def __rmul__(self, other):
        return self._as_tensor(other).__mul__(self)

    def __matmul__(self, other):
        other = self._as_tensor(other)
        a, b = self.data, other.data
        out = Tensor(a @ b, self.requires_grad or other.requires_grad)

        def backward():
            g = out.grad
            if self.requires_grad:
                if a.ndim == 2 and b.ndim == 2:
                    local = g @ np.swapaxes(b, -1, -2)
                elif a.ndim == 1 and b.ndim == 2:
                    local = g @ np.swapaxes(b, -1, -2)
                elif a.ndim == 2 and b.ndim == 1:
                    local = np.outer(g, b)
                elif a.ndim == 1 and b.ndim == 1:
                    local = g * b
                else:
                    local = g @ np.swapaxes(b, -1, -2)
                self._accumulate(self._reduce_gradient(local, a.shape))
            if other.requires_grad:
                if a.ndim == 2 and b.ndim == 2:
                    local = np.swapaxes(a, -1, -2) @ g
                elif a.ndim == 1 and b.ndim == 2:
                    local = np.outer(a, g)
                elif a.ndim == 2 and b.ndim == 1:
                    local = np.swapaxes(a, -1, -2) @ g
                elif a.ndim == 1 and b.ndim == 1:
                    local = g * a
                else:
                    local = np.swapaxes(a, -1, -2) @ g
                other._accumulate(self._reduce_gradient(local, b.shape))

        out._set_graph((self, other), backward)
        return out

    def __rmatmul__(self, other):
        return self._as_tensor(other).__matmul__(self)

    def __neg__(self):
        return self * -1.0

    def relu(self):
        out = Tensor(np.maximum(self.data, 0.0), self.requires_grad)

        def backward():
            if self.requires_grad:
                self._accumulate((self.data > 0.0) * out.grad)

        out._set_graph((self,), backward)
        return out

    def exp(self):
        out = Tensor(np.exp(self.data), self.requires_grad)

        def backward():
            if self.requires_grad:
                self._accumulate(out.data * out.grad)

        out._set_graph((self,), backward)
        return out

    def log(self):
        out = Tensor(np.log(self.data), self.requires_grad)

        def backward():
            if self.requires_grad:
                self._accumulate(out.grad / self.data)

        out._set_graph((self,), backward)
        return out

    def sum(self, axis=None, keepdims=False):
        out = Tensor(self.data.sum(axis=axis, keepdims=keepdims), self.requires_grad)

        def backward():
            if self.requires_grad:
                expanded = np.broadcast_to(out.grad, out.data.shape)
                self._accumulate(np.ones(self.data.shape, dtype=np.float64) * expanded)

        out._set_graph((self,), backward)
        return out

    def mean(self, axis=None, keepdims=False):
        count = self.data.size if axis is None else self.data.shape[axis]
        return self.sum(axis=axis, keepdims=keepdims) * (1.0 / count)

    def backward(self):
        if self.data.size != 1:
            raise ValueError("backward can only be called on a scalar Tensor")
        if self.requires_grad and self.grad is None:
            self.grad = np.ones_like(self.data)

        topo: list[Tensor] = []
        visited: set[Tensor] = set()
        stack = [(self, False)]
        while stack:
            node, processed = stack.pop()
            if processed:
                topo.append(node)
                continue
            if node in visited:
                continue
            visited.add(node)
            stack.append((node, True))
            for parent in node._parents:
                if parent not in visited:
                    stack.append((parent, False))

        for node in reversed(topo):
            if node._backward is not None and node.requires_grad:
                node._backward()


def cross_entropy_logits(logits: Tensor, y) -> Tensor:
    """Mean cross entropy from unnormalized logits and integer labels."""
    labels = np.asarray(y, dtype=np.int64)
    n, classes = logits.data.shape
    shifted = logits - Tensor(logits.data.max(axis=1, keepdims=True))
    log_sum_exp = shifted.exp().sum(axis=1, keepdims=True).log()
    log_probs = shifted - log_sum_exp
    one_hot = np.zeros((n, classes), dtype=np.float64)
    one_hot[np.arange(n), labels] = 1.0
    selected = log_probs * Tensor(one_hot)
    return selected.sum() * (-1.0 / n)


class Net:
    def __init__(self, n_in: int, h1: int, h2: int, n_out: int, seed: int | None = None):
        rng = np.random.default_rng(seed)
        self.W1 = Tensor(rng.normal(0.0, math.sqrt(2.0 / n_in), (n_in, h1)), True)
        self.b1 = Tensor(np.zeros(h1), True)
        self.W2 = Tensor(rng.normal(0.0, math.sqrt(2.0 / h1), (h1, h2)), True)
        self.b2 = Tensor(np.zeros(h2), True)
        self.W3 = Tensor(rng.normal(0.0, math.sqrt(2.0 / h2), (h2, n_out)), True)
        self.b3 = Tensor(np.zeros(n_out), True)

    def params(self):
        return [self.W1, self.b1, self.W2, self.b2, self.W3, self.b3]

    def forward(self, X):
        x = Tensor(X)
        hidden1 = (x @ self.W1 + self.b1).relu()
        hidden2 = (hidden1 @ self.W2 + self.b2).relu()
        return hidden2 @ self.W3 + self.b3

    def predict(self, X):
        return np.argmax(self.forward(X).data, axis=1)

    def fit(self, X, y, epochs: int, lr: float, momentum: float):
        labels = np.asarray(y, dtype=np.int64)
        optimizer = SGD(self.params(), lr=lr, momentum=momentum)
        for _ in range(epochs):
            optimizer.zero_grad()
            loss = cross_entropy_logits(self.forward(X), labels)
            loss.backward()
            optimizer.step()

    def save(self, path: str):
        payload = {
            "W1": self.W1.data.tolist(),
            "b1": self.b1.data.tolist(),
            "W2": self.W2.data.tolist(),
            "b2": self.b2.data.tolist(),
            "W3": self.W3.data.tolist(),
            "b3": self.b3.data.tolist(),
        }
        with open(path, "w", encoding="utf-8") as handle:
            json.dump(payload, handle)


def load_net(path: str) -> Net:
    with open(path, "r", encoding="utf-8") as handle:
        payload = json.load(handle)
    w1 = np.asarray(payload["W1"], dtype=np.float64)
    b1 = np.asarray(payload["b1"], dtype=np.float64)
    w2 = np.asarray(payload["W2"], dtype=np.float64)
    b2 = np.asarray(payload["b2"], dtype=np.float64)
    w3 = np.asarray(payload["W3"], dtype=np.float64)
    b3 = np.asarray(payload["b3"], dtype=np.float64)
    net = Net(w1.shape[0], w1.shape[1], w2.shape[1], w3.shape[1], seed=None)
    for param, value in zip(net.params(), (w1, b1, w2, b2, w3, b3)):
        param.data[...] = value
    return net


class SGD:
    def __init__(self, params, lr: float, momentum: float = 0.0):
        self.params = list(params)
        self.lr = lr
        self.momentum = momentum
        self._velocities = [np.zeros_like(param.data) for param in self.params]

    def zero_grad(self):
        for param in self.params:
            param.grad = None

    def step(self):
        for param, velocity in zip(self.params, self._velocities):
            if param.grad is None:
                continue
            velocity *= self.momentum
            velocity += param.grad
            param.data -= self.lr * velocity
