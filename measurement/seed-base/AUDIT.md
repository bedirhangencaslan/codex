# Sepet Uçtan Uca Denetimi

Bu denetim `sepet/` altındaki Python, SQL ve web frontend kaynaklarını okuyarak hazırlanmıştır. Derleme çıktısı olan `web/dist` ve ikili `api/sepet.db` davranışın kaynağı olarak kullanılmamıştır; veri modeli `schema.sql` üzerinden doğrulanmıştır. Raporun tamamı kaynak satırlarına dayanır.

## 1. Modül haritası

| Modül | Sorumluluklar | Okuduğu tablolar | Yazdığı tablolar |
|---|---|---|---|
| [db.py](sepet/api/db.py:11) | SQLite bağlantısı açar, UTC zaman üretir, PBKDF2 ile parola hash'ler ve doğrular. | Yok (yalnızca bağlantı ve yardımcılar) | Yok (doğrudan yazım yok) |
| [auth.py](sepet/api/auth.py:6) | `Authorization` header'ından token ile kullanıcı çözer; customer/restaurant/courier rol kurallarını uygular. | `auth_token`, `user`, `restaurant`, `courier` | Yok |
| [accounts.py](sepet/api/accounts.py:11) | Kayıt, giriş, `/me` ve müşteri cüzdan yükleme uçlarını sunar; restoran sahibi kaydında restoran profilini de yaratır. | `user` | `user`, `auth_token`, `restaurant` |
| [main.py](sepet/api/main.py:13) | FastAPI uygulamasını kurar; restoran listesi/detayı ve müşteri siparişi oluşturma uçlarını sunar. Genel sipariş detay ucunu da tanımlar. | `restaurant`, `restaurant_review`, `menu_item`, `order`, `order_item`, `user` | `user`, `order`, `order_item` |
| [panels.py](sepet/api/panels.py:10) | Müşteri geçmişi, restoran sipariş/menü/kurye yönetimi, kurye işlemleri, teslim onayı ve restoran kazanç uçlarını sunar. | `order`, `order_item`, `restaurant`, `user`, `courier`, `menu_item` | `user`, `courier`, `menu_item`, `order` |
| [reviews.py](sepet/api/reviews.py:10) | Onaylanmış siparişe göre restoran yorumu oluşturur; tohum puanı ile yorum ortalamasını hesaplar. | `restaurant`, `order`, `restaurant_review` | `restaurant_review` |
| [schema.sql](sepet/api/schema.sql:1) | Sekiz SQLite tablosunu ve anahtar ilişkilerini kurar. | Uygulanmaz | Şemayı yaratır |
| [seed.py](sepet/api/seed.py:162) | Veritabanını siler, şemayı uygular, restoranları/menüleri ve demo kullanıcı-kuryeleri oluşturur. | `restaurant` (kurulum sonrası id arama), `schema.sql` içeriği | `restaurant`, `menu_item`, `user`, `courier` |

## 2. Eşleşme (coupling) haritası

Backend içi doğrudan bağımlılıklar:

- [accounts.py:7](sepet/api/accounts.py:7) `auth.get_current_user`'ı kullanır; [accounts.py:8](sepet/api/accounts.py:8) `db.get_db`, `hash_password`, `utc_now`, `verify_password` fonksiyonlarını kullanır.
- [auth.py:3](sepet/api/auth.py:3) `db.get_db`'ye bağımlıdır; token doğrulama [auth.py:18](sepet/api/auth.py:18) içinde bu bağlantıyı kullanır.
- [main.py:7](sepet/api/main.py:7) router kaydı için `accounts` modülüne bağlıdır.
- [main.py:8](sepet/api/main.py:8) `auth.get_optional_user` ve `require_customer` bağımlılıklarını kullanır.
- [main.py:9](sepet/api/main.py:9) tüm doğrudan sorgular için `db.get_db`'ye bağımlıdır.
- [main.py:10](sepet/api/main.py:10) router kaydı için `panels` modülüne bağlıdır.
- [main.py:11](sepet/api/main.py:11) router kaydı ve ortalama puan hesabı için `reviews.displayed_rating`'e bağlıdır.
- [panels.py:6](sepet/api/panels.py:6) üç rol kısıtlama fonksiyonu için `auth` modülüne bağlıdır.
- [panels.py:7](sepet/api/panels.py:7) `db.get_db`, `hash_password`, `utc_now` fonksiyonlarına bağlıdır.
- [reviews.py:5](sepet/api/reviews.py:5) opsiyonel/zorunlu kullanıcı bağımlılıkları için `auth` modülünü kullanır.
- [reviews.py:6](sepet/api/reviews.py:6) `db.get_db` ve `utc_now` fonksiyonlarına bağlıdır.
- [reviews.py:7](sepet/api/reviews.py:7) istek gövdesini `panels.ReviewInput`'tan alır; bu, panels modülünün istek sözleşmesini başka bir router'a taşır.
- [seed.py:4](sepet/api/seed.py:4) parola hash'i ve zaman üretimi için `db` modülüne bağlıdır.
- [schema.sql](sepet/api/schema.sql:1) ile tüm backend sorguları arasında şema sözleşmesi vardır; özellikle sipariş durumu literal'leri [schema.sql:60](sepet/api/schema.sql:60), [main.py:199](sepet/api/main.py:199) ve [panels.py:174](sepet/api/panels.py:174) arasında çelişir.

