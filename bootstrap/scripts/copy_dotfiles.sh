#!/bin/bash

# Ensure gum is installed
if ! command -v gum &>/dev/null; then
  echo "gum is not installed. Please install it from https://github.com/charmbracelet/gum"
  exit 1
fi

# List of files to copy to ~/.bootstrap
bootstrap_files=("exports" "aliases" "util")

# List of files to copy to home directory
home_files=("bashrc" "bash_profile" "tmux.conf" "gitconfig" "tool-versions")

# Function to backup existing files
backup_files() {
  local src_files=("$@")
  # shellcheck disable=SC2155
  local backup_dir=~/.bootstrap_back/$(date +'%Y%m%d_%H%M%S')
  mkdir -p "$backup_dir"
  for file in "${src_files[@]}"; do
    if [ -f ~/."$file" ]; then
      cp ~/."$file" "$backup_dir/"
    fi
  done
  gum style --foreground 212 --bold "Backup of existing files completed: $backup_dir"
}

# Backup existing dotfiles
gum style --foreground 212 --bold "Backing up existing dotfiles..."
backup_files "${bootstrap_files[@]}" "${home_files[@]}"

# Copy files to ~/.bootstrap
gum style --foreground 212 --bold "Copying files to ~/.bootstrap..."
mkdir -p ~/.bootstrap
for file in "${bootstrap_files[@]}"; do
  if cp ./dotfiles/"$file" ~/.bootstrap/; then
    gum style --foreground 212 --bold "Copied $file to ~/.bootstrap/"
  else
    gum style --foreground 9 --bold "Failed to copy $file to ~/.bootstrap/" >&2
    exit 1
  fi
done

# Copy files to home directory
gum style --foreground 212 --bold "Copying files to home directory..."
for file in "${home_files[@]}"; do
  if cp ./dotfiles/"$file" ~/."$file"; then
    gum style --foreground 212 --bold "Copied $file to ~/"
  else
    gum style --foreground 9 --bold "Failed to copy $file to ~/" >&2
    exit 1
  fi
done

gum style --foreground 212 --bold "Dotfiles copied and sourced successfully."
