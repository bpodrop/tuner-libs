#!/usr/bin/env bash
set -euo pipefail

if [ ! -f Cargo.toml ]; then
  echo "ERROR: missing Cargo.toml workspace manifest" >&2
  exit 1
fi

actual_members=$(python3 - <<'PY'
import tomllib

with open("Cargo.toml", "rb") as f:
    data = tomllib.load(f)

members = data.get("workspace", {}).get("members")
if not isinstance(members, list):
    print("<invalid>")
else:
    print(",".join(members))
PY
)

if [ "$actual_members" != "core,dsp" ]; then
  echo "ERROR: workspace members must be exactly core and dsp" >&2
  exit 1
fi

echo "Workspace structure OK"
