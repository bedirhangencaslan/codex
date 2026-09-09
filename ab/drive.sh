#!/usr/bin/env bash
# Alternate the sides so any drift in provider behaviour hits them equally.
#
#   ./drive.sh [reps] [side ...]
set -u
AB="$(cd "$(dirname "$0")" && pwd)"
REPS="${1:-2}"
shift || true
SIDES=("$@")
[ ${#SIDES[@]} -eq 0 ] && SIDES=(base fork)
for i in $(seq 1 "$REPS"); do
  for s in "${SIDES[@]}"; do
    "$AB/run.sh" "$s" 1
  done
done
echo "ALL DONE"
