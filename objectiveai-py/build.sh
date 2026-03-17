#!/usr/bin/env bash
# Sets up venv, installs requirements, and builds pyo3.
# Output is captured to .logs/build/objectiveai-py.txt.
#
# Usage:
#   bash objectiveai-py/build.sh

set -euo pipefail

MODULE="objectiveai-py"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
VENV_DIR="$SCRIPT_DIR/venv"
LOG_DIR="$REPO_ROOT/.logs/build"
LOG_FILE="$LOG_DIR/$MODULE.txt"

mkdir -p "$LOG_DIR"

run() {
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

  # ── install requirements if missing ─────────────────────────────────────────────

  install_if_missing() {
    local req_file="$1"
    local missing=false
    while IFS= read -r line; do
      [[ -z "$line" || "$line" == \#* || "$line" == -r* || "$line" == ../* ]] && continue
      local pkg
      pkg=$(echo "$line" | sed 's/[><=!].*//' | tr '-' '_')
      if ! "$PYTHON" -c "import $pkg" 2>/dev/null; then
        missing=true
        break
      fi
    done < "$req_file"

    if $missing; then
      echo "Installing requirements from $req_file..."
      "$PIP" install -r "$req_file" --quiet
    fi
  }

  install_if_missing "$SCRIPT_DIR/requirements.txt"

  if ! "$PYTHON" -c "import pytest" 2>/dev/null; then
    echo "Installing dev requirements..."
    "$PIP" install -r "$SCRIPT_DIR/requirements-dev.txt" --quiet
  fi

  # ── pyo3 build + install ────────────────────────────────────────────────────────

  "$PYTHON" "$SCRIPT_DIR/scripts/install_pyo3.py"
}

if run > "$LOG_FILE" 2>&1; then
  echo "$MODULE: SUCCESS"
else
  echo "$MODULE: ERROR"
  exit 1
fi
