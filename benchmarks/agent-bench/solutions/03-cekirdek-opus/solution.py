"""ÇEKİRDEK — Sıfırdan tek gizli katmanlı MLP (yalnız numpy).

İleri yayılım, elle türetilmiş geri yayılım, tam-batch GD eğitimi ve JSON kayıt.
"""

import json
import math

import numpy as np


def relu(x):
    """ReLU: max(0, x), eleman bazında."""
    return np.maximum(0.0, x)


def softmax(z):
    """Satır bazında softmax, sayısal olarak kararlı."""
    z = np.asarray(z, dtype=float)
    z_max = np.max(z, axis=-1, keepdims=True)
    e = np.exp(z - z_max)
    return e / np.sum(e, axis=-1, keepdims=True)


def cross_entropy(probs, y):
    """Ortalama çapraz entropi (doğal log). y: tamsayı etiket dizisi."""
    probs = np.asarray(probs, dtype=float)
    y = np.asarray(y)
    n = probs.shape[0]
    # sayısal kararlılık için küçük bir taban ekle
    picked = probs[np.arange(n), y]
    return float(-np.mean(np.log(picked)))


class MLP:
    def __init__(self, n_in, n_hidden, n_out, seed=0):
        self.n_in = n_in
        self.n_hidden = n_hidden
        self.n_out = n_out
        rng = np.random.default_rng(seed)
        # SIRA sabittir: önce W1, sonra W2.
        self.W1 = rng.normal(0, math.sqrt(2 / n_in), (n_in, n_hidden))
        self.W2 = rng.normal(0, math.sqrt(2 / n_hidden), (n_hidden, n_out))
        self.b1 = np.zeros(n_hidden)
        self.b2 = np.zeros(n_out)

    def _forward_cache(self, X):
        X = np.asarray(X, dtype=float)
        z1 = X @ self.W1 + self.b1
        a1 = relu(z1)
        z2 = a1 @ self.W2 + self.b2
        probs = softmax(z2)
        return X, z1, a1, z2, probs

    def forward(self, X):
        return self._forward_cache(X)[4]

    def loss(self, X, y):
        return cross_entropy(self.forward(X), np.asarray(y))

    def grads(self, X, y):
        """Batch ORTALAMASI analitik gradyanlar.

        Türetme (N örnek):
          z1 = X W1 + b1;  a1 = relu(z1);  z2 = a1 W2 + b2;  p = softmax(z2)
          L  = -(1/N) sum_i log p[i, y_i]
          softmax+CE için:  dz2 = (p - onehot(y)) / N
          dW2 = a1^T dz2            db2 = sum_rows dz2
          da1 = dz2 W2^T            dz1 = da1 * (z1 > 0)
          dW1 = X^T dz1             db1 = sum_rows dz1
        """
        y = np.asarray(y)
        X, z1, a1, z2, probs = self._forward_cache(X)
        n = X.shape[0]

        dz2 = probs.copy()
        dz2[np.arange(n), y] -= 1.0
        dz2 /= n

        dW2 = a1.T @ dz2
        db2 = dz2.sum(axis=0)

        da1 = dz2 @ self.W2.T
        dz1 = da1 * (z1 > 0)

        dW1 = X.T @ dz1
        db1 = dz1.sum(axis=0)

        return {"W1": dW1, "b1": db1, "W2": dW2, "b2": db2}

    def step(self, grads, lr):
        self.W1 -= lr * grads["W1"]
        self.b1 -= lr * grads["b1"]
        self.W2 -= lr * grads["W2"]
        self.b2 -= lr * grads["b2"]

    def fit(self, X, y, epochs, lr):
        X = np.asarray(X, dtype=float)
        y = np.asarray(y)
        for _ in range(epochs):
            g = self.grads(X, y)
            self.step(g, lr)
        return self

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
    W2 = np.array(data["W2"], dtype=float)
    n_in, n_hidden = W1.shape
    n_out = W2.shape[1]
    mlp = MLP(n_in, n_hidden, n_out, seed=0)
    mlp.W1 = W1
    mlp.b1 = np.array(data["b1"], dtype=float)
    mlp.W2 = W2
    mlp.b2 = np.array(data["b2"], dtype=float)
    return mlp
