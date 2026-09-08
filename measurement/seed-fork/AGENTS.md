# Sepet Calisma Notlari

Bu workspace'teki ana proje `sepet/` klasorudur. Demo hesaplarin tam listesi
[SIFRELER.txt](sepet/SIFRELER.txt) dosyasindadir. Ayri SEPET-STATE.md veya
ARCHITECTURE-REVIEW.md dosyalari yoktur; eski baglantilari kullanma.

## Kisa Kurallar

- Kullanici Turkce konusur; cevaplar Turkce olmali.
- `sepet/` icinde Vite + React + TypeScript frontend ve FastAPI + SQLite backend var.
  Kutuphane listesini buyutmek icin once kullaniciya sor.
- `python seed.py` komutu `api/sepet.db` dosyasini silip veritabanini sifirdan kurar.
- Git deposu yok; buyuk dosya degisikliklerini dikkatli yap.
- Kod inceleme yapilirken once kaynagi oku; varsayimla risk analizi yazma.

## Aktif Gorev: AUDIT.md

Kullanici uctan uca kod denetimi istedi ve sonucu kok dizindeki `AUDIT.md`
dosyasina yazilmasini istedi. KAYNAK DOSYALAR DEGISTIRILEMEZ; bu gorevde
yaratilmasi/degistirilmesi izinli tek dosya `AUDIT.md`'dir.

`AUDIT.md` istenen bolumler:

1. Module map: her Python modulu, sorumlulugu, okudugu ve yazdigi tablolar.
2. Coupling: moduller arasi bagimliliklar, `file:line` kaynakli.
3. Request lifecycle: her HTTP ucu icin handler'dan DB'ye ve geriye gecen
   fonksiyonlari adlandirarak izle.
4. Data model: her tablo/kolon ve dokunan endpoint'ler.
5. Risks: gerekceli 10 ciddi dogruluk/guvenlik sorunu, `file:line` ve somut
   degisiklik onerisi.
6. Frontend: istemcinin API konusmasi ve backend uyusmazliklari.

Satir numarasi olmadan genel iddia yazma. `AUDIT.md` henuz yazilmadi.

## Denetim Okuma Durumu

Semantik kaynaklar tam okundu: `api/*.py`, `api/schema.sql`,
`api/requirements.txt`, `web/index.html`, `web/package.json`, Vite/TSconfig
dosyalari, `web/src/**` dosyalari, `README.md`, `AGENTS.md`,
`SIFRELER.txt`. `package-lock.json` ve `sepet.db` semantik kaynak olarak
okunmadi. Satir numaralariyla su bulgular netlesmistir:

- `api/db.py:11-14`: `get_db()` yeni SQLite baglantisi acar, foreign keys
  acilmaz. `db.py:21-31` PBKDF2 hash/verify uygular.
- `api/auth.py:6-15` token'dan kullanici ceker; token yalnizca varligina
  bakilir, expiry yok. `auth.py:23-30` mandatory/optional user.
  `auth.py:33-48` customer/restaurant, `auth.py:51-60` courier guard.
- `api/accounts.py:59-86` register; customer/restaurant icin token uretir ve
  restoran olusturur. `accounts.py:89-98` login. `accounts.py:106-123`
  wallet top-up (yalnizca customer).
- `api/main.py:33-70` restoran listesi ve filtre/siralama.
  `main.py:73-79` mutfaklar. `main.py:82-116` restoran detay.
  `main.py:119-139` siparis detayi auth/ownership yok.
  `main.py:142-214` siparis olusturma: menu dogrulama, min order, bakiye
  kontrolu, bakiye dusme, order/order_item insert.
- `api/panels.py:35-49` order_item serialization. `panels.py:68-103` customer
  history ve restoran siparisleri. `panels.py:106-125` courier/menu list.
  `panels.py:128-157` courier ve menu olusturma. `panels.py:160-187` kurye
  atama. `panels.py:190-223` kurye siparisleri/teslim.
  `panels.py:226-241` musteri teslim onayi. `panels.py:247-272` kazanc ozeti.
- `api/reviews.py:13-15` seed rating + yorum ortalamasi.
  `reviews.py:18-50` onayli siparis sonrasi tek yorum ekleme.
- `api/schema.sql:1-76` tablolar: restaurant, user, auth_token, courier,
  restaurant_review, menu_item, order, order_item. Para alanlari REAL ve
  siparis status default'u mojibake: `schema.sql:60`.
- `api/main.py:109`, `main.py:199` ve `main.py:214` mojibake status literal
  kullanir; `panels.py:174`, `panels.py:238` ve `reviews.py:34` dogru literal
  kullanir. Kurye atamasi mojibake sipariste tikanir.
- `api/seed.py:162-230` DB'yi siler, schema calistirir, restoran/menu/hesap
  seed eder. Tum demo sifre `sepet123`.
- Web: `web/vite.config.ts:6-9` `/api` proxy. `web/src/api.ts:3-21` Bearer
  token/JSON fetch wrapper, `api.ts:32-123` endpoint cagrilari.
  `web/src/AuthContext.tsx:17-66` localStorage auth. `web/src/CartContext.tsx:21-79`
  localStorage cart. Sayfalar: Restaurants, RestaurantDetail, Checkout,
  OrderPage, Account, Panel; paneller Customer/Restaurant/Courier.

## Onemli Riskler

- `GET /api/orders/{id}` public: adres, telefon, not siziyor (`main.py:119-139`).
- Siparis durum literal'leri normalize edilmeli (`schema.sql:60`, `main.py:109,199,214`).
- Foreign key pragma yok (`db.py:11-14`).
- Token suresiz, logout DB tokenini gecersiz kilmaz
  (`accounts.py:50-56`, `auth.py:6-15`, `AuthContext.tsx:56-59`).
- Para alanlari REAL; yuvarlama/hesap hatasi riski (`schema.sql:5-7,18,49,59`).
- Sepet/bakiye UI verisi siparis sonrasi eski kalabilir
  (`Checkout.tsx:47-49`, `CustomerPanel.tsx:38-45`).
- Yeni eklenen menu item `name/description/category` icin length kontrolu yok
  (`panels.py:148-157`); adres/telefon/not backend validasyonu zayif
  (`main.py:25-30`).
- Restoran puanlamasi seed rating ile yorum ortalamasinin ortalamasi; getiri
  raporu teslim/onay durumundan bagimsiz toplam da sayar
  (`reviews.py:13-15`, `panels.py:247-272`).

## Ortam Notlari

- PowerShell varsayilan kabuk. `rg` kurulu degil; `Select-String` kullan.
- `apply_patch` bu oturumda guvenilmezdi; uzun heredoc'lar ve pipe input
  basarisiz oldu. Uzun dosya yaziminda PowerShell .NET `WriteAllText` ve
  UTF-8 no-BOM guvenli calisti.
- PowerShell `Set-Content -Encoding UTF8` BOM ekler; schema gibi dogrudan
  okunan dosyalarda sorun cikarir.
- Port 5173, 5174 ve 5175 onceki oturumlarda dolu olabilir; aktif frontend
  adresini `Get-NetTCPConnection` veya surec cikisindan dogrula.
- Konsol ciktisi Turkce karakterleri bozuk gosterebilir. Gercek dogrulama
  icin Unicode kod noktalari veya Python/SQLite sorgusu kullan.