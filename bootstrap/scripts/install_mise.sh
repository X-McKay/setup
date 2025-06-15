#!/bin/bash

# Ensure gum is installed
if ! command -v gum &>/dev/null; then
  echo "gum is not installed. Please install it from https://github.com/charmbracelet/gum"
  exit 1
fi

# Check if mise is already installed
if command -v mise &>/dev/null; then
  gum style --foreground 212 --bold "mise is already installed. Skipping installation."
else
  gum style --foreground 212 --bold "Installing mise version manager..."
  if curl https://mise.run | bash; then
    gum style --foreground 212 --bold "mise installed successfully."
    # Add mise to PATH for this session if installed to ~/.local/bin
    export PATH="$HOME/.local/bin:$PATH"
  else
    gum style --foreground 9 --bold "Failed to install mise." >&2
    exit 1
  fi

  # shellcheck disable=SC1090
  source ~/.bashrc

  gum style --foreground 212 --bold "mise installation completed successfully."
fi

# Function to add plugins and install versions from .tool-versions
install_mise_plugins() {
  local tool_versions_file=~/.tool-versions
  if [ -f "$tool_versions_file" ]; then
    gum style --foreground 212 --bold "Installing tools specified in .tool-versions using mise..."
    if mise install; then
      gum style --foreground 212 --bold "Tools installed successfully."
    else
      gum style --foreground 9 --bold "Failed to install tools." >&2
      exit 1
    fi
  else
    gum style --foreground 9 --bold ".tool-versions file not found. Skipping plugin installation." >&2
  fi
}

install_mise_plugins
