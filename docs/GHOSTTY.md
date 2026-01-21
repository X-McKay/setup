# Ghostty Terminal Emulator

Ghostty is a fast, feature-rich, and cross-platform terminal emulator that uses platform-native UI and GPU acceleration.

---

## Table of Contents

- [Overview](#overview)
- [Installation](#installation)
- [Configuration](#configuration)
- [Theme](#theme)
- [Keybindings](#keybindings)
- [Integration with Tmux](#integration-with-tmux)
- [Font Setup](#font-setup)
- [Tips & Tricks](#tips--tricks)
- [Troubleshooting](#troubleshooting)

---

## Overview

Ghostty is designed to be:
- **Fast** - GPU-accelerated rendering for smooth scrolling and low latency
- **Native** - Uses platform-native UI elements (GTK on Linux)
- **Feature-rich** - Tabs, splits, ligatures, true color, and more
- **Simple** - Single config file, sensible defaults

Your configuration is at `~/.config/ghostty/config`.

---

## Installation

### From Package Manager (Recommended)

```bash
# Ubuntu/Debian (if available in repos)
sudo apt install ghostty

# Or download the latest release from:
# https://github.com/ghostty-org/ghostty/releases
```

### Building from Source

```bash
# Requires Zig compiler
git clone https://github.com/ghostty-org/ghostty.git
cd ghostty
zig build -Doptimize=ReleaseFast
```

---

## Configuration

Configuration is done through a simple key-value file at `~/.config/ghostty/config`.

### Editing the Config

```bash
# Open in your editor
vghost              # Alias (if configured)
# or
vim ~/.config/ghostty/config
```

### Reloading Configuration

```
Super+Shift+,       # Reload config without restarting
```

Or restart Ghostty for changes to take effect.

### Key Configuration Options

| Setting | Description | Our Value |
|---------|-------------|-----------|
| `font-family` | Primary font | JetBrainsMono Nerd Font |
| `font-size` | Font size in points | 12 |
| `font-thicken` | Make fonts bolder | true |
| `window-padding-x/y` | Inner padding | 8px |
| `cursor-style` | Cursor shape | block |
| `cursor-style-blink` | Blinking cursor | false |
| `scrollback-limit` | History lines | 50,000 |
| `copy-on-select` | Auto-copy selection | clipboard |
| `shell-integration` | Shell type | bash |

---

## Theme

This setup uses the **GitHub Dark High Contrast** theme for consistency across all tools.

### Color Palette

| Color | Hex | Usage |
|-------|-----|-------|
| Background | `#0a0c10` | Main background (pure dark) |
| Foreground | `#f0f3f6` | Default text (high contrast white) |
| Cursor | `#71b7ff` | Blue cursor for visibility |
| Red | `#ff9492` | Errors, deletions |
| Green | `#26cd4d` | Success, additions |
| Yellow | `#f0b72f` | Warnings, modifications |
| Blue | `#71b7ff` | Links, info |
| Magenta | `#cb9eff` | Special, constants |
| Cyan | `#39c5cf` | Strings, paths |

### Visual Preview

```
Normal Colors:
  Black   Red     Green   Yellow  Blue    Magenta Cyan    White
  #7a828e #ff9492 #26cd4d #f0b72f #71b7ff #cb9eff #39c5cf #d9dee3

Bright Colors:
  #9ea7b3 #ffb1af #4ae168 #f7c843 #91cbff #dbb7ff #56d4dd #ffffff
```

---

## Keybindings

Our configuration uses `Super` (Cmd on Mac, Win key on Linux) for Ghostty actions to avoid conflicts with tmux (`Ctrl+a`).

### Window & Tab Management

| Shortcut | Action |
|----------|--------|
| `Super+n` | New window |
| `Super+t` | New tab |
| `Super+w` | Close current tab/pane |
| `Super+1-5` | Switch to tab 1-5 |
| `Super+Shift+Enter` | Toggle fullscreen |

### Split Panes

| Shortcut | Action |
|----------|--------|
| `Ctrl+d` | Split pane right (vertical divider) |
| `Ctrl+Shift+d` | Split pane down (horizontal divider) |

### Clipboard

| Shortcut | Action |
|----------|--------|
| `Super+c` | Copy to clipboard |
| `Super+v` | Paste from clipboard |
| `Super+Shift+c` | Copy (alternative) |
| `Super+Shift+v` | Paste (alternative) |

**Note:** With `copy-on-select = clipboard`, text is automatically copied when you select it with the mouse.

### Font Size

| Shortcut | Action |
|----------|--------|
| `Super+=` | Increase font size |
| `Super+-` | Decrease font size |
| `Super+0` | Reset font size |

### Configuration

| Shortcut | Action |
|----------|--------|
| `Super+Shift+,` | Reload configuration |

---

## Integration with Tmux

Ghostty and tmux are designed to work together seamlessly in this setup.

### Key Binding Strategy

- **Ghostty** uses `Super` (Win/Cmd) key for terminal-level actions
- **Tmux** uses `Ctrl+a` prefix for session/window/pane management
- **No conflicts** - each tool has its own modifier

### Workflow Example

```
Super+t          → Create new Ghostty tab (for separate projects)
Ctrl+a c         → Create new tmux window (within same session)
Ctrl+a |         → Split tmux pane vertically
Ctrl+a -         → Split tmux pane horizontally
```

### When to Use What

| Task | Use |
|------|-----|
| Multiple projects | Ghostty tabs (`Super+t`) |
| Multiple contexts in one project | Tmux sessions |
| Side-by-side terminals | Tmux panes (`Ctrl+a |` or `-`) |
| Quick terminal tab switch | Ghostty (`Super+1-5`) |
| Persistent sessions (survive restart) | Tmux |
| Copy text across applications | Ghostty (`Super+c/v`) |
| Copy within terminal | Tmux copy mode or mouse select |

### True Color Support

Both Ghostty and tmux in this setup support true (24-bit) color:

```bash
# Test true color support
curl -s https://raw.githubusercontent.com/JohnMorales/dotfiles/master/colors/24-bit-color.sh | bash
```

---

## Font Setup

### Installing JetBrains Mono Nerd Font

The install script sets this up automatically, but manually:

```bash
# Download and install
mkdir -p ~/.local/share/fonts
cd ~/.local/share/fonts
curl -fLo "JetBrainsMono.zip" \
  https://github.com/ryanoasis/nerd-fonts/releases/download/v3.1.1/JetBrainsMono.zip
unzip JetBrainsMono.zip
rm JetBrainsMono.zip

# Refresh font cache
fc-cache -fv
```

### Why Nerd Fonts?

Nerd Fonts include:
- Programming ligatures (-> becomes →)
- Powerline symbols for status lines
- File type icons for lazygit, etc.

### Verifying Font Installation

```bash
# Check if font is available
fc-list | grep -i jetbrains

# Test icons in terminal
echo -e "\ue0b0 \ue0b2 \uf1d3 \uf121 \uf7a1"
# Should show: arrow, arrow, git icon, code icon, rocket
```

---

## Tips & Tricks

### 1. Quick Config Editing

```bash
# Edit config and reload instantly
vim ~/.config/ghostty/config
# Press Super+Shift+, to reload (no restart needed)
```

### 2. Mouse Features

- **Click and drag** - Select text (auto-copies with `copy-on-select`)
- **Double-click** - Select word
- **Triple-click** - Select line
- **Shift+click** - Extend selection
- **Middle-click** - Paste (X11 primary selection)

### 3. URL Handling

Ghostty can detect and open URLs:
- Hold `Ctrl` and click on a URL to open in browser

### 4. Scrolling

```
Shift+PageUp/Down    # Scroll by page
Shift+Home/End       # Jump to top/bottom of scrollback
Mouse wheel          # Smooth scroll
```

### 5. Search in Scrollback

While Ghostty itself doesn't have built-in search, use tmux copy mode:
```
Ctrl+a [             # Enter tmux copy mode
/                    # Search forward
?                    # Search backward
n/N                  # Next/previous match
```

### 6. Performance Tuning

If you experience issues:

```ini
# In ~/.config/ghostty/config

# Disable GPU rendering (if graphics issues)
# renderer = software

# Reduce scrollback for memory
scrollback-limit = 10000

# Disable font thickening
font-thicken = false
```

---

## Troubleshooting

### Font Icons Not Displaying

**Problem:** Boxes or question marks instead of icons

**Solution:**
1. Verify Nerd Font is installed: `fc-list | grep -i nerd`
2. Check font name matches config: `font-family = "JetBrainsMono Nerd Font"`
3. Rebuild font cache: `fc-cache -fv`
4. Restart Ghostty

### Colors Look Wrong

**Problem:** Colors don't match expected theme

**Solution:**
1. Ensure `$TERM` is correct: `echo $TERM` (should be `xterm-256color` or `ghostty`)
2. Check true color support: Run the 24-bit color test script above
3. Verify config syntax: Check for typos in hex values (no `#` prefix in Ghostty)

### Keybindings Not Working

**Problem:** `Super` key shortcuts don't work

**Solutions:**
- On some Linux systems, the desktop environment captures `Super` key
- Try: System Settings → Keyboard → Shortcuts → Disable conflicting bindings
- Alternative: Change Ghostty bindings to use `alt+shift` instead of `super`

### Config Changes Not Applied

**Problem:** Changes to config file have no effect

**Solution:**
1. Press `Super+Shift+,` to reload config
2. Check config path: `~/.config/ghostty/config` (no extension)
3. Verify syntax: Each line should be `key = value`
4. Check for errors: Run `ghostty --help` to verify installation

### Tmux Colors Off Inside Ghostty

**Problem:** Tmux theme looks different in Ghostty vs other terminals

**Solution:**
Ensure tmux is using proper terminal type:
```bash
# In ~/.tmux.conf
set -g default-terminal "tmux-256color"
set -sa terminal-features ',xterm-256color:RGB'
```

### Copy/Paste Issues

**Problem:** Can't paste or copy doesn't work

**Solutions:**
- Ensure `copy-on-select = clipboard` is set
- For tmux: Make sure xclip is installed: `sudo apt install xclip`
- Use `Super+c/v` instead of `Ctrl+c/v` (which sends signals)

---

## Quick Reference Card

### Essential Shortcuts

| Shortcut | Action |
|----------|--------|
| `Super+t` | New tab |
| `Super+w` | Close tab |
| `Super+1-5` | Switch tabs |
| `Ctrl+d` | Split right |
| `Ctrl+Shift+d` | Split down |
| `Super+c/v` | Copy/Paste |
| `Super+=/-` | Font size |
| `Super+Shift+Enter` | Fullscreen |
| `Super+Shift+,` | Reload config |

### Config Location

```
~/.config/ghostty/config
```

### Theme Colors

```
Background: #0a0c10 (pure dark)
Foreground: #f0f3f6 (bright white)
Accent:     #71b7ff (blue)
```

---

## Resources

- [Ghostty GitHub](https://github.com/ghostty-org/ghostty)
- [Ghostty Documentation](https://ghostty.org/docs)
- [Nerd Fonts](https://www.nerdfonts.com/)
- [GitHub Dark Theme Reference](https://github.com/primer/primitives)
