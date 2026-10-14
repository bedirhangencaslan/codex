# Agent Bench v2 — 27 Eylül 2026 (deterministik + tam kayıtlı)

Bedirhan'ın taleplerini karşılayan yeniden koşum. Üç ajan (suffice, OpenCode
1.18.32, codex-baseline) glm-5.3-flash @ Z.ai üzerinde; **her istek deterministik**
(do_sample=false, temperature=0, kaydedici relay ile enjekte), **her giren/çıkan
token saklandı**, memories kapalı, compaction 200k, 7 proje (yeni 07-ARŞİV
uzun-bağlam görevi dahil).

## Sonuç (kesin — API usage'ından, üç ajan için de)

| Ajan | Maliyet | Süre | Taze tok | Cache % | Skor | Determinizm |
|---|---|---|---|---|---|---|
| codex-baseline | $0,0653 | 955 s | 159.751 | 83,5% | 21/21 | 61/61 |
| suffice | $0,0717 | 1.215 s | 129.695 | 87,0% | 21/21 | 57/57 |
| opencode | $0,1033 | 2.340 s | 189.954 | 83,8% | 21/21 | 70/70 |

- **Doğruluk ayrışmıyor** (94/94 test × 3, 21/21 hücre tam puan).
- **opencode en pahalı + en yavaş**; 04-mutabakat (959 s / $0,029) ve 06-otograd (682 s) patladı.
- **suffice en az taze token + %87 cache** — cache mühendisliğinin ölçümü.
- **codex-baseline bu turda toplam en ucuz** ama en çok taze token'ı o yakıyor.

Ham veri (her token, oturum dökümleri, cost/score/meta — 79 MB) benchmark
reposunda `runs-v2/` altında: https://github.com/muzafferberkesavas/agent-bench

`v2-rapor.pdf` tam rapor · `v2-data.json` hücre-hücre veri.
