#!/usr/bin/env bash
# Builds objectiveai-rs-pyo3 to dist/.
# Skips the build if the source fingerprint hasn't changed (SHA-based, like cargo).
# Output is captured to .logs/build/objectiveai-rs-pyo3.txt.
#
# Usage:
#   bash objectiveai-rs-pyo3/build.sh

set -euo pipefail

MODULE="objectiveai-rs-pyo3"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
LOG_DIR="$REPO_ROOT/.logs/build"
LOG_FILE="$LOG_DIR/$MODULE.txt"

mkdir -p "$LOG_DIR"

run() {
  # Check fingerprint — returns 1 if dist/ is up to date (not an error).
  if ! source "$SCRIPT_DIR/fingerprint.sh"; then
    return 0
  fi

  # Require maturin from repo root bin/
  MATURIN="$REPO_ROOT/bin/maturin"
  [ -x "$MATURIN" ] || { echo "ERROR: maturin not found at $MATURIN. Run 'bash build-bin.sh' from the repo root first." >&2; return 1; }

  # Build
  echo "Building maturin (release)..."
  "$MATURIN" build --release --manifest-path "$SCRIPT_DIR/Cargo.toml" --out "$SCRIPT_DIR/dist"

  # Stamp the fingerprint
  echo "$CURRENT_FP" > "$FINGERPRINT_FILE"
  echo "Build complete (fingerprint: ${CURRENT_FP:0:12}...)"
}

if run > "$LOG_FILE" 2>&1; then
  echo "$MODULE: SUCCESS"
else
  echo "$MODULE: ERROR"
  exit 1
fi
