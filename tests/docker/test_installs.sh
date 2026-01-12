#!/bin/bash
set -e

SETUP_BIN="/setup/cli/target/release/setup"
PASSED=0
FAILED=0
SKIPPED=0

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m'

pass() {
  echo -e "${GREEN}✓ PASS${NC}: $1"
  ((PASSED++))
}

fail() {
  echo -e "${RED}✗ FAIL${NC}: $1"
  ((FAILED++))
}

skip() {
  echo -e "${YELLOW}○ SKIP${NC}: $1"
  ((SKIPPED++))
}

check_command() {
  local cmd=$1
  local desc=$2
  if command -v "$cmd" &>/dev/null; then
    pass "$desc ($cmd found)"
  else
    fail "$desc ($cmd not found)"
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
check_command "fd" "fd-find installed"
check_command "fzf" "fzf installed"
check_command "bat" "bat installed"
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
check_command "yq" "yq installed"

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
check_command "lazygit" "lazygit installed"

# Test 9: Install just
echo ""
echo "--- Test: Just ---"
$SETUP_BIN install just -y
check_command "just" "just installed"

# Test 10: Install glow
echo ""
echo "--- Test: Glow ---"
$SETUP_BIN install glow -y
check_command "glow" "glow installed"

# Test 11: Install bottom
echo ""
echo "--- Test: Bottom ---"
$SETUP_BIN install bottom -y
check_command "btm" "bottom installed"

# Test 12: Install GitHub CLI
echo ""
echo "--- Test: GitHub CLI ---"
$SETUP_BIN install gh -y
check_command "gh" "GitHub CLI installed"

# Test 13: Install hyperfine
echo ""
echo "--- Test: Hyperfine ---"
$SETUP_BIN install hyperfine -y
check_command "hyperfine" "hyperfine installed"

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
$SETUP_BIN dotfiles sync -y
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
