# DESTEK — Goal'lar

1. **G1 — Önceliklendirme + yönlendirme/atama**: anahtar kelime/kanal/plan kuralları,
   beceri-kapasite-yük sıralı atama, sıralı toplu atama. Test: `tests/test_g1_oncelik_atama.py`
2. **G2 — İş saati ve SLA**: mesai dakikası hesabı (hafta sonu/tatil atlama),
   iş-dakikası ekleme, son teslim üretimi. Test: `tests/test_g2_sla.py`
3. **G3 — Yaşam döngüsü + eskalasyon**: olay yeniden-oynatma, durum makinesi,
   geçersiz geçiş denetimi, yeniden açma penceresi, olay-önü eskalasyon.
   Test: `tests/test_g3_yasam.py`
4. **G4 — Metrik raporu CLI**: replay sonucundan SPEC'teki özet JSON birebir.
   Test: `tests/test_g4_cli.py`
