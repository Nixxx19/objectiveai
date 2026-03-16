#!/usr/bin/env bash
# Builds objectiveai-rs-pyo3 to dist/.
# Skips the build if the source fingerprint hasn't changed (SHA-based, like cargo).
#
# Usage:
#   bash objectiveai-rs-pyo3/build.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# Check fingerprint — exits early if dist/ is up to date.
source "$SCRIPT_DIR/fingerprint.sh"

# Require maturin from repo root bin/
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
MATURIN="$REPO_ROOT/bin/maturin"
[ -x "$MATURIN" ] || { echo "ERROR: maturin not found at $MATURIN. Run 'bash build.sh' from the repo root first." >&2; exit 1; }

# Build
echo "Building maturin (release)..."
"$MATURIN" build --release --manifest-path "$SCRIPT_DIR/Cargo.toml" --out "$SCRIPT_DIR/dist"

# Stamp the fingerprint
echo "$CURRENT_FP" > "$FINGERPRINT_FILE"
echo "Build complete (fingerprint: ${CURRENT_FP:0:12}...)"