Frontend-API sözleşme bağımlılıkları:

- [api.ts:3](sepet/web/src/api.ts:3) genel `request` yardımcısı tüm sayfalardan tek fetch/Authorization hattını kullanır.
- [api.ts:49](sepet/web/src/api.ts:49), [api.ts:83](sepet/web/src/api.ts:83), [api.ts:87-108](sepet/web/src/api.ts:87) ve [api.ts:123](sepet/web/src/api.ts:123) sırasıyla sipariş, geçmiş, restoran/kurye paneli ve kazanç endpoint sözleşmelerine bağlanır.
- [CustomerPanel.tsx:8-11](sepet/web/src/panels/CustomerPanel.tsx:8) ve [RestaurantPanel.tsx:11-14](sepet/web/src/panels/RestaurantPanel.tsx:11) doğru Unicode durum değerlerini bekler; backend'in bozuk literal'leriyle çakışır.

## 3. HTTP istek yaşam döngüsü

Aşağıdaki listede her uçta rota fonksiyonu, kimlik doğrulama fonksiyonları ve DB yolu gösterilmektedir. `get_current_user` ile başlayan her akış önce [auth.py:18](sepet/api/auth.py:18) `resolve_user` -> [auth.py:6](sepet/api/auth.py:6) `_user_from_token` -> [auth.py:23](sepet/api/auth.py:23) `get_current_user` yolunu izler.

