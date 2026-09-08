# main.py incelemesi

## Purpose

Bu dosya FastAPI uygulamasinin ana moduludur. Router'lari ekler, restoran listeleme/detay/mutfak endpoint'lerini ve musteri siparis olusturma endpoint'ini tanimlar. `GET /api/orders/{order_id}` de buradadir ve su an kimlik/ownership kontrolu olmadan siparis detayini dondurdugu icin en kritik guvenlik noktalarindan biridir. Restoran listesi ve detay dogrudan arayuzun ana akisini, siparis olusturma ise bakiye ve sepet akisini tasir.

## Walkthrough

### Uygulama kurulumu
`app = FastAPI(title="Sepet API")` 13. satirda tanimlanir. 15-17 satirlarinda `accounts_router`, `panels_router` ve `reviews_router` uygulamaya eklenir. B Boylece hesap, panel ve yorum endpoint'leri ana uygulamada kullanilabilir olur.

### OrderItemInput
`class OrderItemInput(BaseModel)` 20-22 satirlarindadir. `menu_item_id: int` ve `quantity: int = Field(gt=0)` alir. Donus degeri yoktur. Pydantic negatif veya sifir adedi reddeder. Ayni menu urununun birden fazla satir olarak tekrar edilmesini engellemez.

### OrderInput
`class OrderInput(BaseModel)` 25-30 satirlarindadir. `restaurant_id: int`, `address: str`, `phone: str`, `note: str = ""` ve `items: list[OrderItemInput]` alir. Bos liste burada engellenmez; endpoint icinde 144-145 satirlarinda kontrol edilir. Adres, telefon ve not icin uzunluk veya format dogrulamasi yoktur.

### list_restaurants
`@app.get("/api/restaurants")` ile suslenmis `def list_restaurants(...)` 33-70 satirlarindadir. Parametreler arama terimi `q`, mutfak `cuisine`, minimum puan, maksimum teslimat ucreti, maksimum teslim suresi ve siralama anahtari alir. Donus restoran dict'lerinin listesidir. Kod koşullu SQL WHERE bolumu kurar; arama hem ad hem mutfak alaninda LIKE ile yapilir. Sonra `restaurant_review` tablosundan mutfak bazli ortalama puan ceker, her restorana `displayed_rating` uygular. Filtreler Python tarafinda uygulanir (63). Siralama dictionary ile desteklenen anahtarlara gore yapilir; tanimsiz `sort` degeri varsayilan puana döner.

### list_cuisines
`@app.get("/api/cuisines")` ile suslenmis `def list_cuisines() -> list[str]` 73-79 satirlarindadir. Parametre almaz, mutfak string listesi dondurur. `SELECT DISTINCT cuisine FROM restaurant ORDER BY cuisine` calistirir. Bos tablo durumunda bos liste dondurur.

### get_restaurant
`@app.get("/api/restaurants/{restaurant_id}")` ile suslenmis `def get_restaurant(...)` 82-116 satirlarindadir. Yol parametresi olarak `restaurant_id` alir, opsiyonel token ile `user` alir. Donus degeri restoran, menu, yorumlar ve puan bilgisi iceren dict'tir. Restoran yoksa 404 dondurur. Menu urunleri kategori ve ada gore siralanir, yorumlar kullanici e-postasiyla birlikte desc siralanir. `can_review` ve `has_review` default `False` olur. Eger giris yapan musteri varsa, status literal `onaylandÄ±` kullanilarak onayli siparis kontrol edilir ve yorum var mi diye sorgulanir. Bu status literal schema ve paneldeki dogru Unicode ile uyumsuz olabilir.

### get_order
`@app.get("/api/orders/{order_id}")` ile suslenmis `def get_order(order_id: int) -> dict[str, Any]` 119-139 satirlarindadir. Sadece yol id alir; kimlik dogrulama veya ownership guard yoktur. Siparis yoksa 404 dondurur. Siparis ve restoran adini JOIN ile getirir, sonra urunleri `order_item` tablosundan ceker. Donus siparis tum kolonlarini, restoran adini ve urun listesini icerir. Bu, adres/telefon/not ve musteri id gibi kisisel bilgilerin id bilen herkese acilmasi demektir.

### create_order
`@app.post("/api/orders", status_code=201)` ile suslenmis `def create_order(...)` 142-214 satirlarindadir. Parametre olarak `OrderInput` ve `require_customer` ile dogrulanmis `user` alir. Donus id, status ve total icerir. Once bos item listesi 400 ile reddedilir. Restoran yoksa 404 dondurur. Menu urunleri restoran id'siyle filtrelenerek tek sorguda getirilir; eksik veya baska restorana ait urun varsa 404 dondurur. Ara toplam hesaplanir ve minimum sepet kontrol edilir. Toplam teslimat ucretiyle toplanir, bakiye yetersizse 400 dondurur. Bakiye dusurme `UPDATE ... WHERE id = ? AND balance >= ?` ile kosullu yapilir; dusmuyorsa 400 dondurur. Siparis `hazÄ±rlanÄ±yor` statusu ile eklenir ve urunler `order_item` tablosuna yazilir. Blok cikisinda commit olur; istisna olursa rollback beklenir.

## Data

