# Sepet Proje Durumu

Bu dosya, yeni oturumlarda projenin hangi noktada kaldığını ve gelecekte yapılacak çalışmanın kurallarını tutar.

## Proje

- Proje kökü: `C:\Users\Bedirhan\Desktop\agent_web\sepet`
- Uygulama başlangıçta `C:\Users\Bedirhan\Desktop\sepet` altında kuruldu, sonra kullanıcı isteğiyle `agent_web\sepet` altına taşındı.
- Backend: Python 3.10, FastAPI, Uvicorn, SQLite, `sqlite3`, düz SQL, ORM yok.
- Frontend: Vite + React + TypeScript, `react-router-dom`, ek UI kütüphanesi yok.
- Stil: `web/src/styles.css`, sıcak turuncu tema, CSS değişkenleri, 900px üstü iki kolon.
- `vite.config.ts` `/api` isteklerini `http://127.0.0.1:8000` adresine proxy'ler.
- Windows'ta PowerShell script politikası nedeniyle npm komutları `npm.cmd` ile çalıştırılmalı.
- Frontend runtime bağımlılıkları yalnızca `react`, `react-dom`, `react-router-dom`; yeni kütüphane eklenmemeli.
- Hiçbir bileşen 200 satırı geçmemeli. Büyük paneller alt bileşenlere bölünmeli.

## Temel Akış

- Backend dosyaları: `api/main.py`, `api/schema.sql`, `api/seed.py`, `api/db.py`, `api/auth.py`, `api/accounts.py`, `api/panels.py`, `api/reviews.py`.
- Sipariş durumları: `hazırlanıyor` -> `kuryede` -> `teslim_edildi` -> `onaylandı`.
- Toplam tutar sunucuda hesaplanır: menü fiyatları + restoranın `delivery_fee` değeri.
- `min_order` altında sipariş 400 döner ve eksik tutar mesajda gösterilir.
- Misafir siparişi hâlâ mümkün; giriş yapan müşterinin siparişi `order.customer_id` ile ilişkilendirilir.
- Sipariş geçmişi yalnızca giriş yapan müşteri içindir. Misafir siparişleri restoran panelinde `Misafir` etiketiyle görünür.

## Hesaplar Ve Kimlik Doğrulama

- Roller: `customer`, `restaurant`, `courier`.
- Şema: `user`, `auth_token`, `courier`, `restaurant_review` tabloları eklendi. `restaurant.user_id` eklendi. `order` tablosuna `customer_id`, `courier_id`, `courier_assigned_at`, `delivered_at`, `confirmed_at` eklendi.
- Şifreler `db.py` içinde PBKDF2-SHA256 ile hash'lenir.
- Token auth basit demo amaçlıdır: `secrets.token_urlsafe` ile üretilen token `auth_token` tablosunda saklanır, süresi yoktur.
- Frontend'de token ve user bilgisi `sepet-auth` localStorage anahtarında tutulur. `web/src/AuthContext.tsx` bu durumu yönetir.
- Backend auth yardımcıları `api/auth.py` içindedir: `get_current_user`, `get_optional_user`, `require_customer`, `require_restaurant`, `require_courier`.
- Kurye hesapları yalnızca restoran sahibi tarafından oluşturulur; kurye kendisi kayıt olamaz.

## Demo Hesaplar

- `seed.py` veritabanını sıfırdan silip kurar.
- Tüm demo hesapların şifresi: `sepet123`
- Demo müşteri: `musteri@sepet.test`
- Restoran sahipleri: `sahip.<slug>@sepet.test`
- Kuryeler: `kurye1.<slug>@sepet.test`, `kurye2.<slug>@sepet.test`
- Restoran slug'ları sırasıyla: `anadolu-atesi`, `napoli-firini`, `cadde-burger`, `sanli-cigkofte-evi`, `sakura-sushi-bar`, `baklavaci-memis`, `leventin-mutfagi`, `karadeniz-balik-evi`.
- Bu demo hesap bilgileri README'ye eklenmeli.

## Yorumlar Ve Puan

- Yorum yapmak için kullanıcı giriş yapmış müşteri olmalı ve ilgili restorandan `onaylandı` durumunda siparişi olmalı.
- Bir kullanıcı bir restorana yalnızca bir yorum yapabilir.
- Yorumda 1-5 puan zorunlu, metin zorunlu.
- `restaurant.rating` kolonu seed/base puan olarak korunur. API'de gösterilen puan seed puanı ile kullanıcı yorumlarının ortalamasının ortalamasıdır: `displayed_rating`.
- Restoran detay API'si `reviews`, `can_review`, `has_review` alanları döndürür.

## Backend API Uçları

Mevcut uçlar:

- `POST /api/auth/register`
- `POST /api/auth/login`
- `GET /api/auth/me`
- `POST /api/orders`
- `GET /api/orders/{id}`
- `GET /api/orders/history`
- `GET /api/restaurant/orders`
- `GET /api/restaurant/couriers`
- `POST /api/restaurant/couriers`
- `POST /api/restaurant/menu-items`
- `POST /api/restaurant/orders/{id}/assign-courier`
- `GET /api/courier/orders`
- `POST /api/courier/orders/{id}/deliver`
- `POST /api/orders/{id}/confirm_delivery`
- `POST /api/restaurants/{id}/reviews`

