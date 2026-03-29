#!/usr/bin/env bash
# Builds CLI schema modules by running the builder.
# Output is captured to .logs/build/objectiveai-cli.txt.
#
# Usage:
#   bash objectiveai-cli/build.sh

set -euo pipefail

MODULE="objectiveai-cli"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
LOG_DIR="$REPO_ROOT/.logs/build"
LOG_FILE="$LOG_DIR/$MODULE.txt"

mkdir -p "$LOG_DIR"

if cargo run --package objectiveai-cli-builder > "$LOG_FILE" 2>&1; then
  echo "$MODULE: SUCCESS"
else
  echo "$MODULE: ERROR"
  exit 1
fi
