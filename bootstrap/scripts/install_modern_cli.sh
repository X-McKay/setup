#!/bin/bash
set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

info() { echo -e "${BLUE}[INFO]${NC} $1"; }
success() { echo -e "${GREEN}[OK]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Ensure ~/.local/bin exists and is in PATH
mkdir -p ~/.local/bin
export PATH="$HOME/.local/bin:$PATH"

# ============================================================================
# Lazygit - Terminal UI for git
# ============================================================================
install_lazygit() {
  info "Installing Lazygit..."

  if command -v lazygit &>/dev/null; then
    success "Lazygit already installed: $(lazygit --version | head -1)"
    return 0
  fi

  # Get latest version
  LAZYGIT_VERSION=$(curl -s "https://api.github.com/repos/jesseduffield/lazygit/releases/latest" | grep -Po '"tag_name": "v\K[^"]*')

  if [ -z "$LAZYGIT_VERSION" ]; then
    warn "Could not determine latest version, using 0.44.1"
    LAZYGIT_VERSION="0.44.1"
  fi

  # Detect architecture
  ARCH=$(uname -m)
  case $ARCH in
  x86_64) ARCH="x86_64" ;;
  aarch64) ARCH="arm64" ;;
  armv7l) ARCH="armv6" ;;
  *)
    error "Unsupported architecture: $ARCH"
    return 1
    ;;
  esac

  curl -Lo /tmp/lazygit.tar.gz "https://github.com/jesseduffield/lazygit/releases/latest/download/lazygit_${LAZYGIT_VERSION}_Linux_${ARCH}.tar.gz"
  tar xf /tmp/lazygit.tar.gz -C ~/.local/bin lazygit
  rm /tmp/lazygit.tar.gz

  if command -v lazygit &>/dev/null; then
    success "Lazygit installed: $(lazygit --version | head -1)"
  else
    error "Lazygit installation failed"
    return 1
  fi
}

# ============================================================================
# Eza - Modern replacement for ls (fork of exa)
# ============================================================================
install_eza() {
  info "Installing Eza..."

  if command -v eza &>/dev/null; then
    success "Eza already installed: $(eza --version | head -1)"
    return 0
  fi

  # Try apt first (Ubuntu 24.04+)
  if sudo apt install -y eza 2>/dev/null; then
    success "Eza installed via apt"
    return 0
  fi

  # Fall back to binary download
  EZA_VERSION=$(curl -s "https://api.github.com/repos/eza-community/eza/releases/latest" | grep -Po '"tag_name": "v\K[^"]*')

  if [ -z "$EZA_VERSION" ]; then
    warn "Could not determine latest version, using 0.18.0"
    EZA_VERSION="0.18.0"
  fi

  ARCH=$(uname -m)
  case $ARCH in
  x86_64) ARCH="x86_64" ;;
  aarch64) ARCH="aarch64" ;;
  *)
    error "Unsupported architecture: $ARCH"
    return 1
    ;;
  esac

  curl -Lo /tmp/eza.tar.gz "https://github.com/eza-community/eza/releases/download/v${EZA_VERSION}/eza_${ARCH}-unknown-linux-gnu.tar.gz"
  tar xf /tmp/eza.tar.gz -C ~/.local/bin
  rm /tmp/eza.tar.gz

  if command -v eza &>/dev/null; then
    success "Eza installed: $(eza --version | head -1)"
  else
    error "Eza installation failed"
    return 1
  fi
}

