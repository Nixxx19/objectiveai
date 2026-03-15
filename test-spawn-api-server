#!/usr/bin/env bash
# Spawns the objectiveai-api server on a random free port for testing.
# MOCK_DELAY_MS=0 so tests run fast.
#
# Prints "PORT PID" to stdout once the server is ready, then exits.
# The server continues running as a background process.
# Caller is responsible for killing it.
#
# Usage:
#   read PORT PID < <(bash test-spawn-api-server)
#   OBJECTIVEAI_TEST_PORT=$PORT pytest tests/
#   kill $PID

set -euo pipefail

# Find a free port
get_free_port() {
  python3 -c "import socket; s=socket.socket(); s.bind(('127.0.0.1',0)); print(s.getsockname()[1]); s.close()"
}

PORT=$(get_free_port)

# Start the server in the background
ADDRESS=127.0.0.1 PORT="$PORT" MOCK_DELAY_MS=0 \
  cargo run --package objectiveai-api &
SERVER_PID=$!

# Wait for the server to accept connections (up to 120s)
TIMEOUT=120
ELAPSED=0
while ! (echo > /dev/tcp/127.0.0.1/"$PORT") 2>/dev/null; do
  if ! kill -0 "$SERVER_PID" 2>/dev/null; then
    echo "Server process died" >&2
    exit 1
  fi
  if [ "$ELAPSED" -ge "$TIMEOUT" ]; then
    echo "Server did not become ready within ${TIMEOUT}s" >&2
    kill "$SERVER_PID" 2>/dev/null
    exit 1
  fi
  sleep 0.1
  ELAPSED=$((ELAPSED + 1))
done

echo "$PORT $SERVER_PID"
