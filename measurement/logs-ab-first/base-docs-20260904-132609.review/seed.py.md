# seed.py incelemesi

## Purpose

Bu dosya demo ortami icin veritabanini bastan olusturur. `api/sepet.db` dosyasini siler, `schema.sql` calistirir, 8 restoran ve her restoran icin 6 menu urunu ekler, sonra demo musteri, restoran sahipleri ve her restoran icin 2 kurye hesabi olusturur. `SIFRELER.txt` referansiyla demo hesaplarin kaynagidir. Arastirma, test ve ilk kurulum icin kullanilir; uretim ortaminda bu dosya dogrudan calistirilmamalidir.

## Walkthrough

### Path sabitleri
`DB_PATH = Path(__file__).with_name("sepet.db")` 7. satir ve `SCHEMA_PATH = Path(__file__).with_name("schema.sql")` 8. satir. Parametre almaz, donus degeri sabit `Path` objeleri. Kontrol akisi yoktur. DB ve schema ayni klasorde olacagindan calistirma dizininden bagimsizdir.

### RESTAURANTS
`RESTAURANTS = [...]` 11-148 satirlarindadir. Parametre almaz; donus degeri modul seviyesinde bir list-of-dicts sabitidir. Her dict restoran adi, mutfak, rating, teslimat ucreti, minimum sepet, teslimat suresi, emoji ve `menu` listesi icerir. Menu listesi `(name, description, price, category)` tuple'larindan olusur. Bu sabit, seed isleminde restoran/menu eklemek icin kaynak veri olarak kullanilir.

### RESTAURANT_SLUGS
`RESTAURANT_SLUGS = [...]` 150-159 satirlarindadir. Parametre almaz, string listesi dondurur. Her restoran icin email slug'i saglar. Kontrol akisi yoktur. `main()` icinde `zip(RESTAURANTS, RESTAURANT_SLUGS)` ile kullanilir. Liste uzunlugu 8 restoranla eslesir; uyusmazsa zip sessizce fazla elemanlari atlar.

### main
`def main() -> None:` 162-230 satirlarindadir. Parametre almaz, donus degeri yoktur. Once 163. satirda `DB_PATH.unlink(missing_ok=True)` ile veritabanini siler. Bu, AGENTS kurallarina gore dokumante edilen davranistir. 164-166 satirlarinda SQLite baglantisi acar, schema dosyasini UTF-8 olarak okur ve `executescript` ile calistirir. Baglanti `row_factory = sqlite3.Row` olur.

Restoran eklerken 167-191 satirlarinda her `RESTAURANTS` elemani dongude islenir. Ilgili restoran `restaurant` tablosuna parametreli insert edilir, `cursor.lastrowid` ile yeni id alinir ve menu urunleri `executemany` ile `menu_item` tablosuna yazilir.

193. satirda `"sepet123"` sifresi bir kez `hash_password` ile hash'lenir. 194. satirda `utc_now()` alinir. 195-198 satirlarinda demo musteri `musteri@sepet.test` olarak customer rolunde ve 250.0 bakiyeyle eklenir.

200-227 satirlarinda restoran ve slug listesi zip edilir. Her restoran icin ad uzerinden id bulunur. Bulunamazsa dongu `continue` eder. Restoran bulundugu takdirde sahip hesabi `sahip.{slug}@sepet.test` emaili ile restaurant rolunde ve 0.0 bakiyeyle eklenir, sonra restoran `user_id` guncellenir. Her restoran icin `number in (1, 2)` dongusuyle iki kurye kullanici eklenir ve her kurye ilgili restorana baglanir.

229-230 satirlarinda demo musteri ve restoran sayisi ekrana yazilir. 233-234 satirlarinda dosya dogrudan calistirilirse `main()` tetiklenir.

## Data

- Dosya sistemi: `sepet.db` silinir/olusturulur (7, 163), `schema.sql` okunur (8, 164-166).
- `restaurant` yazilir: 168-181.
- `menu_item` yazilir: 183-191.
- `user` yazilir: musteri 195-198, restoran sahibi 207-210, kuryeler 216-219.
- `restaurant.user_id` guncellenir: 211-214.
- `courier` yazilir: 220-227.
- HTTP endpoint yok; bu dosya bir CLI seed aracidir.
- Client state yok.
- Console output uretilir: 229-230.

## Failure modes

