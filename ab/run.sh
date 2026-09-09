#!/usr/bin/env bash
# One A/B iteration: same prompt, same workspace snapshot, same 80k compaction
# budget, run once through the fork and once through pristine upstream Codex.
#
#   ./run.sh base|fork|nomem [reps] [task]
#
# Tasks are (seed, prompt) pairs:
#   sepet  ab/seed     + ab/prompt.txt      - audit a small FastAPI/React app
#   rs     ab/seed-rs  + ab/prompt-rs.txt   - trace the context filters in the
#                                            fork's own Rust workspace
#
# Each rep resets the workspace from the seed so the sides always start from
# byte-identical files. Only the provider's own billed usage is compared, so no
# fork-specific instrumentation enters the measurement.
set -u

AB="$(cd "$(dirname "$0")" && pwd)"
SIDE="${1:?usage: run.sh base|fork|nomem [reps] [task]}"
REPS="${2:-1}"
TASK="${3:-sepet}"
EXTRA=()

case "$TASK" in
  sepet) SEED="$AB/seed";    PROMPT="$AB/prompt.txt";    SUFFIX="";    SANDBOX=workspace-write ;;
  # The session this task reproduces (01a068f4) ran with danger-full-access, so
  # it is reproduced with the same policy; a narrower sandbox would fail
  # commands the original was allowed to run and change what the model spends.
  rs)    SEED="$AB/seed-rs"; PROMPT="$AB/prompt-rs.txt"; SUFFIX="-rs"; SANDBOX=danger-full-access ;;
  # A deliberately generative task: 22 review documents, sized to land near
  # 100k output tokens so a balance drop is large enough to read off.
  docs)  SEED="$AB/seed";    PROMPT="$AB/prompt-docs.txt"; SUFFIX="-docs"; SANDBOX=workspace-write ;;
  # Exercises the tool-output shrinker: git commit over a large tree is on the
  # allowlist and prints one line per file, which is exactly what it trims.
  git)   SEED="$AB/seed-rs"; PROMPT="$AB/prompt-git.txt";  SUFFIX="-git";  SANDBOX=danger-full-access ;;
  # Exercises apply_patch: six failures across four modules, each fix verified by re-running
  # check.py. Edit-then-verify is the shape that made the model chain a command onto a heredoc
  # and batch several patches into one call in the real sessions.
  patch) SEED="$AB/seed-patch"; PROMPT="$AB/prompt-patch.txt"; SUFFIX="-patch"; SANDBOX=workspace-write ;;
  # The same shape at the smallest size that still tempts the model to bundle: 35 lines over
  # four files, three one-line bugs. Roughly a third of `patch` to run.
  patchmini) SEED="$AB/seed-patchmini"; PROMPT="$AB/prompt-patchmini.txt"; SUFFIX="-patchmini"; SANDBOX=workspace-write ;;
  *) echo "unknown task: $TASK" >&2; exit 2 ;;
esac

case "$SIDE" in
  base)
    BIN="$AB/../codex-baseline/codex-rs/target/debug/codex.exe"
    export CODEX_HOME="C:/Users/Bedirhan/.codex-baseline"
    ;;
  fork)
    BIN="$AB/../codex/codex-rs/target/debug/suffice.exe"
    export SUFFICE_HOME="C:/Users/Bedirhan/.suffice-ab"
    ;;
  *) echo "unknown side: $SIDE" >&2; exit 2 ;;
esac

WORK="$AB/$SIDE$SUFFIX"
AB_WIN='C:\Users\Bedirhan\Desktop\agent\ab\'
WORK_WIN="$AB_WIN$SIDE$SUFFIX"

mkdir -p "$AB/logs"

for i in $(seq 1 "$REPS"); do
  rm -rf "$WORK"
  cp -r "$SEED" "$WORK"
  STAMP="$(date +%Y%m%d-%H%M%S)"
  LOG="$AB/logs/$SIDE$SUFFIX-$STAMP.log"
  echo "== $SIDE/$TASK rep $i/$REPS -> $LOG"
  "$BIN" exec \
    --cd "$WORK_WIN" \
    --skip-git-repo-check \
    -s "$SANDBOX" \
    -c approval_policy='"never"' \
    -o "$AB/logs/$SIDE$SUFFIX-$STAMP.last.md" \
    ${EXTRA[@]+"${EXTRA[@]}"} \
    - < "$PROMPT" > "$LOG" 2>&1
  echo "   exit=$? lines=$(wc -l < "$LOG")"
  # keep the deliverable: the next rep wipes the workspace
  [ -d "$WORK/review" ] && cp -r "$WORK/review" "$AB/logs/$SIDE$SUFFIX-$STAMP.review"
  for f in AUDIT.md CONTEXT-FILTERS.md; do
    [ -f "$WORK/$f" ] && cp "$WORK/$f" "$AB/logs/$SIDE$SUFFIX-$STAMP.$f"
  done
done
