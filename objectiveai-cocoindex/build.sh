#!/usr/bin/env bash
# Sets up venv and installs requirements.
# Output is captured to .logs/build/objectiveai-cocoindex.txt.
#
# Usage:
#   bash objectiveai-cocoindex/build.sh

set -euo pipefail

MODULE="objectiveai-cocoindex"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
VENV_DIR="$SCRIPT_DIR/venv"
LOG_DIR="$REPO_ROOT/.logs/build"
LOG_FILE="$LOG_DIR/$MODULE.txt"

mkdir -p "$LOG_DIR"

run() {
  # ── venv setup ──────────────────────────────────────────────────────────────────

  if [ ! -d "$VENV_DIR" ]; then
    echo "Creating virtual environment..."
    python3 -m venv "$VENV_DIR"
  fi

  # Detect venv layout AFTER the venv exists so a fresh checkout picks the right paths.
  if [ -d "$VENV_DIR/Scripts" ]; then
    PYTHON="$VENV_DIR/Scripts/python.exe"
    PIP="$VENV_DIR/Scripts/pip.exe"
  else
    PYTHON="$VENV_DIR/bin/python"
    PIP="$VENV_DIR/bin/pip"
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

  # Install requirements with the objectiveai pin redirected to the sibling
  # source. requirements.txt declares `objectiveai==X.Y.Z` (canonical pin
  # used when this package is published to PyPI), but for local dev we want
  # the sibling source so changes there are picked up immediately and we
  # don't depend on the PyPI wheel being available. We do this by feeding
  # install_if_missing a temp file that strips the objectiveai line, then
  # editable-install ../objectiveai-py separately.
  REQS_FILTERED=$(mktemp)
  grep -v '^objectiveai[[:space:]]*==' "$SCRIPT_DIR/requirements.txt" > "$REQS_FILTERED" || true
  install_if_missing "$REQS_FILTERED"
  rm -f "$REQS_FILTERED"

  if ! "$PYTHON" -c "import pytest" 2>/dev/null; then
    echo "Installing dev requirements..."
    "$PIP" install -r "$SCRIPT_DIR/requirements-dev.txt" --quiet
  fi

  # Editable install of sibling objectiveai-py (overrides the PyPI pin for
  # local dev). maturin compiles the bundled _pyo3 extension into this venv.
  if ! "$PYTHON" -c "import objectiveai" 2>/dev/null; then
    echo "Installing objectiveai from sibling source ($REPO_ROOT/objectiveai-py)..."
    "$PIP" install -e "$REPO_ROOT/objectiveai-py" --quiet
  fi
}

if run > "$LOG_FILE" 2>&1; then
  echo "$MODULE: SUCCESS"
else
  echo "$MODULE: ERROR"
  exit 1
fi