# ============================================================================
# Fd - Modern replacement for find
# ============================================================================
install_fd() {
  info "Installing fd..."

  if command -v fd &>/dev/null || command -v fdfind &>/dev/null; then
    success "fd already installed"
    return 0
  fi

  # Try apt first
  if sudo apt install -y fd-find 2>/dev/null; then
    # Create symlink for consistent naming
    ln -sf "$(which fdfind)" ~/.local/bin/fd 2>/dev/null || true
    success "fd installed via apt"
    return 0
  fi

  # Fall back to binary download
  FD_VERSION=$(curl -s "https://api.github.com/repos/sharkdp/fd/releases/latest" | grep -Po '"tag_name": "v\K[^"]*')

  if [ -z "$FD_VERSION" ]; then
    warn "Could not determine latest version, using 10.2.0"
    FD_VERSION="10.2.0"
  fi

  ARCH=$(uname -m)
  case $ARCH in
  x86_64) ARCH="x86_64" ;;
  aarch64) ARCH="aarch64" ;;
  *)
    error "Unsupported architecture: $ARCH"
    return 1
    ;;
  esac

  curl -Lo /tmp/fd.tar.gz "https://github.com/sharkdp/fd/releases/download/v${FD_VERSION}/fd-v${FD_VERSION}-${ARCH}-unknown-linux-musl.tar.gz"
  tar xf /tmp/fd.tar.gz -C /tmp
  mv "/tmp/fd-v${FD_VERSION}-${ARCH}-unknown-linux-musl/fd" ~/.local/bin/
  rm -rf /tmp/fd.tar.gz "/tmp/fd-v${FD_VERSION}-${ARCH}-unknown-linux-musl"

  if command -v fd &>/dev/null; then
    success "fd installed: $(fd --version)"
  else
    error "fd installation failed"
    return 1
  fi
}

# ============================================================================
# Bat - Modern replacement for cat
# ============================================================================
install_bat() {
  info "Installing bat..."

  if command -v bat &>/dev/null || command -v batcat &>/dev/null; then
    success "bat already installed"
    return 0
  fi

  # Try apt first
  if sudo apt install -y bat 2>/dev/null; then
    # Create symlink for consistent naming
    ln -sf "$(which batcat)" ~/.local/bin/bat 2>/dev/null || true
    success "bat installed via apt"
    return 0
  fi

  # Fall back to binary download
  BAT_VERSION=$(curl -s "https://api.github.com/repos/sharkdp/bat/releases/latest" | grep -Po '"tag_name": "v\K[^"]*')

  if [ -z "$BAT_VERSION" ]; then
    warn "Could not determine latest version, using 0.24.0"
    BAT_VERSION="0.24.0"
  fi

  ARCH=$(uname -m)
  case $ARCH in
  x86_64) ARCH="x86_64" ;;
  aarch64) ARCH="aarch64" ;;
  *)
    error "Unsupported architecture: $ARCH"
    return 1
    ;;
  esac

  curl -Lo /tmp/bat.tar.gz "https://github.com/sharkdp/bat/releases/download/v${BAT_VERSION}/bat-v${BAT_VERSION}-${ARCH}-unknown-linux-musl.tar.gz"
  tar xf /tmp/bat.tar.gz -C /tmp
  mv "/tmp/bat-v${BAT_VERSION}-${ARCH}-unknown-linux-musl/bat" ~/.local/bin/
  rm -rf /tmp/bat.tar.gz "/tmp/bat-v${BAT_VERSION}-${ARCH}-unknown-linux-musl"

  if command -v bat &>/dev/null; then
    success "bat installed: $(bat --version)"
  else
    error "bat installation failed"
    return 1
  fi
}

# ============================================================================
# Delta - Better git diff
# ============================================================================
install_delta() {
  info "Installing delta..."

  if command -v delta &>/dev/null; then
    success "Delta already installed: $(delta --version)"
    return 0
  fi

  DELTA_VERSION=$(curl -s "https://api.github.com/repos/dandavison/delta/releases/latest" | grep -Po '"tag_name": "\K[^"]*')

  if [ -z "$DELTA_VERSION" ]; then
    warn "Could not determine latest version, using 0.18.2"
    DELTA_VERSION="0.18.2"
  fi

  ARCH=$(uname -m)
  case $ARCH in
  x86_64) ARCH="x86_64" ;;
  aarch64) ARCH="aarch64" ;;
  *)
    error "Unsupported architecture: $ARCH"
    return 1
    ;;
  esac

  curl -Lo /tmp/delta.tar.gz "https://github.com/dandavison/delta/releases/download/${DELTA_VERSION}/delta-${DELTA_VERSION}-${ARCH}-unknown-linux-gnu.tar.gz"
  tar xf /tmp/delta.tar.gz -C /tmp
  mv "/tmp/delta-${DELTA_VERSION}-${ARCH}-unknown-linux-gnu/delta" ~/.local/bin/
  rm -rf /tmp/delta.tar.gz "/tmp/delta-${DELTA_VERSION}-${ARCH}-unknown-linux-gnu"

  if command -v delta &>/dev/null; then
    success "Delta installed: $(delta --version)"
  else
    error "Delta installation failed"
    return 1
  fi
}

