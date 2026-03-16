#!/usr/bin/env bash
# Spawns the API server (if needed) and runs tests.
#
# If OBJECTIVEAI_TEST_PORT is already set, uses that server as-is.
# Otherwise, spawns a new server via test-spawn-api-server.sh and kills it on exit.
#
# Usage:
#   bash objectiveai-py/test.sh                          # run all tests
#   bash objectiveai-py/test.sh --no-server              # skip API server spawning
#   bash objectiveai-py/test.sh -- -k mock_7 -vv         # pass args to pytest
#   bash objectiveai-py/test.sh --no-server -- -k mock_7 # combine flags

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
VENV_DIR="$SCRIPT_DIR/venv"

# Parse flags
NO_SERVER=false
PYTEST_ARGS=()
while [[ $# -gt 0 ]]; do
  case "$1" in
    --no-server) NO_SERVER=true; shift ;;
    --)          shift; PYTEST_ARGS=("$@"); break ;;
    *)           PYTEST_ARGS+=("$1"); shift ;;
  esac
done

# Platform-independent venv path
if [ -d "$VENV_DIR/Scripts" ]; then
  PYTHON="$VENV_DIR/Scripts/python.exe"
else
  PYTHON="$VENV_DIR/bin/python"
fi

# Spawn test server if needed
if ! $NO_SERVER && [ -z "${OBJECTIVEAI_TEST_PORT:-}" ]; then
  read -r PORT PID < <(bash "$REPO_ROOT/test-spawn-api-server.sh")
  echo "API server running on port $PORT (pid $PID)"
  trap 'echo "Shutting down API server (pid $PID)..."; kill "$PID" 2>/dev/null || true' EXIT
  export OBJECTIVEAI_TEST_PORT="$PORT"
fi

# Run tests (PYTHONPATH ensures objectiveai package is importable from repo root)
PYTHONPATH="$SCRIPT_DIR${PYTHONPATH:+:$PYTHONPATH}" "$PYTHON" -m pytest "$SCRIPT_DIR/tests/" -v --tb=long "${PYTEST_ARGS[@]}"
