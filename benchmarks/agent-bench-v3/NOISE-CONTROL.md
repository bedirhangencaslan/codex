# Gürültü kontrolü — Bedirhan'ın iki kritiği ve karşılıkları

## 1. Tek-dosya darlığı → çok-dosya brownfield (çözüldü)
Eski set her projede tek `solution.py` doldurtuyordu; az yüzey = az sinyal =
yüksek model varyansı. v3 seti Tabby mantığında **brownfield**: her proje
çalışan çok-modüllü bir paket (5-9 modül, geçen çekirdek testler) ve ajan
4 goal'ü **birden çok dosyaya yayarak** ekliyor. Ölçülen şey artık gerçek:
var-olan-kodu-okuma + çok-dosyalı yama isabeti.

## 2. Araç-çıktısı-bekleme gürültüsü (önlendi + ölçülüyor)
Model uzun bir komutun çıktısını beklerken bazen düzenli kontrol isteği atar,
bazen atmayı bırakıp devam eder — bu tercih **rastgele**, ajan-verimi değil.

**Önleme (görev tasarımı):**
- Görevlerde uzun-çalışan komut yok; test paketi milisaniyeler sürer.
- `pytest.ini` her projede `addopts = -p no:cacheprovider` — pytest'in
  cache'e yazarken asılı kalıp modeli "bekleme" turuna sokmasını engeller.
- Üç ajan da AYNI sandbox + AYNI deterministik ayarla (do_sample=false) koşar.

**Ölçme (scripts/wait_noise.py):** koşum sonrası her hücrenin wire kaydından
'bekleme kontrolü' isteklerini (çok kısa yanıt + bir öncekiyle neredeyse aynı
prompt) ayıklar; rapor 'saf iş maliyeti' ile 'bekleme gürültüsü'nü AYRI verir.
Böylece iki ajan arasındaki fark, rastgele bekleme davranışına değil gerçek
verime atfedilebilir.

