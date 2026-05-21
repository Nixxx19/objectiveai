#!/usr/bin/env bash
# Installs the faux launcher binary as drop-in replacements for the
# four production executables under `~/.objectiveai/`:
#
#   objectiveai{.exe}         -> cargo run -p objectiveai-cli
#   objectiveai-api{.exe}     -> cargo run -p objectiveai-api
#   objectiveai-viewer{.exe}  -> cargo run -p objectiveai-viewer
#   objectiveai-mcp{.exe}     -> cargo run -p objectiveai-mcp-cli
#
# After running this script, anything that spawns those binaries
# (cli `api spawn` / `viewer spawn`, scripts, the viewer's
# embedded cli invocation, etc.) picks up local source changes
# through a fresh `cargo build` — no per-crate install step needed
# between edit and run.
#
# Backs up any existing real binary to `<name>.real$EXT` (once,
# idempotent) so you can restore later via plain `cp`.
#
# Usage:
#   bash objectiveai-faux-launcher/install.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

case "$(uname -s)" in
  CYGWIN*|MINGW*|MSYS*) EXT=".exe" ;;
  *)                    EXT=""     ;;
esac

INSTALL_DIR="$HOME/.objectiveai"

# Build the launcher with the repo root baked in.
OBJECTIVEAI_REPO_ROOT="$REPO_ROOT" \
  cargo build --release --manifest-path "$SCRIPT_DIR/Cargo.toml"

SRC="$SCRIPT_DIR/target/release/objectiveai-faux-launcher$EXT"
if [ ! -f "$SRC" ]; then
  echo "ERROR: launcher missing at $SRC" >&2
  exit 1
fi

mkdir -p "$INSTALL_DIR"

# Copy the same launcher binary to each of the 4 install paths.
# Back up any existing real binary to <name>.real$EXT once (idempotent —
# only writes the .real backup if absent, so a later run after some
# updater has overwritten the live .exe still preserves the original).
for name in objectiveai objectiveai-api objectiveai-viewer objectiveai-mcp; do
  dst="$INSTALL_DIR/$name$EXT"
  bak="$INSTALL_DIR/$name.real$EXT"
  if [ -f "$dst" ] && [ ! -f "$bak" ]; then
    cp "$dst" "$bak"
    echo "Backed up $dst -> $bak"
  fi
  cp "$SRC" "$dst"
  chmod +x "$dst" 2>/dev/null || true
  echo "Installed faux launcher: $dst (-> cargo run -p <pkg>)"
done

echo ""
echo "To restore the real binaries:"
for name in objectiveai objectiveai-api objectiveai-viewer objectiveai-mcp; do
  echo "  cp '$INSTALL_DIR/$name.real$EXT' '$INSTALL_DIR/$name$EXT'"
done
