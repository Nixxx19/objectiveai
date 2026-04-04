#!/usr/bin/env bash
# Generates C# types from JSON schemas, installs native CFFI libraries,
# and builds the .NET SDK.
# Output is captured to .logs/build/objectiveai-dotnet.txt.
#
# Usage:
#   bash objectiveai-dotnet/build.sh

set -euo pipefail

MODULE="objectiveai-dotnet"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
LOG_DIR="$REPO_ROOT/.logs/build"
LOG_FILE="$LOG_DIR/$MODULE.txt"

mkdir -p "$LOG_DIR"

run() {
  # Generate types from JSON schemas
  dotnet run --project "$SCRIPT_DIR/ObjectiveAI.Builder"

  # Install CFFI native libraries
  bash "$SCRIPT_DIR/scripts/install_cffi.sh"

  # Build .NET SDK
  dotnet build "$SCRIPT_DIR" --configuration Release
}

if run > "$LOG_FILE" 2>&1; then
  echo "$MODULE: SUCCESS"
else
  echo "$MODULE: ERROR"
  exit 1
fi
