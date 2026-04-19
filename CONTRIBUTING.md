# Contributing to Setup

Thank you for contributing! This guide will help you get started.

## Development Setup

1. **Clone the repository:**
   ```bash
   git clone https://github.com/X-McKay/setup.git
   cd setup
   ```

2. **Install pre-commit hooks:**
   ```bash
   pip install pre-commit
   pre-commit install
   pre-commit install --hook-type commit-msg
   ```

3. **Build the CLI:**
   ```bash
   cd cli
   cargo build -p setup
   ```

## Code Style

### Rust Code
- Run `cargo fmt` before committing
- Run `cargo clippy` and fix all warnings
- Use `anyhow::Context` for error handling, not `.expect()` or `.unwrap()`
- Add proper error messages that help users understand what went wrong

### Shell Scripts
- Use `shellcheck` for linting (runs automatically via pre-commit)
- Use `shfmt -i 2` for formatting (runs automatically via pre-commit)
- Quote all variables: `"$var"` not `$var`
- Use `set -e` at the top of scripts

### Commit Messages
We use [Conventional Commits](https://www.conventionalcommits.org/):

```
type(scope): description

[optional body]
```

**Types:** `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`, `revert`

**Examples:**
- `feat(docker): add Docker Compose support`
- `fix(dotfiles): correct tmux config path`
- `docs: update installation instructions`

## Adding a New Installable Component

### 1. Add a manifest entry (`bootstrap/manifest.toml`)

Define the component id, display name, tags, dependencies, and capability flags
(`requires_sudo`, `requires_systemd`, `requires_privileged`, `interactive`).
Profiles should reference the manifest id, not the Rust type name.

### 2. Implement the component (`cli/src/components/<id>.rs`)

Create a unit struct that implements the `Component` trait from
`cli/src/components/mod.rs`.

Minimum methods:
- `id()` must match the manifest id exactly
- `is_installed()` should be a cheap, reliable probe
- `install()` must be idempotent
- override `uninstall()`, `verify()`, `dry_run()`, or `is_reversible()` only when needed

### 3. Register the implementation (`cli/src/components/registry.rs`)

Wire the new component into `Registry::build()`. The manifest and registry are
validated against each other at startup, so a missing registration will fail
fast.

### 4. Add tests

- Add unit coverage in Rust when the component has non-trivial behavior
- Add Docker e2e coverage in `tests/docker/test_installs.sh` when the component
  is Docker-safe
- If the component is intentionally skipped in Docker, document why in the test
  harness comments or skip list

### 5. Update documentation

- Add or update user-facing docs in `README.md` and `docs/*.md`
- Document any dependencies, caveats, or profile implications
- If the component installs or manages a repo-controlled config, document how to
  review it with `setup drift`

## Testing

### Run Unit Tests
```bash
cd cli && cargo test -p setup
```

### Run Integration Tests
```bash
bash tests/docker/run_tests.sh
```

### Validate the Release Installer
```bash
bash -n install.sh
```

### Test Managed Dotfile Changes
```bash
setup drift --dotfiles
setup drift diff --name ghostty/config
```

## Project Structure

```
setup/
├── cli/                    # Rust CLI application
│   └── src/
│       ├── commands/       # install, uninstall, doctor, drift, list, profile, dotfiles, update
│       ├── components/     # Per-component implementations + registry
│       ├── config/         # Dotfile and local config helpers
│       ├── manifest/       # Manifest schema, loader, resolver, intent
│       └── ui/             # User interaction (prompts, colors)
├── bootstrap/
│   ├── dotfiles/           # Repo-managed config files synced to home
│   └── manifest.toml       # Declarative component/profile catalog
├── install.sh              # GitHub release installer for the published CLI
├── .agents/skills/         # Repo-local Codex skills
├── .claude/commands/       # Repo-local Claude Code commands
├── hooks/                  # Git hooks (commit-msg validation)
├── tests/
│   └── docker/             # Docker-based integration tests
└── .github/workflows/      # CI/CD pipelines
```

## Pull Request Process

1. Create a feature branch: `git checkout -b feat/my-feature`
2. Make your changes with proper commits
3. Run tests: `cargo test -p setup`, `bash tests/docker/run_tests.sh`, and `pre-commit run --all-files`
4. Push and create a PR against `main`
5. Ensure CI passes
6. Request review

## Questions?

Open an issue or start a discussion on GitHub.
