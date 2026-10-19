"""A minimal NumPy-only multilayer perceptron for the ÇEKİRDEK spec."""

from __future__ import annotations

import json
from collections.abc import Mapping
from pathlib import Path
from typing import Any

import numpy as np


def relu(x: np.ndarray) -> np.ndarray:
    """Return the elementwise ReLU of *x*."""
    return np.maximum(0.0, np.asarray(x))


def softmax(z: np.ndarray) -> np.ndarray:
    """Return stable row-wise softmax probabilities."""
    logits = np.asarray(z, dtype=float)
    shifted = logits - np.max(logits, axis=-1, keepdims=True)
    exponentials = np.exp(shifted)
    return exponentials / np.sum(exponentials, axis=-1, keepdims=True)


def cross_entropy(probs: np.ndarray, y: np.ndarray) -> float:
    """Return the sample-mean natural-log cross entropy."""
    probabilities = np.asarray(probs, dtype=float)
    labels = np.asarray(y, dtype=np.intp)
    chosen = probabilities[np.arange(probabilities.shape[0]), labels]
    return float(-np.mean(np.log(chosen)))


class MLP:
    """Two-layer MLP with a ReLU hidden layer and softmax output."""

    def __init__(self, n_in: int, n_hidden: int, n_out: int, seed: int) -> None:
        self.n_in = int(n_in)
        self.n_hidden = int(n_hidden)
        self.n_out = int(n_out)

        rng = np.random.default_rng(seed)
        # The order of these draws is part of the public initialization spec.
        self.W1: np.ndarray = rng.normal(
            0.0, np.sqrt(2.0 / self.n_in), (self.n_in, self.n_hidden)
        )
        self.W2: np.ndarray = rng.normal(
            0.0, np.sqrt(2.0 / self.n_hidden), (self.n_hidden, self.n_out)
        )
        self.b1: np.ndarray = np.zeros(self.n_hidden, dtype=float)
        self.b2: np.ndarray = np.zeros(self.n_out, dtype=float)

    def forward(self, X: np.ndarray) -> np.ndarray:
        """Return class probabilities for *X*."""
        hidden_pre = self._forward_hidden_pre(X)
        return softmax(relu(hidden_pre) @ self.W2 + self.b2)

    def loss(self, X: np.ndarray, y: np.ndarray) -> float:
        """Return mean cross entropy under the current parameters."""
        return cross_entropy(self.forward(X), y)

    def grads(self, X: np.ndarray, y: np.ndarray) -> dict[str, np.ndarray]:
        """Return averaged analytical gradients for one full batch."""
        inputs = np.asarray(X, dtype=float)
        labels = np.asarray(y, dtype=np.intp)
        hidden_pre, hidden = self._forward_hidden_pre(inputs, return_hidden=True)
        probabilities = softmax(hidden @ self.W2 + self.b2)

        # d(loss)/d(logits) for softmax followed by cross entropy.
        delta_output = probabilities
        delta_output[np.arange(inputs.shape[0]), labels] -= 1.0
        delta_output /= inputs.shape[0]

        grad_W2 = hidden.T @ delta_output
        grad_b2 = np.sum(delta_output, axis=0)

        delta_hidden = delta_output @ self.W2.T
        # ReLU passes its preactivation exactly where the forward value > 0.
        delta_hidden *= hidden_pre > 0.0

        grad_W1 = inputs.T @ delta_hidden
        grad_b1 = np.sum(delta_hidden, axis=0)

        return {"W1": grad_W1, "b1": grad_b1, "W2": grad_W2, "b2": grad_b2}

    def step(self, grads: Mapping[str, np.ndarray], lr: float) -> None:
        """Apply `parameter -= learning_rate * gradient` in place."""
        self.W1 -= lr * np.asarray(grads["W1"])
        self.b1 -= lr * np.asarray(grads["b1"])
        self.W2 -= lr * np.asarray(grads["W2"])
        self.b2 -= lr * np.asarray(grads["b2"])

    def fit(
        self, X: np.ndarray, y: np.ndarray, epochs: int, lr: float
    ) -> "MLP":
        """Train with full-batch gradient descent."""
        inputs = np.asarray(X, dtype=float)
        labels = np.asarray(y, dtype=np.intp)
        for _ in range(int(epochs)):
            self.step(self.grads(inputs, labels), lr)
        return self

    def predict(self, X: np.ndarray) -> np.ndarray:
        """Return integer labels with the greatest probability."""
        return np.argmax(self.forward(X), axis=-1)

    def save(self, path: str | Path) -> None:
        """Save parameters as JSON."""
        parameters = {
            "W1": self.W1.tolist(),
            "b1": self.b1.tolist(),
            "W2": self.W2.tolist(),
            "b2": self.b2.tolist(),
        }
        Path(path).write_text(json.dumps(parameters))

    def _forward_hidden_pre(
        self, X: np.ndarray, *, return_hidden: bool = False
    ) -> np.ndarray | tuple[np.ndarray, np.ndarray]:
        inputs = np.asarray(X, dtype=float)
        hidden_pre = inputs @ self.W1 + self.b1
        if return_hidden:
            return hidden_pre, relu(hidden_pre)
        return hidden_pre


def load_mlp(path: str | Path) -> MLP:
    """Load an MLP saved by :meth:`MLP.save`."""
    data: dict[str, Any] = json.loads(Path(path).read_text())
    w1 = np.asarray(data["W1"], dtype=float)
    w2 = np.asarray(data["W2"], dtype=float)
    network = MLP(w1.shape[0], w1.shape[1], w2.shape[1], seed=0)
    network.W1[:] = w1
    network.b1[:] = np.asarray(data["b1"], dtype=float)
    network.W2[:] = w2
    network.b2[:] = np.asarray(data["b2"], dtype=float)
    return network
