#!/usr/bin/env bash
# Builds objectiveai-rs-wasm-js to dist/.
# Skips the build if the source fingerprint hasn't changed (SHA-based, like cargo).
#
# Usage:
#   bash objectiveai-rs-wasm-js/build.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# Check fingerprint — exits early if dist/ is up to date.
source "$SCRIPT_DIR/fingerprint.sh"

# Require wasm-pack from repo root bin/
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
WASM_PACK="$REPO_ROOT/bin/wasm-pack"
[ -x "$WASM_PACK" ] || { echo "ERROR: wasm-pack not found at $WASM_PACK. Run 'bash build.sh' from the repo root first." >&2; exit 1; }

# Build
echo "Building wasm-pack (nodejs, release)..."
"$WASM_PACK" build "$SCRIPT_DIR" --target nodejs --release --out-dir dist

# Stamp the fingerprint
echo "$CURRENT_FP" > "$FINGERPRINT_FILE"
echo "Build complete (fingerprint: ${CURRENT_FP:0:12}...)"
