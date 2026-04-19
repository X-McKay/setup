# Setup - Development Environment Configuration

A Rust CLI tool for setting up and maintaining a development environment on Ubuntu. Includes profile-aware installs, drift review for managed configs, modern CLI tools, system monitoring, and backup configuration.

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
- Git
- sudo access (for system packages)

## Quick Start

```bash
# Clone the repository
git clone https://github.com/X-McKay/setup.git
cd setup

# Bootstrap from a fresh machine (installs mise, Rust, and builds the CLI)
./bootstrap.sh

# Run interactive mode
./setup.sh

# Or install everything at once
./setup.sh install --all -y
```

## Installation

### Bootstrap (Recommended)

The bootstrap script handles everything from a fresh Ubuntu install:

```bash
./bootstrap.sh
```

This will:
1. Install system prerequisites (curl, git, build-essential, etc.)
2. Install [mise](https://mise.jdx.dev/) version manager
3. Install Rust and other tools via mise (from `.tool-versions`)
4. Build the setup CLI

### Alternative: Manual Rust Install

If you prefer to manage Rust yourself:

1. **Install Rust** (via rustup):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   ```

2. **Build the CLI**:
   ```bash
   cd cli && cargo build --release
   ```

## Usage

### Interactive Mode

Run without arguments to get an interactive menu:

```bash
setup
```

### Install Components

```bash
# Install a profile
setup install --profile server
setup install --profile workstation --profile ai-heavy

# Install specific components
setup install docker mise claude-code

# Preview without installing
setup install --profile server --dry-run

# Continue past failures
setup install --profile workstation --keep-going

# Roll back on first failure
setup install --profile server --rollback-on-failure

# Everything (non-interactive only)
setup install --all -y
```

### See What's Available

```bash
setup list                       # all components
setup list --profile ai-heavy    # components in a profile
setup list --tag dev             # components with a tag
setup profile list               # known profiles
setup profile show server        # resolved components for a profile
```

### Health and Drift

```bash
setup doctor                       # check declared profiles vs reality
setup doctor --profile workstation # check a specific profile
setup doctor --verify              # also run post-install verify()
setup doctor --warn-only           # never exit non-zero
setup drift                        # focused summary for dotfiles + profile intent
setup drift --json                 # machine-readable drift report for agents
setup drift diff --name ghostty/config
setup drift adopt --name ghostty/config
setup drift sync --force
```

`setup check` is deprecated and forwards to `setup doctor`.
`setup doctor` stays read-only and broad; `setup drift` is the narrower review/reconcile
entrypoint for managed configs and active-profile intent.

### Remove Components

```bash
setup uninstall claude-code
setup uninstall ssh-keys --force
setup uninstall docker --cascade
```

### Profile Intent

`~/.config/setup/active.toml` records which profiles you've selected on this machine.
Doctor reads this when `--profile` isn't passed, so it knows what "should be here."

```bash
setup profile activate server
setup profile deactivate server
```

### Manage Dotfiles

```bash
setup dotfiles sync      # Sync dotfiles from repo to home
setup dotfiles sync -f   # Force sync (overwrite without prompting)
setup dotfiles diff      # Show differences between repo and installed
setup dotfiles list      # List managed dotfiles and their status
setup dotfiles backup    # Backup current dotfiles before syncing
```

For agent-assisted review flows, `setup drift --json` is the canonical machine-readable
entrypoint. Repo-local helpers live in `.agents/skills/setup-drift/` and
`.claude/commands/drift-review.md`.

### Update Components

```bash
setup update system      # apt update/upgrade
setup update mise        # Update mise and managed tools
setup update rust        # Update Rust toolchain and cargo packages
setup update dotfiles    # Sync dotfiles from repo
```

## Available Components

The full catalog lives in [`bootstrap/manifest.toml`](bootstrap/manifest.toml).
Run `setup list` to see it with your local profile and tag filters applied.
See [plans/2026-04-18-manifest-architecture-design.md](plans/2026-04-18-manifest-architecture-design.md)
for the manifest architecture.

## Features

### Modern CLI Tools

The setup installs a curated set of modern CLI replacements:

| Tool | Replaces | Description |
|------|----------|-------------|
| `eza` | `ls` | Modern ls with git status |
| `bat` | `cat` | Cat with syntax highlighting |
| `fd` | `find` | Fast, user-friendly find |
| `ripgrep` | `grep` | Fast recursive search |
| `fzf` | - | Fuzzy finder for files, history, etc. |
| `delta` | `diff` | Beautiful git diffs |
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
| `.config/ghostty/config` | Ghostty terminal config |
| `.config/lazygit/config.yml` | Lazygit config |
| `.config/mise/config.toml` | Mise settings, including Python attestation policy |

Typical review loop for a repo-managed config:

```bash
setup drift --dotfiles
setup drift diff --name ghostty/config
setup drift adopt --name ghostty/config   # home -> repo
setup drift sync --force                  # repo -> home
```

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
- Test the docker-safe installable subset
- Verify binaries are installed to correct locations
- Test profile-based dry-runs and doctor/profile flows
- Test drift summary/diff/sync for managed dotfiles
- Test user-manifest override behavior

### Host-Side Contract Suite

The cross-component contract test is opt-in because it performs real install and
uninstall operations on the current machine.

```bash
SETUP_CONTRACT_TESTS=1 ./tests/docker/run_tests.sh
```

### What's Tested

| Category | Tests |
|----------|-------|
| CLI | Help, version commands |
| Install | APT, tools, jq, yq, lazygit, just, glow, bottom, gh, hyperfine, tldr, mise, neovim, tpm |
| Profiles | `install --profile`, `profile activate/deactivate`, `profile show` |
| Health | `doctor --warn-only`, `doctor --profile server` |
| Overrides | `~/.config/setup/manifest.toml` merge behavior |
| Config | Dotfiles sync, drift summary/diff |

### Skipped Tests

Some components require systemd or user interaction and are skipped in Docker:
- Docker (requires privileged mode)
- Monitoring/Backup (require systemd)
- SSH/GPG keys (require user input)

## Development

### Prerequisites

- Rust toolchain (or `./bootstrap.sh`)
- Docker (for container integration tests)
- pre-commit (for git hooks)

### Setup Development Environment

```bash
# Bootstrap toolchains the same way a fresh machine does
./bootstrap.sh

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
├── bootstrap.sh                # Fresh-machine bootstrap (mise + Rust + build)
├── cli/                        # Rust CLI source
│   ├── src/
│   │   ├── commands/           # install, uninstall, doctor, drift, list, profile, dotfiles, update
│   │   ├── components/         # Per-component install/uninstall implementations
│   │   ├── config/             # Configuration handling
│   │   ├── manifest/           # Manifest schema, loader, resolver, intent
│   │   ├── system/             # Remaining system helpers (health, etc.)
│   │   └── ui/                 # User interface (prompts)
│   ├── tests/
│   │   └── contract.rs         # Gated install/uninstall contract suite
│   └── Cargo.toml
├── .agents/
│   └── skills/                 # Repo-local Codex skills
├── .claude/
│   ├── commands/               # Repo-local Claude Code slash commands
│   └── settings.local.json
├── bootstrap/
│   ├── dotfiles/               # Dotfile templates
│   ├── scripts/
│   │   ├── copy_dotfiles.sh    # Fallback dotfiles script
│   └── templates/
│       └── justfile            # Project justfile template
├── tests/
│   └── docker/
│       ├── Dockerfile          # Test container definition
│       ├── fixtures/           # User-manifest override fixtures
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
- [Modern Unix](https://github.com/ibraheemdev/modern-unix)