| Uç | Rota fonksiyonu ve fonksiyon zinciri | DB erişimi |
|---|---|---|
| `POST /api/auth/register` | `register` ([accounts.py:59](sepet/api/accounts.py:59)) -> `validate_registration` -> `validate_email` -> `get_db` -> `issue_token` | `user` içinde e-posta kontrolü ve kullanıcı ekleme; restoran rolünde `restaurant` ekleme; `auth_token` ekleme ([accounts.py:62-85](sepet/api/accounts.py:62)) |
| `POST /api/auth/login` | `login` ([accounts.py:89](sepet/api/accounts.py:89)) -> `get_db` -> `verify_password` -> `issue_token` -> `user_payload` | `user` arar, `auth_token` yazar ([accounts.py:91-98](sepet/api/accounts.py:91)) |
| `GET /api/auth/me` | `me` ([accounts.py:101](sepet/api/accounts.py:101)) -> `get_current_user` -> `resolve_user` -> `_user_from_token` -> `user_payload` | `auth_token` ve `user` okuma ([auth.py:10-15](sepet/api/auth.py:10)) |
| `POST /api/auth/wallet/topup` | `top_up_wallet` ([accounts.py:110](sepet/api/accounts.py:110)) -> `get_current_user` -> `get_db` | `user` bakiyesini güncelleyip tekrar okur ([accounts.py:115-122](sepet/api/accounts.py:115)) |
| `GET /api/restaurants` | `list_restaurants` ([main.py:33](sepet/api/main.py:33)) -> `get_db` -> `displayed_rating` | `restaurant` ve `restaurant_review` okuma ([main.py:47-53](sepet/api/main.py:47)) |
| `GET /api/cuisines` | `list_cuisines` ([main.py:73](sepet/api/main.py:73)) -> `get_db` | `restaurant` içinde distinct `cuisine` okuma ([main.py:75-78](sepet/api/main.py:75)) |
| `GET /api/restaurants/{restaurant_id}` | `get_restaurant` ([main.py:82](sepet/api/main.py:82)) -> `get_optional_user` -> `resolve_user` -> `_user_from_token` -> `get_db` -> `displayed_rating` | `restaurant`, `menu_item`, `restaurant_review` + `user` okuma; müşteriyse `order` ve `restaurant_review` yetenek kontrolü ([main.py:84-115](sepet/api/main.py:84)) |
| `GET /api/orders/{order_id}` | `get_order` ([main.py:119](sepet/api/main.py:119)) -> `get_db` | `order` + `restaurant` ve `order_item` okuma; kimlik/sahiplik fonksiyonu yok ([main.py:121-138](sepet/api/main.py:121)) |
| `POST /api/orders` | `create_order` ([main.py:142](sepet/api/main.py:142)) -> `require_customer` -> `get_current_user` -> `get_db` | `restaurant`, `menu_item`, `user` okuma; `user` bakiye güncelleme; `order` ve `order_item` ekleme ([main.py:149-212](sepet/api/main.py:149)) |
| `GET /api/orders/history` | `order_history` ([panels.py:68](sepet/api/panels.py:68)) -> `require_customer` -> `get_db` -> `serialize_orders` | `order` + `restaurant`/`user`/`courier` ve `order_item` okuma ([panels.py:70-81](sepet/api/panels.py:70)) |
| `GET /api/restaurant/orders` | `restaurant_orders` ([panels.py:84](sepet/api/panels.py:84)) -> `require_restaurant` -> `get_current_user` -> `get_db` -> `serialize_orders` | `restaurant` profil okuma ([auth.py:42-45](sepet/api/auth.py:42)); sonra `order`, `restaurant`, `user`, `courier`, `order_item` okuma ([panels.py:101-103](sepet/api/panels.py:101)) |
| `GET /api/restaurant/couriers` | `list_couriers` ([panels.py:106](sepet/api/panels.py:106)) -> `require_restaurant` -> `get_db` | `restaurant` ve `courier` okuma ([panels.py:109-113](sepet/api/panels.py:109)) |
| `GET /api/restaurant/menu-items` | `list_menu_items` ([panels.py:117](sepet/api/panels.py:117)) -> `require_restaurant` -> `get_db` | `restaurant` ve `menu_item` okuma ([panels.py:120-124](sepet/api/panels.py:120)) |
| `POST /api/restaurant/couriers` | `create_courier` ([panels.py:128](sepet/api/panels.py:128)) -> `require_restaurant` -> `get_db` -> `hash_password` | `user` e-posta kontrolü ve ekleme, `courier` ekleme ([panels.py:133-143](sepet/api/panels.py:133)) |
| `POST /api/restaurant/menu-items` | `add_menu_item` ([panels.py:148](sepet/api/panels.py:148)) -> `require_restaurant` -> `get_db` | `restaurant` profil okuma, `menu_item` ekleme ([panels.py:151-156](sepet/api/panels.py:151)) |
| `POST /api/restaurant/orders/{order_id}/assign-courier` | `assign_courier` ([panels.py:160](sepet/api/panels.py:160)) -> `require_restaurant` -> `get_db` | `order` ve `courier` okuma, `order` güncelleme ([panels.py:168-185](sepet/api/panels.py:168)) |
| `GET /api/courier/orders` | `courier_orders` ([panels.py:190](sepet/api/panels.py:190)) -> `require_courier` -> `get_current_user` -> `get_db` -> `serialize_orders` | `courier` profil okuma ([auth.py:54-57](sepet/api/auth.py:54)); sonra `order`, `restaurant`, `user`, `courier`, `order_item` okuma ([panels.py:193-204](sepet/api/panels.py:193)) |
| `POST /api/courier/orders/{order_id}/deliver` | `deliver_order` ([panels.py:207](sepet/api/panels.py:207)) -> `require_courier` -> `get_db` | `courier` profil okuma, `order` okuma/güncelleme ([panels.py:211-220](sepet/api/panels.py:211)) |
| `POST /api/orders/{order_id}/confirm_delivery` | `confirm_delivery` ([panels.py:226](sepet/api/panels.py:226)) -> `require_customer` -> `get_db` | `order` okuma/güncelleme ([panels.py:229-239](sepet/api/panels.py:229)) |
| `GET /api/restaurant/earnings` | `restaurant_earnings` ([panels.py:247](sepet/api/panels.py:247)) -> `require_restaurant` -> `get_db` | `restaurant` profil okuma, `order` toplamları okuma ([panels.py:250-263](sepet/api/panels.py:250)) |
| `POST /api/restaurants/{restaurant_id}/reviews` | `create_review` ([reviews.py:18](sepet/api/reviews.py:18)) -> `require_customer` -> `get_db` -> `utc_now` | `restaurant`, `order`, `restaurant_review` okuma; `restaurant_review` ekleme ([reviews.py:26-49](sepet/api/reviews.py:26)) |

