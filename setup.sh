#!/bin/bash
# Setup CLI - Build and run the Rust CLI wrapper
# Usage: ./setup.sh [options]
#
# This script builds and runs the Rust CLI for system setup.
# On a fresh machine, run ./bootstrap.sh first.

set -e

cd "$(dirname "$0")"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

info() { echo -e "${GREEN}==>${NC} $1"; }
warn() { echo -e "${YELLOW}==>${NC} $1"; }
error() { echo -e "${RED}==>${NC} $1"; }

# Find cargo: check PATH first, then try mise
CARGO_CMD=""

if command -v cargo &>/dev/null; then
  CARGO_CMD="cargo"
elif command -v mise &>/dev/null; then
  if mise which cargo &>/dev/null 2>&1; then
    CARGO_CMD="mise exec -- cargo"
  fi
elif [[ -x "$HOME/.local/bin/mise" ]]; then
  if "$HOME/.local/bin/mise" which cargo &>/dev/null 2>&1; then
    CARGO_CMD="$HOME/.local/bin/mise exec -- cargo"
  fi
fi

if [[ -z "$CARGO_CMD" ]]; then
  error "Rust toolchain not found."
  echo ""
  echo "Run ./bootstrap.sh first to install mise, Rust, and build the setup CLI."
  exit 1
fi

# Build CLI if needed
if [[ ! -f cli/target/release/setup ]] || [[ cli/src -nt cli/target/release/setup ]]; then
  info "Building setup CLI..."
  $CARGO_CMD build --release --quiet --manifest-path cli/Cargo.toml
fi

# Run the CLI, passing through all arguments
exec ./cli/target/release/setup "$@"
