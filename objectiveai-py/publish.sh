#!/usr/bin/env bash
# Publishes objectiveai-py to PyPI as the `objectiveai` distribution.
# Output is captured to .logs/publish/objectiveai-py.txt.
#
# Requires twine credentials via env vars (recommended: API token):
#   TWINE_USERNAME=__token__
#   TWINE_PASSWORD=pypi-...
# ...or a configured ~/.pypirc.
#
# Usage:
#   bash objectiveai-py/publish.sh                  # upload to PyPI
#   bash objectiveai-py/publish.sh --test           # upload to TestPyPI
#   bash objectiveai-py/publish.sh --build-only     # build sdist+wheel, skip upload

set -euo pipefail

MODULE="objectiveai-py"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
VENV_DIR="$SCRIPT_DIR/venv"
DIST_DIR="$SCRIPT_DIR/dist"
LOG_DIR="$REPO_ROOT/.logs/publish"
LOG_FILE="$LOG_DIR/$MODULE.txt"

mkdir -p "$LOG_DIR"

REPOSITORY=""
BUILD_ONLY=false
while [[ $# -gt 0 ]]; do
  case "$1" in
    --test)         REPOSITORY="testpypi"; shift ;;
    --build-only)   BUILD_ONLY=true; shift ;;
    *)              echo "Unknown flag: $1" >&2; exit 1 ;;
  esac
done

run() {
  # ── ensure venv + codegen via build.sh ──────────────────────────────────────────
  bash "$SCRIPT_DIR/build.sh"

  if [ -d "$VENV_DIR/Scripts" ]; then
    PYTHON="$VENV_DIR/Scripts/python.exe"
    PIP="$VENV_DIR/Scripts/pip.exe"
  else
    PYTHON="$VENV_DIR/bin/python"
    PIP="$VENV_DIR/bin/pip"
  fi

  # ── install build/twine if missing ──────────────────────────────────────────────
  if ! "$PYTHON" -c "import build" 2>/dev/null; then
    echo "Installing 'build'..."
    "$PIP" install --quiet build
  fi
  if ! "$PYTHON" -c "import twine" 2>/dev/null; then
    echo "Installing 'twine'..."
    "$PIP" install --quiet twine
  fi

  # ── stage LICENSE + README from repo root ───────────────────────────────────────
  # pyproject.toml references local paths (sdists can't include `../`).
  cp "$REPO_ROOT/LICENSE" "$SCRIPT_DIR/LICENSE"
  cp "$REPO_ROOT/README.md" "$SCRIPT_DIR/README.md"
  trap 'rm -f "$SCRIPT_DIR/LICENSE" "$SCRIPT_DIR/README.md"' EXIT

  # ── clean previous build artifacts ──────────────────────────────────────────────
  rm -rf "$DIST_DIR" "$SCRIPT_DIR/build" "$SCRIPT_DIR"/*.egg-info

  # ── build sdist + wheel ─────────────────────────────────────────────────────────
  echo "Building sdist + wheel..."
  ( cd "$SCRIPT_DIR" && "$PYTHON" -m build --sdist --wheel --outdir "$DIST_DIR" )

  echo "Built artifacts:"
  ls -1 "$DIST_DIR"

  if $BUILD_ONLY; then
    echo "--build-only specified; skipping upload."
    return 0
  fi

  # ── upload via twine ────────────────────────────────────────────────────────────
  echo "Validating distributions..."
  "$PYTHON" -m twine check "$DIST_DIR"/*

  if [ -n "$REPOSITORY" ]; then
    echo "Uploading to $REPOSITORY..."
    "$PYTHON" -m twine upload --repository "$REPOSITORY" "$DIST_DIR"/*
  else
    echo "Uploading to PyPI..."
    "$PYTHON" -m twine upload "$DIST_DIR"/*
  fi
}

if run > "$LOG_FILE" 2>&1; then
  echo "$MODULE: PUBLISHED"
else
  echo "$MODULE: ERROR (see $LOG_FILE)"
  exit 1
fi
