# OTOGRAD — Goal'lar

1. **G1 — Tensor ileri işlemler**: aritmetik/matmul/relu/exp/log/sum/mean değer
   doğruluğu, broadcasting, skaler karışımı. Test: `tests/test_g1_ops.py`
2. **G2 — Geri yayılım motoru**: keyfî bileşik ifadelerde sayısal gradyan eşleşmesi;
   elmas yeniden-kullanımda birikim; broadcasting gradyan indirgeme; init determinizmi.
   Test: `tests/test_g2_backward.py`
3. **G3 — Motorla eğitim**: cross_entropy_logits + Net + momentum'lu SGD spiralde
   ≥ 0.95 eğitim doğruluğuna ulaşır; kayıp düşer; determinizm. Test: `tests/test_g3_egitim.py`
4. **G4 — Genelleme ve kayıt**: test kümesinde ≥ 0.88; save/load birebir; parametre
   bütçesi. Test: `tests/test_g4_genelleme.py`
