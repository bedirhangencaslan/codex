# panels.py incelemesi

## Purpose

Bu dosya musteri, restoran sahibi ve kurye panel akislari icin REST endpoint'lerini toplar. Siparis gecmisi, restoran siparis listesi, kurye ve menu yonetimi, kurye atama, kurye siparis listesi, teslim etme, musterinin teslimi onaylamasi ve restoran kazanc ozetini saglar. `main.py:16` tarafindan uygulamaya eklenir; `auth.py` guard'lari ve `db.py` baglanti yardimcilari uzerine kurulur. Bu dosya is akisinin cogunugunu tasidigi icin status literal'leri ve ownership kontrolu kritik oneme sahiptir.

## Walkthrough

### CourierInput
`class CourierInput(BaseModel)` 13-16 satirlarindadir. Alanlar `email`, `password` ve `name`. Donus degeri yoktur. E-posta formati, sifre uzunlugu veya adin boslugu burada dogrulanmaz; sadece endpoint icinde ad kontrol edilir.

### MenuItemInput
`class MenuItemInput(BaseModel)` 19-23 satirlarindadir. Alanlar `name`, `description`, `price: float = Field(gt=0)` ve `category`. Donus degeri yoktur. Fiyat sifirdan buyuk zorunlu; diger alanlar icin uzunluk/tek dogrulama yok.

### AssignCourierInput
`class AssignCourierInput(BaseModel)` 26-27 satirlarindadir. `courier_id: int` alir. Donus degeri yoktur. Kuryenin restorana ait olup olmadigi endpoint'te kontrol edilir.

### ReviewInput
`class ReviewInput(BaseModel)` 30-32 satirlarindadir. `rating` 1-5 arasi zorunlu, `comment` string. Donus degeri yoktur. Bos yorum kontrolu `reviews.py:24-25` icinde yapilir.

### serialize_orders
`def serialize_orders(conn, rows) -> list[dict[str, Any]]` 35-49 satirlarindadir. Parametre olarak baglanti ve siparis `sqlite3.Row` listesi alir, dict listesi dondurur. Her siparis row'u once `dict(row)` ile kopyalanir, sonra `order["items"]` listesi `order_item` sorgusundan doldurulur. Her satira `line_total` eklenir. Kontrol akisi rows uzerinde dongu ve her row icin ayri sorgudur; N+1 query problemi vardir.

### order_with_items
`def order_with_items(conn, order_id: int):` 52-65 satirlarindadir. Parametre baglanti ve siparis id. Donus tek siparis dict'i veya `None`. Siparis, restoran, musteri email ve kurye adi JOIN ile getirilir. Siparis yoksa `None`, varsa `serialize_orders` ile item listesi eklenir. Bu yardimci fonksiyon su an hic endpoint'ten cagrilmiyor gibi gorunuyor; olasi dead code veya gelecek kullanim icin tanimli.

### order_history
`@router.get("/orders/history")` ile suslenmis `def order_history(user=Depends(require_customer)) -> list[dict[str, Any]]` 68-81 satirlarindadir. Parametre olarak yalniz dogrulanmis musteri alir. Donus siparis dict'leri listesidir. Kod sadece `customer_id` kullaniciya esit olan siparisleri desc siralar, sonra `serialize_orders` ile urunleri ekler. Bu, musteri gecmisini guvenli sekilde owner'a sinirlar.

### restaurant_orders
`@router.get("/restaurant/orders")` ile suslenmis `def restaurant_orders(status="", user_restaurant=Depends(require_restaurant))` 84-103 satirlarindadir. Parametre olarak opsiyonel `status` filter ve `(user, restaurant)` tuple alir. Donus siparis listesidir. Sorgu restoran id'sine sinirlanir; `status` verilirse ikinci parametreli WHERE eklenir. Sorgu desc siralanir. Ownership kontrolu guard icinde yapilir.

### list_couriers
`@router.get("/restaurant/couriers")` ile suslenmis `def list_couriers(user_restaurant=Depends(require_restaurant))` 106-114 satirlarindadir. `(user, restaurant)` alir. Donus kurye dict listesidir. Sadece kendi restoranina ait `courier` kayitlarini id, name ve user_id olarak getirir. `user_id` ifsasi burada panel icin bilgilendirici ama disari sunuma yonelik mutlaka gerekli degil.

### list_menu_items
`@router.get("/restaurant/menu-items")` ile suslenmis `def list_menu_items(user_restaurant=Depends(require_restaurant))` 117-125 satirlarindadir. Restoran tuple alir, menu item listesi dondurur. Sorgu kendi restoranina ait urunleri kategori ve ada gore siralar.

