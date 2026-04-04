#!/usr/bin/env bash
# Validates objectiveai-rs-cffi dist/ and copies the native runtime libraries
# into objectiveai-dotnet/ObjectiveAI/runtimes/ for .NET P/Invoke.
#
# Delegates the fingerprint check to objectiveai-rs-cffi/validate.sh.
# If dist/ is missing or stale, exits with an error — run build.sh first.
#
# Usage:
#   bash objectiveai-dotnet/scripts/install_cffi.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DOTNET_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO_ROOT="$(cd "$DOTNET_ROOT/.." && pwd)"

CFFI_DIR="$REPO_ROOT/objectiveai-rs-cffi"
SRC_RUNTIMES="$CFFI_DIR/dist/runtimes"
DST_RUNTIMES="$DOTNET_ROOT/ObjectiveAI/runtimes"
VALIDATE_SCRIPT="$CFFI_DIR/validate.sh"

# 1. Validate dist/ is up to date
bash "$VALIDATE_SCRIPT"

# 2. Check that native runtimes exist
if [ ! -d "$SRC_RUNTIMES" ]; then
  echo "ERROR: No native runtimes in objectiveai-rs-cffi/dist/runtimes/" >&2
  echo "       Run objectiveai-rs-cffi/build.sh first." >&2
  exit 1
fi

# 3. Copy each RID's native library
count=0
for rid_dir in "$SRC_RUNTIMES"/*/; do
  [ -d "$rid_dir" ] || continue
  rid=$(basename "$rid_dir")
  native_dir="$rid_dir/native"
  [ -d "$native_dir" ] || continue

  mkdir -p "$DST_RUNTIMES/$rid/native"
  cp "$native_dir"/* "$DST_RUNTIMES/$rid/native/"
  for lib in "$native_dir"/*; do
    echo "Installed $(basename "$lib") -> runtimes/$rid/native/ ($(wc -c < "$lib") bytes)"
  done
  count=$((count + 1))
done

echo "Installed $count runtime(s)"
