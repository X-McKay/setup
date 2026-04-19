#!/bin/bash
# Bootstrap - Get from a fresh Ubuntu machine to a working setup CLI
#
# This script installs mise (version manager), uses it to install Rust and
# other tools defined in .tool-versions, then builds the setup CLI.
#
# Usage: ./bootstrap.sh

set -e

cd "$(dirname "$0")"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

info() { echo -e "${GREEN}==>${NC} $1"; }
warn() { echo -e "${YELLOW}==>${NC} $1"; }
error() { echo -e "${RED}==>${NC} $1"; }

# --------------------------------------------------------------------------
# 1. Install system prerequisites
# --------------------------------------------------------------------------
info "Installing system prerequisites..."
sudo apt-get update -qq
sudo apt-get install -y -qq curl git build-essential pkg-config libssl-dev >/dev/null

# --------------------------------------------------------------------------
# 2. Install mise
# --------------------------------------------------------------------------
MISE_BIN="$HOME/.local/bin/mise"

if command -v mise &>/dev/null; then
  info "mise already installed"
elif [[ -x "$MISE_BIN" ]]; then
  info "mise found at $MISE_BIN"
else
  info "Installing mise..."
  curl -fsSL https://mise.run | sh
fi

# Ensure mise is on PATH for the rest of this script
export PATH="$HOME/.local/bin:$PATH"

# --------------------------------------------------------------------------
# 3. Copy .tool-versions to home directory (if not already present)
# --------------------------------------------------------------------------
if [[ ! -f "$HOME/.tool-versions" ]]; then
  info "Copying .tool-versions to home directory..."
  cp bootstrap/dotfiles/tool-versions "$HOME/.tool-versions"
else
  # Ensure rust is in the existing .tool-versions
  if ! grep -q '^rust ' "$HOME/.tool-versions"; then
    info "Adding rust to existing .tool-versions..."
    echo "rust latest" >> "$HOME/.tool-versions"
  fi
fi

# --------------------------------------------------------------------------
# 4. Install tools via mise (Rust, Python, etc.)
# --------------------------------------------------------------------------
info "Installing tools via mise (this may take a few minutes)..."
mise install --yes

# --------------------------------------------------------------------------
# 5. Build the setup CLI
# --------------------------------------------------------------------------
info "Building setup CLI..."
mise exec -- cargo build --release --manifest-path cli/Cargo.toml

info "Bootstrap complete!"
echo ""
echo "  Run the setup CLI:"
echo "    ./setup.sh              # interactive mode"
echo "    ./setup.sh install --all -y  # install everything"
echo ""
echo "  To activate mise in your shell permanently, add to your .bashrc:"
echo "    eval \"\$(mise activate bash)\""
echo ""
