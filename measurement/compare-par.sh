#!/usr/bin/env bash
# Runs every agent at the same time, each behind its own relay on its own port.
#
#   ./compare-par.sh <task> <timeout-seconds> [agent ...]
#   ./compare-par.sh wire 1800
#
# Parallel is a trade: `ab/queue-rs.sh` notes that two runs against one provider at once risk
# rate-limit retries, and a retry re-sends the whole prompt, inflating exactly what we measure.
# The relay logs every response status, so a 429 or 5xx shows up in the report rather than hiding
# inside a token count. Wall clock is one run, not the sum, which is the point.
set -u

AB="$(cd "$(dirname "$0")" && pwd)"
AB_WIN='C:\Users\Bedirhan\Desktop\agent\ab'
TASK="${1:-wire}"
TIMEOUT="${2:-1800}"
shift 2 2>/dev/null || true
AGENTS=("$@")
[ ${#AGENTS[@]} -eq 0 ] && AGENTS=(suffice basefix base cline opencode)

case "$TASK" in
  wire)      SEED="$AB/seed-rs";        PROMPT="$AB/prompt-wire.txt" ;;
  patch)     SEED="$AB/seed-patch";     PROMPT="$AB/prompt-patch.txt" ;;
  patchmini) SEED="$AB/seed-patchmini"; PROMPT="$AB/prompt-patchmini.txt" ;;
  *) echo "unknown task: $TASK" >&2; exit 2 ;;
esac

OUT="$AB/logs5"
mkdir -p "$OUT"
RESULTS="$OUT/results.tsv"
[ -f "$RESULTS" ] || printf 'agent\ttask\texit\tseconds\tport\n' > "$RESULTS"

# FORK_BIN can be overridden so a freshly built debug binary can be measured against a recorded
# release run without moving the binary the user actually runs.
FORK_BIN="${FORK_BIN:-}"
if [ -z "$FORK_BIN" ]; then
  FORK_BIN="$AB/../codex/codex-rs/target/release/suffice.exe"
  [ -x "$FORK_BIN" ] || FORK_BIN="$AB/../codex/codex-rs/target/debug/suffice.exe"
fi
BASE_BIN="$AB/../codex-baseline/codex-rs/target/debug/codex.exe"
FIX_BIN="$AB/../codex-baseline/codex-rs/target/debug/codex-freeform.exe"

port_for() {
  case "$1" in
    suffice) echo 8788 ;; base) echo 8789 ;; basefix) echo 8790 ;;
    cline)   echo 8791 ;; opencode) echo 8792 ;; *) echo 8799 ;;
  esac
}

# Each codex side needs its base_url to name its own port, so every side gets its own home.
make_home() {  # $1 dir  $2 port
  mkdir -p "$HOME/$1"
  cat > "$HOME/$1/config.toml" <<TOML
model = "glm-5.3-flash"
model_provider = "zai-relay"
model_reasoning_effort = "high"
${AUTO_COMPACT:+model_auto_compact_token_limit = $AUTO_COMPACT}
${MODEL_INSTRUCTIONS_FILE:+model_instructions_file = \"$MODEL_INSTRUCTIONS_FILE\"}

[model_providers.zai-relay]
name = "Z.ai via relay"
base_url = "http://127.0.0.1:$2/api/paas/v4"
env_key = "ZAI_API_KEY"
wire_api = "chat"

[windows]
sandbox = "unelevated"
TOML
}

free_port() {
  local pid
  for _ in $(seq 1 10); do
    pid=$(netstat -ano 2>/dev/null | grep -E "[:.]${1}[[:space:]]" | grep -i listening \
          | awk '{print $NF}' | head -1)
    [ -z "$pid" ] && return 0
    taskkill //PID "$pid" //F >/dev/null 2>&1
    sleep 0.5
  done
  return 1
}

launch() {  # $1 agent
  local a="$1" port work work_win log t0 code
  port=$(port_for "$a")
  work="$AB/par-$a"
  work_win="$AB_WIN\\par-$a"
  log="$OUT/$a.log"

  # `cp -r src dst` copies *into* dst when dst survives, which silently nests the whole corpus one
  # level down and hands the agent a different task: one run failed its first command with "path not
  # found" and spent its turns hunting for the directory. A stale handle is enough to make `rm -rf`
  # fail, so the wipe is verified rather than assumed.
  rm -rf "$work"
  if [ -e "$work" ]; then
    sleep 2
    rm -rf "$work" || true
  fi
  [ -e "$work" ] && { echo "could not clear $work; a stale process is holding it" >&2; exit 3; }
  cp -r "$SEED" "$work"

  free_port "$port"
  # Truncate per run: relay.py appends, so leaving the previous file in place splices two runs into
  # one trace and every number drawn from it is wrong. This already cost one false conclusion.
  rm -f "$OUT/$a.jsonl"
  python "$AB/relay.py" --port "$port" --label "$a" --out "$OUT/$a.jsonl" \
    > "$OUT/$a.relay.log" 2>&1 &
  local relay=$!
  for _ in $(seq 1 60); do
    (echo > "/dev/tcp/127.0.0.1/$port") 2>/dev/null && break
    sleep 0.25
  done

  t0=$(date +%s)
  case "$a" in
    suffice)
      make_home ".suffice-par" "$port"
      SUFFICE_HOME="$HOME/.suffice-par" timeout "$TIMEOUT" \
        "$FORK_BIN" exec --cd "$work_win" --skip-git-repo-check \
        -s workspace-write -c approval_policy='"never"' - < "$PROMPT" > "$log" 2>&1
      ;;
    base)
      make_home ".codex-par" "$port"
      CODEX_HOME="$HOME/.codex-par" timeout "$TIMEOUT" \
        "$BASE_BIN" exec --cd "$work_win" --skip-git-repo-check \
        -s workspace-write -c approval_policy='"never"' - < "$PROMPT" > "$log" 2>&1
      ;;
    basefix)
      make_home ".codex-fix-par" "$port"
      CODEX_HOME="$HOME/.codex-fix-par" timeout "$TIMEOUT" \
        "$FIX_BIN" exec --cd "$work_win" --skip-git-repo-check \
        -s workspace-write -c approval_policy='"never"' - < "$PROMPT" > "$log" 2>&1
      ;;
    cline)
      local cdir="$AB/.cline-par"
      rm -rf "$cdir"
      cline auth -p openai-compatible -k relay-local -m glm-5.3-flash \
        -b "http://127.0.0.1:$port/v1" --data-dir "$cdir" > "$OUT/$a.auth.log" 2>&1
      timeout "$TIMEOUT" cline --cwd "$work" --data-dir "$cdir" \
        --auto-approve true --thinking high "$(cat "$PROMPT")" > "$log" 2>&1
      ;;
    opencode)
      sed "s|127.0.0.1:8788|127.0.0.1:$port|" "$AB/opencode-cmp.json" > "$work/opencode.json"
      (cd "$work" && timeout "$TIMEOUT" opencode run -m zairelay/glm-5.3-flash \
        "$(cat "$PROMPT")") > "$log" 2>&1
      ;;
  esac
  code=$?
  printf '%s\t%s\t%s\t%s\t%s\n' "$a" "$TASK" "$code" "$(( $(date +%s) - t0 ))" "$port" >> "$RESULTS"
  kill "$relay" 2>/dev/null
  echo "  [$a] bitti exit=$code"
}

echo "=== $TASK, ${TIMEOUT}s tavan, paralel: ${AGENTS[*]}"
for a in "${AGENTS[@]}"; do launch "$a" & done
wait
echo "ALL DONE"
