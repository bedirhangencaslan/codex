# auth.py incelemesi

## Purpose

Bu dosya uygulamanin ortak kimlik ve rol dogrulama katmanidir. HTTP `Authorization` basligindan Bearer token okur, token ile kullaniciyi cozer ve endpoint'lerin kullanabilecegi bagimlilik fonksiyonlarini saglar: opsiyonel kullanici, zorunlu kullanici, musteri, restoran ve kurye rol kontrolu. `main.py`, `accounts.py`, `panels.py` ve `reviews.py` bu dosyanin yardimcilarina baglidir; bu yuzden token dogrulamasi veya rol kontrolu degistikce tum API'nin erisim modeli etkilenir.

## Walkthrough

### _user_from_token
`def _user_from_token(conn, token: str | None):` 6-15 satirlarindadir. Parametre olarak veritabani baglantisi ve ham `Authorization` degeri alir. Donus degeri ya `sqlite3.Row` tipinde bir kullanici ya da `None`'dur. Once token yoksa veya `Bearer ` onekiyle baslamiyorsa 7-8 satirlarinda `None` dondurur. Daha sonra 9. satirda prefix'i kaldirir ve 10-15 satirlarinda `auth_token` ile `user` tablolarini JOIN ederek `u.id`, `u.email`, `u.role`, `u.balance` degerlerini getirir. Sorgu tam eslesme ile token bulur; herhangi bir uyumsuzlukta `fetchone()` zaten `None` dondurur.

### resolve_user
`def resolve_user(authorization: str | None = Header(default=None)):` 18-20 satirlarindadir. FastAPI basligindan `authorization` degerini alir, `get_db()` ile baglanti acar ve token bazli kullaniciyi cozer. Donus degeri opsiyonel kullanici olabilir. Kontrol akisi kisadir ve hata dondurmez; token gecersizse null kullanan endpoint'ler misafir olarak davranir.

### get_current_user
`def get_current_user(user=Depends(resolve_user)):` 23-26 satirlarindadir. Parametre olarak cozulmus kullaniciyi alir. Kullanici yoksa `HTTPException(401, "Giris yapmalisiniz")` yukseltir; yoksa kullaniciyi dondurur. Bu zorunlu kimlik dogrulamasi icin temel guard'dir.

### get_optional_user
`def get_optional_user(user=Depends(resolve_user)):` 29-30 satirlarindadir. Kimlik dogrulamasi yoksa da null dondurur, varsa kullaniciyi dondurur. Hata akisi yoktur. Restoran detay gibi hem misafire hem girisli kullaniciya gore degisen sayfalarda kullanilir.

### require_customer
`def require_customer(user=Depends(get_current_user)):` 33-36 satirlarindadir. Once zorunlu kimlik dogrulamasi tetiklenir; kullanici yoksa 401 doner. Rol `customer` degilse 403 doner. Gecerli musteri icin user objesini dondurur.

### require_restaurant
`def require_restaurant(user=Depends(get_current_user)):` 39-48 satirlarindadir. Once musteri/kurye gibi yanlis roller 403 ile engellenir. Ardindan `user.id` ile `restaurant.user_id` iliskisini sorgular. Restoran profili yoksa 403 dondurur. Basariliysa `(user, restaurant)` tuple'ini dondurur. Bu endpoint'lere iki deger alan bir bagimlilik dondurdugu icin cagri kodlari unpack eder.

### require_courier
`def require_courier(user=Depends(get_current_user)):` 51-60 satirlarindadir. Rol `courier` degilse 403 doner. Ardindan `courier.user_id` ile profil arar. Profil yoksa 403 dondurur. Basariliysa `(user, courier)` tuple'ini dondurur. Bu sayede sadece rol degil, gercek kurye profili de zorunlu olur.

## Data

- HTTP header okunur: `authorization` basligi 18. satirda.
- `auth_token` okunur: token sorgusu 10-15.
- `user` okunur: token ile JOIN 11-14; `restaurant` ve `courier` profilleri de kullanici id uzerinden iliskilendirilir.
- `restaurant` okunur: 42-45.
- `courier` okunur: 54-57.
- HTTP endpoint yok; yardimci fonksiyon dosyasi.

## Failure modes

- Token suresiz oldugu icin eski token'lar sonsuza kadar gecerli kalabilir (6-15).
- Logout akisi olmadigi icin token DB'de kalir; istemci tarafinda silmek sunucuda token'i gecersiz kilmez (10-15).
- Token basligi format olarak kontrol edilir ama token uzunlugu veya karakter bilesimi ekstra dogrulanmaz; DB eslesmesi yeterli olsa da saldiri yuzeyi buyur (7-14).
- `resolve_user` her istekte yeni baglanti acar, yuksek trafikte kaynak maliyeti artar (18-20).
- Rol degisen veya kullanici silinen bir token dogrulanabilir kalir; `ON DELETE` davranisi `PRAGMA foreign_keys` kapaliyken garanti degildir (10-15, schema 21-25).
- Restoran ve kurye profil kontrolu ayri sorgulardir; tek sorguda birlestirilerek round-trip azaltilabilir (39-60).

## Security

- Kimlik dogrulama tamamen `Bearer token` mantigina dayanir (6-15).
- Yetkilendirme rol bazli yapilir: musteri 33-36, restoran 39-48, kurye 51-60.
- SQL injection yok; token sorgusu parametreli (10-15).
- Veri ifsasi sinirli: sorgu sadece id/email/role/balance getirir, password_hash getirmez (11-14).
- Restoran/kurye profil iliskisi kullanici id ile kontrol edilir, kullanicinin baskasina ait panel erisimini engeller (42-57).

## Suggested changes

- Mevcut:
  `"WHERE t.token = ?"`
- Onerilen:
  `"WHERE t.token = ? AND t.expires_at > ?"` ve sorguya `utc_now()` parametresi ekle.

- Mevcut:
  `def get_current_user(user=Depends(resolve_user)):`
- Onerilen:
  dogrulanmis token kaydini da dondurun (örn. `(user, token_row)`), logout icin silme endpoint'i ekleyin.

- Mevcut:
  `def require_restaurant(...): ... with get_db() as conn: ...`
- Onerilen:
  `resolve_user` icinde profil iliskisini JOIN ile tek sorguda getirin veya token dogrulama fonksiyonunu yeniden kullanin.

## Test checklist

1. Authorization basligi yok: `/me` beklenen 401.
2. `Bearer ` oneksiz token: beklenen 401.
3. Gecersiz token: beklenen 401.
4. Gecerli musteri token'i: `/me` beklenen 200.
5. Gecerli restoran token'i: `require_customer` guard'li endpoint beklenen 403.
6. Gecerli musteri token'i: `require_restaurant` guard'li endpoint beklenen 403.
7. Restoran profili silinmis kullanici: beklenen 403.
8. Kurye rolunde profil olan token: kurye guard'li endpoint beklenen 200.
9. Kurye profili olmayan kurye rolunde kullanici: beklenen 403.
10. Baskasinin restoran id'si ile restoran endpoint'i: beklenen kendi restorani disinda erisim engeli.
11. Token DB'den silinmis: beklenen 401.
12. Opsiyonel guard'li endpoint: misafir beklenen 200 ve `user=None`.
13. Opsiyonel guard'li endpoint: girisli musteri beklenen user bilgisi.
14. Token basligi buyuk/kucuk harf farkiyla: mevcut davranisin hata verip vermedigi belgelenmeli.
15. Ayni token'dan iki istek: ikisi de 200 olmali, rate limiting yoksa bu bilinmeli.