# ============================================================================
# Fzf - Fuzzy finder
# ============================================================================
install_fzf() {
  info "Installing fzf..."

  if command -v fzf &>/dev/null; then
    success "fzf already installed: $(fzf --version)"
    return 0
  fi

  if sudo apt install -y fzf 2>/dev/null; then
    success "fzf installed via apt"
    return 0
  fi

  # Fall back to git install
  git clone --depth 1 https://github.com/junegunn/fzf.git ~/.fzf
  ~/.fzf/install --bin
  ln -sf ~/.fzf/bin/fzf ~/.local/bin/fzf

  if command -v fzf &>/dev/null; then
    success "fzf installed"
  else
    error "fzf installation failed"
    return 1
  fi
}

# ============================================================================
# Nerd Fonts - Required for icons
# ============================================================================
install_nerd_font() {
  info "Installing Nerd Fonts..."

  FONT_DIR="$HOME/.local/share/fonts"
  mkdir -p "$FONT_DIR"

  # Install JetBrainsMono Nerd Font (complete font with icons)
  if ls "$FONT_DIR"/JetBrainsMono*.ttf &>/dev/null; then
    success "JetBrainsMono Nerd Font already installed"
  else
    info "  Downloading JetBrainsMono Nerd Font..."
    curl -fLo "$FONT_DIR/JetBrainsMonoNerdFont-Regular.ttf" \
      "https://github.com/ryanoasis/nerd-fonts/raw/HEAD/patched-fonts/JetBrainsMono/Ligatures/Regular/JetBrainsMonoNerdFont-Regular.ttf"

    curl -fLo "$FONT_DIR/JetBrainsMonoNerdFont-Bold.ttf" \
      "https://github.com/ryanoasis/nerd-fonts/raw/HEAD/patched-fonts/JetBrainsMono/Ligatures/Bold/JetBrainsMonoNerdFont-Bold.ttf"

    curl -fLo "$FONT_DIR/JetBrainsMonoNerdFont-Italic.ttf" \
      "https://github.com/ryanoasis/nerd-fonts/raw/HEAD/patched-fonts/JetBrainsMono/Ligatures/Italic/JetBrainsMonoNerdFont-Italic.ttf"

    success "JetBrainsMono Nerd Font installed"
  fi

  # Install Symbols Nerd Font Mono (icons-only fallback font)
  if ls "$FONT_DIR"/SymbolsNerdFont*.ttf &>/dev/null; then
    success "Symbols Nerd Font Mono already installed"
  else
    info "  Downloading Symbols Nerd Font Mono..."
    curl -fLo "$FONT_DIR/SymbolsNerdFontMono-Regular.ttf" \
      "https://github.com/ryanoasis/nerd-fonts/raw/HEAD/patched-fonts/NerdFontsSymbolsOnly/SymbolsNerdFontMono-Regular.ttf"

    success "Symbols Nerd Font Mono installed"
  fi

  # Refresh font cache
  fc-cache -fv

  success "All Nerd Fonts installed"
}

# ============================================================================
# Just - Task runner (Make alternative)
# ============================================================================
install_just() {
  info "Installing Just..."

  if command -v just &>/dev/null; then
    success "Just already installed: $(just --version)"
    return 0
  fi

  # Install via prebuilt binary
  curl --proto '=https' --tlsv1.2 -sSf https://just.systems/install.sh | bash -s -- --to ~/.local/bin

  if command -v just &>/dev/null; then
    success "Just installed: $(just --version)"
  else
    error "Just installation failed"
    return 1
  fi
}

