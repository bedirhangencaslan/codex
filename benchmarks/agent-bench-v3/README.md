# Agent Bench v3 — brownfield + deterministik (28 Eylül 2026)

Bedirhan'ın iki kritiğini karşılayan set: (1) tek-dosya darlığı → **brownfield**
(çalışan çok-modüllü projeyi geliştirme, Tabby modeli — ajan 4 goal'ü birden çok
dosyaya yayarak ekliyor); (2) araç-bekleme gürültüsü önlendi + ölçüldü.

7 proje × 3 ajan (glm-5.3-flash @ Z.ai), her istek deterministik
(do_sample=false, temperature=0, kaydedici relay ile enjekte), her token
saklandı, memories kapalı, compaction 200k.

## Ana sonuç — temiz kıyas (01-06, otomatik goal skoru, 92 goal testi)

| Ajan | Goal (doğruluk) | Maliyet | Süre |
|---|---|---|---|
| codex-baseline | **92/92** | **$0,0991** | 1.360 s |
| suffice | **92/92** | $0,1224 | 1.611 s |
| opencode | **78/92** | $0,1706 | 1.742 s |

- **Artık doğruluk da ayrışıyor:** v2'de (tek-dosya) herkes tam puandı; v3'te
  opencode 01-kanbagi'de goal'lerin 7/21'ini geçebildi. Çok-dosyalı gerçek iş.
- **codex ve suffice kusursuz** (92/92); opencode 14 goal eksik VE en pahalı.
- suffice, opencode'dan **%28 ucuz** ve tam doğru.
- **415/415 istek deterministik; bekleme-gürültüsü $0** (üç ajan).

## 07-tabby (büyük ölçek) — iki anomali, ana kıyas dışında

- **suffice** 900 sn timeout'a takıldı ($0,088) — relay+keep-alive artefaktı, gerçek verim değil.
- **codex** orijinal 76 testten bazılarını değiştirdi (test-kurcalama → REGRESYON, diskalifiye).

Ham veri (her token, oturum dökümleri, cost/score/wait-noise — 126 MB) +
brownfield projeler + koşucular: https://github.com/muzafferberkesavas/agent-bench (runs-v3/)

`v3-rapor.pdf` tam rapor · `v3-data.json` hücre-hücre · `NOISE-CONTROL.md` gürültü önlemleri.
