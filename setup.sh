#!/bin/bash
# Setup CLI - One command to set up a new development machine
# Usage: ./setup.sh [options]
#
# This script builds and runs the Rust CLI for system setup.
# If Rust is not installed, it falls back to bash scripts.

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

# Check if Rust/cargo is available
if ! command -v cargo &>/dev/null; then
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
  echo "To use the full CLI, install Rust first: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
  exit 0
fi

# Build CLI if needed
if [[ ! -f cli/target/release/setup ]] || [[ cli/src -nt cli/target/release/setup ]]; then
  info "Building setup CLI..."
  (cd cli && cargo build --release --quiet)
fi

# Run the CLI, passing through all arguments
exec ./cli/target/release/setup "$@"
