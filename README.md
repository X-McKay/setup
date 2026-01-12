# Setup - Development Environment Configuration

A Rust CLI tool for setting up and maintaining a development environment on Ubuntu. Includes modern CLI tools, dotfiles management, system monitoring, and backup configuration.

## System Requirements

- Ubuntu 22.04 LTS or Ubuntu 24.04 LTS
- Rust toolchain (for building from source)
- Git

## Quick Start

### Build the CLI

```bash
cd cli
cargo build --release
```

### Run Interactive Mode

```bash
./cli/target/release/setup
```

### Install Specific Components

```bash
# Install all components
./cli/target/release/setup install --all

# Install specific component
./cli/target/release/setup install mise
./cli/target/release/setup install docker
./cli/target/release/setup install tools
```

## Available Commands

### `setup install [component]`

Install system components. Without arguments, shows interactive selection.

**Components:**
| Component | Description |
|-----------|-------------|
| `apt` | Basic system packages (curl, git, build-essential, etc.) |
| `tools` | Extra CLI tools (ripgrep, fd, fzf, bat, eza, delta) |
| `mise` | Mise version manager + installs tools from ~/.tool-versions |
| `docker` | Docker and adds user to docker group |
| `monitoring` | System monitoring (htop, netdata, fail2ban, logwatch) + health checks |
| `backup` | Backup utilities (rsync, timeshift) + automated backup scripts |
| `starship` | Starship prompt |
| `zoxide` | Smarter cd command |
| `lazygit` | Terminal UI for git |
| `just` | Task runner |
| `glow` | Markdown renderer |
| `bottom` | System monitor (btm) |
| `gh` | GitHub CLI |
| `hyperfine` | Command benchmarking |
| `jq` | JSON processor |
| `yq` | YAML processor |
| `tldr` | Simplified man pages |

### `setup dotfiles <action>`

Manage dotfiles synchronization.

```bash
setup dotfiles sync      # Sync dotfiles from repo to home
setup dotfiles diff      # Show differences
setup dotfiles list      # List managed dotfiles
setup dotfiles backup    # Backup current dotfiles
```

### `setup check [category]`

Check system status.

```bash
setup check tools        # Check installed tools
setup check dotfiles     # Check dotfile sync status
setup check system       # Check system info
setup check all          # Check everything
```

### `setup update [component]`

Update installed components.

```bash
setup update system      # apt update/upgrade
setup update mise        # Update mise and tools
setup update rust        # Update Rust toolchain and cargo packages
setup update dotfiles    # Sync dotfiles
```

## Features

### Modern CLI Tools

The setup installs a curated set of modern CLI tools:

- **eza** - Modern ls replacement with icons and git status
- **bat** - Cat with syntax highlighting
- **fd** - Fast, user-friendly find alternative
- **ripgrep** - Fast grep alternative
- **fzf** - Fuzzy finder
- **delta** - Beautiful git diffs
- **zoxide** - Smarter cd that learns your habits
- **starship** - Cross-shell prompt
- **lazygit** - Terminal UI for git
- **just** - Modern task runner
- **bottom** - System monitor

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

Plus automated health checks:
- Daily health reports at midnight
- Reports stored in `~/.monitoring/health_report.log`
- Manual check: `/usr/local/bin/check_monitoring.sh`

### Backup System

The `backup` component sets up:

- **rsync** - File synchronization
- **timeshift** - System snapshots (handled by Timeshift's built-in scheduler)
- **duplicity** - Encrypted incremental backups

Automated backup scripts:
- `~/.backup/backup.sh` - Daily backup at 2 AM
- `~/.backup/restore.sh` - Restore from backup
- 7-day retention policy for config and system backups

### Dotfiles Management

Managed dotfiles include:
- `.bashrc`, `.bash_profile`, `.aliases`, `.exports`, `.util`
- `.tmux.conf`
- `.gitconfig`
- `.tool-versions` (for mise)
- `~/.config/starship.toml`
- `~/.config/ghostty/config`
- `~/.config/lazygit/config.yml`

## Directory Structure

```
.
├── cli/                    # Rust CLI source
│   ├── src/
│   │   ├── commands/       # CLI commands
│   │   ├── config/         # Configuration handling
│   │   ├── system/         # System operations
│   │   └── ui/             # User interface
│   └── Cargo.toml
├── bootstrap/
│   ├── dotfiles/           # Dotfile templates
│   ├── scripts/
│   │   ├── copy_dotfiles.sh      # Fallback dotfiles script
│   │   └── install_modern_cli.sh # Standalone bash installer
│   └── templates/
│       └── justfile        # Project justfile template
├── docs/
│   └── TOOLS.md            # Tool usage guide
└── README.md
```

## Git Workflow

### Pre-commit Configuration

The repository uses pre-commit hooks for:
- YAML/JSON validation
- Shell script linting (shellcheck)
- Code formatting (shfmt)
- Security checks
- Large file prevention

### Conventional Commits

Commit messages must follow the format: `type(scope): description`

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`, `revert`

## Alternative: Bash Installation

For environments without Rust, use the standalone bash script:

```bash
./bootstrap/scripts/install_modern_cli.sh
```

## Credits

- [pre-commit](https://pre-commit.com/)
- [Conventional Commits](https://www.conventionalcommits.org/)
- [Starship](https://starship.rs/)
- [Modern Unix](https://github.com/ibraheemdev/modern-unix)