### create_courier
`@router.post("/restaurant/couriers", status_code=201)` ile suslenmis `def create_courier(...)` 128-145 satirlarindadir. `CourierInput` ve restoran tuple alir. Donus olusturulan kurye dict'idir. Ad bos ise 400 dondurur. E-posta zaten varsa 409 dondurur. Yeni `user` kaydi courier rolunde eklenir ve `courier` tablosuna restoran iliskisiyle yazilir. E-posta format/sifre uzunlugu kontrol edilmedigi icin olasi bozuk hesaplar olusabilir.

### add_menu_item
`@router.post("/restaurant/menu-items", status_code=201)` ile suslenmis `def add_menu_item(...)` 148-157 satirlarindadir. `MenuItemInput` ve restoran tuple alir. Donus olusturulan menu item dict'idir. Yeni `menu_item` satiri kendi restoran id'siyle eklenir. Bos ad/kategori/aciklama engellenmez.

### assign_courier
`@router.post("/restaurant/orders/{order_id}/assign-courier")` ile suslenmis `def assign_courier(...)` 160-187 satirlarindadir. Siparis id, `AssignCourierInput`, restoran tuple alir. Donus status ve kurye adi. Kod once siparisi id ile arar; siparis yoksa veya baska restorana aitse 404 dondurur. Status dogru Unicode `"hazırlanıyor"` degilse 409 dondurur. Kurye id restoran id ile birlikte dogrulanir; baska restorana ait kurye ise 403 dondurur. Sonra siparis `kuryede` statusuna gecirilir ve atama zamanlani. Bu endpoint ownership dogru; ancak main.py status mojibake olursa bu kontrol siparisi bulamaz.

### courier_orders
`@router.get("/courier/orders")` ile suslenmis `def courier_orders(courier_data=Depends(require_courier))` 190-204 satirlarindadir. `(user, courier)` tuple alir. Donus kuryenin atanmis siparisleri. Sorgu `JOIN courier c ON c.id = o.courier_id` ve `WHERE o.courier_id = ?` kullanir. Siparis urunleri `serialize_orders` ile eklenir.

### deliver_order
`@router.post("/courier/orders/{order_id}/deliver")` ile suslenmis `def deliver_order(...)` 207-223 satirlarindadir. Siparis id ve kurye tuple alir. Donus teslim statusu. Siparis yoksa veya kurye eslesmiyorsa 404 dondurur. Status `"kuryede"` degilse 409 dondurur. Gecerli durumda siparis `"teslim_edildi"` statusuna gecirilir ve `delivered_at` yazilir.

### confirm_delivery
`@router.post("/orders/{order_id}/confirm_delivery")` ile suslenmis `def confirm_delivery(...)` 226-241 satirlarindadir. Siparis id ve musteri tuple alir. Siparis yoksa veya musteri owner degilse 404 dondurur. Status `"teslim_edildi"` degilse 409 dondurur. Gecerli durumda `"onaylandı"` statusuna gecer ve `confirmed_at` yazilir. Ownership dogru.

### restaurant_earnings
`@router.get("/restaurant/earnings")` ile suslenmis `def restaurant_earnings(user_restaurant=Depends(require_restaurant))` 247-272 satirlarindadir. Restoran tuple alir, kazanc dict dondurur. Once tum siparislerin adet ve toplam gelirini, sonra status `"onayland\u0131"` olan siparislerin ayni degerlerini hesaplar. Ortalama siparis degeri tum siparislerin gelirine gore hesaplanir. Toplam, onayli ve bekleyen gelir ile adetler dondurulur.

## Data

- `order` okunur/yazilir: gecmis 71-80, restoran listesi 90-102, kurye listesi 194-203, atama 169-186, teslim 212-222, onay 230-240, kazanc 251-263.
- `order_item` okunur: 39-46.
- `restaurant` okunur: JOIN 54-59, 72-77, 90-95, 195-200.
- `user` okunur: musteri email JOIN 54-59, 72-77, 90-95, 195-200; yazilir kurye hesabi 136-139.
- `courier` okunur/yazilir: listeleme 110-113, kurye olusturma 141-144, atama dogrulama 176-179, JOIN 54-59.
- `menu_item` okunur/yazilir: 117-125, 151-156.
- `restaurant_review` dolayli degil; yorum dogrulamasi `reviews.py`.
- HTTP endpoint'leri: 68, 84, 106, 117, 128, 148, 160, 190, 207, 226, 247.

