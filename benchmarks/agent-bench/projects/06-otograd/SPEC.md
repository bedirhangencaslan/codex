# OTOGRAD — Ters-Mod Otomatik Türev Motoru (yalnız numpy)

ÇEKİRDEK'in ileri seviyesi: elle türetilmiş geri yayılım YOK. Bunun yerine
mikro bir autograd motoru yazılır (PyTorch'un çekirdek fikri) ve ağ, kayıp ve
eğitim **tamamen bu motorun üzerine** kurulur. Testler motoru keyfî ifade
grafikleriyle sınar; sabit-kodlu türevlerle geçilemez.

Yalnız `numpy` + standart kütüphane. Başka ML kütüphanesi yasak.

## Veri (fixtures/)

- `train.json`, `test.json`: `{"X": [[f1,f2],...], "y": [0|1|2,...]}` — 3 sınıflı
  spiral, 300 eğitim / 75 test örneği.

## Zorunlu API (`solution.py`)

### Tensor

```python
class Tensor:
    def __init__(self, data, requires_grad=False)   # data: numpy'a çevrilebilir
    data: np.ndarray        # float64
    grad: np.ndarray | None # backward sonrası; requires_grad=False ise None kalır
    requires_grad: bool
```

İşlemler (hepsi yeni `Tensor` döndürür, grafik kurar):
`a + b`, `a - b`, `a * b` (eleman bazında, **numpy broadcasting kurallarıyla**;
b `Tensor` ya da skaler olabilir), `a @ b` (matmul), `a.relu()`, `a.exp()`,
`a.log()`, `a.sum()` (skalere), `a.mean()` (skalere).

`t.backward()`: yalnız skaler (`t.data.size == 1`) üzerinde çağrılır; grafikteki
`requires_grad=True` yapraklara gradyan yazar. Kurallar:

- Topolojik sıra doğru işlenir; **aynı düğüm birden çok yerde kullanılırsa**
  gradyanlar **toplanır** (ör. `z = x*y + x`).
- Broadcasting'li işlemlerde gradyan, yayınlanan eksenler üzerinden **toplanarak**
  orijinal şekle indirgenir.
- Ardışık `backward` çağrıları arasında gradyanlar `zero_grad()` ile sıfırlanır
  (aşağıdaki `SGD.zero_grad`); tek `backward` içinde birikim doğru olmalıdır.

### Kayıp ve ağ

```python
cross_entropy_logits(logits: Tensor, y: np.ndarray) -> Tensor  # skaler; sayısal kararlı
class Net(n_in, h1, h2, n_out, seed):
    # rng = np.random.default_rng(seed); SIRAYLA:
    #   W1 = rng.normal(0, sqrt(2/n_in), (n_in, h1))
    #   W2 = rng.normal(0, sqrt(2/h1),   (h1, h2))
    #   W3 = rng.normal(0, sqrt(2/h2),   (h2, n_out))
    # b1,b2,b3 = zeros. Hepsi requires_grad=True Tensor.
    params() -> [W1, b1, W2, b2, W3, b3]        # bu sırayla
    forward(X: np.ndarray) -> Tensor            # logits; relu(X@W1+b1) → relu(@W2+b2) → @W3+b3
    predict(X) -> np.ndarray                    # argmax etiketleri
    fit(X, y, epochs, lr, momentum)             # tam-batch; her epoch: zero_grad → CE → backward → adım
    save(path)                                  # JSON {"W1":...,"b1":...,...}
load_net(path) -> Net

class SGD(params, lr, momentum):
    zero_grad()      # tüm grad'ları None ya da 0 yapar
    step()           # v = momentum*v + grad; p.data -= lr*v   (v başlangıçta 0)
```

`fit` içindeki ileri/geri hesap **yalnız Tensor işlemleriyle** yapılır.

## Kabul eşikleri

- Sayısal gradyan doğrulaması (merkezi fark, keyfî bileşik ifadeler, broadcasting
  ve elmas-yeniden-kullanım dahil): göreli hata < 1e-4.
- `Net(2, 32, 16, 3, seed=0)` + `fit(train, epochs=1200, lr=0.3, momentum=0.9)`:
  eğitim doğruluğu ≥ 0.95, test doğruluğu ≥ 0.88, süre < 120 sn.
- Parametre sayısı < 5000. Aynı seed → bit-bit aynı sonuç.
