#!/bin/bash
#
# Run Docker integration tests locally
#
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "Building Docker test image..."
docker build -f "$SCRIPT_DIR/Dockerfile" -t setup-test "$PROJECT_ROOT"

echo ""
echo "Running integration tests..."
docker run --rm setup-test
