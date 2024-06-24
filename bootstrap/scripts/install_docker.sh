#!/bin/bash

# Ensure gum is installed
if ! command -v gum &>/dev/null; then
  echo "gum is not installed. Please install it from https://github.com/charmbracelet/gum"
  exit 1
fi

# Check if Docker is already installed
if command -v docker &>/dev/null; then
  gum style --foreground 212 --bold "Docker is already installed. Skipping installation."
else
  gum style --foreground 212 --bold "Installing Docker..."
  if sudo apt update; then
    gum style --foreground 212 --bold "Package list updated successfully."
  else
    gum style --foreground 9 --bold "Failed to update package list." >&2
    exit 1
  fi

  if sudo apt install -y apt-transport-https ca-certificates curl software-properties-common; then
    gum style --foreground 212 --bold "Required packages installed successfully."
  else
    gum style --foreground 9 --bold "Failed to install required packages." >&2
    exit 1
  fi

  if curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo gpg --dearmor -o /usr/share/keyrings/docker-archive-keyring.gpg; then
    gum style --foreground 212 --bold "Docker GPG key added successfully."
  else
    gum style --foreground 9 --bold "Failed to add Docker GPG key." >&2
    exit 1
  fi

  if echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/docker-archive-keyring.gpg] https://download.docker.com/linux/ubuntu $(lsb_release -cs) stable" | sudo tee /etc/apt/sources.list.d/docker.list >/dev/null; then
    gum style --foreground 212 --bold "Docker repository added successfully."
  else
    gum style --foreground 9 --bold "Failed to add Docker repository." >&2
    exit 1
  fi

  if sudo apt update && sudo apt install -y docker-ce; then
    gum style --foreground 212 --bold "Docker installed successfully."
  else
    gum style --foreground 9 --bold "Failed to install Docker." >&2
    exit 1
  fi

  # Add the user to the docker group
  if sudo usermod -aG docker "${USER}"; then
    gum style --foreground 212 --bold "User added to Docker group successfully. You may need to log out and back in to apply the group changes."
  else
    gum style --foreground 9 --bold "Failed to add user to Docker group." >&2
    exit 1
  fi
fi