Korunan orijinal uçlar:

- `GET /api/restaurants?q=&cuisine=`
- `GET /api/restaurants/{id}`
- `GET /api/cuisines`

Kurallar:

- Restoran sadece kendi siparişine ve kendi kuryesine işlem yapabilir.
- Kurye yalnızca kendisine atanmış siparişi teslim edebilir.
- Kurye atama yalnızca `hazırlanıyor` durumunda yapılabilir.
- Kurye teslimi yalnızca `kuryede` durumunda yapılabilir.
- Müşteri onayı yalnızca `teslim_edildi` durumunda ve kendi siparişi için yapılabilir.
- Yetkisiz işlem 403, bulunamayan kayıt 404, geçersiz durum geçişi 409 döndürür.

## Frontend Durumu

Tamamlanan başlangıç uygulaması:

- `/` restoran listesi, backend'e giden arama ve mutfak filtresi.
- `/r/:id` restoran detayı ve sepet.
- `/checkout` sipariş formu.
- `/order/:id` sipariş özeti.
- Sepet React Context ve localStorage ile yönetilir.
- Sepet tek restorana aittir; farklı restorandan ürün eklenirse onay istenir.

Yeni özellik çalışması:

- `web/src/types.ts` güncellendi: `User`, `Review`, `Courier`, sipariş ve restoran alanları eklendi.
- `web/src/api.ts` güncellendi: Bearer token desteği ve yeni auth/panel/review API çağrıları eklendi.
- `web/src/AuthContext.tsx` eklendi.

Henüz yapılacaklar:

- `AuthProvider` uygulama ağacına eklenmeli.
- Header auth-aware hale getirilmeli: giriş durumu, çıkış, role göre panel linki.
- `/giris` kayıt/giriş sayfası eklenmeli.
- `/panel` rol bazlı panel eklenmeli.
- Panel bileşenleri ayrılmalı: `CustomerPanel`, `RestaurantPanel`, `CourierPanel`.
- Restoran panelinde: gelen siparişler, kurye atama, kurye oluşturma, menüye kalem ekleme, yorumlar.
- Kurye panelinde: atanmış siparişler ve "teslim ettim" işlemi.
- Müşteri panelinde: sipariş geçmişi ve teslim onayı.
- Restoran detay sayfasına yorum listesi ve `can_review` varsa yorum formu eklenmeli.
- Checkout giriş yapan kullanıcıyı siparişe bağlamalı; misafir siparişi de çalışmaya devam etmeli.
- `README.md` demo hesaplar ve yeni akışla güncellenmeli.

## Doğrulama Ve Testler

Çalıştırma:

```powershell
cd C:\Users\Bedirhan\Desktop\agent_web\sepet\api
python seed.py
python -m uvicorn main:app --port 8000
```

```powershell
cd C:\Users\Bedirhan\Desktop\agent_web\sepet\web
npm.cmd install
npm.cmd run dev
npm.cmd run build
```

Adresler:

- Frontend: `http://localhost:5173`
- Backend: `http://127.0.0.1:8000`

Yeni özellik için test edilecekler:

1. `python seed.py` hatasız çalışmalı, demo hesaplar oluşmalı.
2. Kayıt, giriş, `/api/auth/me`, çıkış akışı çalışmalı.
3. Girişli müşteri siparişi hesaba bağlanmalı ve geçmişte görünmeli.
4. Misafir siparişi çalışmalı ve restoran panelinde görünmeli.
5. Restoran sahibi kendi kuryesini oluşturup siparişe atayabilmeli.
6. Kurye yalnızca kendi siparişini teslim edebilmeli.
7. Müşteri teslim edilen siparişi onaylayabilmeli.
8. Onay sonrası müşteri yorum yapabilmeli; ikinci yorum 409 dönmeli.
9. Restoran puanı yorumla güncellenmeli.
10. Yanlış rol, yanlış restoran ve yanlış kurye işlemleri 403 döndürmeli.
11. Geçersiz durum geçişleri 409 döndürmeli.
12. `npm.cmd run build` TypeScript hatasız tamamlanmalı.

## Bilinen Notlar

- Başlangıç uygulaması tamamen test edilmişti; hesap/yorum/panel özelliği hâlâ yarı yolda.
- Son doğrulanan seed başarılıydı: 8 restoran ve demo hesaplar oluştu.
- Son backend ve frontend değişikliklerinden sonra build veya uçtan uca test çalıştırılmadı.
- `README.md` henüz yalnızca başlangıç uygulamasının kurulumunu anlatıyor.
- `restaurant` tablosu `user` tablosuna referans verdiği için şemada `user` tanımı `restaurant` tanımından önce gelmelidir; mevcut `schema.sql` bu sırayı zaten kullanıyor.
- Türkçe karakterler dosyalarda UTF-8 olarak duruyor; terminal çıktısında bozuk görünebilir ama dosya içeriği bozulmuş değildir.
- `restaurant.rating` API yanıtında hesaplanan puanla değişebilir, ancak veritabanındaki seed/base puan değiştirilmez.
