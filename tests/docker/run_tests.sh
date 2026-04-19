#!/bin/bash
#
# Run Docker integration tests locally
#
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

if [ "${SETUP_CONTRACT_TESTS:-0}" = "1" ]; then
  echo "--- Host: component contract suite ---"
  (
    cd "$PROJECT_ROOT/cli"
    SETUP_CONTRACT_TESTS=1 \
    SETUP_MANIFEST="$PROJECT_ROOT/bootstrap/manifest.toml" \
    cargo test --test contract -- --nocapture
  )
  echo ""
else
  echo "--- Host: component contract suite skipped (set SETUP_CONTRACT_TESTS=1 to enable) ---"
  echo ""
fi

echo "Building Docker test image..."
docker build -f "$SCRIPT_DIR/Dockerfile" -t setup-test "$PROJECT_ROOT"

echo ""
echo "Running integration tests..."
docker run --rm setup-test
