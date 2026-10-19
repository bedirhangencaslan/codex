# Agent Bench — Suffice / OpenCode / Codex karşılaştırma sahası

Dört kodlama ajanının (suffice, opencode, codex-baseline, claude-opus) **aynı
görevlerde** performansını ve maliyetini karşılaştırmak için altı proje. İlk üçü
aynı modeli (glm-5.3-flash @ Z.ai) konuşur — kıyasları saf harness verimidir;
Opus kolu farklı model sınıfından bağımsız bir referans noktasıdır (maliyeti
Anthropic token'larıyla ayrı yazılır). Her proje: arka planı olan bir SPEC,
ölçülebilir **4 goal** ve her goal için hazır pytest dosyaları. Testler önceden
doğrulanmıştır (bir referans çözüm tüm testleri geçirmiştir; referans bu repoda
yoktur — üretmek ajanların işidir).

## Projeler

| # | Proje | Konu | Neden seçildi |
|---|-------|------|----------------|
| 01 | **KANBAĞI** | Kan bağışı koordinasyonu (uyumluluk matrisi, SKT/FIFO stok, bağış uygunluğu, talep eşleştirme) | Hac-Umre tarzı, kural-yoğun gerçek alan yazılımı |
| 02 | **SAHRA** | Afet yardım lojistiği (öncelik skoru, ambar tahsisi, araç yükleme, sevkiyat planı) | İkinci alan yazılımı; paylaşımlı durum + açgözlü algoritmalar |
| 03 | **ÇEKİRDEK** | Sıfırdan sinir ağı (yalnız numpy: ileri/geri yayılım, gradyan doğrulama, eğitim, genelleme) | Sıfırdan YZ eğitim taskı; matematiksel doğruluk testlenebilir |
| 04 | **MUTABAKAT** | Fatura–banka ekstresi mutabakatı (TR sayı/tarih formatları, kur çevrimi, eşleştirme motoru, rapor) | İşletmelerin en yaygın, YZ'nin maliyet düşürdüğü ofis işi |
| 05 | **DESTEK** | Kurumsal destek talep sistemi (önceliklendirme+yönlendirme, SLA/iş saatleri, olay yeniden-oynatma + durum makinesi + eskalasyon, metrik raporu) | İleri düzey, çok-özellikli, her şirketin kurabileceği sistem |
| 06 | **OTOGRAD** | Ters-mod otomatik türev motoru (broadcasting'li backward, keyfî ifade grafikleri) + motorla spiral sınıflandırma eğitimi | Sıfırdan YZ eğitiminin ileri seviyesi |

## Klasör düzeni

```
projects/<id>/SPEC.md      görev tanımı (ajanın okuyacağı tek gerçek)
projects/<id>/GOALS.md     4 ölçülebilir goal
projects/<id>/PROMPT.txt   üç ajana da verilecek BİREBİR AYNI başlangıç promptu
projects/<id>/fixtures/    veri dosyaları (değiştirilemez)
projects/<id>/tests/       goal başına bir pytest dosyası (değiştirilemez)
solutions/<id>-<agent>/    ajanın çözümü BURAYA yazılır (solution.py zorunlu giriş noktası)
```

## Kurulum

```bash
python3 -m venv .venv && .venv/bin/pip install -r requirements.txt
```

## Bir çözümü test etme

```bash
.venv/bin/python bench.py --project 01-kanbagi --solution suffice   # tek hücre
.venv/bin/python bench.py --all                                     # 4×3 matris + tablo
```

`bench.py`, `BENCH_SOLUTION` ortam değişkenini ayarlayıp her goal dosyasını ayrı
çalıştırır ve goal başına geçen/toplam test sayısını raporlar; `results.json`'a yazar.

## Hücre koşumu (otomatik)

```bash
./scripts/run_cell.sh <proje> <ajan>     # ör: ./scripts/run_cell.sh 01-kanbagi codex
```

Betik: ajanı çözüm klasöründe `PROMPT.txt` ile headless başlatır, `logs/`'a yazar,
süreyi ölçer, `bench.py` skorunu basar ve `projects/` bütünlüğünü `git status`
ile denetler. Ajan-özel notlar:

- **Model eşitliği**: üç ajan da **glm-5.3-flash @ Z.ai** konuşur; tek gereken
  `export ZAI_API_KEY=...` (anahtar: https://z.ai/manage-apikey/apikey-list —
  Z.ai hesabına girip API Keys sayfasından oluşturulur; GLM-5.3-Flash için ayrı
  bir işlem gerekmez, anahtar tüm modelleri kapsar).
- **codex (düz)**: npm codex DEĞİL — fork'un `baseline` branch'inden derlenen
  binary (`bin/codex-baseline`): upstream codex + minimal Z.ai kablosu. Koşum
  `-c model_provider=zai -c model=glm-5.3-flash` ve izole `CODEX_HOME` ile.
  Derleme: `git worktree add ../codex-baseline origin/baseline && cd
  ../codex-baseline/codex-rs && cargo build -p codex-cli --bin codex`.
- **opus (Claude)**: bağımsız bir Claude Opus ajanı, hücre klasöründe bayt-aynı
  PROMPT.txt ile çalıştırılır (Claude Code Agent aracı üzerinden); model farklı
  olduğundan tabloda referans kolu olarak işaretlenir.
- **opencode**: `opencode run -m zai/glm-5.3-flash` (önce `opencode auth login`
  ile Z.ai anahtarı ya da ZAI_API_KEY env).
- **suffice**: `bin/suffice` (derleme: `cd <suffice-repo>/codex-rs && cargo build
  -p codex-cli --bin suffice`, binary'yi `bin/`e kopyala). Maliyet sidecar'ı için
  `~/.suffice/config.toml`'a şunu ekleyin (varsayılan KAPALI):

  ```toml
  [features]
  request_stats = true
  ```

  Z.ai anahtarı: `ZAI_API_KEY` ortam değişkeni. Koşum sonrası
  `scripts/collect_suffice_stats.py` en yeni sidecar'dan taze/önbellekli/çıktı
  token ve maliyeti basar.

## Koşum protokolü (adillik)

1. Ajan, **kendi çözüm klasöründe** başlatılır (ör. `solutions/01-kanbagi-suffice/`).
2. Başlangıç promptu olarak projenin `PROMPT.txt` içeriği **birebir** verilir —
   üç ajana da aynı bayt dizisi. Ek yönlendirme, ipucu, düzeltme turu yok;
   ajanın soru sorması "tek tur ek açıklama" hakkı olarak sayılır ve nota geçirilir.
3. Ajan `projects/<id>/` altındaki SPEC/GOALS/fixtures/tests'i **okuyabilir**,
   ancak `projects/` altına yazamaz; testler ve fixture'lar değiştirilemez.
   (Test dosyasını değiştiren çözüm diskalifiye — koşum sonrası `git status` ile denetlenir.)
4. Çözüm yalnızca kendi klasörüne yazılır; zorunlu giriş noktası `solution.py`.
5. Koşum bitince: `bench.py` skoru + aşağıdaki metrikler `RESULTS.md`'ye işlenir.

## Ölçülecek metrikler (her hücre için)

- **Goal skoru**: goal başına geçen test / toplam (bench.py üretir)
- **Maliyet**: suffice → `~/.suffice/analytics/<thread>.jsonl` sidecar toplamları;
  opencode/codex → kendi kullanım ekranları. Taze/önbellekli/çıktı token ayrı yazılır.
- **İstek sayısı** ve **duvar saati süresi** (başlangıç→son test yeşili)
- **Tur sayısı**: insan müdahalesi/yeniden prompt sayısı (ideal: 1)

Sonuç şablonu: `RESULTS.md`.

## Kurallar (çözümler için — PROMPT.txt de bunları söyler)

- Python ≥3.12, yalnız standart kütüphane; **03-cekirdek ve 06-otograd ayrıca numpy kullanır (zorunlu), başka ML kütüphanesi yasak**.
- Fixture verisini ezberleyen/testlere özel dallanan çözüm geçersizdir (kod insan
  gözüyle de incelenir).
- Determinizm şart: aynı girdi → aynı çıktı (sıralamalar SPEC'te tanımlı).