# ============================================================================
# Glow - Markdown renderer for terminal
# ============================================================================
install_glow() {
  info "Installing Glow..."

  if command -v glow &>/dev/null; then
    success "Glow already installed: $(glow --version)"
    return 0
  fi

  # Try apt first (may be available)
  if sudo apt install -y glow 2>/dev/null; then
    success "Glow installed via apt"
    return 0
  fi

  # Fall back to binary download
  GLOW_VERSION=$(curl -s "https://api.github.com/repos/charmbracelet/glow/releases/latest" | grep -Po '"tag_name": "v\K[^"]*')

  if [ -z "$GLOW_VERSION" ]; then
    warn "Could not determine latest version, using 2.0.0"
    GLOW_VERSION="2.0.0"
  fi

  ARCH=$(uname -m)
  case $ARCH in
  x86_64) ARCH="x86_64" ;;
  aarch64) ARCH="arm64" ;;
  *)
    error "Unsupported architecture: $ARCH"
    return 1
    ;;
  esac

  curl -Lo /tmp/glow.tar.gz "https://github.com/charmbracelet/glow/releases/download/v${GLOW_VERSION}/glow_${GLOW_VERSION}_Linux_${ARCH}.tar.gz"
  tar xf /tmp/glow.tar.gz -C /tmp
  mv "/tmp/glow_${GLOW_VERSION}_Linux_${ARCH}/glow" ~/.local/bin/
  rm -rf /tmp/glow.tar.gz "/tmp/glow_${GLOW_VERSION}_Linux_${ARCH}"

  if command -v glow &>/dev/null; then
    success "Glow installed: $(glow --version)"
  else
    error "Glow installation failed"
    return 1
  fi
}

# ============================================================================
# Bottom (btm) - System monitor
# ============================================================================
install_bottom() {
  info "Installing Bottom (btm)..."

  if command -v btm &>/dev/null; then
    success "Bottom already installed: $(btm --version)"
    return 0
  fi

  # Try apt first
  if sudo apt install -y bottom 2>/dev/null; then
    success "Bottom installed via apt"
    return 0
  fi

  # Fall back to binary download
  BTM_VERSION=$(curl -s "https://api.github.com/repos/ClementTsang/bottom/releases/latest" | grep -Po '"tag_name": "\K[^"]*')

  if [ -z "$BTM_VERSION" ]; then
    warn "Could not determine latest version, using 0.10.2"
    BTM_VERSION="0.10.2"
  fi

  ARCH=$(uname -m)
  case $ARCH in
  x86_64) ARCH="x86_64" ;;
  aarch64) ARCH="aarch64" ;;
  *)
    error "Unsupported architecture: $ARCH"
    return 1
    ;;
  esac

  curl -Lo /tmp/bottom.tar.gz "https://github.com/ClementTsang/bottom/releases/download/${BTM_VERSION}/bottom_${ARCH}-unknown-linux-gnu.tar.gz"
  tar xf /tmp/bottom.tar.gz -C ~/.local/bin btm
  rm /tmp/bottom.tar.gz

  if command -v btm &>/dev/null; then
    success "Bottom installed: $(btm --version)"
  else
    error "Bottom installation failed"
    return 1
  fi
}

