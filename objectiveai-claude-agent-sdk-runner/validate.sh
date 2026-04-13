#!/usr/bin/env bash
# Validates that venv/ exists and its fingerprint matches the current source.
#
# Usage:
#   bash objectiveai-claude-agent-sdk-runner/validate.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
VENV_DIR="$SCRIPT_DIR/venv"
FINGERPRINT_FILE="$VENV_DIR/.fingerprint"

if [ ! -d "$VENV_DIR" ] || [ ! -f "$FINGERPRINT_FILE" ]; then
  echo "ERROR: venv/ is missing. Run build.sh first." >&2
  exit 1
fi

# Compute current fingerprint (source exits early if up to date, so we
# suppress that by only importing the compute logic).
source "$SCRIPT_DIR/fingerprint.sh" || true

STORED_FP=$(cat "$FINGERPRINT_FILE")
if [ "$CURRENT_FP" != "$STORED_FP" ]; then
  echo "ERROR: venv/ is stale. Fingerprint mismatch:" >&2
  echo "  stored:  ${STORED_FP:0:12}..." >&2
  echo "  current: ${CURRENT_FP:0:12}..." >&2
  echo "Run build.sh to rebuild." >&2
  exit 2
fi

echo "venv/ is valid (fingerprint: ${CURRENT_FP:0:12}...)"
