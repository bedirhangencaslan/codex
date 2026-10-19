# Sonuçlar — 26 Eylül 2026 koşumu

Model eşitliği: suffice / opencode / codex-baseline → **glm-5.3-flash @ Z.ai**
(fiyat: $0,15 taze / $0,03 cached / $0,50 çıktı per 1M). opus → Claude Opus
(Anthropic; ayrı model sınıfı, referans kolu). Prompt üç GLM ajanına bayt-aynı;
opus aynı metni Agent-aracı sarmalayıcısıyla aldı. Müdahale: 0. `projects/`
bütünlüğü: 24/24 hücrede temiz.

## Skorlar — 24/24 hücre TAM PUAN (85/85 × 4 ajan)

Tüm hücreler: 01: 15/15 · 02: 11/11 · 03: 12/12 · 04: 14/14 · 05: 16/16 · 06: 17/17.
Goal kırılımı `results.json`'da. Skor ayrıştırmadı; ayrışma verim metriklerinde.

## Süreler (saniye)

| Proje | suffice | opencode | codex-b | opus* |
|---|---|---|---|---|
| 01-kanbagi | 145 | 103 | **97** | 61 |
| 02-sahra | 282 | 248 | **225** | 81 |
| 03-cekirdek | 130 | **95** | 305 | 42 |
| 04-mutabakat | **140** | 1.044 | 200 | 127 |
| 05-destek | 308 | 375 | **144** | 190 |
| 06-otograd | 332 | 1.324 | **137** | 99 |
| **Toplam** | **1.337 (22,3 dk)** | 3.189 (53,2 dk) | **1.108 (18,5 dk)** | ~600 (paralelde 3,2 dk) |

\* opus farklı model sınıfı; süre kıyası yalnız ilk üç kol arasında anlamlıdır.

## Maliyet (Z.ai paneli koşum sonrası: ~$0,26)

| Kol | Kaynak | Taze tok | Cached tok | Çıktı tok | Maliyet |
|---|---|---|---|---|---|
| suffice | sidecar (KESİN) | 152.944 | 911.552 (%86) | 51.595 | **$0,076** |
| codex-b | log göstergesi güvenilmez | — | — | — | ~$0,03 (kalan bakiyeden tahmin) |
| opencode | istemci ölçmüyor | — | — | — | ~$0,15 (panel farkından) |
| opus | Anthropic token | 282.730 token (6 hücre) | — | — | ayrı fatura (plan içi) |

Suffice, 1,12M token trafiğinin %86'sını önbellekten (5× ucuz) işledi — keep-alive
+ compaction mühendisliğinin sahadaki karşılığı. opencode'un iki patlayan hücresi
(04: 17,4 dk, 06: 22 dk) maliyetin aslan payını yedi.

## Gizlilik probu (kanaryalı nettest)

| Kol | Kanarya | api.z.ai dışı temas | Sınıf |
|---|---|---|---|
| suffice | ✅ sızmadı | chatgpt.com (568 B giden/4,4 KB gelen), github.com (909 B/39,7 KB) | sürüm/duyuru kontrolü — İNDİRME şekilli, sızdırma değil |
| codex-b | ✅ sızmadı | aynı ikili (bayt-bayt aynı boyutlar → upstream ortak kod yolu) | aynı |
| opencode | ✅ sızmadı | models.opencode.ai (919 B/350 KB gelen) | model kataloğu indirme |

Not: `check_for_update_on_startup=false` bu temasları KESMEDİ — kaynağı ayrı bir
kod yolu (Bedirhan backlog'una: çağrı yerini bulup bayrağa bağlamak). Sertleştirme
istenirse `nettest/netwatch.py` allowlist-proxy olarak üretimde de sarabilir.
Sınır: TLS içeriği açılmıyor (meta veri + kanarya); içerik-düzeyi denetim için
MITM fazı ayrıca kurulur.
