# Setup - Development Environment Configuration

A Rust CLI tool for setting up and maintaining a development environment on Ubuntu. Includes modern CLI tools, dotfiles management, system monitoring, and backup configuration.

## Table of Contents

- [System Requirements](#system-requirements)
- [Quick Start](#quick-start)
- [Installation](#installation)
- [Usage](#usage)
- [Available Components](#available-components)
- [Features](#features)
- [Testing](#testing)
- [Development](#development)
- [Directory Structure](#directory-structure)

## System Requirements

- Ubuntu 22.04 LTS or Ubuntu 24.04 LTS
- Rust toolchain (for building from source)
- Git
- sudo access (for system packages)

## Quick Start

```bash
# Clone the repository
git clone https://github.com/X-McKay/setup.git
cd setup

# Build the CLI
cd cli && cargo build --release && cd ..

# Run interactive mode
./cli/target/release/setup

# Or install everything at once
./cli/target/release/setup install --all -y
```

## Installation

### Building from Source

1. **Install Rust** (if not already installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   ```

2. **Clone and build**:
   ```bash
   git clone https://github.com/X-McKay/setup.git
   cd setup/cli
   cargo build --release
   ```

3. **Optional: Add to PATH**:
   ```bash
   sudo cp target/release/setup /usr/local/bin/
   ```

### Alternative: Bash Installation

For environments without Rust, use the standalone bash script:

```bash
./bootstrap/scripts/install_modern_cli.sh
```

## Usage

### Interactive Mode

Run without arguments to get an interactive menu:

```bash
setup
```

### Install Components

```bash
# Install all components (non-interactive)
setup install --all -y

# Install specific component
setup install mise
setup install docker
setup install tools

# Install multiple components
setup install apt tools mise starship
```

### Manage Dotfiles

```bash
setup dotfiles sync      # Sync dotfiles from repo to home
setup dotfiles sync -f   # Force sync (overwrite without prompting)
setup dotfiles diff      # Show differences between repo and installed
setup dotfiles list      # List managed dotfiles and their status
setup dotfiles backup    # Backup current dotfiles before syncing
```

### Check System Status

```bash
setup check tools        # Check which tools are installed
setup check dotfiles     # Check dotfile sync status
setup check system       # Check system info
setup check all          # Check everything
```

### Update Components

```bash
setup update system      # apt update/upgrade
setup update mise        # Update mise and managed tools
setup update rust        # Update Rust toolchain and cargo packages
setup update dotfiles    # Sync dotfiles from repo
```

## Available Components

| Component | Description |
|-----------|-------------|
| `apt` | Basic system packages (curl, git, build-essential, etc.) |
| `tools` | Extra CLI tools (ripgrep, fd, fzf, bat, eza, delta) |
| `mise` | Mise version manager + installs tools from ~/.tool-versions |
| `docker` | Docker and adds user to docker group |
| `monitoring` | System monitoring (htop, netdata, fail2ban, logwatch) + health checks |
| `backup` | Backup utilities (rsync, timeshift) + automated backup scripts |
| `starship` | Starship cross-shell prompt |
| `zoxide` | Smarter cd command that learns your habits |
| `lazygit` | Terminal UI for git |
| `just` | Modern task runner |
| `glow` | Terminal markdown renderer |
| `bottom` | System monitor (btm) |
| `gh` | GitHub CLI |
| `hyperfine` | Command-line benchmarking tool |
| `jq` | JSON processor |
| `yq` | YAML processor |
| `tldr` | Simplified man pages |
| `neovim` | Neovim editor with sensible defaults |
| `tpm` | Tmux Plugin Manager |
| `ssh-keys` | Generate ED25519 SSH keys (interactive) |
| `gpg` | Generate GPG keys for commit signing (interactive) |

## Features

### Modern CLI Tools

The setup installs a curated set of modern CLI replacements:

| Tool | Replaces | Description |
|------|----------|-------------|
| `eza` | `ls` | Modern ls with icons and git status |
| `bat` | `cat` | Cat with syntax highlighting |
| `fd` | `find` | Fast, user-friendly find |
| `ripgrep` | `grep` | Fast recursive search |
| `fzf` | - | Fuzzy finder for files, history, etc. |
| `delta` | `diff` | Beautiful git diffs |
| `zoxide` | `cd` | Smarter cd that learns your habits |
| `bottom` | `top` | Beautiful system monitor |

See [docs/TOOLS.md](docs/TOOLS.md) for detailed usage of each tool.

### System Monitoring

The `monitoring` component installs and configures:

- **htop** - Interactive process viewer
- **iotop** - I/O monitoring
- **nethogs** - Network traffic monitoring
- **sysstat** - System performance tools
- **netdata** - Real-time monitoring dashboard (http://localhost:19999)
- **logwatch** - Log analysis and reporting
- **fail2ban** - Intrusion prevention

Automated health checks:
- Daily health reports at midnight via cron
- Reports stored in `~/.monitoring/health_report.log`
- Manual check: `/usr/local/bin/check_monitoring.sh`

### Backup System

The `backup` component sets up:

- **rsync** - File synchronization
- **timeshift** - System snapshots (uses Timeshift's built-in scheduler)
- **duplicity** - Encrypted incremental backups

Automated backup scripts:
- `~/.backup/backup.sh` - Daily backup at 2 AM via cron
- `~/.backup/restore.sh` - Restore from backup
- 7-day retention policy for config and system backups

### Dotfiles Management

Managed dotfiles include:

| File | Description |
|------|-------------|
| `.bashrc` | Bash configuration |
| `.bash_profile` | Login shell config |
| `.aliases` | Shell aliases |
| `.exports` | Environment variables |
| `.util` | Utility functions |
| `.tmux.conf` | Tmux configuration |
| `.gitconfig` | Git configuration |
| `.tool-versions` | Mise tool versions |
| `.config/starship.toml` | Starship prompt config |
| `.config/ghostty/config` | Ghostty terminal config |
| `.config/lazygit/config.yml` | Lazygit config |
| `.config/nvim/init.lua` | Neovim config (created by neovim component) |

## Testing

### Docker Integration Tests

Run the full test suite in a Docker container (simulates a fresh Ubuntu install):

```bash
# Run tests
./tests/docker/run_tests.sh

# Or manually
docker build -f tests/docker/Dockerfile -t setup-test .
docker run --rm setup-test
```

The Docker tests:
- Run as a non-root user (`testuser`) to simulate real usage
- Test all installable components
- Verify binaries are installed to correct locations
- Test dotfiles synchronization

### What's Tested

| Category | Tests |
|----------|-------|
| CLI | Help, version commands |
| APT | curl, git, wget, unzip |
| Tools | ripgrep, fd, fzf, bat, eza, delta |
| Utilities | jq, yq, starship, zoxide, lazygit, just |
| Apps | glow, bottom, gh, hyperfine, tldr |
| Dev | mise, neovim, tpm |
| Config | Dotfiles sync |

### Skipped Tests

Some components require systemd or user interaction and are skipped in Docker:
- Docker (requires privileged mode)
- Monitoring/Backup (require systemd)
- SSH/GPG keys (require user input)

## Development

### Prerequisites

- Rust 1.70+
- Docker (for running tests)
- pre-commit (for git hooks)

### Setup Development Environment

```bash
# Install pre-commit hooks
pip install pre-commit
pre-commit install

# Build in debug mode
cd cli
cargo build

# Run tests
cargo test

# Run clippy
cargo clippy
```

### CI/CD

The repository includes GitHub Actions workflows:

- **CI** (`ci.yml`): Runs on push/PR
  - Rust formatting check
  - Clippy linting
  - Build verification
  - Unit tests
  - ShellCheck for bash scripts
  - Docker integration tests

- **Release** (`release.yml`): Runs on version tags
  - Builds release binary
  - Creates GitHub release with tarball

### Pre-commit Hooks

The repository uses pre-commit hooks for:
- YAML/JSON validation
- Shell script linting (shellcheck)
- Shell formatting (shfmt)
- Security checks (gitleaks)
- Large file prevention

### Conventional Commits

Commit messages follow the format: `type(scope): description`

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`, `revert`

## Directory Structure

```
.
├── cli/                        # Rust CLI source
│   ├── src/
│   │   ├── commands/           # CLI commands (install, dotfiles, check, update)
│   │   ├── config/             # Configuration handling
│   │   ├── system/             # System operations (packages.rs)
│   │   └── ui/                 # User interface (prompts)
│   └── Cargo.toml
├── bootstrap/
│   ├── dotfiles/               # Dotfile templates
│   ├── scripts/
│   │   ├── copy_dotfiles.sh    # Fallback dotfiles script
│   │   └── install_modern_cli.sh  # Standalone bash installer
│   └── templates/
│       └── justfile            # Project justfile template
├── tests/
│   └── docker/
│       ├── Dockerfile          # Test container definition
│       ├── test_installs.sh    # Integration test script
│       └── run_tests.sh        # Test runner
├── .github/
│   └── workflows/
│       ├── ci.yml              # CI workflow
│       └── release.yml         # Release automation
├── docs/
│   └── TOOLS.md                # Tool usage guide
├── CHANGELOG.md                # Version history
└── README.md
```

## License

MIT

## Credits

- [pre-commit](https://pre-commit.com/)
- [Conventional Commits](https://www.conventionalcommits.org/)
- [Starship](https://starship.rs/)
- [Modern Unix](https://github.com/ibraheemdev/modern-unix)
