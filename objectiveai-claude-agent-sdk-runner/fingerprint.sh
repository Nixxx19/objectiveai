#!/usr/bin/env bash
# Computes a SHA256 fingerprint of all source files that affect the build.
# This mirrors cargo's approach: hash every input, so any change is detected
# regardless of filesystem timestamps, clock skew, or copied files.
#
# Usage:
#   source fingerprint.sh
#
# Exports: CURRENT_FP, FINGERPRINT_FILE
# Returns 0 if fingerprint changed (build needed), 1 if up to date.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VENV_DIR="$SCRIPT_DIR/venv"
FINGERPRINT_FILE="$VENV_DIR/.fingerprint"

compute_fingerprint() {
  {
    echo "$SCRIPT_DIR/main.py"
    echo "$SCRIPT_DIR/requirements.txt"
  } | while IFS= read -r file; do
    relpath="${file#"$SCRIPT_DIR/"}"
    printf '%s\n' "$relpath"
    sha256sum "$file"
  done | sha256sum | awk '{print $1}'
}

CURRENT_FP=$(compute_fingerprint)
export CURRENT_FP FINGERPRINT_FILE

if [ -f "$FINGERPRINT_FILE" ]; then
  STORED_FP=$(cat "$FINGERPRINT_FILE")
  if [ "$CURRENT_FP" = "$STORED_FP" ]; then
    echo "venv/ is up to date (fingerprint: ${CURRENT_FP:0:12}...)"
    return 1 2>/dev/null || exit 1
  fi
  echo "Fingerprint changed: ${STORED_FP:0:12}... -> ${CURRENT_FP:0:12}..."
fi
