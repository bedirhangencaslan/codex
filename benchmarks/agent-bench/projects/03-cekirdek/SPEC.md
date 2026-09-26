# ÇEKİRDEK — Sıfırdan Sinir Ağı (yalnız numpy)

Hiçbir ML kütüphanesi olmadan (yalnız `numpy` + standart kütüphane) tek gizli
katmanlı bir MLP: ileri yayılım, elle türetilmiş geri yayılım, eğitim ve kayıt.
`torch`, `sklearn`, `jax` vb. **yasaktır**.

Çözüm: `solution.py`.

## Veri (fixtures/)

- `train.json`, `test.json`: `{"X": [[f1, f2], ...], "y": [0|1, ...]}` — "iki ay"
  (two moons) veri kümesi, 2 öznitelik, 2 sınıf.

## Zorunlu API

```python
relu(x)                      # np.ndarray -> np.ndarray
softmax(z)                   # satır bazında, sayısal olarak kararlı
cross_entropy(probs, y)      # doğal log, örnek ortalaması; y tamsayı etiket dizisi

class MLP(n_in, n_hidden, n_out, seed):
    # Ağırlık başlatma SIRASI ve dağılımı sabittir:
    #   rng = numpy.random.default_rng(seed)
    #   W1 = rng.normal(0, sqrt(2/n_in),    (n_in, n_hidden))   (önce W1)
    #   W2 = rng.normal(0, sqrt(2/n_hidden), (n_hidden, n_out)) (sonra W2)
    #   b1 = zeros(n_hidden); b2 = zeros(n_out)
    # Öznitelikler dışarıdan erişilebilir: W1, b1, W2, b2 (numpy dizileri).

    forward(X)        # -> olasılıklar (softmax çıktısı), gizli katman ReLU
    loss(X, y)        # -> float (cross_entropy(forward(X), y))
    grads(X, y)       # -> {"W1","b1","W2","b2"}: batch ORTALAMASI gradyanlar
    step(grads, lr)   # parametreleri yerinde günceller: p -= lr * g
    fit(X, y, epochs, lr)   # tam-batch (full-batch) GD; her epoch: grads + step
    predict(X)        # -> tamsayı etiketler (argmax)
    save(path)        # JSON: {"W1": [[...]], "b1": [...], "W2": ..., "b2": ...}

load_mlp(path) -> MLP   # save edilmiş dosyadan aynı ağı geri kurar
```

## Kabul eşikleri

- Gradyan doğrulama: merkezi sonlu farkla göreli hata < 1e-4.
- `fit(train, epochs=400, lr=1.0, seed=0, n_hidden=16)` sonrası eğitim doğruluğu ≥ 0.97,
  test doğruluğu ≥ 0.90; toplam parametre sayısı < 2000; eğitim 60 saniyeyi aşmaz.
- Determinizm: aynı `seed` ile iki kez kurulan ağın `W1`'i bit-bit aynıdır.
