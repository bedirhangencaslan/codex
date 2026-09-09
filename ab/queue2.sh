#!/usr/bin/env bash
set -u
AB="$(cd "$(dirname "$0")" && pwd)"
"$AB/run.sh" nomem 1 sepet   # replaces the rep lost to a broken path in run.sh
"$AB/run.sh" base  1 rs
"$AB/run.sh" fork  1 rs
"$AB/run.sh" nomem 1 rs
echo "QUEUE2 DONE"
