import json
import math

import numpy as np


def relu(x):
    return np.maximum(np.asarray(x, dtype=float), 0.0)


def softmax(z):
    z = np.asarray(z, dtype=float)
    z = z - z.max(axis=-1, keepdims=True)
    e = np.exp(z)
    return e / e.sum(axis=-1, keepdims=True)


def cross_entropy(probs, y):
    probs = np.asarray(probs, dtype=float)
    y = np.asarray(y, dtype=int)
    p = np.clip(probs[np.arange(y.shape[0]), y], 1e-12, None)
    return float(-np.mean(np.log(p)))


class MLP:
    def __init__(self, n_in, n_hidden, n_out, seed=0):
        self.n_in = n_in
        self.n_hidden = n_hidden
        self.n_out = n_out
        rng = np.random.default_rng(seed)
        self.W1 = rng.normal(0.0, math.sqrt(2.0 / n_in), (n_in, n_hidden))
        self.W2 = rng.normal(0.0, math.sqrt(2.0 / n_hidden), (n_hidden, n_out))
        self.b1 = np.zeros(n_hidden)
        self.b2 = np.zeros(n_out)

    def _forward_cache(self, X):
        Z1 = X @ self.W1 + self.b1
        A1 = relu(Z1)
        Z2 = A1 @ self.W2 + self.b2
        return Z1, A1, Z2

    def forward(self, X):
        X = np.asarray(X, dtype=float)
        _, _, Z2 = self._forward_cache(X)
        return softmax(Z2)

    def loss(self, X, y):
        return cross_entropy(self.forward(X), y)

    def grads(self, X, y):
        X = np.asarray(X, dtype=float)
        y = np.asarray(y, dtype=int)
        n = X.shape[0]
        Z1, A1, Z2 = self._forward_cache(X)
        P = softmax(Z2)

        # dL/dZ2 = (softmax - onehot) / N   (fused softmax + cross-entropy)
        dZ2 = P.copy()
        dZ2[np.arange(n), y] -= 1.0
        dZ2 /= n

        dW2 = A1.T @ dZ2
        db2 = dZ2.sum(axis=0)

        dA1 = dZ2 @ self.W2.T
        dZ1 = dA1 * (Z1 > 0.0)

        dW1 = X.T @ dZ1
        db1 = dZ1.sum(axis=0)

        return {"W1": dW1, "b1": db1, "W2": dW2, "b2": db2}

    def step(self, grads, lr):
        self.W1 -= lr * grads["W1"]
        self.b1 -= lr * grads["b1"]
        self.W2 -= lr * grads["W2"]
        self.b2 -= lr * grads["b2"]

    def fit(self, X, y, epochs, lr):
        X = np.asarray(X, dtype=float)
        y = np.asarray(y, dtype=int)
        for _ in range(int(epochs)):
            self.step(self.grads(X, y), lr)

    def predict(self, X):
        return np.argmax(self.forward(X), axis=1)

    def save(self, path):
        data = {
            "W1": self.W1.tolist(),
            "b1": self.b1.tolist(),
            "W2": self.W2.tolist(),
            "b2": self.b2.tolist(),
        }
        with open(path, "w") as f:
            json.dump(data, f)


def load_mlp(path):
    with open(path) as f:
        data = json.load(f)
    W1 = np.array(data["W1"], dtype=float)
    b1 = np.array(data["b1"], dtype=float)
    W2 = np.array(data["W2"], dtype=float)
    b2 = np.array(data["b2"], dtype=float)
    mlp = MLP(W1.shape[0], W1.shape[1], W2.shape[1], seed=0)
    mlp.W1, mlp.b1, mlp.W2, mlp.b2 = W1, b1, W2, b2
    return mlp