- `restaurant` okunur: liste 44-54, detay 85-87, siparis 150-152.
- `restaurant_review` okunur: liste ortalama 51-53, detay yorumlari 94-99, yorum yapma durumu 112-115.
- `menu_item` okunur: detay 90-93, siparis 157-161.
- `order` okunur: detay 122-128, yorum yapma uygunlugu 107-111; yazilir: siparis ekleme 189-203.
- `order_item` okunur/yazilir: detay 131-136, siparis urunleri 207-212.
- `user` okunur/yazilir: musteri bakiyesi token uzerinden gelir, bakiye dusulur 181-183.
- HTTP endpoint'leri: `GET /api/restaurants`, `GET /api/cuisines`, `GET /api/restaurants/{id}`, `GET /api/orders/{id}`, `POST /api/orders`.

## Failure modes

- Siparis detay endpoint'i kimliksiz acik ve ownership yok (119-139); id bilen herkes kisisel bilgileri gorebilir.
- `main.py` ve `schema.sql` bozuk Unicode status literal kullanirken `panels.py` dogru Unicode kullanir (109, 199, schema 60 vs panels 174, 238). Restart/reseed sonrasi kurye atamasi tikanabilir.
- `create_order` yalnizca `require_customer` guard'iyla islem görür, ancak 146-147 satirindaki rol kontrolu redundant ve `user` None durumunu tam kapsamaz.
- Arama `LIKE` wildcard karakterlerini kacirmaz; `%` veya `_` iceren terim istenmeden genis eslesme yapar (37-40).
- SQL f-string sadece placeholders sayisi icin kullanilir; degisken dogrudan gecirilmez, ama dinamik query yapisinda dikkat gerekir (156-160).
- Restoran listesindeki filtreleme veritabaninda degil Python'da yapilir; buyuk veri setinde performans dususebilir (63).
- Minimum sepet kontrolu subtotal uzerinden yapilir; quantity toplami veya 0 fiyat urunleri ozel durum yaratmaz ama float hassasiyeti olabilir (166-172).
- Bakiye dusme ve siparis insert'i ayni transaction'dadir; hata olursa geri alinir, ancak mantik hatalari commit olabilir (181-212).

## Security

- Kimlik dogrulama: restoran listesi ve mutfak listesi halka acik (33-79). Restoran detay opsiyonel token kullanir (82-116).
- Yetkilendirme: siparis olusturma sadece musteriler icin `require_customer` ile (142-143).
- Siparis detayda auth ve ownership yok; IDOR acik (119-139).
- SQL injection riski dusuk: parametreli sorgular kullanilir (37-48, 85-99, 122-136, 157-160, 181-183, 207-212).
- Veri ifsasi: siparis detay `o.*` ile adres, telefon, not, musteri id, courier id ve zaman damgalarini dondurur (123, 137-139).
- Bakiye dusurme kosullu WHERE ile yapilir; race condition'da cift dusme riski azaltilir (181-184).

## Suggested changes

- Mevcut:
  `def get_order(order_id: int) -> dict[str, Any]:`
- Onerilen:
  `def get_order(order_id: int, user=Depends(get_current_user)) -> dict[str, Any]:` ve `order.customer_id != user["id"]` durumunda 404 dondurun.

- Mevcut:
  `"hazÄ±rlanÄ±yor"`
- Onerilen:
  `"hazırlanıyor"` hem schema, hem main.py, hem panels.py ayni literal olacak sekilde.

- Mevcut:
  `search = f"%{q.lower()}%"`
- Onerilen:
  `safe_q = q.replace("\\", "\\\\").replace("%", "\\%").replace("_", "\\_"); search = f"%{safe_q.lower()}%"` ve SQL'e uygun `ESCAPE` clause ekleyin.

- Mevcut:
  `if user and user["role"] != "customer":`
- Onerilen:
  `require_customer` guard'i yeterli oldugundan bu satiri kaldirin veya user tipini netlestirin.

## Test checklist

1. Restoran listesi bos DB'de bos donmeli.
2. Q parametresi ada gore eslesmeli.
3. Q parametresi mutfak adiyla eslesmeli.
4. Cuisine filtresi tek restoran donmeli.
5. Minimum rating filtresi dusuk puanli restoranlari elemeli.
6. Maksimum teslimat ucreti ve sure filtresi birlikte calismali.
7. Gecersiz sort degeri varsayilan puana dönmeli.
8. Mutfak listesi DISTINCT ve alfabetik donmeli.
9. Olmayan restoran id detayi 404 donmeli.
10. Girisli musteri onayli siparisi varsa `can_review=true` olmali.
11. Girisli musteri yorum yaptiysa `has_review=true` olmali.
12. Siparis detayi kimliksiz istekte 401 olmali (guncellemeden sonra).
13. Baskasinin siparisini girisli musteri isteyince 404 olmali.
14. Siparis olusturma bos sepetle 400 olmali.
15. Menu urunu baska restorandan secildiginde 404 olmali.
16. Minimum sepet altinda 400 olmali.
17. Bakiye yetersizse 400 olmali ve bakiye degismemeli.
18. Basarili siparis sonrasi bakiye tam toplam kadar dusmeli.
19. Ayni menu urunu birden fazla satirda gelirse toplam dogru hesaplanmali.
20. Olusan siparis status'u panels.py'daki dogru literal ile eslesmeli.