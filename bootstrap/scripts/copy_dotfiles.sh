#!/bin/bash
set -e

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"
DOTFILES_DIR="$SCRIPT_DIR/../dotfiles"

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m'

info() { echo -e "${BLUE}[INFO]${NC} $1"; }
success() { echo -e "${GREEN}[OK]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Files to copy to home directory (as dotfiles)
HOME_DOTFILES=(
  "bashrc:.bashrc"
  "bash_profile:.bash_profile"
  "tmux.conf:.tmux.conf"
  "gitconfig:.gitconfig"
  "tool-versions:.tool-versions"
  "aliases:.aliases"
  "exports:.exports"
  "util:.util"
)

# Files to copy to ~/.config/
CONFIG_FILES=(
  "ghostty:ghostty"
  "lazygit:lazygit"
)

# Backup directory
BACKUP_DIR="$HOME/.dotfiles_backup/$(date +'%Y%m%d_%H%M%S')"

# Function to backup a file
backup_file() {
  local target="$1"
  local backup_path
  if [ -e "$target" ]; then
    mkdir -p "$BACKUP_DIR"
    backup_path="$BACKUP_DIR/$(basename "$target")"
    cp -r "$target" "$backup_path"
    return 0
  fi
  return 1
}

# Function to copy a file/directory
copy_item() {
  local src="$1"
  local dest="$2"

  # Ensure parent directory exists
  mkdir -p "$(dirname "$dest")"

  if [ -d "$src" ]; then
    # It's a directory - remove existing and copy fresh to avoid nesting
    rm -rf "$dest"
    cp -r "$src" "$dest"
  else
    # It's a file
    cp "$src" "$dest"
  fi
}

# Main
main() {
  echo ""
  echo "========================================"
  echo "  Dotfiles Installer"
  echo "========================================"
  echo ""

  # Verify dotfiles directory exists
  if [ ! -d "$DOTFILES_DIR" ]; then
    error "Dotfiles directory not found: $DOTFILES_DIR"
    exit 1
  fi

  info "Backing up existing dotfiles to $BACKUP_DIR"

  # Copy home dotfiles
  info "Copying dotfiles to home directory..."
  for mapping in "${HOME_DOTFILES[@]}"; do
    src="${mapping%%:*}"
    dest_name="${mapping##*:}"
    src_path="$DOTFILES_DIR/$src"
    dest_path="$HOME/$dest_name"

    if [ -e "$src_path" ]; then
      backup_file "$dest_path" && info "  Backed up existing $dest_name"
      copy_item "$src_path" "$dest_path"
      success "  $dest_name"
    else
      warn "  Source not found: $src"
    fi
  done

  # Copy config files
  info "Copying config files to ~/.config/..."
  mkdir -p "$HOME/.config"

  for mapping in "${CONFIG_FILES[@]}"; do
    src="${mapping%%:*}"
    dest_name="${mapping##*:}"
    src_path="$DOTFILES_DIR/$src"
    dest_path="$HOME/.config/$dest_name"

    if [ -e "$src_path" ]; then
      backup_file "$dest_path" && info "  Backed up existing $dest_name"
      copy_item "$src_path" "$dest_path"
      success "  ~/.config/$dest_name"
    else
      warn "  Source not found: $src"
    fi
  done

  echo ""
  echo "========================================"
  success "Dotfiles installed successfully!"
  echo "========================================"
  echo ""

  if [ -d "$BACKUP_DIR" ]; then
    info "Backups saved to: $BACKUP_DIR"
  fi

  echo ""
  info "Run 'source ~/.bashrc' to apply changes"
  echo ""
}

# Run
main "$@"
