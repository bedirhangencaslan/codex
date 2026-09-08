# schema.sql incelemesi

## Purpose

Bu dosya SQLite veritabaninin tam semasini tanimlar: restoran, kullanici, auth token, kurye, restoran yorumu, menu urunu, siparis ve siparis satiri tablolari. `seed.py` bu dosyayi okuyup tum tablolari olusturur; backend kodu tum CRUD islemlerini bu semaya baglidir. Dolayisiyla kolon tipleri, UNIQUE/CHECK kuralilari ve foreign key tanimlari uygulamanin butunlugu icin referans noktasidir.

## Walkthrough

### restaurant tablosu
`CREATE TABLE IF NOT EXISTS restaurant` 1-11 satirlarindadir. Kolonlar `id`, `name`, `cuisine`, `rating`, `delivery_fee`, `min_order`, `eta_minutes`, `image_emoji`, `user_id`. Kontrol akisi yoktur; tanim sabit. `id` otomatik artan primary key. `name`, `cuisine`, `rating`, `delivery_fee`, `min_order`, `eta_minutes`, `image_emoji` NOT NULL. `user_id` `user(id)` referansi, ama nullable. Yani restoran profili sahipsiz olabilir. Para alanlari `REAL`, yani floating point.

### user tablosu
`CREATE TABLE IF NOT EXISTS "user"` 13-19 satirlarindadir. `id` otomatik primary key. `email` NOT NULL UNIQUE. `password_hash` NOT NULL. `role` CHECK ile yalniz `customer`, `restaurant`, `courier` olabilir. `balance` `REAL NOT NULL DEFAULT 0`. Parola hash tek string olarak saklanir; token, email dogrulama veya password reset tablosu yok.

### auth_token tablosu
`CREATE TABLE IF NOT EXISTS auth_token` 21-25 satirlarindadir. `token` primary key. `user_id` NOT NULL ve `user(id)` referansi. `created_at` NOT NULL. Su satiral `expires_at`, `revoked_at` veya son kullanma yok. Bu, token suresizlik riskini semada gomulu yapar.

### courier tablosu
`CREATE TABLE IF NOT EXISTS courier` 27-32 satirlarindadir. `id` otomatik primary key. `user_id` NOT NULL UNIQUE ve `user(id)` referansi, yani bir kullanici yalniz bir kurye profili olabilir. `restaurant_id` NOT NULL ve `restaurant(id)` referansi. `name` NOT NULL. Kurye bir restorana baglidir; birden fazla restorana ayni anda baglanamaz.

### restaurant_review tablosu
`CREATE TABLE IF NOT EXISTS restaurant_review` 34-42 satirlarindadir. `id` primary key. `restaurant_id` ve `user_id` NOT NULL ve foreign key. `rating` CHECK 1-5. `comment` NOT NULL. `created_at` NOT NULL. `(restaurant_id, user_id)` UNIQUE, yani bir kullanici restoran basina bir yorum yapabilir.

### menu_item tablosu
`CREATE TABLE IF NOT EXISTS menu_item` 44-51 satirlarindadir. `id` primary key. `restaurant_id` NOT NULL FK. `name`, `description`, `price`, `category` NOT NULL. Fiyat `REAL`, pozitiflik CHECK yok; backend katmaninda enforced. Stok, durum veya gorsel yok.

### order tablosu
`CREATE TABLE IF NOT EXISTS "order"` 53-67 satirlarindadir. `id` primary key. `restaurant_id` NOT NULL FK. `address`, `phone`, `note` metinleri. `note` default `''`. `total` `REAL NOT NULL`. `status` TEXT ve default `hazÄ±rlanÄ±yor`. `created_at` NOT NULL. `customer_id` FK, nullable. `courier_id` FK, nullable. `courier_assigned_at`, `delivered_at`, `confirmed_at` nullable. Status CHECK kuralilari yok; degiskenleri tamamen uygulama kontrol eder.

### order_item tablosu
`CREATE TABLE IF NOT EXISTS order_item` 69-76 satirlarindadir. `id` primary key. `order_id` NOT NULL FK. `menu_item_id` NOT NULL FK. `name`, `unit_price` ve `quantity` NOT NULL. Fiyat/quantity CHECK yok; backend uzerinde yapilir. Siparis oluşturulduktan sonra menu fiyat degisirse satirdeki `unit_price` gecmişi korur.

## Data

- Tablo tanimlari: `restaurant` 1-11, `user` 13-19, `auth_token` 21-25, `courier` 27-32, `restaurant_review` 34-42, `menu_item` 44-51, `order` 53-67, `order_item` 69-76.
- HTTP endpoint yok.
- Client state yok.
- Iliskiler: `restaurant.user_id -> user.id`, `auth_token.user_id -> user.id`, `courier.user_id -> user.id`, `courier.restaurant_id -> restaurant.id`, `restaurant_review.restaurant_id -> restaurant.id`, `restaurant_review.user_id -> user.id`, `menu_item.restaurant_id -> restaurant.id`, `order.restaurant_id -> restaurant.id`, `order.customer_id -> user.id`, `order.courier_id -> courier.id`, `order_item.order_id -> order.id`, `order_item.menu_item_id -> menu_item.id`.

