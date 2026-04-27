#!/usr/bin/env bash
# Runs the MCP proxy compliance suite: rmcp-driven Rust integration
# tests AND @modelcontextprotocol/sdk-driven TypeScript tests, both
# against the same proxy + test-upstream subprocesses.
#
# Output is captured to .logs/test/objectiveai-mcp-proxy.txt.
# Prints exactly one line: "$MODULE: PASS N/N" or "$MODULE: FAIL N/N".
#
# Usage:
#   bash objectiveai-mcp-proxy/test.sh

set -euo pipefail

MODULE="objectiveai-mcp-proxy"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
LOG_DIR="$REPO_ROOT/.logs/test"
LOG_FILE="$LOG_DIR/$MODULE.txt"

mkdir -p "$LOG_DIR"
> "$LOG_FILE"

run_all() {
  echo "==> Building release binaries"
  cargo build --release \
    --manifest-path "$REPO_ROOT/Cargo.toml" \
    -p objectiveai-mcp-proxy \
    -p test-upstream
  echo "==> Rust integration tests (rmcp client → proxy)"
  cargo test \
    --manifest-path "$REPO_ROOT/Cargo.toml" \
    -p objectiveai-mcp-proxy \
    --tests
  echo "==> TypeScript integration tests (@modelcontextprotocol/sdk → proxy)"
  pnpm --filter tests-ts install
  pnpm --filter tests-ts run test
}

EXIT=0
run_all >> "$LOG_FILE" 2>&1 || EXIT=$?

# Strip ANSI escape codes once for vitest counting; cargo output is
# usually plain when redirected, but stripping is harmless either way.
CLEAN=$(sed 's/\x1b\[[0-9;]*m//g' "$LOG_FILE")

# Cargo: each test binary prints `test result: ok. N passed; M failed; ...`
RUST_PASSED=$(echo "$CLEAN" | grep -E '^test result:' \
  | sed -n 's/.*[^0-9]\([0-9][0-9]*\) passed.*/\1/p' \
  | awk '{s+=$1} END {print s+0}')
RUST_FAILED=$(echo "$CLEAN" | grep -E '^test result:' \
  | sed -n 's/.*[^0-9]\([0-9][0-9]*\) failed.*/\1/p' \
  | awk '{s+=$1} END {print s+0}')

# Vitest: final summary line `     Tests  18 passed (18)` (with the
# `Tests Files` line just above it being similar — narrow to lines
# starting with whitespace + `Tests ` exactly).
TS_PASSED=$(echo "$CLEAN" | grep -E '^[[:space:]]*Tests[[:space:]]' \
  | sed -n 's/.*[^0-9]\([0-9][0-9]*\) passed.*/\1/p' \
  | tail -1)
TS_FAILED=$(echo "$CLEAN" | grep -E '^[[:space:]]*Tests[[:space:]]' \
  | sed -n 's/.*[^0-9]\([0-9][0-9]*\) failed.*/\1/p' \
  | tail -1)

PASSED=$(( RUST_PASSED + ${TS_PASSED:-0} ))
FAILED=$(( RUST_FAILED + ${TS_FAILED:-0} ))
TOTAL=$(( PASSED + FAILED ))

if [ "$EXIT" -eq 0 ]; then
  if [ "$TOTAL" -gt 0 ]; then
    echo "$MODULE: PASS $PASSED/$TOTAL"
  else
    echo "$MODULE: PASS"
  fi
else
  if [ "$TOTAL" -gt 0 ]; then
    echo "$MODULE: FAIL $PASSED/$TOTAL"
  else
    echo "$MODULE: FAIL"
  fi
  exit 1
fi