# ============================================================================
# Hyperfine - Command benchmarking
# ============================================================================
install_hyperfine() {
  info "Installing Hyperfine..."

  if command -v hyperfine &>/dev/null; then
    success "Hyperfine already installed: $(hyperfine --version)"
    return 0
  fi

  # Try apt first
  if sudo apt install -y hyperfine 2>/dev/null; then
    success "Hyperfine installed via apt"
    return 0
  fi

  # Fall back to binary download
  HYPERFINE_VERSION=$(curl -s "https://api.github.com/repos/sharkdp/hyperfine/releases/latest" | grep -Po '"tag_name": "v\K[^"]*')

  if [ -z "$HYPERFINE_VERSION" ]; then
    warn "Could not determine latest version, using 1.18.0"
    HYPERFINE_VERSION="1.18.0"
  fi

  ARCH=$(uname -m)
  case $ARCH in
  x86_64) ARCH="x86_64" ;;
  aarch64) ARCH="aarch64" ;;
  *)
    error "Unsupported architecture: $ARCH"
    return 1
    ;;
  esac

  curl -Lo /tmp/hyperfine.tar.gz "https://github.com/sharkdp/hyperfine/releases/download/v${HYPERFINE_VERSION}/hyperfine-v${HYPERFINE_VERSION}-${ARCH}-unknown-linux-musl.tar.gz"
  tar xf /tmp/hyperfine.tar.gz -C /tmp
  mv "/tmp/hyperfine-v${HYPERFINE_VERSION}-${ARCH}-unknown-linux-musl/hyperfine" ~/.local/bin/
  rm -rf /tmp/hyperfine.tar.gz "/tmp/hyperfine-v${HYPERFINE_VERSION}-${ARCH}-unknown-linux-musl"

  if command -v hyperfine &>/dev/null; then
    success "Hyperfine installed: $(hyperfine --version)"
  else
    error "Hyperfine installation failed"
    return 1
  fi
}

# ============================================================================
# jq - JSON processor
# ============================================================================
install_jq() {
  info "Installing jq..."

  if command -v jq &>/dev/null; then
    success "jq already installed: $(jq --version)"
    return 0
  fi

  # Try apt first (usually available)
  if sudo apt install -y jq 2>/dev/null; then
    success "jq installed via apt"
    return 0
  fi

  # Fall back to binary download
  JQ_VERSION=$(curl -s "https://api.github.com/repos/jqlang/jq/releases/latest" | grep -Po '"tag_name": "jq-\K[^"]*')

  if [ -z "$JQ_VERSION" ]; then
    warn "Could not determine latest version, using 1.7.1"
    JQ_VERSION="1.7.1"
  fi

  ARCH=$(uname -m)
  case $ARCH in
  x86_64) ARCH="amd64" ;;
  aarch64) ARCH="arm64" ;;
  *)
    error "Unsupported architecture: $ARCH"
    return 1
    ;;
  esac

  curl -Lo ~/.local/bin/jq "https://github.com/jqlang/jq/releases/download/jq-${JQ_VERSION}/jq-linux-${ARCH}"
  chmod +x ~/.local/bin/jq

  if command -v jq &>/dev/null; then
    success "jq installed: $(jq --version)"
  else
    error "jq installation failed"
    return 1
  fi
}

# ============================================================================
# yq - YAML processor (like jq but for YAML)
# ============================================================================
install_yq() {
  info "Installing yq..."

  if command -v yq &>/dev/null; then
    success "yq already installed: $(yq --version)"
    return 0
  fi

  # Download binary directly (mikefarah/yq)
  YQ_VERSION=$(curl -s "https://api.github.com/repos/mikefarah/yq/releases/latest" | grep -Po '"tag_name": "v\K[^"]*')

  if [ -z "$YQ_VERSION" ]; then
    warn "Could not determine latest version, using 4.44.1"
    YQ_VERSION="4.44.1"
  fi

  ARCH=$(uname -m)
  case $ARCH in
  x86_64) ARCH="amd64" ;;
  aarch64) ARCH="arm64" ;;
  *)
    error "Unsupported architecture: $ARCH"
    return 1
    ;;
  esac

  curl -Lo ~/.local/bin/yq "https://github.com/mikefarah/yq/releases/download/v${YQ_VERSION}/yq_linux_${ARCH}"
  chmod +x ~/.local/bin/yq

  if command -v yq &>/dev/null; then
    success "yq installed: $(yq --version)"
  else
    error "yq installation failed"
    return 1
  fi
}

