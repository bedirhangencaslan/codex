"""ÇEKİRDEK — single-hidden-layer MLP with hand-derived backprop (numpy only).

Forward pass:
    Z1 = X @ W1 + b1          (N, h)
    H  = relu(Z1)             (N, h)
    Z2 = H @ W2 + b2          (N, k)
    P  = softmax(Z2)          row-wise
    L  = -(1/N) * sum_i log P[i, y_i]

Backward pass (derived by hand, batch-mean gradients):
    Let Y be the one-hot encoding of y. For softmax followed by the mean
    cross-entropy loss, dL/dZ2 = (P - Y) / N.
    dW2 = H.T @ dZ2
    db2 = dZ2.sum(axis=0)
    dH  = dZ2 @ W2.T
    dZ1 = dH * (Z1 > 0)          # ReLU gate
    dW1 = X.T @ dZ1
    db1 = dZ1.sum(axis=0)
"""

from __future__ import annotations

import json

import numpy as np


def relu(x: np.ndarray) -> np.ndarray:
    """Element-wise rectified linear unit."""
    return np.maximum(x, 0.0)


def softmax(z: np.ndarray) -> np.ndarray:
    """Row-wise softmax, numerically stable via per-row max subtraction."""
    z = np.asarray(z, dtype=float)
    shifted = z - z.max(axis=1, keepdims=True)
    e = np.exp(shifted)
    return e / e.sum(axis=1, keepdims=True)


def cross_entropy(probs: np.ndarray, y: np.ndarray) -> float:
    """Mean cross-entropy (natural log) for integer labels y."""
    probs = np.asarray(probs, dtype=float)
    y = np.asarray(y, dtype=int)
    n = probs.shape[0]
    return float(-np.log(probs[np.arange(n), y]).mean())


class MLP:
    """Two-layer perceptron: input -> ReLU hidden -> softmax output."""

    def __init__(self, n_in: int, n_hidden: int, n_out: int, seed: int = 0):
        self.n_in = n_in
        self.n_hidden = n_hidden
        self.n_out = n_out
        rng = np.random.default_rng(seed)
        # Initialization order and distribution are fixed by the spec:
        # W1 first, then W2, both normal with He-style std.
        self.W1 = rng.normal(0.0, np.sqrt(2.0 / n_in), (n_in, n_hidden))
        self.W2 = rng.normal(0.0, np.sqrt(2.0 / n_hidden), (n_hidden, n_out))
        self.b1 = np.zeros(n_hidden)
        self.b2 = np.zeros(n_out)

    def forward(self, X: np.ndarray) -> np.ndarray:
        """Return softmax probabilities of shape (N, n_out)."""
        X = np.asarray(X, dtype=float)
        h = relu(X @ self.W1 + self.b1)
        return softmax(h @ self.W2 + self.b2)

    def loss(self, X: np.ndarray, y: np.ndarray) -> float:
        return cross_entropy(self.forward(X), y)

    def grads(self, X: np.ndarray, y: np.ndarray) -> dict[str, np.ndarray]:
        """Batch-mean analytic gradients of the cross-entropy loss."""
        X = np.asarray(X, dtype=float)
        y = np.asarray(y, dtype=int)
        n = X.shape[0]

        z1 = X @ self.W1 + self.b1
        h = relu(z1)
        probs = softmax(h @ self.W2 + self.b2)

        # dL/dZ2 for softmax + mean cross-entropy:
        dz2 = probs.copy()
        dz2[np.arange(n), y] -= 1.0
        dz2 /= n

        dW2 = h.T @ dz2
        db2 = dz2.sum(axis=0)

        dh = dz2 @ self.W2.T
        dz1 = dh * (z1 > 0)
        dW1 = X.T @ dz1
        db1 = dz1.sum(axis=0)

        return {"W1": dW1, "b1": db1, "W2": dW2, "b2": db2}

    def step(self, grads: dict[str, np.ndarray], lr: float) -> None:
        """In-place SGD update: p -= lr * g."""
        self.W1 -= lr * grads["W1"]
        self.b1 -= lr * grads["b1"]
        self.W2 -= lr * grads["W2"]
        self.b2 -= lr * grads["b2"]

    def fit(self, X: np.ndarray, y: np.ndarray, epochs: int, lr: float) -> None:
        """Full-batch gradient descent: one grads+step pair per epoch."""
        X = np.asarray(X, dtype=float)
        y = np.asarray(y, dtype=int)
        for _ in range(epochs):
            self.step(self.grads(X, y), lr)

    def predict(self, X: np.ndarray) -> np.ndarray:
        """Integer class labels (argmax of the softmax output)."""
        return np.argmax(self.forward(X), axis=1)

    def save(self, path: str) -> None:
        data = {
            "W1": self.W1.tolist(),
            "b1": self.b1.tolist(),
            "W2": self.W2.tolist(),
            "b2": self.b2.tolist(),
        }
        with open(path, "w") as f:
            json.dump(data, f)


def load_mlp(path: str) -> MLP:
    """Rebuild an MLP from a file written by MLP.save."""
    with open(path) as f:
        data = json.load(f)
    W1 = np.array(data["W1"], dtype=float)
    b1 = np.array(data["b1"], dtype=float)
    W2 = np.array(data["W2"], dtype=float)
    b2 = np.array(data["b2"], dtype=float)
    mlp = MLP(W1.shape[0], W1.shape[1], W2.shape[1])
    mlp.W1 = W1
    mlp.b1 = b1
    mlp.W2 = W2
    mlp.b2 = b2
    return mlp
