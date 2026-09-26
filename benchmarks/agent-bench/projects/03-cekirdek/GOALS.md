# ÇEKİRDEK — Goal'lar

1. **G1 — İleri yayılım**: relu/softmax/cross_entropy tam değerler; elle verilen
   ağırlıklarla forward birebir; init determinizmi. Test: `tests/test_g1_ileri.py`
2. **G2 — Geri yayılım**: analitik gradyanlar sayısal (merkezi fark) gradyanlarla
   < 1e-4 göreli hata içinde. Test: `tests/test_g2_gradyan.py`
3. **G3 — Eğitim**: tam-batch GD fixture eğitim kümesinde ≥ 0.97 doğruluğa 60 sn
   içinde ulaşır. Test: `tests/test_g3_egitim.py`
4. **G4 — Genelleme ve kayıt**: test kümesinde ≥ 0.90; save/load birebir aynı
   tahminler; parametre bütçesi < 2000. Test: `tests/test_g4_genelleme.py`
