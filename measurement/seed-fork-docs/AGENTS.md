# Sepet Calisma Notlari

Bu workspace'teki ana proje `sepet/` klasorudur. Guncel proje durumu icin
[SEPET-STATE.md](SEPET-STATE.md), detayli backend/mimari incelemesi icin
[ARCHITECTURE-REVIEW.md](ARCHITECTURE-REVIEW.md) dosyasina bak. Demo hesaplarin
tam listesi [SIFRELER.txt](sepet/SIFRELER.txt) dosyasindadir.

## Kisa Kurallar

- Kullanici Turkce konusur; cevaplar Turkce olmali.
- `sepet/` icinde Vite + React + TypeScript frontend ve FastAPI + SQLite
  backend var. Kutuphane listesini buyutmek icin once kullaniciya sor.
- `python seed.py` komutu `api/sepet.db` dosyasini silip veritabanini
  sifirdan kurar.
- Git deposu yok; buyuk dosya degisikliklerini dikkatli yap.
- Modern UI yenilemesi yapildi; parcali kozmetik dokunus yerine mevcut
  tasarim sistemini koru ve yeni ekranlari ayni sisteme bagla.
- Kod inceleme yapilirken once kaynagi oku; varsayimla risk analizi yazma.

## Aktif Konu: Mimari Iyilestirme

Son is kod okuma ve backend mimari incelemesiydi; kod degisikligi yapilmadi.
Ayrintilar [ARCHITECTURE-REVIEW.md](ARCHITECTURE-REVIEW.md) icinde.

Once yapilacaklar:

1. Siparis durumu literal'lerini normalize et. `main.py` ve `schema.sql`
   cift kodlanmis/mojibake `hazirlaniyor` dizisi icerirken `panels.py`
   dogru Unicode dizisi kullanir. Backend restart/reseed sonrasi kurye
   atamasi tikanabilir.
2. `GET /api/orders/{order_id}` auth ve ownership kontrolu olmadan adres,
   telefon ve not bilgisini donduruyor; erisimi kisitla.
3. `get_db()` icinde SQLite `PRAGMA foreign_keys = ON` ac.
4. Backend restart sonrasi cuzdan yukleme, bakiye dusme, yetersiz bakiye ve
   kurye atama akisini test et.

Ayrica bilinen riskler: `auth_token` suresiz ve logout DB tokenini gecersiz
kilmaz; para alanlari `REAL`; sepet/bakiye UI verisi siparis sonrasi eski
kabilir; repoda test dosyasi yok.

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