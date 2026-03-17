#!/usr/bin/env bash
# Builds JSON schema files by running the builder.
# Output is captured to .logs/build/objectiveai-json-schema.txt.
#
# Usage:
#   bash objectiveai-json-schema/build.sh

set -euo pipefail

MODULE="objectiveai-json-schema"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
LOG_DIR="$REPO_ROOT/.logs/build"
LOG_FILE="$LOG_DIR/$MODULE.txt"

mkdir -p "$LOG_DIR"

if cargo run --package objectiveai-json-schema-builder > "$LOG_FILE" 2>&1; then
  echo "$MODULE: SUCCESS"
else
  echo "$MODULE: ERROR"
  exit 1
fi