- `DB_PATH.unlink` uzerinden veritabani yok edilir; calisir uygulamada tekrar calistirilirsa mevcut veriler kaybolur (163).
- Schema `executescript` ile calistirilir; otomatik transaction davranisindan farkli olarak script icindeki hatayi geri almak garanti degildir (164-166).
- Kurye ve restoran owner sifreleri ayni demo sifreden uretilir; uretim icin guvensizdir (193, 209, 218).
- Email formatlari sablondan uretilir; slug bozuksa veya Unicode sorunlari varsa hatali hesaplar olusabilir (200-227).
- Restoran `SELECT id FROM restaurant WHERE name = ?` ile ayni ada gore bulunur; ayni isimli restoranlar olusursa iliskiler yanlis olabilir (200-213).
- Zip uzunlugu eslesmezse sessizce kisalir; hata dondurmez (200).
- Schema'daki default status `"hazÄ±rlanÄ±yor"` mojibake ise yeni kayitlarda bu literal kullanilabilir; `panels.py` dogru literal ile uyumsuz kalir.
- Kurye number 1 ve 2 olarak sabit; ayni email uretilecek sekilde slug tekrari olursa UNIQUE hatasi ile seed yarida kesilir.
- Seed ciktisi UTF-8 konsolda bozuk gorunebilir; dogrulama DB sorgusu veya kod noktalari ile yapilmali.
- `utc_now()` alinir fakat bu dosyada direkt kullanilmaz gibi gorunuyor; dead variable olabilir (194).

## Security

- `hash_password` ile PBKDF2 sifre hashleme kullanilir (193, 209, 218).
- Demo hesap email ve sifreleri kod/arkasinda varsayilan olarak bilinir; uretimde hizli erisim riski yaratir.
- SQL parametreleri kullanildigi icin injection riski dusuktur (168-181, 183-191, 195-198, 207-219, 220-227).
- Dosya silme isi lokal calisma dizininde yapilir; yanlis yerde calistirilmiyorsa risk dusuk, ancak DB yolu kaynak dosyaya bagli oldugundan dogru klasoru hedefler (7, 163).
- Şema foreign key tanimlarina ragmen bu dosyada PRAGMA foreign_keys acilmaz; gecersiz iliski yazilmasi teorik olarak engellenmez.

## Suggested changes

- Mevcut:
  `DB_PATH.unlink(missing_ok=True)`
- Onerilen:
  `if DB_PATH.exists() and "--force" not in sys.argv: raise SystemExit("DB silinmeden once --force gerekli")`

- Mevcut:
  `with sqlite3.connect(DB_PATH) as conn, open(SCHEMA_PATH, encoding="utf-8") as schema_file:`
- Onerilen:
  `with sqlite3.connect(DB_PATH) as conn, open(SCHEMA_PATH, encoding="utf-8") as schema_file: conn.execute("PRAGMA foreign_keys = ON")` ve script calistirirken hata durumunu acik kontrol edin.

- Mevcut:
  `"sepet123"`
- Onerilen:
  demo sifresini env/args ile alin veya sadece test konfigurasyonunda kullanin.

- Mevcut:
  `for restaurant, slug in zip(RESTAURANTS, RESTAURANT_SLUGS):`
- Onerilen:
  `if len(RESTAURANTS) != len(RESTAURANT_SLUGS): raise ValueError("Restoran ve slug listesi uzunluklari eslesmiyor")`.

## Test checklist

1. Seed oncesi mevcut DB yedeginin alindigini dogrula.
2. Seed calistiginda eski DB silinmeli.
3. Schema calistirilip 8 tablo olusmali.
4. 8 restoran eklenmeli.
5. Her restoran icin 6 menu urunu eklenmeli.
6. Demo musteri customer rolunde 250 bakiyeyle olusmali.
7. Her restoran icin bir sahip hesabi olusmali.
8. Her restoran `user_id` ile sahibine baglanmali.
9. Her restoran icin 2 kurye hesabi olusmali.
10. Kurye `restaurant_id` dogru restorana baglanmali.
11. Demo sifresi ile giris calismali.
12. Ayni seed ikinci kez calistirildiginda eski verilerin silindigi dogrulanmali.
13. Restoran email slug'lari dogru olmali.
14. Seed ciktisinda restoran sayisi 8 olmali.
15. Unicode restoran ve menu adlari DB'de bozulmadan saklanmali.
16. Sifre hash algoritmasi ve salt uretimi dogru calismali.
17. Schema foreign key tanimlari dogrulanmali.
18. Seed sonrasi `auth_token` tablosu bos olmali.