## 4. Veri modeli ve endpoint erişimi

| Tablo | Sütunlar | Veri yazan/okuyan uçlar |
|---|---|---|
| `restaurant` | `id`, `name`, `cuisine`, `rating`, `delivery_fee`, `min_order`, `eta_minutes`, `image_emoji`, `user_id` ([schema.sql:1-10](sepet/api/schema.sql:1)) | Yazar: `POST /api/auth/register` ([accounts.py:71-83](sepet/api/accounts.py:71)), seed ([seed.py:167-181](sepet/api/seed.py:167)). Okur: restoran listesi/detayı, cuisines, sipariş oluşturma, panels, yorum ve auth rol bağımlılıkları. |
| `user` | `id`, `email`, `password_hash`, `role`, `balance` ([schema.sql:13-18](sepet/api/schema.sql:13)) | Yazar: kayıt, cüzdan yükleme, sipariş bakiyesi düşme, kurye oluşturma, seed. Okur: giriş, `/me`, token çözümü, rol bağımlılıkları, sipariş/detay/join'leri. |
| `auth_token` | `token`, `user_id`, `created_at` ([schema.sql:21-24](sepet/api/schema.sql:21)) | Yazar: `register`, `login` ([accounts.py:50-56](sepet/api/accounts.py:50)). Okur: token'a bağlı her auth uç/fonksiyon ([auth.py:10-15](sepet/api/auth.py:10)). |
| `courier` | `id`, `user_id`, `restaurant_id`, `name` ([schema.sql:27-31](sepet/api/schema.sql:27)) | Yazar: `POST /api/restaurant/couriers` ([panels.py:141-143](sepet/api/panels.py:141)), seed. Okur: courier/restaurant rol bağımlılıkları, kurye listesi, kurye atama, sipariş join'leri. |
| `restaurant_review` | `id`, `restaurant_id`, `user_id`, `rating`, `comment`, `created_at`; `restaurant_id,user_id` unique ([schema.sql:34-41](sepet/api/schema.sql:34)) | Yazar: `POST /api/restaurants/{id}/reviews` ([reviews.py:44-49](sepet/api/reviews.py:44)). Okur: yorum oluşturma kontrolü, restoran listesi/detayı. |
| `menu_item` | `id`, `restaurant_id`, `name`, `description`, `price`, `category` ([schema.sql:44-50](sepet/api/schema.sql:44)) | Yazar: `POST /api/restaurant/menu-items` ([panels.py:151-156](sepet/api/panels.py:151)), seed. Okur: restoran detayı, sipariş oluşturma, restoran menü listesi. |
| `order` | `id`, `restaurant_id`, `address`, `phone`, `note`, `total`, `status`, `created_at`, `customer_id`, `courier_id`, `courier_assigned_at`, `delivered_at`, `confirmed_at` ([schema.sql:53-66](sepet/api/schema.sql:53)) | Yazar: `POST /api/orders` ([main.py:189-202](sepet/api/main.py:189)), kurye atama/teslim/müşteri onayı ([panels.py:182-184](sepet/api/panels.py:182), [panels.py:219-221](sepet/api/panels.py:219), [panels.py:237-239](sepet/api/panels.py:237)). Okur: sipariş detayı/geçmişi, restoran/kurye paneli, yorum uygunluğu, kazanç toplamları. |
| `order_item` | `id`, `order_id`, `menu_item_id`, `name`, `unit_price`, `quantity` ([schema.sql:69-75](sepet/api/schema.sql:69)) | Yazar: `POST /api/orders` ([main.py:205-212](sepet/api/main.py:205)). Okur: tüm sipariş listeleme/detay uçları ([panels.py:35-48](sepet/api/panels.py:35), [main.py:131-136](sepet/api/main.py:131)). |