# ============================================================================
# tldr - Simplified man pages with examples
# ============================================================================
install_tldr() {
  info "Installing tldr..."

  if command -v tldr &>/dev/null; then
    success "tldr already installed"
    return 0
  fi

  # Try apt first
  if sudo apt install -y tldr 2>/dev/null; then
    success "tldr installed via apt"
    # Update the cache
    tldr --update 2>/dev/null || true
    return 0
  fi

  # Fall back to tealdeer (Rust implementation - faster)
  TEALDEER_VERSION=$(curl -s "https://api.github.com/repos/dbrgn/tealdeer/releases/latest" | grep -Po '"tag_name": "v\K[^"]*')

  if [ -z "$TEALDEER_VERSION" ]; then
    warn "Could not determine latest version, using 1.6.1"
    TEALDEER_VERSION="1.6.1"
  fi

  ARCH=$(uname -m)
  case $ARCH in
  x86_64) ARCH="x86_64" ;;
  aarch64) ARCH="aarch64" ;;
  *)
    error "Unsupported architecture: $ARCH"
    return 1
    ;;
  esac

  curl -Lo ~/.local/bin/tldr "https://github.com/dbrgn/tealdeer/releases/download/v${TEALDEER_VERSION}/tealdeer-linux-${ARCH}-musl"
  chmod +x ~/.local/bin/tldr

  # Update the cache
  ~/.local/bin/tldr --update 2>/dev/null || true

  if command -v tldr &>/dev/null; then
    success "tldr (tealdeer) installed"
  else
    error "tldr installation failed"
    return 1
  fi
}

# ============================================================================
# GitHub CLI (gh)
# ============================================================================
install_gh() {
  info "Installing GitHub CLI..."

  if command -v gh &>/dev/null; then
    success "GitHub CLI already installed: $(gh --version | head -1)"
    return 0
  fi

  # Install via official apt repository
  (type -p wget >/dev/null || (sudo apt update && sudo apt install wget -y)) &&
    sudo mkdir -p -m 755 /etc/apt/keyrings &&
    wget -nv -O- https://cli.github.com/packages/githubcli-archive-keyring.gpg | sudo tee /etc/apt/keyrings/githubcli-archive-keyring.gpg >/dev/null &&
    sudo chmod go+r /etc/apt/keyrings/githubcli-archive-keyring.gpg &&
    echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | sudo tee /etc/apt/sources.list.d/github-cli.list >/dev/null &&
    sudo apt update &&
    sudo apt install gh -y

  if command -v gh &>/dev/null; then
    success "GitHub CLI installed: $(gh --version | head -1)"
  else
    error "GitHub CLI installation failed"
    return 1
  fi
}

# ============================================================================
# Main
# ============================================================================
main() {
  echo ""
  echo "========================================"
  echo "  Modern CLI Tools Installer"
  echo "========================================"
  echo ""

  install_lazygit
  install_eza
  install_fd
  install_bat
  install_delta
  install_fzf
  install_just
  install_glow
  install_bottom
  install_gh
  install_hyperfine
  install_jq
  install_yq
  install_tldr
  install_nerd_font

  echo ""
  echo "========================================"
  success "All tools installed!"
  echo "========================================"
  echo ""
  info "Add this to your .bashrc to enable fzf keybindings:"
  echo ""
  echo '  [ -f ~/.fzf.bash ] && source ~/.fzf.bash'
  echo ""
  info "Other tools ready to use:"
  echo "  just      - Run 'just --list' in any project with a justfile"
  echo "  glow      - Run 'glow README.md' to render markdown"
  echo "  btm       - Run 'btm' for system monitoring"
  echo "  gh        - Run 'gh auth login' to authenticate with GitHub"
  echo "  hyperfine - Run 'hyperfine cmd1 cmd2' to benchmark commands"
  echo "  jq        - Run 'cat file.json | jq .key' to process JSON"
  echo "  yq        - Run 'cat file.yaml | yq .key' to process YAML"
  echo "  tldr      - Run 'tldr command' for simplified man pages"
  echo ""
}

# Run if executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
  main "$@"
fi
