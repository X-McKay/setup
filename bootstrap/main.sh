#!/bin/bash

# Get the directory where the script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"

# Initialize variables for error handling
current_command=""
last_command=""

# Logging function
log() {
  local level=$1
  local message=$2
  local timestamp
  timestamp=$(date '+%Y-%m-%d %H:%M:%S')
  echo "[$timestamp] [$level] $message" | tee -a "$HOME/.setup.log"
}

# Error handling
set -e
trap 'last_command=$current_command; current_command=$BASH_COMMAND' DEBUG
trap 'if [ $? -ne 0 ]; then log "ERROR" "Command failed: $last_command"; fi' EXIT

# Ensure default shell is bash
if [[ "${SHELL}" != "/bin/bash" ]]; then
  log "INFO" "Setting shell to bash"
  chsh -s /bin/bash
fi

# Function to display menu using gum if available, otherwise use select
show_menu() {
  options=(
    "Update and Install Basic Packages"
    "Copy Dotfiles"
    "Install mise"
    "Install Docker"
    "Update and Install Extra Packages"
    "Install gum"
    "Install System Monitoring Tools"
    "Setup Backup and Recovery"
    "Exit"
  )
  if command -v gum &>/dev/null; then
    choice=$(gum choose "${options[@]}")
  else
    log "INFO" "gum is not installed. Using basic menu."
    select opt in "${options[@]}"; do
      choice="$opt"
      break
    done
  fi
  echo "$choice"
}

# Function to handle selection
handle_selection() {
  log "INFO" "Processing selection: $1"
  case "$1" in
  "Update and Install Basic Packages")
    "$SCRIPT_DIR/scripts/basic_apt_installs.sh"
    ;;
  "Copy Dotfiles")
    "$SCRIPT_DIR/scripts/copy_dotfiles.sh"
    ;;
  "Install mise")
    "$SCRIPT_DIR/scripts/install_mise.sh"
    ;;
  "Install Docker")
    "$SCRIPT_DIR/scripts/install_docker.sh"
    ;;
  "Update and Install Extra Packages")
    "$SCRIPT_DIR/scripts/extra_apt_installs.sh"
    ;;
  "Install gum")
    "$SCRIPT_DIR/scripts/install_gum.sh"
    ;;
  "Install System Monitoring Tools")
    "$SCRIPT_DIR/scripts/install_monitoring.sh"
    ;;
  "Setup Backup and Recovery")
    "$SCRIPT_DIR/scripts/setup_backup.sh"
    ;;
  "Exit")
    log "INFO" "Exiting setup script"
    exit 0
    ;;
  *)
    log "ERROR" "Invalid option: $1"
    ;;
  esac
}

# Main loop
log "INFO" "Starting setup script"
while true; do
  choice=$(show_menu)
  log "INFO" "Selected option: $choice"
  handle_selection "$choice"
done
