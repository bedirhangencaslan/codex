#!/bin/sh
# Register this fork's merge drivers with the current clone.
#
# `.gitattributes` names the drivers; this teaches git what they are. Git does not carry
# driver definitions in the repository - that would let a clone run arbitrary code on
# checkout - so every clone runs this once. A driver git has not been told about is
# ignored silently, and the merge falls back to the ordinary one.
#
#     sh scripts/merge/install.sh
set -e

repo=$(git rev-parse --show-toplevel)
python=$(command -v python3 || command -v python || true)
[ -n "$python" ] || { echo "python not found on PATH; the rename driver needs it" >&2; exit 1; }
driver="$repo/scripts/merge/rename-merge.py"
[ -f "$driver" ] || { echo "missing $driver" >&2; exit 1; }

# %O base, %A ours (and the file to write), %B theirs, %L marker size, %P path.
git config merge.rename.name "fold this fork's rename, merge, restore the load-bearing names"
git config merge.rename.driver "\"$python\" \"$driver\" %O %A %B %L %P"

# Generated outputs: keep ours, report success, regenerate afterwards.
git config merge.generated.name "keep ours; the file is regenerated after the merge"
git config merge.generated.driver "true"

# `.gitattributes` itself. A conflicted attributes file is not a valid one, so git stops
# applying every rule in it and the drivers above quietly do not run. Keeping ours means
# the rules survive the merge that needs them.
git config merge.ours.name "keep ours"
git config merge.ours.driver "true"

# Replay resolutions if the same conflict is seen twice - it already saved this merge once.
git config rerere.enabled true

echo "kayitli surucular:"
git config --get-regexp '^merge\.(rename|generated)\.' | sed 's/^/  /'
echo
echo "dogrulamak icin: python scripts/merge/test-driver.py"
