#!/bin/bash

# Ensure default shell is bash
if [[ "${SHELL}" != "/bin/bash" ]]; then
  echo_warn "Setting shell to bash"
  chsh -s /bin/bash
fi

# Ensure gum is installed
if ! command -v gum &>/dev/null; then
  echo "gum is not installed. Please install it from https://github.com/charmbracelet/gum"
  exit 1
fi

# Function to display menu using gum
show_menu() {
  options=("Update and Install Basic Packages" "Copy Dotfiles" "Install ASDF" "Install Docker" "Update and Install Extra Packages" "Exit")
  choice=$(gum choose "${options[@]}")
  echo "$choice"
}

# Function to handle selection
handle_selection() {
  case "$1" in
  "Update and Install Basic Packages")
    ./scripts/basic_apt_installs.sh
    ;;
  "Copy Dotfiles")
    ./scripts/copy_dotfiles.sh
    ;;
  "Install ASDF")
    ./scripts/install_asdf.sh
    ;;
  "Install Docker")
    ./scripts/install_docker.sh
    ;;
  "Update and Install Extra Packages")
    ./scripts/extra_apt_installs.sh
    ;;
  "Exit")
    echo "Exiting..."
    exit 0
    ;;
  *)
    echo "Invalid option: $1"
    ;;
  esac
}

# Main loop
while true; do
  choice=$(show_menu)
  echo "You selected: $choice" # Debugging output
  handle_selection "$choice"
done
