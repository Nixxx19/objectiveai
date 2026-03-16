#!/usr/bin/env bash
# Runs objectiveai-rs tests.
#
# Usage:
#   bash objectiveai-rs/test.sh
#   bash objectiveai-rs/test.sh -- --test-threads=1   # pass args to cargo test

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# Parse flags
CARGO_ARGS=()
while [[ $# -gt 0 ]]; do
  case "$1" in
    --) shift; CARGO_ARGS=("$@"); break ;;
    *)  CARGO_ARGS+=("$1"); shift ;;
  esac
done

cargo test --manifest-path "$SCRIPT_DIR/Cargo.toml" "${CARGO_ARGS[@]}"