## 5. En ciddi on düzeltme önceliği

1. **Sipariş detayı kimliksiz ve sahipsiz.** `GET /api/orders/{order_id}` herhangi bir auth bağımlılığı almadan adres, telefon ve notu döndürür ([main.py:119-139](sepet/api/main.py:119)); frontend bu veriyi ekranda basar ([OrderPage.tsx:37-38](sepet/web/src/pages/OrderPage.tsx:37)). Düzeltme: `user=Depends(get_current_user)` ekleyin; müşteriyse `order.customer_id`, kuryeyse `order.courier_id`, restoran rolünde ise bağlı restoran `restaurant_id` ile eşleşmeyen isteği 403/404 yapın.
2. **Sipariş durumu kodlaması teslimat akışını kırar.** Yeni sipariş `hazÃ„Â±rlanÃ„Â±yor` literal'i ile yazılır ([main.py:199](sepet/api/main.py:199)), şema varsayılanı da bozuktur ([schema.sql:60](sepet/api/schema.sql:60)); atama ise doğru görünen `hazÄ±rlanÄ±yor` değerini bekler ([panels.py:174](sepet/api/panels.py:174)). Restoran paneli de aynı doğru literal ile filtreler ([RestaurantPanel.tsx:105](sepet/web/src/panels/RestaurantPanel.tsx:105)). Düzeltme: `order_status` enum'u veya ASCII değerler (`preparing`, `assigned`, `delivered`, `confirmed`) kullanın, şema varsayılanını ve tüm insert/update/filtreleri tek tanımdan üretin; mevcut veri için migration yazın.
3. **Foreign key bütünlüğü kapalı.** `get_db()` yalnızca bağlantı açar ve `PRAGMA foreign_keys = ON` çalıştırmaz ([db.py:11-14](sepet/api/db.py:11)); şemadaki ilişkiler bu yüzden çalışma zamanında güvence olmaz ([schema.sql:10](sepet/api/schema.sql:10), [schema.sql:23](sepet/api/schema.sql:23)). Düzeltme: `conn.execute("PRAGMA foreign_keys = ON")` ekleyin ve bağlantı açılışında doğrulayın.
4. **Token'lar süresizdir ve logout geçersiz kılmaz.** Login/register her seferinde yeni token yazar ([accounts.py:50-56](sepet/api/accounts.py:50)); doğrulama sadece token eşleşmesine bakar ([auth.py:10-15](sepet/api/auth.py:10)), frontend logout yalnızca localStorage temizler ([AuthContext.tsx:56-59](sepet/web/src/AuthContext.tsx:56)). Düzeltme: `expires_at` ekleyip token doğrulamada süre kontrolü yapın, logout için `DELETE /api/auth/logout` ekleyin ve eski tokenları periyodik silin.
5. **Para `REAL` ve işlem sırasında kayan noktalıdır.** `balance`, `price`, `delivery_fee`, `min_order`, `total` `REAL` ([schema.sql:5-9](sepet/api/schema.sql:5), [schema.sql:18](sepet/api/schema.sql:18), [schema.sql:49](sepet/api/schema.sql:49), [schema.sql:59](sepet/api/schema.sql:59)); toplam float ile hesaplanır ([main.py:166-174](sepet/api/main.py:166)). Düzeltme: para alanlarını integer kuruş veya SQLite `NUMERIC`/`DECIMAL` kurallarıyla saklayın, API katmanında kuruş hesabı yapıp yalnızca görüntüde biçimlendirin.
6. **Restoran kaydı ekonomik alanları güvensizce kabul eder.** `delivery_fee`, `min_order` ve `eta_minutes` için pozitiflik/aralık kısıtı yoktur ([accounts.py:14-22](sepet/api/accounts.py:14)); restoranda yalnızca ad/mutfak kontrol edilir ([accounts.py:40-47](sepet/api/accounts.py:40)). Böylece negatif teslimat/min sipariş değerleri gelebilir. Düzeltme: `Field(ge=...)`, üst sınır ve `eta_minutes` doğrulaması ekleyin; hesap oluşturma için de ayrı bir eşik/rate limit uygulayın.
7. **Kurye hesabı doğrulaması tutarsızdır.** Genel kayıtta e-posta ve minimum parola kontrolü varken ([accounts.py:35-43](sepet/api/accounts.py:35)) restoran paneli kurye hesabında bunları doğrulamaz ve `email/password/name` alanlarını olduğu gibi DB'ye gönderir ([panels.py:128-139](sepet/api/panels.py:128)). Düzeltme: ortak `validate_email` + parola politikası kullanın, ad/şifre boşluk ve uzunluk kontrollerini backend'e taşıyın.
8. **Unique e-posta ihlalleri 500'e dönüşebilir.** Hem register ([accounts.py:62-67](sepet/api/accounts.py:62)) hem kurye oluşturma ([panels.py:133-137](sepet/api/panels.py:133)) önce SELECT sonra INSERT yapar; eşzamanlı iki istek aynı e-postayı görebilir ve `user.email` unique kısıtı ([schema.sql:15](sepet/api/schema.sql:15)) beklenmeyen `IntegrityError` üretir. Düzeltme: INSERT'i tek adımda yapın, `sqlite3.IntegrityError` yakalayıp 409 döndürün veya insert işlemini belirli bir senkronizasyon/sorgu-INSERT tek transaction kuralıyla yönetin.
9. **Kimlik doğrulama katmanı token ve kullanıcı verisini rol fonksiyonlarında tekrar DB'ye açar.** `require_restaurant` ve `require_courier` `get_db` ile ikinci bağlantı açar ([auth.py:42-45](sepet/api/auth.py:42), [auth.py:54-57](sepet/api/auth.py:54)); bu her istekte ek DB bağlantısı ve rol/profil çift kontrolü yaratır. Düzeltme: `resolve_user` sonucunu request state'te tutun, restaurant/courier profilini aynı bağlantıda join ile alın ve bağımlılık fabrikasına taşıyın.
10. **Puanlama modeli yorum sayısıyla dengesiz biçimde tohum puana çeker.** Ortalama her seferinde tohum puanını tek oy gibi dahil eder ([reviews.py:13-15](sepet/api/reviews.py:13)); restoran detayı yorum satırlarını aynen geçirir ([main.py:103](sepet/api/main.py:103)). 100 gerçek oy olsa bile 4.8 tohum puanı sonucu yaklaşık 4.85'e çeker. Düzeltme: ortalama yalnızca `restaurant_review` üzerinden hesaplanmalı; bayat/tohum puanı istenirse ayrı alan veya Bayesian formül belgelenmelidir.

