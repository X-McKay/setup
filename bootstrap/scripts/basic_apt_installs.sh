#!/bin/bash

# Get the directory where the script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"

# Source logging function if available
MAIN_SCRIPT="${SCRIPT_DIR}/../main.sh"
if [ -f "$MAIN_SCRIPT" ]; then
  # shellcheck source=../main.sh
  source "$MAIN_SCRIPT"
else
  # Fallback logging function
  log() {
    local level=$1
    local message=$2
    local timestamp
    timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo "[$timestamp] [$level] $message" | tee -a "$HOME/.setup.log"
  }
fi

# Function to verify package installation
verify_package() {
  local package=$1
  if dpkg -l | grep -q "^ii  $package "; then
    log "INFO" "Package $package verified successfully"
    return 0
  else
    log "ERROR" "Package $package installation verification failed"
    return 1
  fi
}

# Function to clean up unnecessary packages
cleanup_packages() {
  log "INFO" "Cleaning up unnecessary packages and dependencies"
  sudo apt autoremove -y
  sudo apt clean
}

# Update package list and upgrade existing packages
log "INFO" "Updating package list and upgrading existing packages..."
if ! sudo apt update; then
  log "ERROR" "Failed to update package list"
  exit 1
fi

if ! sudo apt upgrade -y; then
  log "ERROR" "Failed to upgrade packages"
  exit 1
fi

# List of essential packages to install
PACKAGES=(
  curl
  wget
  binutils
  git
  gcc
  build-essential
  libreadline-dev
  libsqlite3-dev
  zlib1g-dev
  libbz2-dev
  libffi-dev
  libssl-dev
  libncurses-dev
  xz-utils
  libedit-dev
  unzip
  lzma-dev
)

# Install packages with verification
log "INFO" "Installing basic packages..."
for package in "${PACKAGES[@]}"; do
  log "INFO" "Installing $package..."
  if sudo apt install -y "$package"; then
    if ! verify_package "$package"; then
      log "ERROR" "Failed to verify installation of $package"
      exit 1
    fi
  else
    log "ERROR" "Failed to install $package"
    exit 1
  fi
done

# Cleanup
cleanup_packages

log "INFO" "Basic package installation completed successfully"
