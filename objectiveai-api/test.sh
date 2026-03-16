#!/usr/bin/env bash
# Runs objectiveai-api tests.
#
# Usage:
#   bash objectiveai-api/test.sh
#   bash objectiveai-api/test.sh -- -k client_tests     # pass args to cargo test
#   UPDATE_SNAPSHOTS=1 bash objectiveai-api/test.sh      # regenerate all snapshots

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# If UPDATE_SNAPSHOTS is set, propagate to all 6 snapshot env vars.
if [ "${UPDATE_SNAPSHOTS:-}" = "1" ]; then
  export UPDATE_AGENT_COMPLETIONS_CLIENT_TESTS_SNAPSHOTS=1
  export UPDATE_AGENT_COMPLETIONS_MOCK_CLIENT_TESTS_SNAPSHOTS=1
  export UPDATE_VECTOR_COMPLETIONS_CLIENT_TESTS_SNAPSHOTS=1
  export UPDATE_FUNCTIONS_EXECUTIONS_CLIENT_TESTS_SNAPSHOTS=1
  export UPDATE_FUNCTIONS_INVENTIONS_CLIENT_TESTS_SNAPSHOTS=1
  export UPDATE_FUNCTIONS_INVENTIONS_RECURSIVE_CLIENT_TESTS_SNAPSHOTS=1
fi

# Parse flags
CARGO_ARGS=()
while [[ $# -gt 0 ]]; do
  case "$1" in
    --) shift; CARGO_ARGS=("$@"); break ;;
    *)  CARGO_ARGS+=("$1"); shift ;;
  esac
done

cargo test --manifest-path "$SCRIPT_DIR/Cargo.toml" "${CARGO_ARGS[@]}"
