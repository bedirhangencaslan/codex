#!/usr/bin/env bash
# Tam matris: 6 proje × 3 ajan, KOLLAR SERPİŞTİRİLMİŞ (FINDINGS §3 kuralı —
# aynı projenin üç kolu art arda koşar ki oturum-içi kayma tek kola yüklenmesin).
set -uo pipefail
cd "$(dirname "$0")/.."
for proj in 01-kanbagi 02-sahra 03-cekirdek 04-mutabakat 05-destek 06-otograd; do
  for agent in suffice opencode codex; do
    echo "######## $proj / $agent — $(date +%H:%M:%S) ########"
    ./scripts/run_cell.sh "$proj" "$agent" || echo "!! hücre hata: $proj/$agent"
  done
done
echo "######## MATRİS BİTTİ — $(date +%H:%M:%S) ########"
.venv/bin/python bench.py --all
