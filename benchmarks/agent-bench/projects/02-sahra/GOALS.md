# SAHRA — Goal'lar

1. **G1 — Öncelik skoru**: formül birebir; saat tabanı floor; sıralama determinizmi.
   Test: `tests/test_g1_oncelik.py`
2. **G2 — Ambar tahsisi**: en-çok-stoklu-ambar kuralı, kalem alfabetik işleme,
   yerinde stok düşümü, kısmi karşılama ve `unmet`. Test: `tests/test_g2_tahsis.py`
3. **G3 — Araç yükleme**: ağırlık-azalan satır sırası, kapasite-azalan araç sırası,
   satır bölme, `unshipped` ve paylaşımlı `used`. Test: `tests/test_g3_yukleme.py`
4. **G4 — Günlük plan CLI**: sıralı-paylaşımlı işlem zinciri fixtures üzerinde
   SPEC'teki JSON'u üretir. Test: `tests/test_g4_cli.py`