## Failure modes

- Default siparis status `"hazÄ±rlanÄ±yor"` mojibake 60. satirda; `panels.py` dogru `"hazırlanıyor"` kullanir. Bu uyumsuzluk kurye atamasini engeller.
- Foreign key iliskileri tanimli ama `db.py` `PRAGMA foreign_keys = ON` acmadigindan uygulama icinde efektif olmayabilir.
- Para alanlari `REAL`: `restaurant.rating/delivery_fee/min_order`, `user.balance`, `menu_item.price`, `order.total`, `order_item.unit_price`. Yuvarlama ve floating point hatalari riskli.
- Token tablosunda expiry yok; sifre yerine DB'de sonsuza kadar gecerli token kalabilir (21-25).
- `order.status` serbest TEXT, CHECK kurali yok; uygulama hatasi ile gecersiz status yazilabilir (60).
- `order.address` ve `phone` uzunluk/format kontrolu yok (56-57).
- `restaurant.user_id` nullable oldugu icin sahipsiz restoran olusabilir (10).
- `courier.restaurant_id` birden fazla restoran iliskisi desteklemez; gercek dunyada kurye parki modeli gerekirse yetersiz kalir (29-30).
- Yorum UNIQUE kurali iyi; ancak schema seviyesinde yorum metni uzunluk siniri yok.
- `order_item.quantity` CHECK yok; backend hatasi ile 0/negatif kayit yazilabilir (75).

## Security

- Sifre hash'i tek string `user.password_hash` (16). Algoritma/iterasyon metadata icin ayri kolon yok.
- Token `token TEXT PRIMARY KEY` (22); token'in veritabaninda indeksli oldugunu garantiler, ama expiry/revocation yok.
- `role` CHECK kurali rol enumlarini kisitlar (17).
- Sensitive adres/telefon `order` tablosunda raw saklanir (56-57); erisim kontrolu tamamen API katmaninda yapilmali.
- Yorumlar user id'ye bagli (37); anonim yorum yok.
- Foreign key tanimlari injection'i degil, veri butunlugu iliskisini ilgilendirir.

## Suggested changes

- Mevcut:
  `status TEXT NOT NULL DEFAULT 'hazÄ±rlanÄ±yor'`
- Onerilen:
  `status TEXT NOT NULL DEFAULT 'hazırlanıyor' CHECK (status IN ('hazırlanıyor', 'kuryede', 'teslim_edildi', 'onaylandı'))`

- Mevcut:
  `balance REAL NOT NULL DEFAULT 0`
- Onerilen:
  `balance INTEGER NOT NULL DEFAULT 0` (kurus cinsi) veya SQLite 3.31+ icin CHECK + DECIMAL-emulasyon politikasi.

- Mevcut:
  `created_at TEXT NOT NULL`
- Onerilen:
  `created_at TEXT NOT NULL, expires_at TEXT NOT NULL, revoked_at TEXT NULL` token tablosunda.

- Mevcut:
  `quantity INTEGER NOT NULL`
- Onerilen:
  `quantity INTEGER NOT NULL CHECK (quantity > 0)`

## Test checklist

1. `restaurant` tablosu olusmali ve id otomatik artmali.
2. Ayni e-postayla ikinci user eklenmemeli.
3. Gecersiz role insert'i CHECK ile engellenmeli.
4. `auth_token` ayni token degeriyle ikinci kayit kabul etmemeli.
5. Foreign key ON iken olmayan user_id ile token insert'i engellenmeli.
6. Kurye user_id UNIQUE olmali; ayni kullanici ikinci kurye profili olusturamamali.
7. Rating 0 insert'i CHECK ile engellenmeli.
8. Rating 6 insert'i CHECK ile engellenmeli.
9. Ayni user ayni restorana ikinci yorum ekleyememeli.
10. Menu item fiyat negatif olabilir; fix sonrasi CHECK engellemeli.
11. Order status default degeri mojibake olmamali.
12. Order address/phone bos kayit yazilabiliyor mu test edilmeli.
13. Order_item quantity 0 yazilabiliyor mu test edilmeli.
14. Restoran user_id NULL ile olusturulabiliyor mu test edilmeli.
15. Siparis silinince order_item FK davranisi foreign key ON/OFF ayri test edilmeli.
16. Kurye silinince order.courier_id FK davranisi test edilmeli.
17. Para alanlari 0.1 toplamlarinda yuvarlama hatasi uretip uretmedigi test edilmeli.
18. Schema seed sonrasi tablo sayisi ve kolon tipleri dogrulanmali.