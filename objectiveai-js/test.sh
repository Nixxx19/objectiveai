#!/usr/bin/env bash
# Spawns the API server (if needed) and runs tests.
#
# If OBJECTIVEAI_TEST_PORT is already set, uses that server as-is.
# Otherwise, spawns a new server via test-spawn-api-server.sh and kills it on exit.
#
# Usage:
#   bash objectiveai-js/test.sh

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

# Spawn test server only if not already provided
if [ -z "${OBJECTIVEAI_TEST_PORT:-}" ]; then
  read -r PORT PID < <(bash "$REPO_ROOT/test-spawn-api-server.sh")
  echo "API server running on port $PORT (pid $PID)"
  trap 'echo "Shutting down API server (pid $PID)..."; kill "$PID" 2>/dev/null || true' EXIT
  export OBJECTIVEAI_TEST_PORT="$PORT"
fi

npm test --workspace=objectiveai -- --reporter=verbose
