#!/bin/bash

# Ensure gum is installed
if ! command -v gum &>/dev/null; then
  echo "gum is not installed. Please install it from https://github.com/charmbracelet/gum"
  exit 1
fi

# Check if ASDF is already installed
if [ -d "$HOME/.asdf" ]; then
  gum style --foreground 212 --bold "ASDF is already installed. Skipping installation."
else
  gum style --foreground 212 --bold "Installing ASDF version manager..."
  if git clone https://github.com/asdf-vm/asdf.git ~/.asdf --branch v0.10.0; then
    gum style --foreground 212 --bold "ASDF cloned successfully."
  else
    gum style --foreground 9 --bold "Failed to clone ASDF." >&2
    exit 1
  fi

  # shellcheck disable=SC1090
  source ~/.bashrc

  gum style --foreground 212 --bold "ASDF installation completed successfully."
fi

# Function to add plugins and install versions from .tool-versions
install_asdf_plugins() {
  local tool_versions_file=~/.tool-versions
  if [ -f "$tool_versions_file" ]; then
    # shellcheck disable=SC2034
    while read -r plugin version; do
      if asdf plugin-list | grep -q "^$plugin\$"; then
        gum style --foreground 212 --bold "Plugin $plugin is already added."
      else
        if asdf plugin-add "$plugin"; then
          gum style --foreground 212 --bold "Added plugin $plugin."
        else
          gum style --foreground 9 --bold "Failed to add plugin $plugin." >&2
          exit 1
        fi
      fi
    done <"$tool_versions_file"

    gum style --foreground 212 --bold "Installing versions specified in .tool-versions..."
    if asdf install; then
      gum style --foreground 212 --bold "Versions installed successfully."
    else
      gum style --foreground 9 --bold "Failed to install versions." >&2
      exit 1
    fi

    gum style --foreground 212 --bold "Reshimming ASDF..."
    if asdf reshim; then
      gum style --foreground 212 --bold "ASDF reshim completed successfully."
    else
      gum style --foreground 9 --bold "Failed to reshim ASDF." >&2
      exit 1
    fi
  else
    gum style --foreground 9 --bold "$tool_versions_file not found. Skipping plugin installation."
  fi
}

# Install plugins and versions
install_asdf_plugins
