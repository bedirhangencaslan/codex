# MUTABAKAT — Fatura ↔ Banka Ekstresi Mutabakatı

Muhasebe ekiplerinin her ay elle yaptığı iş: kesilen faturaları banka ekstresindeki
tahsilatlarla eşleştir, farkları ve açıkta kalanları raporla.

Çözüm: `solution.py` (yalnız standart kütüphane).

## Veri (fixtures/)

- `invoices.csv` — `;` ayraçlı, başlık satırı var: `fatura_no;musteri;tutar;para_birimi;tarih`
- `bank.csv` — `;` ayraçlı, başlık satırı var: `tarih;aciklama;tutar;para_birimi`
- `rates.json` — `{ "YYYY-MM-DD": {"EUR": kur, "USD": kur}, ... }` (1 birim → TRY)

Biçimler: tutar TR yazımı `1.234,56`; tarih `GG.AA.YYYY` **veya** `YYYY-AA-GG`.

## Kurallar

1. **Yükleme**: bozuk satır (parse edilemeyen tutar, boş/bozuk tarih, eksik alan)
   çökertmez; `errors` listesine `{"line": dosyadaki_satır_no, "reason": metin}`
   olarak yazılır (başlık satırı 1'dir). Ödemelere veri satırı sırasına göre
   `p1, p2, …` kimliği verilir. Tarihler ISO `YYYY-MM-DD`'ye normalize edilir.
2. **Kur çevrimi** `to_try(amount, currency, date, rates)`: TRY ise aynen; değilse
   `date` kuru, yoksa **en yakın önceki** günün kuru (en fazla 7 gün geriye),
   o da yoksa `ValueError`. Sonuç `round(x, 2)`.
3. **Eşleştirme** (`match`): faturalar dosya sırasıyla işlenir; her ödeme en fazla
   bir faturaya gider. Tutar karşılaştırması: iki tarafın para birimi aynıysa
   orijinal tutarla, değilse TRY karşılığıyla; `round(|fark|, 2) <= 0.01` tolerans.
   Kural sırası (ilk tutan kazanır, adaylar ödeme id sırasıyla denenir):
   1. **Kesin**: açıklama `fatura_no`'yu içeriyor VE tutar toleransta → tek ödeme.
   2. **Tutar+tarih+müşteri**: tutar toleransta VE `|tarih farkı| <= 3 gün` VE
      müşteri adının ilk kelimesi (`str.lower()`) açıklamada (`str.lower()`) geçiyor.
   3. **Bölünmüş**: açıklamasında `fatura_no` geçen ödemelerden **tam iki tanesinin**
      toplamı toleransta → o iki ödeme (id sırasıyla).
4. `fark_try`: fatura ile ödeme toplamı **aynı para birimindeyse** önce o birimde
   `|fark|` alınır, sonra **fatura tarihiyle** TRY'ye çevrilir (kur oynaklığı sahte
   fark üretmesin diye). Para birimleri farklıysa TRY karşılıklarının farkı alınır.
   Her durumda sonuç `round(x, 2)`.

## Zorunlu API (`solution.py`)

```python
parse_amount(s: str) -> float                # "12.500,00" -> 12500.0
load_invoices(path) -> (rows, errors)
# rows: {"fatura_no","musteri","tutar": float,"para_birimi","tarih": "YYYY-MM-DD"}
load_payments(path) -> rows
# rows: {"id","tarih","aciklama","tutar": float,"para_birimi"}
to_try(amount: float, currency: str, date: str, rates: dict) -> float
match(invoices, payments, rates) -> dict
# {"matched": [{"fatura_no","payment_ids": [...], "fark_try": x}...],  # fatura dosya sırası
#  "unmatched_invoices": [fatura_no...], "unmatched_payments": [id...]}
```

## CLI

```
python3 solution.py report --fixtures <DIR>
```

stdout'a `json.dumps` ile:

```json
{"matched": n, "unmatched_invoices": [...], "unmatched_payments": [...],
 "toplam_fark_try": x, "hatali_satirlar": n}
```

`toplam_fark_try` = eşleşmelerin `fark_try` toplamı, `round(x, 2)`.
`hatali_satirlar` = fatura dosyasındaki hata satırı sayısı.