Ek ciddiyet notları: `get_db` bağlantıları çoğu yerde açıkça kapatılmaz ([main.py:47](sepet/api/main.py:47), [panels.py:101-103](sepet/api/panels.py:101)); sipariş sonrası frontend `refreshUser` çağırmaz ([Checkout.tsx:46-49](sepet/web/src/pages/Checkout.tsx:46)), bu nedenle bakiye eski kalır. Bu maddeler yukarıdaki güvenlik/düzeltme sırasıyla birlikte ele alınmalıdır.

## 6. Frontend-API hattı ve çelişkiler

Frontend tüm API erişimini [api.ts](sepet/web/src/api.ts:3) içindeki `request` fonksiyonuna indirger. Token `localStorage`'daki `sepet-auth` değerinden okunur ([api.ts:4-5](sepet/web/src/api.ts:4)) ve `Authorization: Bearer ...` header'ı eklenir ([api.ts:7-10](sepet/web/src/api.ts:7)). Hata yanıtlarındaki `detail` string'i kullanıcıya gösterilir ([api.ts:13-19](sepet/web/src/api.ts:13)). AuthContext kayıt/girişte token ve kullanıcıyı kalıcılaştırır ([AuthContext.tsx:41-54](sepet/web/src/AuthContext.tsx:41)), açılışta `/api/auth/me` ile yeniler ([AuthContext.tsx:29-38](sepet/web/src/AuthContext.tsx:29)) ve `refreshUser` sağlar ([AuthContext.tsx:61-66](sepet/web/src/AuthContext.tsx:61)).

