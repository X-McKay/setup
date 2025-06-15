#!/bin/bash

# Install gum (Charmbracelet Gum)
if command -v gum &>/dev/null; then
  echo "gum is already installed."
  exit 0
fi

# Add Charmbracelet APT repo and install gum
if command -v apt &>/dev/null; then
  echo "Adding Charmbracelet APT repository and installing gum..."
  sudo mkdir -p /etc/apt/keyrings
  curl -fsSL https://repo.charm.sh/apt/gpg.key | sudo gpg --dearmor -o /etc/apt/keyrings/charm.gpg
  echo "deb [signed-by=/etc/apt/keyrings/charm.gpg] https://repo.charm.sh/apt/ * *" | sudo tee /etc/apt/sources.list.d/charm.list
  sudo apt update && sudo apt install -y gum
  exit $?
fi

# Fallback: install via script from Charmbracelet
if command -v curl &>/dev/null; then
  echo "Installing gum using Charmbracelet install script..."
  curl -sSfL "https://github.com/charmbracelet/gum/releases/latest/download/gum_$(uname -s)_$(uname -m).tar.gz" | tar -xz -C /tmp
  sudo mv /tmp/gum /usr/local/bin/
  exit $?
fi

echo "Could not install gum automatically. Please install it manually from https://github.com/charmbracelet/gum"
exit 1
