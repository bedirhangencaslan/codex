# db.py incelemesi

## Purpose

Bu dosya backend'in veri ve sifre yardimci katmanidir. SQLite veritabani baglantisi olusturur, UTC zaman damgasi uretir ve PBKDF2 ile sifre hash'leme/dogrulama yapar. Tum API modulleri dogrudan veya dolayli olarak bu dosyaya baglidir; ozellikle `get_db()` cagrildigi her yerde transaction ve baglanti davranisi belirlenir.

## Walkthrough

### DB_PATH
`DB_PATH = Path(__file__).with_name("sepet.db")` 8. satirdadir. Bagli bir parametre almaz; donus degeri modul seviyesinde sabit bir `Path` objesidir. Kontrol akisi yoktur. Bu, veritabaninin Python kaynak dosyasinin yanina yerlesmesini garanti eder ve `seed.py` ile ayni dosyayi kullanmasini saglar.

### get_db
`def get_db() -> sqlite3.Connection:` 11-14 satirlarindadir. Parametre almaz. Donus degeri yeni bir `sqlite3.Connection` objesidir. Baglanti `DB_PATH` uzerinden acilir ve `row_factory = sqlite3.Row` set edilir. Bu sayede sorgu sonuclari hem indeksle hem kolon adiyla erisilebilir. Fonksiyon baglantiyi kapatmaz veya foreign key pragma'sini acmaz. Kullanan kod genellikle `with get_db() as conn` yapar; SQLite'ta bu, blok normal bittiginde commit ve exception'da rollback saglar, ama connection close etmez. Yani connection lifecycle tamamen cagri koduna birakilmistir.

### utc_now
`def utc_now() -> str:` 17-18 satirlarindadir. Parametre almaz. Donus degeri ISO 8601 formatli UTC zaman string'idir. `datetime.now(timezone.utc).isoformat()` ile uretilir. Kontrol akisi tek adimlidir. `auth_token.created_at`, siparis zamanlari, teslim ve onay zamanlari bu formatla yazilir.

### hash_password
`def hash_password(password: str) -> str:` 21-24 satirlarindadir. Parametre `password` string'idir. Donus degeri `salt$digest` formatinda bir string'dir. 16 bayt (32 hex karakter) rastgele tuz uretir. PBKDF2-HMAC-SHA256 algoritmasi 120000 iterasyonla calisir. Kontrol akisi salt uretimi, diger hesaplama ve formatlama seklinde ilerler. Her cagri yeni tuz urettigi icin ayni sifre farkli hash uretir.

### verify_password
`def verify_password(password: str, stored: str) -> bool:` 27-33 satirlarindadir. Parametre olarak ham sifre ve saklanmis hash string'i alir. Donus degeri bool'dir. `try` blogunda `stored` degeri `$` ile ayirir, ayni salt ve iterasyon sayisiyla PBKDF2 digest hesaplar ve `secrets.compare_digest` ile sabit zamanli karsilastirma yapar. `ValueError` olursa (orn. format bozuk) `False` dondurur. Ancak `TypeError` gibi farkli istisnalar yakalanmaz; yanlis veri tipi beklenmedik hataya yol acabilir.

## Data

- Dosya sistemi okunur/yazilir: SQLite dosyasi `sepet.db` (8, 12).
- Tum tablolara erisim bu baglanti uzerinden olur; dosya kendisi tablo yazmaz/okumaz.
- HTTP endpoint yok.
- Client state yok.
- Zaman verisi uretir: `auth_token.created_at`, `order.created_at`, `courier_assigned_at`, `delivered_at`, `confirmed_at` kullanim yerleri buna baglidir.
- Sifre hash'i uretir/dogrur: `user.password_hash`.

## Failure modes

- `PRAGMA foreign_keys = ON` acilmadigi icin referans butunlugu SQLite varsayilan olarak devre disidir (11-14). Bu, gecersiz `user_id`, `restaurant_id` veya `courier_id` yazilmasina yol acabilir.
- `with get_db() as conn` connection close etmedigi icin kaynak sizmasi mumkundur (11-14).
- Eski SQLite surumleri veya ayni dosyaya yogun erisimde lock riski artar (12).
- PBKDF2 iterasyonu kod icerisine gomulu; ileride coklu hash surumu desteklemek istersen mevcut `salt$digest` formati yeterli degil (21-24, 27-31).
- `verify_password` yalniz `ValueError` yakalar; bozuk tipler veya beklenmeyen kodlama hatalari istisna verebilir (27-32).
- Veritabani dosyasi silinir/reseed edilirken uygulama calisiyorsa yeni baglantilar dosya yolu uzerinden acilir, eski baglantilar farkli durumda kalabilir (8, 12).

## Security

- Kimlik dogrulama bu dosyada yapilmaz, ama sifre dogrulama temelini saglar (21-33).
- Rastgele tuz `secrets.token_hex(16)` ile uretilir (22), tekrar kullanilabilir tuz riski dusuktur.
- `secrets.compare_digest` kullanildigi icin klasik timing attack riski azalir (31).
- PBKDF2 iterasyon sayisi 120000'dir (23, 30); saldiri maliyetini artirir ama cagin standartlarina gore surekli guncellenmesi gerekir.
- SQL sorgu yok; injection riski bu dosyada yoktur.
- Veritabani dosyasi uygulama klasorunde tutulur (8); yedekleme/erisim izinleri operasyonel gizlilik icin kritiktir.

## Suggested changes

- Mevcut:
  `conn = sqlite3.connect(DB_PATH); conn.row_factory = sqlite3.Row; return conn`
- Onerilen:
  `conn = sqlite3.connect(DB_PATH); conn.row_factory = sqlite3.Row; conn.execute("PRAGMA foreign_keys = ON"); return conn`

- Mevcut:
  `return f"{salt}${digest.hex()}"`
- Onerilen:
  `return f"pbkdf2_sha256$120000${salt}${digest.hex()}"` ve `verify_password` icinde algoritma/iterasyon ayristirmasi ekle.

- Mevcut:
  kullanicilarin `with get_db() as conn` sonrasinda connection close etmemesi.
- Onerilen:
  FastAPI dependency ile `finally: conn.close()` davranisi sagla veya contextmanager yaz.

## Test checklist

1. `get_db()` donduren baglantida `row_factory` `sqlite3.Row` olmali.
2. Ayni fonksiyon iki kez cagrildiginda ayri connection objeleri donmeli.
3. `utc_now()` ciktisi ISO 8601 ve UTC olmali.
4. Ayni sifre iki kez hash'lenince farkli salt/digest uretilmeli.
5. Dogru sifre `verify_password` ile True donmeli.
6. Yanlis sifre False donmeli.
7. Hash formati bozuk oldugunda False donmeli.
8. Sadece salt veya sadece digest varsa False donmeli.
9. Yeni baglantida `PRAGMA foreign_keys` 1 olmali.
10. Foreign key acikken gecersiz `user_id` ile token insert'i engellenmeli.
11. Transaction exception oldugunda degisiklikler rollback olmali.
12. Connection close edildikten sonra sorgu beklenmedik hata vermeli.
13. Unicode sifre ile hash/dogrulama calismali.
14. Cok uzun sifre ile performans ve hata davranisi test edilmeli.
15. Dosya yoksa yeni bos DB olusmali, app endpoint'leri tablo hatasi vermemeli icin seed oncesi kontrol edilmeli.