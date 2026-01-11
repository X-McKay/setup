# Tmux Quick Reference

Tmux is a terminal multiplexer - it lets you run multiple terminal sessions inside a single window, and keeps them running even when you disconnect.

---

## Table of Contents

- [Core Concepts](#core-concepts)
- [Sessions](#sessions)
- [Windows](#windows)
- [Panes](#panes)
- [Copy Mode](#copy-mode)
- [Common Workflows](#common-workflows)
- [Configuration](#configuration)

---

## Core Concepts

**Prefix Key:** All tmux commands start with a prefix. Our config uses `Ctrl+a` (instead of the default `Ctrl+b`).

To execute a tmux command:
1. Press `Ctrl+a`
2. Release
3. Press the command key

Example: To split the window horizontally, press `Ctrl+a` then `-`

**Hierarchy:**
```
Session (a workspace, can have multiple windows)
  └── Window (like a tab, can have multiple panes)
        └── Pane (an individual terminal)
```

---

## Sessions

Sessions are workspaces that persist even when you disconnect.

### Session Commands (from terminal)

```bash
# Create a new session
tmux                        # New session with default name
tmux new -s project         # New session named "project"
tn project                  # Alias for above

# List sessions
tmux ls                     # List all sessions
tl                          # Alias for above

# Attach to a session
tmux attach                 # Attach to last session
tmux attach -t project      # Attach to "project" session
ta project                  # Alias for above

# Kill a session
tmux kill-session -t project
tk project                  # Alias for above
```

### Session Commands (inside tmux)

| Shortcut | Action |
|----------|--------|
| `Ctrl+a d` | Detach from session (keeps it running) |
| `Ctrl+a $` | Rename current session |
| `Ctrl+a s` | List/switch sessions |
| `Ctrl+a S` | Create new session (custom binding) |
| `Ctrl+a K` | Kill current session (custom binding) |
| `Ctrl+a (` | Previous session |
| `Ctrl+a )` | Next session |

---

## Windows

Windows are like tabs within a session.

| Shortcut | Action |
|----------|--------|
| `Ctrl+a c` | Create new window |
| `Ctrl+a ,` | Rename current window |
| `Ctrl+a w` | List all windows (interactive) |
| `Ctrl+a n` | Next window |
| `Ctrl+a p` | Previous window |
| `Ctrl+a 1-9` | Switch to window 1-9 |
| `Alt+1-9` | Switch to window 1-9 (no prefix needed) |
| `Ctrl+a &` | Kill current window |
| `Ctrl+a l` | Toggle to last window |

**Tips:**
- Windows are numbered starting from 1 (not 0)
- Use `Alt+1` through `Alt+5` for quick window switching without prefix

---

## Panes

Panes split a window into multiple terminals.

### Creating Panes

| Shortcut | Action |
|----------|--------|
| `Ctrl+a |` | Split horizontally (side by side) |
| `Ctrl+a -` | Split vertically (top and bottom) |

### Navigating Panes

| Shortcut | Action |
|----------|--------|
| `Ctrl+a h` | Move to left pane |
| `Ctrl+a j` | Move to down pane |
| `Ctrl+a k` | Move to up pane |
| `Ctrl+a l` | Move to right pane |
| `Alt+←` | Move to left pane (no prefix) |
| `Alt+→` | Move to right pane (no prefix) |
| `Alt+↑` | Move to up pane (no prefix) |
| `Alt+↓` | Move to down pane (no prefix) |
| `Ctrl+a o` | Cycle through panes |
| `Ctrl+a ;` | Toggle to last active pane |
| `Ctrl+a q` | Show pane numbers (press number to jump) |

### Resizing Panes

| Shortcut | Action |
|----------|--------|
| `Ctrl+a H` | Resize left (5 cells) |
| `Ctrl+a J` | Resize down (5 cells) |
| `Ctrl+a K` | Resize up (5 cells) |
| `Ctrl+a L` | Resize right (5 cells) |

Hold `Ctrl+a` and press `H/J/K/L` repeatedly for continuous resizing.

### Pane Management

| Shortcut | Action |
|----------|--------|
| `Ctrl+a x` | Kill current pane |
| `Ctrl+a z` | Toggle pane zoom (fullscreen) |
| `Ctrl+a !` | Break pane into new window |
| `Ctrl+a Space` | Cycle through pane layouts |
| `Ctrl+a {` | Swap with previous pane |
| `Ctrl+a }` | Swap with next pane |

---

## Copy Mode

Copy mode lets you scroll, search, and copy text.

### Entering Copy Mode

| Shortcut | Action |
|----------|--------|
| `Ctrl+a [` | Enter copy mode |
| Mouse scroll | Enter copy mode (if mouse enabled) |

### Navigation in Copy Mode

| Key | Action |
|-----|--------|
| `h/j/k/l` | Move cursor (vim-style) |
| `Ctrl+u` | Page up |
| `Ctrl+d` | Page down |
| `g` | Go to top |
| `G` | Go to bottom |
| `/` | Search forward |
| `?` | Search backward |
| `n` | Next search result |
| `N` | Previous search result |

### Selecting and Copying

| Key | Action |
|-----|--------|
| `v` | Begin selection |
| `y` | Copy selection to clipboard |
| `Enter` | Copy selection to clipboard |
| `Escape` | Clear selection |
| `q` | Exit copy mode |

**Copying Workflow:**
1. `Ctrl+a [` to enter copy mode
2. Navigate to start of text
3. `v` to start selection
4. Navigate to end of text
5. `y` to copy
6. `Ctrl+a ]` to paste

**Mouse Selection:**
With mouse mode enabled, you can:
- Click and drag to select text
- Selection is automatically copied
- Right-click to paste (in some terminals)

---

## Common Workflows

### Development Environment

Set up a typical dev environment with one command:

```bash
# Create a session with multiple windows
tmux new-session -s dev \; \
  send-keys 'vim .' C-m \; \
  new-window -n 'server' \; \
  send-keys 'npm run dev' C-m \; \
  new-window -n 'terminal' \; \
  select-window -t 1
```

Or create a script:

```bash
#!/bin/bash
# ~/bin/dev-session.sh

SESSION="dev"
PROJECT_DIR="$1"

tmux new-session -d -s $SESSION -c $PROJECT_DIR

# Window 1: Editor
tmux rename-window -t $SESSION:1 'editor'
tmux send-keys -t $SESSION:1 'vim .' C-m

# Window 2: Server
tmux new-window -t $SESSION:2 -n 'server' -c $PROJECT_DIR
tmux send-keys -t $SESSION:2 'npm run dev' C-m

# Window 3: Terminal
tmux new-window -t $SESSION:3 -n 'terminal' -c $PROJECT_DIR

# Window 4: Git (with lazygit)
tmux new-window -t $SESSION:4 -n 'git' -c $PROJECT_DIR
tmux send-keys -t $SESSION:4 'lg' C-m

# Attach to session on window 1
tmux select-window -t $SESSION:1
tmux attach-session -t $SESSION
```

### Split for Logs

Watch logs while working:

```
Ctrl+a |          # Split horizontally
Ctrl+a l          # Move to right pane
tail -f logs.txt  # Start log tail
Ctrl+a h          # Move back to left pane
# Work in left pane while watching logs on right
```

### Quick Reference Layout

Keep a reference visible:

```
Ctrl+a -          # Split vertically
Ctrl+a j          # Move to bottom pane
man tmux          # or 'glow ~/docs/TMUX.md'
Ctrl+a z          # Zoom the man page when needed
Ctrl+a z          # Unzoom to see both
```

---

## Configuration

Your tmux config is at `~/.tmux.conf`. To edit:

```bash
vtmux              # Opens in your editor (alias)
# or
vim ~/.tmux.conf
```

After editing, reload without restarting:

```
Ctrl+a r           # Reload config (shows "Config reloaded!")
```

### Key Customizations in Your Config

| Feature | Setting |
|---------|---------|
| Prefix | `Ctrl+a` (instead of `Ctrl+b`) |
| Mouse | Enabled |
| Base index | 1 (windows start at 1, not 0) |
| Pane base index | 1 |
| History limit | 50,000 lines |
| Status bar | Top of screen |
| Theme | GitHub Dark High Contrast |
| Escape time | 10ms (fast for vim) |

### Theme Colors

The status bar uses GitHub Dark High Contrast colors:
- Background: `#0a0c10`
- Active elements: `#71b7ff` (blue)
- Success indicators: `#26cd4d` (green)
- Warnings: `#f0b72f` (yellow)
- Errors: `#ff9492` (red)

---

## Quick Reference Card

### Most Used Commands

| Shortcut | Action |
|----------|--------|
| `Ctrl+a d` | Detach |
| `Ctrl+a c` | New window |
| `Ctrl+a |` | Split horizontal |
| `Ctrl+a -` | Split vertical |
| `Alt+1-5` | Switch windows |
| `Alt+arrows` | Switch panes |
| `Ctrl+a z` | Zoom pane |
| `Ctrl+a [` | Copy mode |
| `Ctrl+a r` | Reload config |

### Terminal Commands

| Alias | Command |
|-------|---------|
| `tn name` | `tmux new -s name` |
| `ta name` | `tmux attach -t name` |
| `tl` | `tmux list-sessions` |
| `tk name` | `tmux kill-session -t name` |

---

## Troubleshooting

**Prefix not working?**
- Make sure you're pressing `Ctrl+a`, not `Ctrl+b`
- Release the prefix before pressing the command key

**Colors look wrong?**
- Make sure your terminal supports 256 colors
- Check that `$TERM` is set to `xterm-256color` or `tmux-256color`

**Mouse selection not copying?**
- Hold `Shift` while selecting to use terminal's native selection
- Or use copy mode: `Ctrl+a [`, select, `y` to copy

**Pane borders invisible?**
- This can happen with some color schemes
- The config uses subtle borders; they're there but minimal
