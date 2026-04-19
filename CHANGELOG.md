# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2026-04-19

### Added
- Declarative component manifest (`bootstrap/manifest.toml`) as the source of truth for components and profiles
- Composable machine-shape profiles: `base`, `server`, `workstation`, `ai-heavy`
- Persistent intent file at `~/.config/setup/active.toml`
- Component dependency graph with topological install order and transitive auto-pull
- `setup install --dry-run`
- `setup install --rollback-on-failure`
- `setup install --keep-going`
- `setup install --verify`
- `setup uninstall` with `--force` and `--cascade`
- `setup doctor` for health and drift reporting
- `setup drift` for focused config reconciliation (`summary`, `--json`, `diff`, `sync`, `adopt`)
- `install.sh` for curl-based installation from GitHub Releases
- `setup list [--profile X] [--tag T]`
- `setup profile list / show / activate / deactivate`
- Optional `~/.config/setup/manifest.toml` user override
- Repo-local Codex and Claude helpers for drift review workflows
- Neovim installation with sensible default configuration
- Tmux Plugin Manager (tpm) installation
- SSH key generation helper (ED25519)
- GPG key setup with git signing configuration
- kubectl and helm via mise tool-versions
- GitHub Actions CI workflow (build, lint, test)
- Release automation workflow for tagged releases

### Changed
- Components are now Rust structs implementing a `Component` trait, dispatched by id via `Registry::build()`
- `setup check` is deprecated and now forwards to `setup doctor`
- Enhanced mise installation to run `mise install` from .tool-versions
- Shared `mise` Python configuration now disables attestations only for Python precompiled downloads, matching the current `python-build-standalone` artifact gap
- Improved apt package installation argument handling
- Docker e2e coverage now includes managed-config drift summary/diff/sync checks
- Ghostty config adds split-pane navigation on `Ctrl+Shift+W/A/S/D`
- GitHub Releases now publish installer/checksum assets alongside the Linux tarball

### Removed
- `cli/src/system/packages/` module (install logic moved to `cli/src/components/<id>.rs`)
- Hardcoded `Component` enum in `cli/src/commands/install.rs`

### Migration Notes
- Existing `setup install <component>` invocations still work
- Consider `setup profile activate workstation` or `setup profile activate server` so `setup doctor` has intent to compare against
- Run `setup doctor` after upgrading to see drift
- Use `setup drift` when reviewing or syncing repo-managed dotfile changes over time

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

[Unreleased]: https://github.com/X-McKay/setup/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/X-McKay/setup/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/X-McKay/setup/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/X-McKay/setup/releases/tag/v0.1.0
