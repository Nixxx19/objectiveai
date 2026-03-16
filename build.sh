#!/usr/bin/env bash
# Installs build tools (wasm-pack, maturin) into ./bin/ using versions
# from [workspace.metadata.tools] in Cargo.toml.
#
# Usage:
#   bash build.sh

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")" && pwd)"

# Parse tool versions from Cargo.toml [workspace.metadata.tools]
WASM_PACK_VERSION=$(sed -n 's/^wasm-pack *= *"\(.*\)"/\1/p' "$REPO_ROOT/Cargo.toml")
MATURIN_VERSION=$(sed -n 's/^maturin *= *"\(.*\)"/\1/p' "$REPO_ROOT/Cargo.toml")

[ -n "$WASM_PACK_VERSION" ] || { echo "ERROR: Could not read wasm-pack version from Cargo.toml" >&2; exit 1; }
[ -n "$MATURIN_VERSION" ] || { echo "ERROR: Could not read maturin version from Cargo.toml" >&2; exit 1; }

BIN_DIR="$REPO_ROOT/bin"

install_if_needed() {
  local name="$1" version="$2"
  local bin="$BIN_DIR/$name"
  if [ -x "$bin" ]; then
    local installed
    installed=$("$bin" --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1)
    if [ "$installed" = "$version" ]; then
      echo "$name $version already installed, skipping."
      return
    fi
  fi
  echo "Installing $name $version..."
  cargo install "$name" --version "$version" --locked --root "$REPO_ROOT"
}

install_if_needed wasm-pack "$WASM_PACK_VERSION"
install_if_needed maturin "$MATURIN_VERSION"

echo "Done. Tools at $BIN_DIR/"
