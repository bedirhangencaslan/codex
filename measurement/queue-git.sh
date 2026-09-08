#!/usr/bin/env bash
set -u
AB="$(cd "$(dirname "$0")" && pwd)"
"$AB/run.sh" base 1 git
echo "=== BASE BITTI ==="
"$AB/run.sh" fork 1 git
echo "=== FORK BITTI ==="
