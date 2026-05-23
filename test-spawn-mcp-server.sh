#!/usr/bin/env bash
# Spawns objectiveai-mcp-filesystem on a random free port for testing.
#
# Prints "URL PID" to stdout once the server is ready, then exits.
# The server continues running as a background process.
# Caller is responsible for killing it.
#
# Usage:
#   read URL PID < <(bash test-spawn-mcp-server.sh)
#   OBJECTIVEAI_MCP_ADDRESS=$URL ...
#   kill $PID

set -euo pipefail

# Find a free port
get_free_port() {
  python3 -c "import socket; s=socket.socket(); s.bind(('127.0.0.1',0)); print(s.getsockname()[1]); s.close()"
}

PORT=$(get_free_port)

# Build the server binary, then run from a copy so the original is not locked.
# Windows locks running executables, which blocks cargo test from relinking.
cargo build --package objectiveai-mcp-filesystem --quiet >&2
REPO_ROOT="$(cd "$(dirname "$0")" && pwd)"
BINARY="$REPO_ROOT/target/debug/objectiveai-mcp-filesystem"
if [ -f "$BINARY.exe" ]; then BINARY="$BINARY.exe"; fi
TMPDIR="$(mktemp -d)"
TMPBIN="$TMPDIR/$(basename "$BINARY")"
cp "$BINARY" "$TMPBIN"
ADDRESS=127.0.0.1 PORT="$PORT" SUPPRESS_OUTPUT=1 "$TMPBIN" &
SERVER_PID=$!

# Wait for the server to accept connections (up to 60s)
TIMEOUT=60
ELAPSED=0
while ! (echo > /dev/tcp/127.0.0.1/"$PORT") 2>/dev/null; do
  if ! kill -0 "$SERVER_PID" 2>/dev/null; then
    echo "MCP server process died" >&2
    exit 1
  fi
  if [ "$ELAPSED" -ge "$TIMEOUT" ]; then
    echo "MCP server did not become ready within ${TIMEOUT}s" >&2
    kill "$SERVER_PID" 2>/dev/null
    exit 1
  fi
  sleep 0.1
  ELAPSED=$((ELAPSED + 1))
done

echo "http://127.0.0.1:$PORT $SERVER_PID"
