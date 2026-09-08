# accounts.py incelemesi

## Purpose

Bu dosya, Sepet backend'inin hesap yonetimi yuzeyini tanimlar: musterinin veya restoran sahibinin kayit olmasi, giris yapmasi, aktif kullaniciyi gormesi ve musterilerin cuzdanina bakiye yuklemesi. Dosya `APIRouter` objesini `/api/auth` onekiyle kurar ve `main.py:15` tarafindan uygulamaya eklenir; bu yuzden `auth.py` icindeki `get_current_user` yardimcisina ve `db.py` icindeki baglanti/sifre yardimcilarina dogrudan baglidir. Kullanici yaratildiginda `user` tablosu yazilir, restoran rolunde ek olarak `restaurant` tablosu yazilir, token uretimi `auth_token` tablosuna yazilir ve bakiye guncellemesi `user.balance` uzerinden yapilir. Bu dosyanin bozulmasi dogrudan giris, kayit, oturum ve odeme bakiyesi akisini etkiler.

## Walkthrough

### RegisterInput sinifi
`class RegisterInput(BaseModel)` 14-23 satirlarinda tanimli. Alanlar `email: str`, `password: str`, `role: str`, `restaurant_name: str = ""`, `cuisine: str = ""`, `delivery_fee: float = 10.0`, `min_order: float = 50.0`, `eta_minutes: int = 30` ve `image_emoji` varsayilanidir. Pydantic dogrulamasi burada tip ve varsayilan degeri zorlar, ancak karakter uzunlugu veya e-posta formati daha sonra yardimci fonksiyonlarda kontrol edilir. `delivery_fee` ve `min_order` icin alt/ust sinir yoktur; `eta_minutes` icin de negatif olamaz kontrolu yoktur.

### LoginInput sinifi
`class LoginInput(BaseModel)` 26-28 satirlarinda tanimli. Parametre olarak `email: str` ve `password: str` alir. Donus degeri yoktur cunku bu bir veri tasiyici siniftir. Kontrol akisi Pydantic tarafindan tip dogrulamasi ile baslar; bos string hala gecerlidir, hatali kimlik durumunda `login` icinde 404/401 mantigi devreye girer.

### user_payload
`def user_payload(user) -> dict` 31-32 satirlarindadir. `user` adinda genellikle `sqlite3.Row` olan bir kayit alir ve `id`, `email`, `role` ile iki ondalige yuvarlanmis `balance` degerini iceren bir `dict` dondurur. Kontrol akisi tek adimlidir; `user["balance"]` None olamaz cunku semada NOT NULL'dur. Bu fonksiyon yanitlarin hassas `password_hash` alanini disari vermemesine yardimci olur.

### validate_email
`def validate_email(email: str) -> None` 35-37 satirlarindadir. Regex `[^@\s]+@[^@\s]+\.[^@\s]+` ile tam eslesme ister. Gecersizse `HTTPException(400, "Gecerli bir e-posta girin")` yukseltir. Kontrol akisi ya return yoksa exception'dir. Regex pratik ama eksiktir; `a@b.c` kabul edilir, alan adi formati daha derin dogrulanmaz.

### validate_registration
`def validate_registration(payload: RegisterInput) -> None` 40-47 satirlarindadir. Once e-postayi dogrular, sonra sifre uzunlugu 6'dan kucukse 400 dondurur. Rol `customer` veya `restaurant` degilse 400 dondurur. Rol restoran ise `restaurant_name` ve `cuisine` zorunlu tutulur. Kurye kaydi burada ozellikle engellenmistir; kurye profilleri `panels.py` uzerinden restoran tarafindan olusturulur.

### issue_token
`def issue_token(conn, user_id: int) -> str` 50-56 satirlarindadir. 32 baytlik rastgele veriden URL-safe bir token uretir, `auth_token` tablosuna `token`, `user_id` ve `utc_now()` ile `created_at` ekler ve token string'ini dondurur. Fonksiyon cagrildigi transaction bittiginde token kalici olur. Token'a son kullanma tarihi eklenmez.

### register
`@router.post("/register", status_code=201)` ile suslenen `def register(payload: RegisterInput) -> dict` 59-86 satirlarindadir. Once `validate_registration` cagrilir. Ardindan `get_db()` ile baglanti acilir; ayni e-posta varsa 409 dondurulur. Yeni `user` satiri `hash_password` ile yazilir ve yeni musteri 100.0 bakiye ile baslar. Rol restoran ise `restaurant` tablosuna kayit acilir; mutfak kucuk harfe cevrilir, rating 4.0 sabitlenir. Daha sonra token uretilir. Kod `with get_db() as conn` blok bittiginde commit bekler. Yanit token, id, e-posta, rol ve bakiye döndurur.

### login
`@router.post("/login)` ile suslenen `def login(payload: LoginInput) -> dict` 89-98 satirlarindadir. E-postaya gore `user` tablosunu arar, kullanici yoksa veya `verify_password` basarisizsa ayni 401 mesajini dondurur. Basariliysa token uretir ve `user_payload` ile genel kullanici bilgisi dondurur. Kontrol akisi tek sorgu ve tek sifre dogrulamasi etrafinda kuruludur.

