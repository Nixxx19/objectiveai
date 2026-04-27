#!/usr/bin/env bash
# Run the full proxy compliance suite (Rust rmcp + TypeScript SDK).
#
# Both test suites spawn objectiveai-mcp-proxy and objectiveai-mcp-test-upstream
# binaries as subprocesses. They look in target/release first then
# target/debug, so we build release once up front for fast test cycles.
#
# Usage:
#   bash scripts/test-all.sh
#   bash scripts/test-all.sh --rust-only
#   bash scripts/test-all.sh --ts-only

set -euo pipefail

WORKSPACE="$(cd "$(dirname "$0")/.." && pwd)"
cd "$WORKSPACE"

mode="all"
case "${1:-}" in
  --rust-only) mode="rust" ;;
  --ts-only)   mode="ts" ;;
  "")          mode="all" ;;
  *) echo "unknown flag: $1" >&2; exit 2 ;;
esac

echo "==> Building release binaries"
cargo build --release \
  -p objectiveai-mcp-proxy \
  -p test-upstream

if [[ "$mode" == "all" || "$mode" == "rust" ]]; then
  echo "==> Rust integration tests (rmcp client → proxy)"
  cargo test -p objectiveai-mcp-proxy --tests
fi

if [[ "$mode" == "all" || "$mode" == "ts" ]]; then
  echo "==> TypeScript integration tests (@modelcontextprotocol/sdk client → proxy)"
  pnpm --filter tests-ts install
  pnpm --filter tests-ts run test
fi

echo "==> All compliance tests passed."
