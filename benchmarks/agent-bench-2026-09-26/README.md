# Agent Bench — 26 Eylül 2026 koşum sonuçları

Suffice / OpenCode 1.18.32 / Codex-Baseline (üçü glm-5.3-flash @ Z.ai, bayt-aynı
prompt) + bağımsız Claude Opus referans kolu; 6 proje × 4 goal, önceden referans
çözümle doğrulanmış 85 test.

- **Skor:** 24/24 hücre tam puan (85/85 × 4) — ayrışma verimde.
- **Süre:** codex-b 18,5 dk · suffice 22,3 dk · opencode 53,2 dk (2 hücrede patladı).
- **Maliyet:** suffice sidecar-kesin **$0,076** (girdinin %86'sı cached); panel ~$0,26,
  kalan pay ağırlıkla opencode.
- **Gizlilik (kanaryalı nettest):** sızıntı yok; API-dışı temaslar indirme-şekilli
  (sürüm kontrolü / katalog). `check_for_update_on_startup=false` bu temasları
  KESMİYOR → backlog.

Detay: `benchmark-raporu.pdf` (tam rapor) · `RESULTS.md` (tablolar) ·
`results.json` (goal kırılımı) · `opus-metrics.jsonl`.
Benchmark altyapısının tamamı (spec'ler, testler, koşucular, nettest, çözümler):
https://github.com/muzafferberkesavas/agent-bench
