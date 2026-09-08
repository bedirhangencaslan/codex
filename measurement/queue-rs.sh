#!/usr/bin/env bash
# Wait for the in-flight round to drain, then run the Rust-workspace task on
# both sides. Sequential on purpose: two runs against the same provider at once
# risk rate-limit retries, and a retry re-sends the prompt.
set -u
AB="$(cd "$(dirname "$0")" && pwd)"
while tasklist 2>/dev/null | grep -qiE '^(codex|suffice)\.exe'; do
  sleep 20
done
echo "== previous round drained, starting the rs task"
"$AB/run.sh" base  1 rs
"$AB/run.sh" fork  1 rs
"$AB/run.sh" nomem 1 rs
echo "RS DONE"
