# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Neovim installation with sensible default configuration
- Tmux Plugin Manager (tpm) installation
- SSH key generation helper (ED25519)
- GPG key setup with git signing configuration
- kubectl and helm via mise tool-versions
- GitHub Actions CI workflow (build, lint, test)
- Release automation workflow for tagged releases

### Changed
- Enhanced mise installation to run `mise install` from .tool-versions
- Improved apt package installation argument handling

## [0.2.0] - 2026-01-11

### Added
- Full Rust CLI implementation replacing gum/bash scripts
- Interactive component selection with dialoguer
- Progress indicators with indicatif
- 16 installable components:
  - Basic APT packages
  - Extra CLI tools (ripgrep, bat, fd, fzf, eza, delta)
  - Mise version manager
  - Docker and Docker Compose
  - Starship prompt
  - Zoxide directory jumper
  - Lazygit terminal UI
  - Just task runner
  - Glow markdown renderer
  - Bottom system monitor
  - GitHub CLI
  - Hyperfine benchmarking
  - jq JSON processor
  - yq YAML processor
  - tldr simplified man pages
  - Monitoring tools (htop, netdata, fail2ban, logwatch)
  - Backup utilities (rsync, timeshift, duplicity)
- Dotfiles management commands (sync, diff, list, backup)
- System check commands (tools, dotfiles, system)
- Update commands (system, mise, rust, dotfiles)
- Automated health check scripts for monitoring
- Automated backup scripts with 7-day retention

### Changed
- Migrated from gum TUI to native Rust with dialoguer
- Backup system uses Timeshift's built-in scheduler

### Removed
- Gum-based bash scripts
- bootstrap/main.sh entry point
- Test scripts (replaced by Rust tests)

## [0.1.0] - 2024-01-01

### Added
- Initial gum-based bash setup scripts
- Basic apt package installation
- Dotfiles management
- Docker installation
- Mise version manager setup

[Unreleased]: https://github.com/X-McKay/setup/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/X-McKay/setup/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/X-McKay/setup/releases/tag/v0.1.0