Vite geliştirme sunucusu `/api` isteklerini `http://127.0.0.1:8000`'e proxy'ler ([vite.config.ts:6-9](sepet/web/vite.config.ts:6)). Router üç role göre panel seçer ([Panel.tsx:30-32](sepet/web/src/pages/Panel.tsx:30)); CustomerPanel sipariş geçmişi, cüzdan ve teslim onayı; RestaurantPanel sipariş, menü, kurye ve kazanç uçlarını kullanır ([CustomerPanel.tsx:23-49](sepet/web/src/panels/CustomerPanel.tsx:23), [RestaurantPanel.tsx:25-46](sepet/web/src/panels/RestaurantPanel.tsx:25)).

Öne çıkan çelişkiler ve riskler:

1. **Sipariş durumu çakışması.** Backend bozuk literal yazar ([main.py:199](sepet/api/main.py:199)); CustomerPanel ([CustomerPanel.tsx:7-12](sepet/web/src/panels/CustomerPanel.tsx:7)) ve RestaurantPanel ([RestaurantPanel.tsx:10-15](sepet/web/src/panels/RestaurantPanel.tsx:10)) doğru literal bekler. Bu, restoran ekranında kurye atama alanı görünse bile atama isteği backend'de 409 döner.
2. **Onaylanmış sipariş kontrolü çakışması.** Restoran detayında yorum uygunluğu bozuk `onaylandÃ„Â±` değerini arar ([main.py:107-111](sepet/api/main.py:107)), yorum endpoint'i doğru `onaylandı` değerini arar ([reviews.py:32-35](sepet/api/reviews.py:32)); frontend `can_review` alanına güvenir ([RestaurantDetail.tsx:101-109](sepet/web/src/pages/RestaurantDetail.tsx:101)).
3. **Sipariş detay sayfası kimlik modelini umursamaz.** Frontend `fetchOrder(Number(id))` çağırırken token varsa yine de gönderir ama ownership için kontrole güvenmez ([OrderPage.tsx:14-19](sepet/web/src/pages/OrderPage.tsx:14)); backend de kontrol yapmadığından her ID herkese açıktır.
4. **Bakiye UI'sı sipariş sonrası bayatlar.** Checkout submit bakiye verisini güncellemeden cart'ı temizler ve sipariş sayfasına gider ([Checkout.tsx:46-49](sepet/web/src/pages/Checkout.tsx:46)); `/me` refresh yalnızca cüzdan yükleme akışında çağrılır ([CustomerPanel.tsx:42-45](sepet/web/src/panels/CustomerPanel.tsx:42)).
5. **Restoran işlemlerinde hata yönetimi yok.** Menü ekleme, kurye ekleme, kurye atama ve teslim etme çağrıları `await` edilir ama ayrı hata mesajı/kontrol yoktur ([RestaurantPanel.tsx:42-45](sepet/web/src/panels/RestaurantPanel.tsx:42), [RestaurantPanel.tsx:134-143](sepet/web/src/panels/RestaurantPanel.tsx:134), [RestaurantPanel.tsx:159-165](sepet/web/src/panels/RestaurantPanel.tsx:159), [CourierPanel.tsx:20-23](sepet/web/src/panels/CourierPanel.tsx:20)). Backend'in 400/403/409 mesajları kullanıcıya ulaşmadan unhandled promise olur.
6. **Auth logout yalnızca istemci tarafıdır.** UI çıkış yapınca token localStorage'dan silinir ([App.tsx:30-35](sepet/web/src/App.tsx:30), [AuthContext.tsx:56-59](sepet/web/src/AuthContext.tsx:56)) ama DB'deki token yaşamaya devam eder; token sızarsa süresiz kullanılabilir.
7. **Restoran sahibi kaydında backend-default'lar frontend'de görünmez.** Account formu `email/password/restaurant_name/cuisine/role` gönderir ([Account.tsx:70-75](sepet/web/src/pages/Account.tsx:70)); backend teslimat ücreti, min sipariş, ETA ve emoji default'larını kullanır ([accounts.py:20-22](sepet/api/accounts.py:20)). Bu veriler doğrulanmadığı için risk 6 ile birleşir.

Sonuç: Proje küçük ve okunabilir, parametreli sorgular genelde doğru kullanılmış; ancak sipariş detay erişimi, durum literal'leri ve token/foreign key yaşam döngüsü prodüksiyon öncesi mutlaka düzeltilmelidir.
