#!/usr/bin/env bash
# Runs .NET tests.
# Output is captured to .logs/test/objectiveai-dotnet.txt.
#
# Usage:
#   bash objectiveai-dotnet/test.sh

set -euo pipefail

MODULE="objectiveai-dotnet"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
LOG_DIR="$REPO_ROOT/.logs/test"
LOG_FILE="$LOG_DIR/$MODULE.txt"

mkdir -p "$LOG_DIR"

run() {
  dotnet test "$SCRIPT_DIR/ObjectiveAI.Tests" --no-build --configuration Release -v normal
}

if run > "$LOG_FILE" 2>&1; then
  PASSED=$(grep -c "Passed" "$LOG_FILE" || true)
  echo "$MODULE: PASS"
else
  echo "$MODULE: FAIL"
  exit 1
fi
