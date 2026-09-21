# Register this fork's merge drivers with the current clone.
#
# `.gitattributes` names the drivers; this teaches git what they are. Git does not carry
# driver definitions in the repository - that would let a clone run arbitrary code on
# checkout - so every clone runs this once. A driver git has not been told about is
# ignored silently, and the merge falls back to the ordinary one.
#
#     pwsh scripts/merge/install.ps1

$ErrorActionPreference = "Stop"
$repo = (git rev-parse --show-toplevel).Trim()
if (-not $repo) { throw "not inside a git repository" }

$python = (Get-Command python -ErrorAction SilentlyContinue).Source
if (-not $python) { $python = (Get-Command python3 -ErrorAction SilentlyContinue).Source }
if (-not $python) { throw "python not found on PATH; the rename driver needs it" }

$driver = Join-Path $repo "scripts/merge/rename-merge.py"
if (-not (Test-Path $driver)) { throw "missing $driver" }

# Forward slashes, and quoted. Git hands this line to `sh`, where a Windows path's
# backslashes are escape characters: an unquoted `C:\Users\...` arrives mangled, the driver
# never starts, and git reports the file as conflicted. That failure is silent and looks
# exactly like the driver deciding it could not merge - a trial run produced 611 conflicts
# where an ordinary merge produces 256, and this was why.
$pythonSh = $python -replace '\\', '/'
$driverSh = $driver -replace '\\', '/'

# %O base, %A ours (and the file to write), %B theirs, %L marker size, %P path.
git config merge.rename.name "fold this fork's rename, merge, restore the load-bearing names"
git config merge.rename.driver "'$pythonSh' '$driverSh' %O %A %B %L %P"

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

Write-Host "kayitli surucular:"
git config --get-regexp "^merge\.(rename|generated)\." | ForEach-Object { "  $_" }
Write-Host ""
Write-Host "dogrulamak icin: python scripts/merge/test-driver.py"
