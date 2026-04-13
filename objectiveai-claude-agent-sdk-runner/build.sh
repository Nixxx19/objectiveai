#!/usr/bin/env bash
# Sets up venv and installs requirements.
# Skips the build if the source fingerprint hasn't changed (SHA-based, like cargo).
# Output is captured to .logs/build/objectiveai-claude-agent-sdk-runner.txt.
#
# Usage:
#   bash objectiveai-claude-agent-sdk-runner/build.sh

set -euo pipefail

MODULE="objectiveai-claude-agent-sdk-runner"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
VENV_DIR="$SCRIPT_DIR/venv"
LOG_DIR="$REPO_ROOT/.logs/build"
LOG_FILE="$LOG_DIR/$MODULE.txt"

mkdir -p "$LOG_DIR"

run() {
  # Check fingerprint — returns 1 if venv/ is up to date (not an error).
  if ! source "$SCRIPT_DIR/fingerprint.sh"; then
    return 0
  fi

  # Platform-independent venv paths
  if [ -d "$VENV_DIR/Scripts" ]; then
    PYTHON="$VENV_DIR/Scripts/python.exe"
    PIP="$VENV_DIR/Scripts/pip.exe"
  else
    PYTHON="$VENV_DIR/bin/python"
    PIP="$VENV_DIR/bin/pip"
  fi

  # ── venv setup ──────────────────────────────────────────────────────────────────

  if [ ! -d "$VENV_DIR" ]; then
    echo "Creating virtual environment..."
    python3 -m venv "$VENV_DIR"
  fi

  # ── install requirements ────────────────────────────────────────────────────────

  echo "Installing requirements..."
  "$PIP" install -r "$SCRIPT_DIR/requirements.txt" --quiet

  # Stamp the fingerprint only after successful build.
  echo "$CURRENT_FP" > "$FINGERPRINT_FILE"
  echo "Build complete (fingerprint: ${CURRENT_FP:0:12}...)"
}

if run > "$LOG_FILE" 2>&1; then
  echo "$MODULE: SUCCESS"
else
  echo "$MODULE: ERROR"
  exit 1
fi