## Failure modes

- `serialize_orders` her siparis icin ayri sorgu calistirir; buyuk listelerde N+1 query verimlilik kaybi (35-49).
- Siparis listelerinde sadece musteri email `u.email AS customer_email` getirilir; panelde gereksiz PVA/sifre olmayan ama yine de veri ifsasi buyuyor (54-59, 72-77, 90-95, 195-200).
- `create_courier` email format ve sifre uzunlugunu dogrulamaz (128-145).
- `add_menu_item` bos ad/kategori/aciklama kabul eder (148-157).
- Restoran listesi status filter istemciden dogrudan alir; gecersiz status sadece bos sonuc dondurur, tip hatasi vermez (84-100).
- Kurye atamasi dogru literal `"hazırlanıyor"` ister (174), main.py/schema SQL bozuk literal kullaniyor olabilir; bu uyumsuzluk akisi tikar.
- `restaurant_earnings` pending gelirini `overall - confirmed` ile hesaplar; status degismezse veya farkli literal varsa yanlis gelir raporu (251-268).
- Panel siparisleri canli guncellenmez; UI stale state kalabilir (190-204).

## Security

- Kimlik dogrulama: musteri 68, restoran 84/106/117/128/148/160/247, kurye 190/207, teslim onay 226.
- Yetkilendirme: `order_history` customer_id sinirli (78), restoran endpoint'leri `require_restaurant` ile kendi restaurant id'sini kullanir, kurye endpoint'leri `require_courier` ile kendi courier id'sini kullanir.
- Siparis atamasi ownership kontrolu `order["restaurant_id"] != restaurant["id"]` (172).
- Kurye atamasi ownership kontrolu `courier.id = ? AND courier.restaurant_id = ?` (176-179).
- SQL injection riski dusuk; tum dinamik parametreler bound parameters ile gecirilir.
- Kurye olusturma sirasinda e-posta zaten varsa 409 dondurulur, fakat hassas bilgi olarak sadece genel mesaj var (134-135).

## Suggested changes

- Mevcut:
  `for row in rows: order = dict(row); order["items"] = [...conn.execute(...)...]`
- Onerilen:
  siparis id'leriyle `IN (...)` sorgusu tek istekte item'lari cekip Python'da gruplayin.

- Mevcut:
  `email: str, password: str, name: str`
- Onerilen:
  `email: EmailStr; password: str = Field(min_length=6); name: str = Field(min_length=1)` ve email normalized.

- Mevcut:
  `MenuItemInput` alanlari plain string.
- Onerilen:
  `name: str = Field(min_length=1); description: str = Field(min_length=1); category: str = Field(min_length=1)`.

- Mevcut:
  `u.email AS customer_email` her panel listesinde dondurulur.
- Onerilen:
  panelde sadece gerekli bilgi döndurun; gerekmiyorsa e-postayi cikarin.

## Test checklist

1. Musteri gecmisi kimliksiz istekte 401.
2. Musteri sadece kendi siparislerini gormeli.
3. Restoran sadece kendi siparislerini gormeli.
4. Status filter `"kuryede"` sadece kuryede olanlari donmeli.
5. Gecersiz status bos liste donmeli.
6. Restoran kuryeleri sadece kendi profillerinden gelmeli.
7. Baskasinin restoranina kurye eklemeye calisan restoran 403.
8. Kurye olusturma basarili durumda 201 ve yeni user/courier kayitlari olusmali.
9. Ayni e-posta ile ikinci kurye 409 olmali.
10. Ad bos kurye olusturma 400.
11. Menu item ekleme 201 ve fiyat > 0 zorunlu.
12. Fiyat 0 veya negatif menu item 422.
13. Restoran kendi siparisine kendi kuryesini atayabilmeli.
14. Baska restoran kuryesi atanmaya calisilinca 403.
15. Zaten kuryede siparise tekrar atama 409.
16. Status `"hazÄ±rlanÄ±yor"` mojibake olan siparis dogru literal atanamamali; fix sonrasi atanmali.
17. Kurye sadece kendisine atanmis siparisleri gormeli.
18. Baskasinin siparisini teslim etmeye calisan kurye 404.
19. Kuryede olmayan siparis teslim edilirken 409.
20. Teslim edilen siparis musteri tarafindan onaylaninca `"onaylandı"` olmali.
21. Baskasinin siparisini onaylayan musteri 404.
22. Kazanc raporu toplam/onayli/bekleyen degerleri dogru donmeli.