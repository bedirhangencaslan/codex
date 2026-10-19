# KANBAĞI — Goal'lar

1. **G1 — Uyumluluk ve stok envanteri**: `can_donate` 64 kombinasyonun tamamında doğru;
   `valid_stock` SKT kuralını (42. gün hariç) uygulayıp 8 grubun tümünü sayar.
   Test: `tests/test_g1_uyum_stok.py`
2. **G2 — Bağış uygunluğu**: yaş sınırları (18/65 dahil, doğum günü hesabı) ve
   cinsiyete göre bağış aralığı (E≥90, K≥120, eşitlik dahil, null serbest).
   Test: `tests/test_g2_uygunluk.py`
3. **G3 — Talep eşleştirme**: FIFO stok seçimi, acil/şehir kuralları, bağışçı çağrı
   sıralaması ve `shortfall` hesabı; determinizm. Test: `tests/test_g3_eslestirme.py`
4. **G4 — Rapor CLI**: `report` alt komutu fixtures üzerinde SPEC'teki JSON'u
   alan alanına doğru üretir. Test: `tests/test_g4_cli.py`
