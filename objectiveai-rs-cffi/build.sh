#!/usr/bin/env bash
# Builds objectiveai-rs-cffi to dist/ (wasm32-unknown-unknown).
# Skips the build if the source fingerprint hasn't changed (SHA-based, like cargo).
# Output is captured to .logs/build/objectiveai-rs-cffi.txt.
#
# Usage:
#   bash objectiveai-rs-cffi/build.sh

set -euo pipefail

MODULE="objectiveai-rs-cffi"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
LOG_DIR="$REPO_ROOT/.logs/build"
LOG_FILE="$LOG_DIR/$MODULE.txt"

mkdir -p "$LOG_DIR"

run() {
  # Check fingerprint — returns 1 if dist/ is up to date (not an error).
  if ! source "$SCRIPT_DIR/fingerprint.sh"; then
    return 0
  fi

  TARGET="wasm32-unknown-unknown"

  # Build
  echo "Building cffi (wasm32-unknown-unknown, release)..."
  if ! cargo build --manifest-path "$SCRIPT_DIR/Cargo.toml" --target "$TARGET" --release; then
    return 1
  fi

  # Copy wasm to dist/
  mkdir -p "$SCRIPT_DIR/dist"
  if ! cp "$REPO_ROOT/target/$TARGET/release/objectiveai_cffi.wasm" "$SCRIPT_DIR/dist/"; then
    return 1
  fi

  # Stamp the fingerprint only after successful build + copy
  echo "$CURRENT_FP" > "$FINGERPRINT_FILE"
  echo "Build complete (fingerprint: ${CURRENT_FP:0:12}...)"
}

if run > "$LOG_FILE" 2>&1; then
  echo "$MODULE: SUCCESS"
else
  echo "$MODULE: ERROR"
  exit 1
fi
