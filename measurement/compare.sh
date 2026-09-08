#!/usr/bin/env bash
# Four-way agent comparison on one task, metered by one relay.
#
#   ./compare.sh [task] [reps] [agent ...]
#   ./compare.sh patch 2                     # the full matrix
#   ./compare.sh patchmini 1 suffice cline    # smoke
#
# Agents are interleaved inside the rep loop, the way drive.sh does it, so provider drift hits all
# four equally and rep 1 is already a complete matrix if the budget runs out.
#
# Every run: wipe the workspace from the seed, start a fresh relay labelled with the run, run the
# agent under a hard timeout, then decide completion objectively with `python check.py`.
set -u

AB="$(cd "$(dirname "$0")" && pwd)"
AB_WIN='C:\Users\Bedirhan\Desktop\agent\ab'
TASK="${1:-patch}"
REPS="${2:-2}"
shift 2 2>/dev/null || true
AGENTS=("$@")
[ ${#AGENTS[@]} -eq 0 ] && AGENTS=(suffice base cline opencode)

case "$TASK" in
  patch)     SEED="$AB/seed-patch";     PROMPT="$AB/prompt-patch.txt" ;;
  patchmini) SEED="$AB/seed-patchmini"; PROMPT="$AB/prompt-patchmini.txt" ;;
  *) echo "unknown task: $TASK" >&2; exit 2 ;;
esac

PORT=8788
OUT="$AB/logs4"
RESULTS="$OUT/results.tsv"
mkdir -p "$OUT"
[ -f "$RESULTS" ] || printf 'agent\trep\ttask\texit\tcompleted\tseconds\n' > "$RESULTS"

FORK_BIN="$AB/../codex/codex-rs/target/release/suffice.exe"
[ -x "$FORK_BIN" ] || FORK_BIN="$AB/../codex/codex-rs/target/debug/suffice.exe"
BASE_BIN="$AB/../codex-baseline/codex-rs/target/debug/codex.exe"
FIX_BIN="$AB/../codex-baseline/codex-rs/target/debug/codex-freeform.exe"
TIMEOUT="${TIMEOUT:-420}"

free_port() {
  # A stale listener is worse than none: the readiness probe below cannot tell it apart from ours,
  # so a run would be metered into someone else's log and silently look like it worked. This bit
  # once already. Clear the port before every run.
  local pid
  for _ in $(seq 1 10); do
    pid=$(netstat -ano 2>/dev/null | grep -E "[:.]${PORT}[[:space:]]" | grep -i listening \
          | awk '{print $NF}' | head -1)
    [ -z "$pid" ] && return 0
    taskkill //PID "$pid" //F >/dev/null 2>&1
    sleep 0.5
  done
  echo "port $PORT still held" >&2
  return 1
}

start_relay() {  # $1 label
  free_port || return 1
  rm -f "$OUT/$1.jsonl"
  python "$AB/relay.py" --port "$PORT" --label "$1" --out "$OUT/$1.jsonl" \
    > "$OUT/$1.relay.log" 2>&1 &
  RELAY_PID=$!
  for _ in $(seq 1 40); do
    if ! kill -0 "$RELAY_PID" 2>/dev/null; then
      echo "relay exited at startup:" >&2
      cat "$OUT/$1.relay.log" >&2
      return 1
    fi
    (echo > "/dev/tcp/127.0.0.1/$PORT") 2>/dev/null && return 0
    sleep 0.25
  done
  echo "relay did not come up" >&2
  return 1
}

stop_relay() {
  kill "$RELAY_PID" 2>/dev/null
  wait "$RELAY_PID" 2>/dev/null
  # A killed parent does not guarantee a drained child; wait the way queue-rs.sh does.
  for _ in $(seq 1 30); do
    tasklist 2>/dev/null | grep -qiE '^(codex|suffice)\.exe' || break
    sleep 2
  done
}

run_agent() {  # $1 agent  $2 workspace  $3 log
  local a="$1" work="$2" log="$3"
  local work_win="$AB_WIN\\cmp-$a"
  case "$a" in
    suffice)
      SUFFICE_HOME="C:/Users/Bedirhan/.suffice-cmp" timeout "$TIMEOUT" \
        "$FORK_BIN" exec --cd "$work_win" --skip-git-repo-check \
        -s workspace-write -c approval_policy='"never"' - < "$PROMPT" > "$log" 2>&1
      ;;
    basefix)
      # Upstream Codex plus one cherry-picked commit: the Chat wire's missing representation for
      # freeform tools. Isolates that wire fix from everything else the fork does, so the
      # comparison stops measuring a translation gap and starts measuring the agents.
      CODEX_HOME="C:/Users/Bedirhan/.codex-fix-cmp" timeout "$TIMEOUT" \
        "$FIX_BIN" exec --cd "$work_win" --skip-git-repo-check \
        -s workspace-write -c approval_policy='"never"' - < "$PROMPT" > "$log" 2>&1
      ;;
    base)
      CODEX_HOME="C:/Users/Bedirhan/.codex-cmp" timeout "$TIMEOUT" \
        "$BASE_BIN" exec --cd "$work_win" --skip-git-repo-check \
        -s workspace-write -c approval_policy='"never"' - < "$PROMPT" > "$log" 2>&1
      ;;
    cline)
      timeout "$TIMEOUT" cline --cwd "$work" --data-dir "$AB/.cline-cmp" \
        --auto-approve true --thinking high "$(cat "$PROMPT")" > "$log" 2>&1
      ;;
    opencode)
      (cd "$work" && timeout "$TIMEOUT" opencode run -m zairelay/glm-5.3-flash \
        "$(cat "$PROMPT")") > "$log" 2>&1
      ;;
    *) echo "unknown agent: $a" >&2; return 2 ;;
  esac
}

for rep in $(seq 1 "$REPS"); do
  for a in "${AGENTS[@]}"; do
    label="$a-rep$rep"
    work="$AB/cmp-$a"
    echo "=== $label ($TASK) ==="

    rm -rf "$work"
    cp -r "$SEED" "$work"
    # OpenCode needs its provider config beside the code it is working on.
    [ "$a" = opencode ] && cp "$AB/opencode-cmp.json" "$work/opencode.json"

    start_relay "$label" || exit 1
    t0=$(date +%s)
    run_agent "$a" "$work" "$OUT/$label.log"
    code=$?
    t1=$(date +%s)
    stop_relay

    if (cd "$work" && python check.py > "$OUT/$label.check" 2>&1); then
      done_=yes
    else
      done_=no
    fi
    secs=$((t1 - t0))
    printf '%s\t%s\t%s\t%s\t%s\t%s\n' "$a" "$rep" "$TASK" "$code" "$done_" "$secs" >> "$RESULTS"
    echo "    exit=$code completed=$done_ ${secs}s"
  done
done

echo "ALL DONE"
