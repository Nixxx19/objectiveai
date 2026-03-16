#!/usr/bin/env bash
# Builds JSON schema files by running the builder.
#
# Usage:
#   bash objectiveai-json-schema/build.sh

set -euo pipefail

cargo run --package objectiveai-json-schema-builder
