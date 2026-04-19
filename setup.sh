#!/bin/bash
# Setup CLI - One command to set up a new development machine
# Usage: ./setup.sh [options]
#
# This script builds and runs the Rust CLI for system setup.
# If Rust is not installed, it suggests running bootstrap.sh first.

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
  warn "Rust not found. Using bash fallback..."
  echo ""

  if [[ -f bootstrap/scripts/install_modern_cli.sh ]]; then
    info "Installing modern CLI tools..."
    ./bootstrap/scripts/install_modern_cli.sh
  fi

  if [[ -f bootstrap/scripts/copy_dotfiles.sh ]]; then
    info "Installing dotfiles..."
    ./bootstrap/scripts/copy_dotfiles.sh
  fi

  info "Basic setup complete!"
  echo ""
  echo "To use the full CLI, run ./bootstrap.sh first to install mise and Rust."
  exit 0
fi

# Build CLI if needed
if [[ ! -f cli/target/release/setup ]] || [[ cli/src -nt cli/target/release/setup ]]; then
  info "Building setup CLI..."
  $CARGO_CMD build --release --quiet --manifest-path cli/Cargo.toml
fi

# Run the CLI, passing through all arguments
exec ./cli/target/release/setup "$@"
