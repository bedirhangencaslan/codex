# reviews.py incelemesi

## Purpose

Bu dosya restoran yorumlari icin bir endpoint ve genel puan hesaplama yardimcisini tanimlar. `main.py` hem `displayed_rating` fonksiyonunu hem de router'i kullanir; `Account/RestaurantDetail` arayuzu de yorum olusturma endpoint'ine baglanir. Dosya bir musterinin ancak onayli siparis sonrasi ve restoran basina bir kez yorum yapabilmesini zorunlu tutar.

## Walkthrough

### displayed_rating
`def displayed_rating(seed_rating: float, review_rows) -> float` 13-15 satirlarindadir. Parametre olarak restoranin seed puanini ve review satirlarini alir. Donus degeri iki ondalige yuvarlanmis float. `ratings` listesi seed rating ile her review satirindaki `row["rating"]` degerlerini toplar. Sonra `sum / len` ile ortalama alir. Kontrol akisi bos review listesi olsa da seed rating oldugu icin hata vermez. Ancak `seed_rating` None ise `TypeError` verir.

### create_review
`@router.post("/restaurants/{restaurant_id}/reviews", status_code=201)` ile suslenmis `def create_review(...)` 18-50 satirlarindadir. Parametreler `restaurant_id`, `ReviewInput` ve `require_customer` ile dogrulanmis `user`. Donus degeri id, rating, comment ve `user_email` iceren dict. Once yorum metninin `strip()` sonrasi bos olup olmadigini kontrol eder; bossa 400 dondurur. Baglanti acar. Restoran yoksa 404 dondurur. Kullanici onayli siparise sahip degilse 403 dondurur. Restoran basina daha once yorum varsa 409 dondurur. Gecerli durumda `restaurant_review` tablosuna rating, strip edilmis comment ve `utc_now()` ile ekler. `cursor.lastrowid` ile id donduruur. Blok cikisinda commit olur.

## Data

- `restaurant` okunur: varlik kontrolu 27-29.
- `order` okunur: onayli siparis kontrolu 32-36.
- `restaurant_review` okunur: duplicate kontrol 39-42; yazilir: insert 44-49.
- HTTP endpoint: `POST /api/restaurants/{restaurant_id}/reviews` 18-50.
- Client state yok.
- `main.py` `displayed_rating` cagirarak liste/detay puanini hesaplar (main.py 59-60 ve 103).

## Failure modes

- Yorum yapma sarti `"onaylandı"` literaline bagli (34). Eger backend'in diger katmanlari mojibake status uretiyorsa musteriler yorum yapamaz.
- `displayed_rating` sadece seed ve review rating'leri icerir; onaylanmamis veya degistirilmis yorum yoksa gercekten dogru. Ama yorum silme/moderasyon akisi yok.
- Yorum metni strip edilerek kaydedilir, ancak uzunluk siniri yoktur; cok buyuk metin DB ve UI performansini bozabilir (44-49).
- Rating 1-5 arasi Pydantic ile zorunlu (`panels.py:31`), ama yorumda olasi race durumunda ayni kullanici iki kez insert etmeye calisirsa `IntegrityError` HTTP'ye cevrilmez; UNIQUE kontrol once yapildigindan pratikte dusuk olasilikla 500 olabilir (39-49).
- Yorum aninda silinen restoran veya baskasinin siparisini kullanmaya calisma guard'larla engellenir, fakat transaction izolasyon seviyesi özel olarak set edilmediginden ayni anda ikinci yorum insert teorik race yaratabilir (26-49).
- `user_email` yanitida dondurulur; bu kullaniciya ozel bilgi degil ama privacy tercihine gore sorun olabilir (50).
- Yorumlar yorum sahibinin emailini public detayda dondurur; `main.py:94-99` buraya baglidir.

## Security

- Kimlik dogrulama: `require_customer` ile (22).
- Yetkilendirme: yalniz musteriler yorum yapabilir; onayli siparis sarti 32-38.
- Ownership/injection yok; restoran id ve user id parametreli sorgularla gecirilir (27-49).
- Duplicate kontrol `restaurant_id + user_id` uzerinden (39-42) ve schema UNIQUE (schema 41) ile desteklenir.
- Veri ifsasi: yanit yorum sahibi e-postasini icerir (50); public listelerde de email doner (`main.py:94-99`).
- Rating degeri sadece 1-5 arasi kabul edilir (panels.py:30-32), boylece ortalama puan manipulasyonu form dogrulamasiyla sinirli.

## Suggested changes

- Mevcut:
  `comment: str`
- Onerilen:
  `comment: str = Field(min_length=1, max_length=500)` veya endpoint icinde 500 karakter siniri.

- Mevcut:
  `if conn.execute("SELECT 1 FROM restaurant_review WHERE restaurant_id = ? AND user_id = ?", ...).fetchone():`
- Onerilen:
  race durumu icin insert'i `try` icine alin, `sqlite3.IntegrityError`'u 409'a cevirin.

- Mevcut:
  `"user_email": user["email"]`
- Onerilen:
  public listelerde email yerine maskeli ad/username dondurun.

## Test checklist

1. Kimliksiz yorum: 401.
2. Restoran rolunde yorum: 403.
3. Kurye rolunde yorum: 403.
4. Musteri onayli siparis yok: 403.
5. Onayli siparis var: 201.
6. Rating 0: 422.
7. Rating 6: 422.
8. Rating 1 ve 5 gecerli olmali.
9. Bos yorum: 400.
10. Sadece bosluk yorum: 400.
11. Olmayan restoran: 404.
12. Ayni restorana ikinci yorum: 409.
13. Basarili yorum sonrasi restoran ortalama puan guncellenmeli.
14. Yorum metnindeki on/off character korunmali.
15. Cok uzun yorumlar belirlenen sinira gore reddedilmeli.
16. Ayni kullanici paralel iki yorum istegi gonderirse yalniz biri 201 olmali.
17. Baskasinin restoran id'sine yorum yapmaya calisan musteri onayli siparisi yoksa 403.
18. Yorum sonrasi `has_review=true` olmali.
19. Yorum sonrasi listelemede yeni yorum ve guncel rating gorunmeli.
20. Public yorum listesinde hassas bilgiler adres/telefon gibi yer almayali.