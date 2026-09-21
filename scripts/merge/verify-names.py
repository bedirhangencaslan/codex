#!/usr/bin/env python3
r"""Find names the rename left half-changed, before the compiler does.

Strings and comments can say either word without consequence. Identifiers cannot: if a
declaration says one and a use says the other, the build fails - and a merge produces that
state silently, because each file on its own looks consistent.

So this looks for both spellings of the same name living in the same tree:

    X::Suffice and X::Codex          an enum whose variant is spelled both ways
    SUFFICE_FOO and CODEX_FOO        a constant renamed in one place and not another
    fn suffice_foo / fn codex_foo    likewise for functions
    class Suffice / Codex(           the Python SDK's exported class

Strings are stripped first, so `"Restart Codex"` next to `Product::Suffice` is not a finding.

    python scripts/merge/verify-names.py
    exit 0 clean, 1 if anything is half-changed
"""
import io
import os
import re
import subprocess
import sys
import collections

REPO = subprocess.run(["git", "rev-parse", "--show-toplevel"],
                      capture_output=True, encoding="utf-8",
                      cwd=os.path.dirname(os.path.abspath(__file__))).stdout.strip()

STRIP = re.compile(r'"(?:[^"\\]|\\.)*"' + r"|'(?:[^'\\]|\\.)*'"
                   + r"|//[^\n]*" + r"|/\*.*?\*/" + r"|^\s*#[^\n]*",
                   re.S | re.M)

QUALIFIED = re.compile(r"\b([A-Z][A-Za-z0-9_]*)::(Suffice|Codex)\b")
CONSTANT = re.compile(r"\b(SUFFICE|CODEX)_([A-Z0-9_]+)\b")
FUNCTION = re.compile(r"\bfn\s+(suffice|codex)_([a-z0-9_]+)")
PY_CLASS = re.compile(r"\bclass\s+(Suffice|Codex)\b")

SKIP_DIRS = {".git", "node_modules", "target", "__pycache__", "_labs", "dist", "build"}
SKIP_PREFIX = ("scripts/merge/",)
TEXT_EXT = {".rs", ".py", ".ts", ".tsx", ".js", ".toml", ".json", ".sh", ".ps1"}

qualified = collections.defaultdict(lambda: collections.defaultdict(set))
constant = collections.defaultdict(lambda: collections.defaultdict(set))
function = collections.defaultdict(lambda: collections.defaultdict(set))
pyclass = collections.defaultdict(set)

files = subprocess.run(["git", "ls-files"], cwd=REPO, capture_output=True,
                       encoding="utf-8").stdout.splitlines()
for rel in files:
    if any(p in SKIP_DIRS for p in rel.split("/")) or rel.startswith(SKIP_PREFIX):
        continue
    if os.path.splitext(rel)[1] not in TEXT_EXT:
        continue
    try:
        text = io.open(os.path.join(REPO, rel.replace("/", os.sep)),
                       encoding="utf-8", errors="replace").read()
    except (OSError, IOError):
        continue
    if "odex" not in text and "ODEX" not in text and "uffice" not in text:
        continue
    code = STRIP.sub(" ", text)
    for owner, which in QUALIFIED.findall(code):
        qualified[owner][which].add(rel)
    for which, rest in CONSTANT.findall(code):
        constant[rest][which].add(rel)
    for which, rest in FUNCTION.findall(code):
        function[rest][which].add(rel)
    for which in PY_CLASS.findall(code):
        pyclass[which].add(rel)

findings = []


def check(table, label, a, b):
    for key, sides in sorted(table.items()):
        if a in sides and b in sides:
            findings.append((label, key, sorted(sides[a])[:3], sorted(sides[b])[:3]))


check(qualified, "enum/varyant", "Suffice", "Codex")
check(constant, "sabit", "SUFFICE", "CODEX")
check(function, "fonksiyon", "suffice", "codex")
if "Suffice" in pyclass and "Codex" in pyclass:
    findings.append(("python sinifi", "class", sorted(pyclass["Suffice"])[:3],
                     sorted(pyclass["Codex"])[:3]))

if not findings:
    print("tutarli: hicbir tanimlayici iki yazimla birden yasamiyor.")
    sys.exit(0)

print("YARIM KALAN %d isim:\n" % len(findings))
for label, key, a, b in findings:
    print("  %s  %s" % (label, key))
    print("     Suffice yazimi: %s" % ", ".join(a))
    print("     Codex   yazimi: %s" % ", ".join(b))
sys.exit(1)
