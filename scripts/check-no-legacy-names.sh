#!/usr/bin/env bash
set -euo pipefail

python3 - <<'PY'
from pathlib import Path
import sys

LEGACY = b"acc" + b"ordeur"
SKIP_DIRS = {".git", "target", "node_modules", "dist"}
SKIP_FILES = {"CHANGELOG.md"}

hits = []
for path in Path(".").rglob("*"):
    if not path.is_file():
        continue
    if any(part in SKIP_DIRS for part in path.parts):
        continue
    if path.name in SKIP_FILES:
        continue

    try:
        content = path.read_bytes()
    except OSError:
        continue

    if LEGACY in content.lower():
        hits.append(path.as_posix())

if hits:
    legacy = LEGACY.decode("ascii")
    print(f"ERROR: found legacy name '{legacy}' in active files:", file=sys.stderr)
    for hit in hits:
        print(f"  - {hit}", file=sys.stderr)
    raise SystemExit(1)

print("No legacy names detected.")
PY
