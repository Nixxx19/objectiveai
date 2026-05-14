#!/usr/bin/env bash
# Publishes objectiveai-viewer-sdk to npm.
#
# Dispatches the GitHub Actions workflow that builds + publishes the
# package. Local --build-only mode runs `pnpm run build && pnpm pack`
# so you can inspect the tarball without uploading.
#
# Unscoped name; the first publish needs to be a manual interactive
# `npm publish` to claim the name on npm (same as objectiveai-sdk
# needed). After that, the automation token handles updates.
#
# Usage:
#   bash objectiveai-viewer-sdk/publish.sh                # npm (via GHA)
#   bash objectiveai-viewer-sdk/publish.sh --build-only   # local build + pack
#
# Setup (one-time):
#   - NPM_TOKEN with publish access (repo secret).
#   - `gh` CLI authenticated.

set -euo pipefail

MODULE="objectiveai-viewer-sdk"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
LOG_DIR="$REPO_ROOT/.logs/publish"
LOG_FILE="$LOG_DIR/$MODULE.txt"
WORKFLOW_FILE=".github/workflows/publish-$MODULE.yml"

mkdir -p "$LOG_DIR"

BUILD_ONLY=false
while [[ $# -gt 0 ]]; do
  case "$1" in
    --test)         echo "ERROR: npm has no test registry; --test is not supported." >&2; exit 1 ;;
    --build-only)   BUILD_ONLY=true; shift ;;
    *)              echo "Unknown flag: $1" >&2; exit 1 ;;
  esac
done

if $BUILD_ONLY; then
  run_local() {
    echo "Running pnpm run build..."
    ( cd "$SCRIPT_DIR" && pnpm run build ) || return $?
    echo "Running pnpm pack..."
    ( cd "$SCRIPT_DIR" && pnpm pack ) || return $?
  }

  if run_local > "$LOG_FILE" 2>&1; then
    echo "$MODULE: BUILT (pnpm pack)"
  else
    echo "$MODULE: ERROR (see $LOG_FILE)"
    exit 1
  fi
  exit 0
fi

run_remote() {
  if ! command -v gh >/dev/null 2>&1; then
    echo "ERROR: gh CLI not found. Install it (https://cli.github.com/) or use --build-only." >&2
    return 1
  fi

  if ! gh auth status >/dev/null 2>&1; then
    echo "ERROR: gh CLI not authenticated. Run 'gh auth login' first." >&2
    return 1
  fi

  echo "Triggering $WORKFLOW_FILE on default branch..."
  gh workflow run "$WORKFLOW_FILE" \
    --repo "$(gh repo view --json nameWithOwner -q .nameWithOwner)"

  echo
  echo "Workflow dispatched. Watch progress with:"
  echo "  gh run list --workflow=$WORKFLOW_FILE"
  echo "  gh run watch \$(gh run list --workflow=$WORKFLOW_FILE --limit=1 --json databaseId -q '.[0].databaseId')"
}

if run_remote 2>&1 | tee "$LOG_FILE"; then
  echo "$MODULE: WORKFLOW DISPATCHED"
else
  echo "$MODULE: ERROR (see $LOG_FILE)"
  exit 1
fi
