#!/bin/bash

gum style --foreground 212 --bold "Updating package list and upgrading existing packages..."
if sudo apt update && sudo apt upgrade -y; then
  gum style --foreground 212 --bold "Package list updated and packages upgraded successfully."
else
  gum style --foreground 9 --bold "Failed to update or upgrade packages." >&2
  exit 1
fi

gum style --foreground 212 --bold "Installing extra packages..."
if sudo apt install -y ranger ripgrep jq bat; then
  gum style --foreground 212 --bold "Extra packages installed successfully."
else
  gum style --foreground 9 --bold "Failed to install extra packages." >&2
  exit 1
fi
