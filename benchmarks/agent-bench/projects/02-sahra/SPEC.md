# SAHRA — Afet Yardım Lojistiği

AFAD benzeri bir koordinasyon merkezi için: ihtiyaç önceliklendirme, ambarlardan
tahsis, araçlara yükleme ve günlük sevkiyat planı.

Çözüm: `solution.py` (yalnız standart kütüphane). Sıralamalar deterministiktir.
Tarih-saatler ISO `YYYY-MM-DDTHH:MM` biçimindedir.

## Veri (fixtures/)

- `catalog.json`: `{item: birim_kg}`
- `warehouses.json`: `{id, city, stock: {item: adet}}`
- `vehicles.json`: `{id, warehouse, capacity_kg}`
- `needs.json`: `{id, city, items: {item: adet}, population, severity (1-5), created_at}`

## Kurallar

1. **Öncelik skoru** (tamsayı):
   `priority = severity*100 + bekleme_saati*2 + population // 1000`
   `bekleme_saati = floor((now - created_at) / 1 saat)`.
2. **Tahsis** (`allocate`): ihtiyacın kalemleri **alfabetik** sırayla işlenir. Her kalem
   için: o kalemden stoğu **en çok** olan ambardan alınır (eşitlikte ambar `id` küçük olan);
   stok yetmezse kalan miktar için sıradaki en çok stoklu ambara geçilir. Verilen
   `warehouses` listesi **yerinde güncellenir** (stok düşülür). Karşılanamayan miktarlar
   `unmet`'e yazılır (yalnız > 0 olanlar).
3. **Yükleme** (`load_vehicles`): tahsis satırları ambar bazında gruplanır. Her ambarda
   satırlar **toplam ağırlık azalan** (eşitlikte item alfabetik) sırayla; o ambarın
   araçları **orijinal kapasite azalan** (eşitlikte `id` artan) sabit sırasıyla doldurulur.
   Bir satır araçlara **bölünebilir** (adet bazında): araca sığan adet
   `min(kalan_adet, floor(kalan_kapasite / birim_kg))`. Hiçbir araca sığmayan kalan,
   `unshipped`'e yazılır. `used` sözlüğü (araç → kullanılmış kg) **yerinde güncellenir**;
   böylece ardışık çağrılar aynı araç filosunu paylaşır.
4. **Günlük plan** (CLI): ihtiyaçlar öncelik **azalan** (eşitlikte `id` artan) sırayla,
   **paylaşımlı** ambar stoğu ve araç kapasitesiyle sırayla işlenir.

## Zorunlu API (`solution.py`)

```python
priority(need: dict, now: str) -> int
allocate(need: dict, warehouses: list[dict]) -> dict
# {"allocations": [{"warehouse": id, "item": item, "qty": n}...], "unmet": {item: n}}
# allocations sırası: kalem alfabetik, kalem içinde alınış sırası.
load_vehicles(allocations: list[dict], vehicles: list[dict], catalog: dict, used: dict) -> dict
# {"loads": {vehicle_id: [{"item": i, "qty": n, "kg": k}...]}, "unshipped": [{"warehouse","item","qty"}...]}
# loads yalnız yük alan araçları içerir; unshipped (warehouse, item) artan sıralı.
```

## CLI

```
python3 solution.py plan --fixtures <DIR> --now YYYY-MM-DDTHH:MM
```

stdout'a `json.dumps` ile:

```json
{
  "now": "...",
  "ranked": ["öncelik azalan ihtiyaç id'leri"],
  "unmet": {"yalnız >0 toplamlar, anahtar alfabetik"},
  "loaded_kg": {"yalnız >0 kg toplamı olan araçlar"},
  "unshipped": {"yalnız >0 toplamlar, anahtar alfabetik"}
}
```
