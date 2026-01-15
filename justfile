# ============================================================================
# Setup Repository Justfile
# Run 'just' to see available commands
# ============================================================================

# Default - show help
default:
    @just --list

# ============================================================================
# Docker Testing
# ============================================================================

# Build the test Docker image
docker-build:
    docker build -f Dockerfile.test -t setup-test .

# Run interactive test container
docker-test: docker-build
    docker run -it --rm setup-test

# Run quick automated test of install script
docker-quick-test:
    docker compose -f docker-compose.test.yml run --rm quick-test

# Test dotfiles installation
docker-dotfiles-test:
    docker compose -f docker-compose.test.yml run --rm dotfiles-test

# Run all Docker tests
docker-test-all: docker-quick-test docker-dotfiles-test
    @echo "All tests passed!"

# Clean up Docker test resources
docker-clean:
    docker compose -f docker-compose.test.yml down -v
    docker rmi setup-test 2>/dev/null || true

# ============================================================================
# Quick Start
# ============================================================================

# Run the setup CLI (builds if needed)
run *ARGS:
    ./setup.sh {{ARGS}}

# ============================================================================
# Local Development
# ============================================================================

# Build the Rust CLI
build:
    cd cli && cargo build --release

# Build CLI in debug mode
build-dev:
    cd cli && cargo build

# Run CLI tests
test:
    cd cli && cargo test

# Check CLI compiles
check:
    cd cli && cargo check

# Format Rust code
fmt:
    cd cli && cargo fmt

# Lint Rust code
lint:
    cd cli && cargo clippy

# ============================================================================
# Installation (use with caution - affects local system)
# ============================================================================

# Install CLI to ~/.local/bin
install-cli: build
    cp cli/target/release/setup ~/.local/bin/

# Run the modern CLI tools installer
install-tools:
    ./bootstrap/scripts/install_modern_cli.sh

# Copy dotfiles to home directory
install-dotfiles:
    ./bootstrap/scripts/copy_dotfiles.sh

# Full local install (tools + dotfiles + CLI)
install-all: install-tools install-dotfiles install-cli
    @echo "Installation complete! Run 'source ~/.bashrc' to apply changes."

# ============================================================================
# Utilities
# ============================================================================

# Show what would be installed
dry-run:
    @echo "Tools that would be installed:"
    @echo "  - starship (prompt)"
    @echo "  - zoxide (smarter cd)"
    @echo "  - lazygit (git TUI)"
    @echo "  - eza (better ls)"
    @echo "  - bat (better cat)"
    @echo "  - fd (better find)"
    @echo "  - fzf (fuzzy finder)"
    @echo "  - delta (better diff)"
    @echo "  - just (task runner)"
    @echo "  - glow (markdown viewer)"
    @echo "  - btm (system monitor)"
    @echo "  - gh (GitHub CLI)"
    @echo "  - hyperfine (benchmarking)"
    @echo "  - jq (JSON processor)"
    @echo "  - yq (YAML processor)"
    @echo "  - tldr (simplified man pages)"
    @echo ""
    @echo "Dotfiles that would be copied:"
    @echo "  - bashrc, bash_profile, aliases, exports, util"
    @echo "  - tmux.conf, gitconfig, tool-versions"
    @echo "  - starship.toml, ghostty/config, lazygit/config.yml"

# Render README in terminal
readme:
    glow README.md 2>/dev/null || cat README.md
