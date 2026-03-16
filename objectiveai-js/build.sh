#!/usr/bin/env bash
# Builds objectiveai-js.
#
# Usage:
#   bash objectiveai-js/build.sh

set -euo pipefail

npm run build --workspace=objectiveai
