#!/usr/bin/env bash
# Tek hücre koşumu: ./scripts/run_cell.sh <proje> <ajan>
# Ajanı çözüm klasöründe PROMPT.txt ile headless başlatır, süreyi ölçer,
# bitince goal skorunu alır ve projects/ bütünlüğünü denetler.
#
# Üç ajan da AYNI modeli konuşur: glm-5.3-flash @ Z.ai (ZAI_API_KEY şart).
# "codex" kolu, fork'un `baseline` branch'inden derlenen binary'dir
# (upstream codex + minimal Z.ai kablosu) — npm codex değil.
set -euo pipefail
cd "$(dirname "$0")/.."
ROOT="$PWD"
[ -f "$ROOT/.env" ] && source "$ROOT/.env"
PROJ="$1"; AGENT="$2"
CELL="$ROOT/solutions/$PROJ-$AGENT"
PROMPT="$(cat "$ROOT/projects/$PROJ/PROMPT.txt")"
mkdir -p "$ROOT/logs"
LOG="$ROOT/logs/$PROJ-$AGENT.log"

: "${ZAI_API_KEY:?ZAI_API_KEY ortam değişkeni gerekli (https://z.ai/manage-apikey/apikey-list)}"

START=$(date +%s)
case "$AGENT" in
  codex)
    # baseline binary; izole home — npm codex'in ~/.codex'ine dokunmaz
    export CODEX_HOME="$ROOT/.homes/codex-baseline"
    mkdir -p "$CODEX_HOME"
    (cd "$CELL" && "$ROOT/bin/codex-baseline" exec \
        -c model_provider=zai -c model=glm-5.3-flash \
        --sandbox workspace-write --skip-git-repo-check \
        --output-last-message "$ROOT/logs/$PROJ-$AGENT.last.md" "$PROMPT") \
        >"$LOG" 2>&1 || true
    ;;
  opencode)
    (cd "$CELL" && opencode run -m zai/glm-5.3-flash "$PROMPT") >"$LOG" 2>&1 || true
    ;;
  suffice)
    # ~/.suffice/config.toml: [features] request_stats = true (maliyet sidecar'ı)
    (cd "$CELL" && "$ROOT/bin/suffice" exec --skip-git-repo-check \
        --output-last-message "$ROOT/logs/$PROJ-$AGENT.last.md" "$PROMPT") \
        >"$LOG" 2>&1 || true
    ;;
  *) echo "bilinmeyen ajan: $AGENT" >&2; exit 2 ;;
esac
END=$(date +%s)
echo "----- koşum bitti: $((END-START)) sn -----" >>"$LOG"

echo "== $PROJ / $AGENT : $((END-START)) sn =="
grep -i -m2 "model:\|glm-5.3" "$LOG" | head -2 || true   # GLM doğrulaması log'dan
.venv/bin/python bench.py --project "$PROJ" --solution "$AGENT" || true

if [ "$AGENT" = "suffice" ]; then
  .venv/bin/python scripts/collect_suffice_stats.py || true
fi

if [ -n "$(git status --porcelain projects)" ]; then
  echo "!! DİKKAT: projects/ altında değişiklik var — hücre diskalifiye adayı" | tee -a "$LOG"
  git status --porcelain projects | tee -a "$LOG"
fi
