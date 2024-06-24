#!/bin/bash

gum style --foreground 212 --bold "Updating package list and upgrading existing packages..."
if sudo apt update && sudo apt upgrade -y; then
  gum style --foreground 212 --bold "Package list updated and packages upgraded successfully."
else
  gum style --foreground 9 --bold "Failed to update or upgrade packages." >&2
  exit 1
fi

gum style --foreground 212 --bold "Installing basic packages..."
if sudo apt install -y curl wget binutils git gcc build-essential libreadline-dev libsqlite3-dev zlib1g-dev libbz2-dev libffi-dev libssl-dev libncurses-dev xz-utils libedit-dev unzip lzma-dev; then
  gum style --foreground 212 --bold "Basic packages installed successfully."
else
  gum style --foreground 9 --bold "Failed to install basic packages." >&2
  exit 1
fi
