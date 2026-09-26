# KANBAĞI — Kan Bağışı Koordinasyon Sistemi

İl sağlık müdürlüğü için kan bağışı koordinasyon çekirdeği: bağışçı uygunluğu,
stok (SKT/FIFO), kan grubu uyumluluğu ve taleplerin karşılanması.

Çözüm: bu klasördeki `solution.py` (yalnız Python standart kütüphanesi).
Tüm tarihler ISO `YYYY-MM-DD` string'idir. Tüm sıralamalar deterministiktir.

## Veri (fixtures/)

- `donors.json`: `{id, name, blood_type, sex ("E"|"K"), birth_date, last_donation (null olabilir), city}`
- `stock.json`: torba listesi `{id, blood_type, collected_on, city}`
- `requests.json`: `{id, blood_type, units, urgent (bool), city, created_at}`

Kan grupları: `O-, O+, A-, A+, B-, B+, AB-, AB+` (tam bu yazımla).

## Kurallar

1. **Uyumluluk (bağışçı → alıcı)**: ABO+Rh standardı. O herkese ABO verir; A → A,AB;
   B → B,AB; AB → AB. Rh(-) hem (-) hem (+) alıcıya verebilir; Rh(+) yalnız (+)'ya.
2. **SKT**: bir torbanın son kullanma tarihi `collected_on + 42 gün`dür.
   `on_date` itibarıyla geçerli sayılması için `(on_date - collected_on) < 42 gün` olmalıdır
   (42. gün dahil değildir).
3. **Bağış uygunluğu** (`on_date` itibarıyla): yaş 18–65 dahil (yaş, doğum günü henüz
   geçmediyse bir eksik sayılır); son bağıştan bu yana **erkek ≥ 90**, **kadın ≥ 120 gün**
   geçmiş olmalı (eşitlik uygundur); `last_donation` null ise aralık şartı yoktur.
4. **Talep karşılama** sırası:
   a. Önce stok: alıcı grubuna uyumlu ve geçerli torbalar. Acil değilse yalnız talep
      şehrindeki torbalar; **acilse tüm şehirler**. Sıralama: `collected_on` artan (FIFO),
      eşitlikte torba `id` artan. En fazla `units` torba alınır.
   b. Eksik kalırsa bağışçı çağrısı: uygun (kural 3) ve uyumlu bağışçılar. Acil değilse
      yalnız talep şehri; acilse önce talep şehri sonra diğerleri. Grup içi sıralama:
      `last_donation` null olanlar önce, sonra `last_donation` artan, eşitlikte `id` artan.
      En fazla kalan ihtiyaç kadar bağışçı çağrılır (bağışçı başına 1 ünite varsayılır).
   c. `shortfall = units - stoktan alınan - çağrılan bağışçı` (negatif olamaz).

## Zorunlu API (`solution.py`)

```python
can_donate(donor_type: str, recipient_type: str) -> bool
valid_stock(units: list[dict], on_date: str) -> dict[str, int]   # 8 grubun TAMAMI anahtar (0 dahil)
eligible(donor: dict, on_date: str) -> bool
match_request(request: dict, units: list[dict], donors: list[dict], on_date: str) -> dict
# dönüş: {"from_stock": [torba_id...], "donor_calls": [bagisci_id...], "shortfall": int}
```

## CLI

```
python3 solution.py report --fixtures <DIR> --date YYYY-MM-DD
```

stdout'a TEK satırlık olmayan, `json.dumps` ile yazılmış şu JSON basılır:

```json
{
  "date": "...",
  "stock": {"O-": n, "O+": n, "A-": n, "A+": n, "B-": n, "B+": n, "AB-": n, "AB+": n},
  "critical": ["geçerli stoğu 2'nin ALTINDA olan gruplar, Python sorted() sırasıyla"],
  "expiring_7d": ["SKT'sine 7 gün ve daha az kalmış geçerli torba id'leri, id artan"],
  "open_requests": talep_sayısı
}
```
