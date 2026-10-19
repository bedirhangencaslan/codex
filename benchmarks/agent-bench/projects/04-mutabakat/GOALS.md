# MUTABAKAT — Goal'lar

1. **G1 — Dayanıklı yükleme**: TR sayı/iki tarih biçimi; bozuk satırlar çökertmeden
   satır numarasıyla toplanır; ödeme kimlikleri sıralı. Test: `tests/test_g1_yukleme.py`
2. **G2 — Kur çevrimi**: doğrudan kur, ≤7 gün geriye düşüş, 7 günü aşınca ValueError,
   TRY geçişi, 2 hane yuvarlama. Test: `tests/test_g2_kur.py`
3. **G3 — Eşleştirme motoru**: üç kuralın önceliği, tolerans, bölünmüş ödeme,
   ödeme tekilliği ve determinizm. Test: `tests/test_g3_eslestirme.py`
4. **G4 — Mutabakat raporu CLI**: fixtures üzerinde SPEC'teki özet JSON birebir.
   Test: `tests/test_g4_cli.py`
