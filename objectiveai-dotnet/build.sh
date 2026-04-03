#!/usr/bin/env bash
# Generates C# types from JSON schemas, builds the native CFFI library for
# the host platform, and builds the .NET SDK.
# Output is captured to .logs/build/objectiveai-dotnet.txt.
#
# Usage:
#   bash objectiveai-dotnet/build.sh

set -euo pipefail

MODULE="objectiveai-dotnet"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
LOG_DIR="$REPO_ROOT/.logs/build"
LOG_FILE="$LOG_DIR/$MODULE.txt"

mkdir -p "$LOG_DIR"

# ---------------------------------------------------------------------------
# Detect host RID and corresponding cargo target
# ---------------------------------------------------------------------------

detect_host() {
  local os arch

  case "$(uname -s)" in
    Linux*)  os="linux" ;;
    Darwin*) os="osx" ;;
    MINGW*|MSYS*|CYGWIN*|*_NT*) os="win" ;;
    *) echo "Unsupported OS: $(uname -s)" >&2; exit 1 ;;
  esac

  case "$(uname -m)" in
    x86_64|amd64) arch="x64" ;;
    aarch64|arm64) arch="arm64" ;;
    *) echo "Unsupported arch: $(uname -m)" >&2; exit 1 ;;
  esac

  RID="${os}-${arch}"

  case "$RID" in
    win-x64)     CARGO_TARGET="x86_64-pc-windows-msvc";    LIB_NAME="objectiveai_cffi.dll" ;;
    win-arm64)   CARGO_TARGET="aarch64-pc-windows-msvc";    LIB_NAME="objectiveai_cffi.dll" ;;
    linux-x64)   CARGO_TARGET="x86_64-unknown-linux-gnu";   LIB_NAME="libobjectiveai_cffi.so" ;;
    linux-arm64) CARGO_TARGET="aarch64-unknown-linux-gnu";   LIB_NAME="libobjectiveai_cffi.so" ;;
    osx-x64)     CARGO_TARGET="x86_64-apple-darwin";        LIB_NAME="libobjectiveai_cffi.dylib" ;;
    osx-arm64)   CARGO_TARGET="aarch64-apple-darwin";       LIB_NAME="libobjectiveai_cffi.dylib" ;;
    *) echo "No cargo target mapping for RID: $RID" >&2; exit 1 ;;
  esac
}

# ---------------------------------------------------------------------------
# Build
# ---------------------------------------------------------------------------

run() {
  detect_host
  echo "Host RID: $RID (cargo target: $CARGO_TARGET)"

  # 1. Generate C# types from JSON schemas
  echo "Generating types..."
  dotnet run --project "$SCRIPT_DIR/ObjectiveAI.Builder"

  # 2. Build native CFFI for host platform
  echo "Building objectiveai-rs-cffi for $CARGO_TARGET..."
  cargo build \
    --manifest-path "$REPO_ROOT/objectiveai-rs-cffi/Cargo.toml" \
    --target "$CARGO_TARGET" \
    --release

  # 3. Copy native library into runtimes/{rid}/native/
  local src="$REPO_ROOT/target/$CARGO_TARGET/release/$LIB_NAME"
  local dst_dir="$SCRIPT_DIR/ObjectiveAI/runtimes/$RID/native"
  mkdir -p "$dst_dir"
  cp "$src" "$dst_dir/"
  echo "Installed $LIB_NAME -> runtimes/$RID/native/ ($(wc -c < "$src") bytes)"

  # 4. Build .NET SDK
  echo "Building .NET SDK..."
  dotnet build "$SCRIPT_DIR" --configuration Release
}

if run > "$LOG_FILE" 2>&1; then
  echo "$MODULE: SUCCESS"
else
  echo "$MODULE: ERROR"
  exit 1
fi
