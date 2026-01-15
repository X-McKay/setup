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
   cargo build
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

### 1. Add to Rust CLI (`cli/src/system/packages.rs`)

```rust
pub fn install_mycomponent() -> Result<()> {
    // Check if already installed
    if which::which("mycomponent").is_ok() {
        return Ok(());
    }

    // Try apt first
    if run_sudo("apt", &["install", "-y", "mycomponent"]).is_ok() {
        return Ok(());
    }

    // Fall back to binary download with version detection
    let home = dirs::home_dir().context("Could not find home directory")?;
    let bin_dir = home.join(".local").join("bin");
    std::fs::create_dir_all(&bin_dir)?;

    let version = fetch_github_version("owner/mycomponent", "1.0.0");
    // ... download and install

    Ok(())
}
```

### 2. Add to Shell Script (`bootstrap/scripts/install_modern_cli.sh`)

```bash
install_mycomponent() {
  info "Installing mycomponent..."

  if command -v mycomponent &>/dev/null; then
    success "mycomponent already installed"
    return 0
  fi

  # Installation logic here

  if command -v mycomponent &>/dev/null; then
    success "mycomponent installed"
  else
    error "mycomponent installation failed"
    return 1
  fi
}
```

### 3. Add Tests (`tests/docker/test_installs.sh`)

```bash
test_mycomponent() {
  if command -v mycomponent &>/dev/null; then
    success "mycomponent"
  else
    fail "mycomponent"
  fi
}
```

### 4. Update Documentation

- Add to the component list in `README.md`
- Document any dependencies or special requirements

## Testing

### Run Unit Tests
```bash
cd cli && cargo test
```

### Run Integration Tests
```bash
docker build -f tests/docker/Dockerfile -t setup-test .
docker run --rm setup-test
```

### Test Shell Scripts Locally
```bash
# Test a specific install function
source bootstrap/scripts/install_modern_cli.sh
install_mycomponent
```

## Project Structure

```
setup/
├── cli/                    # Rust CLI application
│   └── src/
│       ├── commands/       # CLI command implementations
│       ├── config/         # Configuration and dotfile management
│       ├── system/         # System operations (packages, services)
│       └── ui/             # User interaction (prompts, colors)
├── bootstrap/
│   ├── dotfiles/           # Configuration files to deploy
│   └── scripts/            # Shell-based installers (legacy)
├── hooks/                  # Git hooks (commit-msg validation)
├── tests/
│   └── docker/             # Docker-based integration tests
└── .github/workflows/      # CI/CD pipelines
```

## Pull Request Process

1. Create a feature branch: `git checkout -b feat/my-feature`
2. Make your changes with proper commits
3. Run tests: `cargo test` and `pre-commit run --all-files`
4. Push and create a PR against `main`
5. Ensure CI passes
6. Request review

## Questions?

Open an issue or start a discussion on GitHub.
