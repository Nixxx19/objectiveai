#!/usr/bin/env bash
# Builds objectiveai-rs-wasm-js to dist/.
# Skips the build if the source fingerprint hasn't changed (SHA-based, like cargo).
#
# Usage:
#   bash objectiveai-rs-wasm-js/build.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
DIST_DIR="$SCRIPT_DIR/dist"
FINGERPRINT_FILE="$DIST_DIR/.fingerprint"

# Compute a SHA256 fingerprint of all source files that affect the build.
# This mirrors cargo's approach: hash every input, so any change is detected
# regardless of filesystem timestamps, clock skew, or copied files.
compute_fingerprint() {
  # Collect all files that affect the build output, sorted for determinism.
  # - Rust sources and Cargo.toml in this crate and its local dependency
  # - Cargo.lock (pins exact dependency versions)
  {
    find "$SCRIPT_DIR/src" -type f -name '*.rs' | sort
    echo "$SCRIPT_DIR/Cargo.toml"
    find "$REPO_ROOT/objectiveai-rs/src" -type f -name '*.rs' | sort
    echo "$REPO_ROOT/objectiveai-rs/Cargo.toml"
    echo "$REPO_ROOT/Cargo.lock"
  } | while IFS= read -r file; do
    # Hash each file's path (relative to repo root, for portability) and contents.
    # Including the path means a file rename is detected even if contents are identical.
    relpath="${file#"$REPO_ROOT/"}"
    printf '%s\n' "$relpath"
    sha256sum "$file"
  done | sha256sum | awk '{print $1}'
}

CURRENT_FP=$(compute_fingerprint)

# Check if dist/ is already up to date
if [ -f "$FINGERPRINT_FILE" ]; then
  STORED_FP=$(cat "$FINGERPRINT_FILE")
  if [ "$CURRENT_FP" = "$STORED_FP" ]; then
    echo "dist/ is up to date (fingerprint: ${CURRENT_FP:0:12}...)"
    exit 0
  fi
  echo "Fingerprint changed: ${STORED_FP:0:12}... -> ${CURRENT_FP:0:12}..."
fi

# Build
echo "Building wasm-pack (nodejs, release)..."
wasm-pack build "$SCRIPT_DIR" --target nodejs --release --out-dir dist

# Stamp the fingerprint
echo "$CURRENT_FP" > "$FINGERPRINT_FILE"
echo "Build complete (fingerprint: ${CURRENT_FP:0:12}...)"
