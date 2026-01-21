# Modern CLI Tools Guide

A quick reference for all the tools included in this setup. These tools are designed to make your terminal workflow faster and more enjoyable.

---

## Table of Contents

- [Navigation & Files](#navigation--files)
- [Git Workflow](#git-workflow)
- [Search & Find](#search--find)
- [Task Running](#task-running)
- [System Monitoring](#system-monitoring)
- [Data Processing](#data-processing)
- [Utilities](#utilities)

---

## Navigation & Files

### Eza - Modern ls Replacement

Better directory listings with colors and git integration.

```bash
# Basic listing (aliased to ls)
ls                  # List files
ll                  # Long format with git status
la                  # Show hidden files
lt                  # Tree view (2 levels deep)
ldir                # List only directories

# Useful flags
eza -l --git        # Show git status for each file
eza -T --level=3    # Tree view, 3 levels deep
eza -l --sort=mod   # Sort by modification time
eza -l --sort=size  # Sort by size
```

---

### Bat - Better cat with Syntax Highlighting

View files with syntax highlighting and line numbers.

```bash
# View a file (aliased to cat)
cat file.py         # Syntax highlighted output
bat file.py         # Same thing, explicit

# Useful options
bat -n file.py      # Show line numbers only (no decorations)
bat -p file.py      # Plain output (no line numbers or headers)
bat -A file.py      # Show non-printable characters
bat --diff file.py  # Show git diff for file

# View multiple files
bat *.js            # Concatenate with headers

# Use as a pager
git diff | bat      # Colorized git diff
man ls | bat -l man # Colorized man pages (already configured)
```

---

## Git Workflow

### Lazygit - Terminal UI for Git

A powerful terminal interface for git that makes complex operations simple.

```bash
# Launch lazygit
lg                  # Opens in current repo
lazygit             # Same thing

# Or launch in a specific directory
lazygit -p /path/to/repo
```

**Keyboard Shortcuts (inside lazygit):**

| Key | Action |
|-----|--------|
| `?` | Show all keybindings |
| `h/l` | Switch between panels |
| `j/k` | Navigate up/down |
| `Enter` | Focus on item / expand |
| `Space` | Stage/unstage file |
| `a` | Stage all files |
| `c` | Commit |
| `P` | Push |
| `p` | Pull |
| `b` | Branch operations |
| `m` | Merge |
| `r` | Rebase |
| `s` | Stash |
| `S` | Stash options menu |
| `[` / `]` | Previous/next tab |
| `@` | Open command log |
| `+` | Expand all / collapse all |
| `q` | Quit |

**Common Workflows:**

```
# Stage specific lines (interactive staging)
1. Navigate to file in Files panel
2. Press Enter to see diff
3. Use Space to stage individual lines/hunks

# Interactive rebase
1. Go to Commits panel
2. Navigate to commit to start from
3. Press 'e' to edit, 's' to squash, 'd' to drop

# Cherry-pick commits
1. Go to Commits panel
2. Press 'C' to copy commit
3. Switch to target branch
4. Press 'V' to paste (cherry-pick)
```

---

### GitHub CLI (gh)

Interact with GitHub from your terminal.

```bash
# Authentication
gh auth login       # Authenticate with GitHub
gh auth status      # Check auth status

# Repository operations
gh repo clone owner/repo    # Clone a repo
gh repo view               # View current repo info
gh repo view --web         # Open repo in browser
ghrv                       # Alias for above

# Pull Requests
ghpr                       # Create a PR (alias)
ghprl                      # List PRs
ghprv                      # View current PR
ghprc 123                  # Checkout PR #123
gh pr merge                # Merge current PR
gh pr checks               # View CI status

# Issues
ghis                       # List issues (alias)
ghic                       # Create issue
ghiv 123                   # View issue #123

# Workflow/Actions
gh run list                # List workflow runs
gh run view                # View a run
gh run watch               # Watch a run in progress

# Useful commands
gh pr create --fill        # Create PR with auto-filled title/body
gh pr create --draft       # Create as draft PR
gh issue create --label bug --assignee @me
```

---

### Delta - Beautiful Git Diffs

Delta is configured automatically for lazygit. For command-line git:

```bash
# Delta is automatically used for git diff
git diff                   # Shows colorized side-by-side diff
git show HEAD              # Colorized commit view
git log -p                 # Colorized patch view

# Configure git to use delta (already done in .gitconfig)
git config --global core.pager delta
git config --global interactive.diffFilter 'delta --color-only'
```

---

## Search & Find

### fzf - Fuzzy Finder

A general-purpose fuzzy finder that integrates everywhere.

```bash
# Search command history (Ctrl+R)
# Press Ctrl+R, type part of command, select with arrows

# Search files (Ctrl+T)
# Press Ctrl+T to insert a file path into current command

# Change directory (Alt+C)
# Press Alt+C to fuzzy-find and cd to a directory

# Pipe anything to fzf
cat file.txt | fzf         # Fuzzy search lines
ps aux | fzf               # Find a process
env | fzf                  # Search environment variables

# Preview files while searching
fzf --preview 'bat --color=always {}'

# Select multiple items
fzf -m                     # Use Tab to select multiple

# Common patterns
vim $(fzf)                 # Open selected file in vim
cd $(find . -type d | fzf) # cd to selected directory
kill $(ps aux | fzf | awk '{print $2}')  # Kill selected process
```

---

### fd - Modern find Replacement

A faster, user-friendly alternative to `find`.

```bash
# Basic search (searches current directory recursively)
fd pattern              # Find files matching pattern
fd "\.py$"              # Find all Python files
fd config               # Find anything with "config" in name

# Search specific file types
fd -e py                # Find .py files
fd -e js -e ts          # Find .js and .ts files

# Search options
fd -H pattern           # Include hidden files
fd -I pattern           # Don't ignore .gitignore patterns
fd -t f pattern         # Files only
fd -t d pattern         # Directories only

# Combine with other tools
fd -e py | xargs wc -l  # Count lines in all Python files
fd -e test.js -x npm test {}  # Run npm test on each test file

# Search from specific path
fd pattern /path/to/search
```

---

### Ripgrep (rg) - Fast Text Search

Blazingly fast text search (already aliased in many tools).

```bash
# Basic search
rg "pattern"            # Search in current directory
rg "TODO" --type py     # Search only in Python files
rg "function" src/      # Search in specific directory

# Common options
rg -i "pattern"         # Case-insensitive
rg -w "word"            # Match whole words only
rg -l "pattern"         # Only show file names
rg -c "pattern"         # Show count per file
rg -C 3 "pattern"       # Show 3 lines of context

# Advanced usage
rg "pattern" -g "*.js"  # Only in .js files
rg "pattern" -g "!test*" # Exclude test files
rg "^import" --type ts  # Regex: lines starting with import
```

---

## Task Running

### Just - Command Runner

A modern alternative to Make for running project tasks.

```bash
# List available tasks
just --list             # or 'jl' alias
just -l                 # Short form

# Run a task
just build              # Run the 'build' task
just test               # Run tests
j test                  # Using alias

# Run with arguments
just test --verbose     # Pass args to the recipe
just deploy prod        # Pass 'prod' as argument

# Run multiple tasks
just clean build test   # Run in sequence

# Choose recipe interactively
just --choose           # Fuzzy select with fzf
```

**Creating a Justfile:**

```just
# Justfile - place in project root

# Default recipe (runs when you just type 'just')
default:
    @just --list

# Build the project
build:
    cargo build --release

# Run tests
test *args:
    cargo test {{args}}

# Format code
fmt:
    cargo fmt

# Run dev server
dev:
    cargo watch -x run

# Deploy to production
deploy env:
    ./scripts/deploy.sh {{env}}
```

**Tips:**
- Create a `justfile` in each project for common tasks
- Use `just --choose` for interactive selection
- Copy the template from `bootstrap/templates/justfile`

---

## System Monitoring

### Bottom (btm) - System Monitor

A beautiful, cross-platform system monitor.

```bash
# Launch bottom
btm                     # Default view

# Keyboard shortcuts inside btm:
```

| Key | Action |
|-----|--------|
| `?` | Help |
| `q` | Quit |
| `e` | Expand/collapse selected widget |
| `Tab` | Move to next widget |
| `Shift+Tab` | Move to previous widget |
| `/` | Search (in process list) |
| `s` | Sort by column |
| `I` | Invert sort order |
| `dd` | Kill selected process |
| `c` | CPU view |
| `m` | Memory view |
| `n` | Network view |
| `p` | Process view |

**Views:**
- Press `1-9` to switch to different layouts
- Press `b` for basic mode (less detailed)

---

## Data Processing

### jq - JSON Processor

The Swiss Army knife for JSON data processing.

```bash
# Basic usage
cat data.json | jq '.'           # Pretty print JSON
cat data.json | jq '.key'        # Extract a key
cat data.json | jq '.items[]'    # Iterate over array

# Common operations
jq '.name' file.json             # Get field
jq '.users[0]' file.json         # First array element
jq '.users | length' file.json   # Count items
jq -r '.name' file.json          # Raw output (no quotes)

# Filtering
jq '.[] | select(.age > 30)'     # Filter by condition
jq '.[] | select(.name | contains("john"))'

# Transformation
jq '{name: .username, id: .user_id}'  # Reshape object
jq '[.[] | {name, age}]'              # Project fields

# Combining with other tools
curl -s api.example.com | jq '.data'
kubectl get pods -o json | jq '.items[].metadata.name'
```

**Aliases:**
- `jqc` - Colorized output (`jq -C`)
- `jqs` - Sort keys (`jq -S`)

---

### yq - YAML Processor

Like jq, but for YAML files. Essential for Kubernetes, Docker Compose, etc.

```bash
# Basic usage
yq '.' file.yaml                 # Pretty print
yq '.metadata.name' file.yaml    # Extract field
yq -o json file.yaml             # Convert to JSON

# Reading values
yq '.spec.replicas' deployment.yaml
yq '.services | keys' docker-compose.yml

# Modifying YAML
yq '.spec.replicas = 3' deployment.yaml           # Set value
yq '.metadata.labels.env = "prod"' file.yaml      # Add field
yq 'del(.metadata.annotations)' file.yaml         # Delete field

# Multiple documents
yq eval-all '. | select(.kind == "Deployment")' manifests.yaml

# In-place editing
yq -i '.version = "2.0"' config.yaml

# Merge files
yq '. *= load("override.yaml")' base.yaml
```

**Common Use Cases:**
```bash
# Update image in deployment
yq -i '.spec.template.spec.containers[0].image = "nginx:1.25"' deployment.yaml

# Extract all service names from docker-compose
yq '.services | keys | .[]' docker-compose.yml

# Convert between formats
yq -o json config.yaml > config.json
yq -P config.json > config.yaml
```

---

### Hyperfine - Command Benchmarking

Precise benchmarking for command-line programs.

```bash
# Basic benchmark
hyperfine 'sleep 0.3'

# Compare two commands
hyperfine 'fd pattern' 'find . -name "*pattern*"'

# With warmup runs (recommended)
hyperfine --warmup 3 'cargo build'
benchw 'cargo build'              # Alias

# Multiple runs
hyperfine --runs 20 'npm test'

# Export results
hyperfine 'cmd1' 'cmd2' --export-markdown results.md
hyperfine 'cmd1' 'cmd2' --export-json results.json

# With setup/cleanup
hyperfine --prepare 'make clean' 'make build'

# Parameter ranges
hyperfine --parameter-range threads 1 8 'program --threads {threads}'

# Compare shell commands
hyperfine 'grep -r pattern .' 'rg pattern'
```

**Output Example:**
```
Benchmark 1: fd .rs
  Time (mean ± σ):      23.4 ms ±   1.2 ms
  Range (min … max):    21.8 ms …  28.1 ms

Benchmark 2: find . -name "*.rs"
  Time (mean ± σ):     312.5 ms ±  14.3 ms
  Range (min … max):   298.2 ms … 341.7 ms

Summary
  'fd .rs' ran 13.35 ± 0.89 times faster than 'find . -name "*.rs"'
```

**Aliases:**
- `bench` - Run hyperfine
- `benchw` - Run with 3 warmup iterations

---

### tldr - Simplified Man Pages

Community-maintained help pages with practical examples.

```bash
# Get quick help for a command
tldr tar                  # Shows common tar examples
tldr git-rebase          # Git subcommands use dashes
tldr ffmpeg              # Complex tools made simple

# Using the alias
help tar                  # Same as tldr tar

# Update the local cache
tldr --update

# List all available pages
tldr --list

# Search for pages
tldr --search compress
```

**Example Output for `tldr tar`:**
```
  tar

  Archiving utility.
  Often combined with a compression method, such as gzip or bzip2.

  - Create an archive from files:
    tar cf target.tar file1 file2 file3

  - Create a gzipped archive:
    tar czf target.tar.gz file1 file2 file3

  - Extract an archive in the current directory:
    tar xf source.tar

  - Extract a gzipped archive:
    tar xzf source.tar.gz

  - Extract to a directory:
    tar xf source.tar -C directory
```

**Tips:**
- Use `tldr` before `man` - it shows what you actually need
- Great for commands you use rarely (tar, ffmpeg, rsync)
- Pages are updated regularly with new examples

---

## Utilities

### Glow - Markdown Renderer

Render markdown beautifully in the terminal.

```bash
# View a markdown file
glow README.md          # Render with paging
md README.md            # Alias
readme                  # View README.md in current dir

# Options
glow -p README.md       # Plain output (no pager)
glow -w 80 README.md    # Set width

# Browse markdown files
glow                    # Opens file browser
```

---

## Quick Reference Card

### Essential Shortcuts

| Shortcut | Tool | Action |
|----------|------|--------|
| `lg` | lazygit | Open git TUI |
| `j` / `jl` | just | Run task / list tasks |
| `Ctrl+R` | fzf | Search command history |
| `Ctrl+T` | fzf | Insert file path |
| `Alt+C` | fzf | Change directory |
| `btm` | bottom | System monitor |
| `md <file>` | glow | Render markdown |
| `help <cmd>` | tldr | Quick command examples |
| `bench` | hyperfine | Benchmark commands |

### File Operations

| Command | Description |
|---------|-------------|
| `ll` | Long listing with git status |
| `lt` | Tree view |
| `cat <file>` | View with syntax highlighting |
| `fd <pattern>` | Find files |
| `rg <pattern>` | Search in files |

### Data Processing

| Command | Description |
|---------|-------------|
| `jq '.key' file.json` | Extract JSON field |
| `yq '.key' file.yaml` | Extract YAML field |
| `jq -r` | Raw output (no quotes) |
| `yq -o json` | Convert YAML to JSON |

### Git Shortcuts

| Alias | Command |
|-------|---------|
| `gs` | `git status` |
| `ga` | `git add` |
| `gc` | `git commit` |
| `gp` | `git push` |
| `gl` | `git pull` |
| `gd` | `git diff` |
| `glog` | `git log --oneline --graph` |
| `ghpr` | `gh pr create` |
| `ghprl` | `gh pr list` |

---

## Tips for Efficient Development

1. **Master lazygit** - It makes git operations 10x faster. Press `?` to learn shortcuts

2. **Create justfiles** - Every project should have common tasks documented in a justfile

3. **Use fzf integration** - `Ctrl+R` for history, `Ctrl+T` for files, `Alt+C` for directories

4. **Let tools help you** - bat for viewing, fd for finding, rg for searching - they're all faster than the defaults

5. **Use tldr before man** - `help tar` gives you practical examples instead of 50 pages of options

6. **Benchmark before optimizing** - Use `hyperfine` to measure actual performance differences

7. **Master jq/yq** - Essential for working with JSON/YAML APIs, configs, and Kubernetes

8. **Learn one tool at a time** - Don't try to memorize everything at once
