#!/bin/bash
set -e

SETUP_BIN="/setup/cli/target/release/setup"
PASSED=0
FAILED=0
SKIPPED=0

# Ensure ~/.local/bin is in PATH
export PATH="$HOME/.local/bin:$PATH"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m'

pass() {
  echo -e "${GREEN}✓ PASS${NC}: $1"
  PASSED=$((PASSED + 1))
}

fail() {
  echo -e "${RED}✗ FAIL${NC}: $1"
  FAILED=$((FAILED + 1))
}

skip() {
  echo -e "${YELLOW}○ SKIP${NC}: $1"
  SKIPPED=$((SKIPPED + 1))
}

check_command() {
  local cmd=$1
  local desc=$2
  if command -v "$cmd" &>/dev/null; then
    pass "$desc ($cmd found)"
    return 0
  else
    fail "$desc ($cmd not found)"
    return 1
  fi
}

# Check for command with fallback (for Ubuntu renamed binaries)
check_command_or() {
  local cmd1=$1
  local cmd2=$2
  local desc=$3
  if command -v "$cmd1" &>/dev/null; then
    pass "$desc ($cmd1 found)"
  elif command -v "$cmd2" &>/dev/null; then
    pass "$desc ($cmd2 found)"
  else
    fail "$desc ($cmd1 or $cmd2 not found)"
  fi
}

check_file() {
  local file=$1
  local desc=$2
  if [ -f "$file" ]; then
    pass "$desc ($file exists)"
  else
    fail "$desc ($file not found)"
  fi
}

check_dir() {
  local dir=$1
  local desc=$2
  if [ -d "$dir" ]; then
    pass "$desc ($dir exists)"
  else
    fail "$desc ($dir not found)"
  fi
}

echo "========================================"
echo "Setup CLI Integration Tests"
echo "========================================"
echo "User: $(whoami)"
echo "Home: $HOME"
echo "PATH: $PATH"
echo ""

# Test 1: CLI builds and runs
echo "--- Test: CLI Basic ---"
if $SETUP_BIN --help &>/dev/null; then
  pass "CLI runs with --help"
else
  fail "CLI failed to run"
  exit 1
fi

if $SETUP_BIN --version &>/dev/null; then
  pass "CLI runs with --version"
else
  fail "CLI failed to show version"
fi

# Test 2: Install APT packages
echo ""
echo "--- Test: APT Packages ---"
$SETUP_BIN install apt -y
check_command "curl" "curl installed"
check_command "git" "git installed"
check_command "wget" "wget installed"
check_command "unzip" "unzip installed"

# Test 3: Install extra tools
echo ""
echo "--- Test: Extra CLI Tools ---"
$SETUP_BIN install tools -y
check_command "rg" "ripgrep installed"
check_command_or "fd" "fdfind" "fd-find installed"
check_command "fzf" "fzf installed"
check_command_or "bat" "batcat" "bat installed"
check_command "eza" "eza installed"
check_command "delta" "delta installed"

# Test 4: Install jq
echo ""
echo "--- Test: jq ---"
$SETUP_BIN install jq -y
check_command "jq" "jq installed"

# Test 5: Install yq
echo ""
echo "--- Test: yq ---"
$SETUP_BIN install yq -y
check_file "$HOME/.local/bin/yq" "yq binary"

# Test 6: Install starship
echo ""
echo "--- Test: Starship ---"
$SETUP_BIN install starship -y
check_command "starship" "starship installed"

# Test 7: Install zoxide
echo ""
echo "--- Test: Zoxide ---"
$SETUP_BIN install zoxide -y
check_command "zoxide" "zoxide installed"

# Test 8: Install lazygit
echo ""
echo "--- Test: Lazygit ---"
$SETUP_BIN install lazygit -y
check_file "$HOME/.local/bin/lazygit" "lazygit binary"

# Test 9: Install just
echo ""
echo "--- Test: Just ---"
$SETUP_BIN install just -y
check_file "$HOME/.local/bin/just" "just binary"

# Test 10: Install glow
echo ""
echo "--- Test: Glow ---"
$SETUP_BIN install glow -y
check_file "$HOME/.local/bin/glow" "glow binary"

# Test 11: Install bottom
echo ""
echo "--- Test: Bottom ---"
$SETUP_BIN install bottom -y
check_file "$HOME/.local/bin/btm" "bottom binary"

# Test 12: Install GitHub CLI
echo ""
echo "--- Test: GitHub CLI ---"
$SETUP_BIN install gh -y
check_command "gh" "GitHub CLI installed"

# Test 13: Install hyperfine
echo ""
echo "--- Test: Hyperfine ---"
$SETUP_BIN install hyperfine -y
# Check for hyperfine in .local/bin or system PATH
if [ -f "$HOME/.local/bin/hyperfine" ] || command -v hyperfine &>/dev/null; then
  pass "hyperfine installed"
else
  fail "hyperfine not found"
fi

# Test 14: Install tldr
echo ""
echo "--- Test: tldr ---"
$SETUP_BIN install tldr -y
check_command "tldr" "tldr installed"

# Test 15: Install mise
echo ""
echo "--- Test: Mise ---"
$SETUP_BIN install mise -y
check_file "$HOME/.local/bin/mise" "mise binary"

# Test 16: Install Neovim
echo ""
echo "--- Test: Neovim ---"
$SETUP_BIN install neovim -y
check_command "nvim" "neovim installed"
check_file "$HOME/.config/nvim/init.lua" "neovim config"

# Test 17: Install TPM
echo ""
echo "--- Test: TPM ---"
$SETUP_BIN install tpm -y
check_dir "$HOME/.tmux/plugins/tpm" "TPM directory"

# Test 18: Dotfiles sync
echo ""
echo "--- Test: Dotfiles ---"
$SETUP_BIN dotfiles sync --force
check_file "$HOME/.bashrc" "bashrc synced"
check_file "$HOME/.aliases" "aliases synced"
check_file "$HOME/.exports" "exports synced"
check_file "$HOME/.config/starship.toml" "starship config synced"

# Skipped tests (require user input or special setup)
echo ""
echo "--- Skipped Tests ---"
skip "Docker (requires systemd/privileged mode)"
skip "Monitoring (requires systemd)"
skip "Backup (requires systemd)"
skip "SSH keys (requires user input)"
skip "GPG keys (requires user input)"

# Summary
echo ""
echo "========================================"
echo "Test Summary"
echo "========================================"
echo -e "${GREEN}Passed${NC}: $PASSED"
echo -e "${RED}Failed${NC}: $FAILED"
echo -e "${YELLOW}Skipped${NC}: $SKIPPED"
echo ""

if [ $FAILED -gt 0 ]; then
  echo -e "${RED}Some tests failed!${NC}"
  exit 1
else
  echo -e "${GREEN}All tests passed!${NC}"
  exit 0
fi
