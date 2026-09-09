#!/usr/bin/env bash
# Repeat the `wire` task on the fork alone and keep every artifact of every run.
#
#   ./run-read.sh <reps> <tag>
#
# `compare-par.sh` protects only `logs5/<agent>.jsonl` with `rm -f`; the stdout log is clobbered by
# a plain `>` and the rollout is never touched at all. That is how the stdout of two earlier read
# runs was lost, and the rollout is the only place a `read` call's arguments are recorded, because
# the exec renderer prints nothing for builtin function tools. So each run is archived immediately.
set -u

AB="$(cd "$(dirname "$0")" && pwd)"
REPS="${1:-5}"
TAG="${2:-read}"
# Third argument picks the task, so a reworded prompt can be run without editing the one that
# every earlier measurement used.
TASK="${3:-wire}"
OUT="$AB/logs5"
ARCHIVE="$AB/logs-$TAG"
mkdir -p "$ARCHIVE"

# Never let the default pick the release binary that happens to be on disk: it is older than the
# change under test, and the run would silently measure the wrong build.
FORK_BIN="${FORK_BIN:?pin FORK_BIN to the binary under test}"
[ -x "$FORK_BIN" ] || { echo "no such binary: $FORK_BIN" >&2; exit 2; }
export FORK_BIN

echo "binary : $FORK_BIN ($(date -r "$FORK_BIN" '+%Y-%m-%d %H:%M'))"
echo "reps   : $REPS -> $ARCHIVE"
echo "task   : $TASK"

for rep in $(seq 1 "$REPS"); do
  echo "=== $TAG rep $rep ==="
  "$AB/compare-par.sh" "$TASK" 1800 suffice || echo "  (compare-par.sh exited $?)"

  for ext in jsonl log relay.log; do
    [ -f "$OUT/suffice.$ext" ] && mv "$OUT/suffice.$ext" "$ARCHIVE/$TAG-rep$rep.$ext"
  done
  # The newest rollout is this run's: the session directory is written as the run ends.
  rollout=$(ls -t "$HOME"/.suffice-par/sessions/*/*/*/rollout-*.jsonl 2>/dev/null | head -1)
  [ -n "$rollout" ] && cp "$rollout" "$ARCHIVE/$TAG-rep$rep.rollout.jsonl"

  cp -r "$AB/par-suffice/WIRE.md" "$ARCHIVE/$TAG-rep$rep.WIRE.md" 2>/dev/null
  echo "  arsivlendi: $ARCHIVE/$TAG-rep$rep.*"
done
echo "ALL DONE"