### me
`@router.get("/me")` ile suslenen `def me(user=Depends(get_current_user)) -> dict` 101-103 satirlarindadir. Kimlik dogrulamasi bagimliligi basarili olmazsa 401 doner. Basariliysa zaten token sorgusundan gelen kullaniciyi `user_payload` ile dondurur. Yeni veritabani sorgusu yapmaz.

### TopUpInput sinifi
`class TopUpInput(BaseModel)` 106-107 satirlarindadir. `amount: float = Field(gt=0, le=1000)` alanini tanimlar. Boylece sifir, negatif deger ve 1000 uzeri tutarlar Pydantic tarafindan reddedilir. Ancak floating point tutarina ozel bir yuvarlama kurali model seviyesinde yoktur.

### top_up_wallet
`@router.post("/wallet/topup")` ile suslenen `def top_up_wallet(payload: TopUpInput, user=Depends(get_current_user)) -> dict` 110-123 satirlarindadir. Ilk olarak rol kontrolu yapar ve musteriler disinda 403 doner. Tutar `round(..., 2)` ile yuvarlanir. `UPDATE "user" SET balance = balance + ?` ile bakiyeyi artirir; guncelleme olmazsa 404 dondurur. Sonra guncel bakiyeyi sorgular, yuvarlayarak dondurur. Kontrol akisi `with get_db()` blok cikisinda commit edilir.

## Data

- `user` okunur: login sorgusu 92-94, `me` icin token tarafindan okunur, top-up bakiye okuma 122.
- `user` yazilir: kayit ekleme 65-68, bakiye artirma 116-119.
- `restaurant` yazilir: restoran kaydi 70-84.
- `auth_token` yazilir: token ekleme 50-56.
- HTTP endpoint'leri: `POST /api/auth/register` 59-86, `POST /api/auth/login` 89-98, `GET /api/auth/me` 101-103, `POST /api/auth/wallet/topup` 110-123.

## Failure modes

- Mojibake varsayilan emoji 23. satirda bozuktur; UI bozuk karakter gosterebilir.
- Restoran kaydinda `delivery_fee`, `min_order`, `eta_minutes` icin alt sinir yoktur; negatif degerler kabul edilebilir (18-22).
- Email dogrulamasi yuzeyseldir; gercek sistemde alan dogrulamasi gerekir (35-37).
- Restoran tablosu eklenirken olasi veritabani hatasi tum transaction'i geri alir, ancak hata mesaji genellestirilebilir (70-84).
- Kayit ve token uretimi ayni transaction'da oldugu icin basarili yanit donmeden hata olursa user/token tutarli kalir, fakat hata mesaji kullaniciya ozel degildir (59-86).
- Top-up sayi yuvarlamasi floating point hassasiyetinden etkilenir (114).

## Security

- Kimlik dogrulama: `/me` ve `/wallet/topup` `get_current_user` bagimliligini kullanir (101-111).
- Yetkilendirme: bakiye yukleme sadece `customer` rolunde (112-113).
- Sifre saklama ve dogrulama `db.py` yardimcilarina devredilir (67, 95).
- SQL injection riski dusuktur cunku parametreli sorgular kullanilir (63, 65-67, 92-94, 116-119).
- Token 32 bayt rastgele uretilir (51), fakat suresizdir (52-55) ve logout endpointsiz oldugu icin sunucuda gecersiz kalinir.

## Suggested changes

- Mevcut:
  `delivery_fee: float = 10.0, min_order: float = 50.0, eta_minutes: int = 30`
- Onerilen:
  `delivery_fee: float = Field(default=10.0, ge=0); min_order: float = Field(default=50.0, ge=0); eta_minutes: int = Field(default=30, ge=1)`

- Mevcut:
  `"INSERT INTO auth_token (token, user_id, created_at) VALUES (?, ?, ?)"`
- Onerilen:
  `"INSERT INTO auth_token (token, user_id, created_at, expires_at) VALUES (?, ?, ?, ?)"` ve token dogrulama/sorgularina suresini ekleyin.

- Mevcut:
  `image_emoji: str = "Ã„Å¸Ã…Â¸Ã‚ÂÃ‚Âª"`
- Onerilen:
  `image_emoji: str = "🍽"` veya guvenli ASCII bir varsayilan.

## Test checklist

1. Bos istek ile kayit: beklenen 422.
2. Gecersiz e-posta ile kayit: beklenen 400.
3. 5 karakter sifre ile kayit: beklenen 400.
4. Gecerli muster kaydi: beklenen 201, token ve bakiye 100.
5. Ayni e-posta ikinci kayit: beklenen 409.
6. Restoran kaydi eksik mutfak: beklenen 400.
7. Gecerli restoran kaydi: beklenen 201 ve restoran profili olusur.
8. Kurye rolune dogrudan kayit: beklenen 400.
9. Gecerli giris: beklenen 200, token ve kullanici payload.
10. Yanlis sifre: beklenen 401.
11. Kimliksiz `/me`: beklenen 401.
12. Gecerli token ile `/me`: beklenen kullanici bilgisi.
13. Giris yapmis restoran top-up: beklenen 403.
14. Giris yapmis musteri 100 TL yukleme: beklenen bakiye 100 artar.
15. 0 veya negatif yukleme: beklenen 422.
16. 1001 TL yukleme: beklenen 422.