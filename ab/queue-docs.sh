#!/usr/bin/env bash
set -u
AB="$(cd "$(dirname "$0")" && pwd)"
"$AB/run.sh" base 1 docs
echo "=== BASE BITTI ==="
"$AB/run.sh" fork 1 docs
echo "=== FORK BITTI ==="
