//! Individual development tool installers.

use anyhow::{Context, Result};
use std::fs;
use super::utils::{
    ensure_bin_dir, fallback_versions, fetch_github_version, get_arch, get_arch_alt,
    path_to_str, run_command, run_sudo,
};

/// Install mise version manager.
pub fn install_mise() -> Result<()> {
    if which::which("mise").is_ok() {
        run_mise_install()?;
        return Ok(());
    }

    let script = run_command("curl", &["-fsSL", "https://mise.run"])?;
    run_command("sh", &["-c", &script])?;
    run_mise_install()?;

    Ok(())
}

fn run_mise_install() -> Result<()> {
    let home = dirs::home_dir().context("Could not find home directory")?;
    let tool_versions = home.join(".tool-versions");

    if tool_versions.exists() {
        let mise_path = home.join(".local").join("bin").join("mise");
        if mise_path.exists() {
            let _ = run_command(path_to_str(&mise_path)?, &["install"]);
        } else if which::which("mise").is_ok() {
            let _ = run_command("mise", &["install"]);
        }
    }
    Ok(())
}

/// Install Docker.
pub fn install_docker() -> Result<()> {
    if which::which("docker").is_ok() {
        return Ok(());
    }

    run_sudo("apt", &["update"])?;
    run_sudo("apt", &["install", "-y", "ca-certificates", "curl", "gnupg", "lsb-release"])?;

    let script = run_command("curl", &["-fsSL", "https://get.docker.com"])?;
    run_sudo("sh", &["-c", &script])?;

    let user = std::env::var("USER").unwrap_or_else(|_| "al".to_string());
    run_sudo("usermod", &["-aG", "docker", &user])?;

    Ok(())
}

/// Install lazygit TUI.
pub fn install_lazygit() -> Result<()> {
    if which::which("lazygit").is_ok() {
        return Ok(());
    }

    let bin_dir = ensure_bin_dir()?;
    let version = fetch_github_version("jesseduffield/lazygit", fallback_versions::LAZYGIT);

    let arch = match std::env::consts::ARCH {
        "x86_64" => "x86_64",
        "aarch64" => "arm64",
        _ => return Err(anyhow::anyhow!("Unsupported architecture")),
    };

    let url = format!(
        "https://github.com/jesseduffield/lazygit/releases/download/v{}/lazygit_{}_Linux_{}.tar.gz",
        version, version, arch
    );

    run_command("sh", &["-c", &format!(
        "curl -Lo /tmp/lazygit.tar.gz '{}' && tar xf /tmp/lazygit.tar.gz -C {} lazygit",
        url, bin_dir.display()
    )])?;

    Ok(())
}

/// Install just task runner.
pub fn install_just() -> Result<()> {
    if which::which("just").is_ok() {
        return Ok(());
    }

    let bin_dir = ensure_bin_dir()?;
    run_command("sh", &["-c", &format!(
        "curl --proto '=https' --tlsv1.2 -sSf https://just.systems/install.sh | bash -s -- --to {}",
        bin_dir.display()
    )])?;

    Ok(())
}

/// Install glow markdown renderer.
pub fn install_glow() -> Result<()> {
    if which::which("glow").is_ok() {
        return Ok(());
    }

    if run_sudo("apt", &["install", "-y", "glow"]).is_ok() {
        return Ok(());
    }

    let bin_dir = ensure_bin_dir()?;
    let version = fetch_github_version("charmbracelet/glow", fallback_versions::GLOW);

    let arch = match std::env::consts::ARCH {
        "x86_64" => "x86_64",
        "aarch64" => "arm64",
        _ => return Err(anyhow::anyhow!("Unsupported architecture")),
    };

    let url = format!(
        "https://github.com/charmbracelet/glow/releases/download/v{}/glow_{}_Linux_{}.tar.gz",
        version, version, arch
    );

    run_command("sh", &["-c", &format!(
        "curl -Lo /tmp/glow.tar.gz '{}' && tar xf /tmp/glow.tar.gz -C /tmp && mv /tmp/glow_*/glow {}",
        url, bin_dir.display()
    )])?;

    Ok(())
}

/// Install bottom system monitor.
pub fn install_bottom() -> Result<()> {
    if which::which("btm").is_ok() {
        return Ok(());
    }

    if run_sudo("apt", &["install", "-y", "bottom"]).is_ok() {
        return Ok(());
    }

    let bin_dir = ensure_bin_dir()?;
    let version = fetch_github_version("ClementTsang/bottom", fallback_versions::BOTTOM);
    let arch = get_arch()?;

    let url = format!(
        "https://github.com/ClementTsang/bottom/releases/download/{}/bottom_{}-unknown-linux-gnu.tar.gz",
        version, arch
    );

    run_command("sh", &["-c", &format!(
        "curl -Lo /tmp/bottom.tar.gz '{}' && tar xf /tmp/bottom.tar.gz -C {} btm",
        url, bin_dir.display()
    )])?;

    Ok(())
}

/// Install GitHub CLI.
pub fn install_gh() -> Result<()> {
    if which::which("gh").is_ok() {
        return Ok(());
    }

    run_command("sh", &["-c", r#"(type -p wget >/dev/null || (sudo apt update && sudo apt install wget -y)) \
        && sudo mkdir -p -m 755 /etc/apt/keyrings \
        && out=$(mktemp) && wget -nv -O$out https://cli.github.com/packages/githubcli-archive-keyring.gpg \
        && cat $out | sudo tee /etc/apt/keyrings/githubcli-archive-keyring.gpg > /dev/null \
        && sudo chmod go+r /etc/apt/keyrings/githubcli-archive-keyring.gpg \
        && echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | sudo tee /etc/apt/sources.list.d/github-cli.list > /dev/null \
        && sudo apt update \
        && sudo apt install gh -y"#])?;

    Ok(())
}

/// Install hyperfine benchmarking tool.
pub fn install_hyperfine() -> Result<()> {
    if which::which("hyperfine").is_ok() {
        return Ok(());
    }

    if run_sudo("apt", &["install", "-y", "hyperfine"]).is_ok() {
        return Ok(());
    }

    let bin_dir = ensure_bin_dir()?;
    let version = fetch_github_version("sharkdp/hyperfine", fallback_versions::HYPERFINE);
    let arch = get_arch()?;

    let url = format!(
        "https://github.com/sharkdp/hyperfine/releases/download/v{}/hyperfine-v{}-{}-unknown-linux-musl.tar.gz",
        version, version, arch
    );

    run_command("sh", &["-c", &format!(
        "curl -Lo /tmp/hyperfine.tar.gz '{}' && tar xf /tmp/hyperfine.tar.gz -C /tmp && mv /tmp/hyperfine-*/hyperfine {}",
        url, bin_dir.display()
    )])?;

    Ok(())
}

/// Install jq JSON processor.
pub fn install_jq() -> Result<()> {
    if which::which("jq").is_ok() {
        return Ok(());
    }

    if run_sudo("apt", &["install", "-y", "jq"]).is_ok() {
        return Ok(());
    }

    let bin_dir = ensure_bin_dir()?;
    let arch = get_arch_alt()?;

    run_command("sh", &["-c", &format!(
        "curl -Lo {}/jq 'https://github.com/jqlang/jq/releases/latest/download/jq-linux-{}' && chmod +x {}/jq",
        bin_dir.display(), arch, bin_dir.display()
    )])?;

    Ok(())
}

/// Install yq YAML processor.
pub fn install_yq() -> Result<()> {
    if which::which("yq").is_ok() {
        return Ok(());
    }

    let bin_dir = ensure_bin_dir()?;
    let arch = get_arch_alt()?;

    run_command("sh", &["-c", &format!(
        "curl -Lo {}/yq 'https://github.com/mikefarah/yq/releases/latest/download/yq_linux_{}' && chmod +x {}/yq",
        bin_dir.display(), arch, bin_dir.display()
    )])?;

    Ok(())
}

/// Install tldr simplified man pages.
pub fn install_tldr() -> Result<()> {
    if which::which("tldr").is_ok() {
        return Ok(());
    }

    if run_sudo("apt", &["install", "-y", "tldr"]).is_ok() {
        let _ = run_command("tldr", &["--update"]);
        return Ok(());
    }

    let bin_dir = ensure_bin_dir()?;
    let arch = get_arch()?;

    run_command("sh", &["-c", &format!(
        "curl -Lo {}/tldr 'https://github.com/dbrgn/tealdeer/releases/latest/download/tealdeer-linux-{}-musl' && chmod +x {}/tldr && {}/tldr --update",
        bin_dir.display(), arch, bin_dir.display(), bin_dir.display()
    )])?;

    Ok(())
}

/// Install neovim editor.
pub fn install_neovim() -> Result<()> {
    if !which::which("nvim").is_ok() {
        if run_sudo("apt", &["install", "-y", "neovim"]).is_err() {
            let bin_dir = ensure_bin_dir()?;
            run_command("sh", &["-c", &format!(
                "curl -Lo {}/nvim https://github.com/neovim/neovim/releases/latest/download/nvim.appimage && chmod +x {}/nvim",
                bin_dir.display(), bin_dir.display()
            )])?;
        }
    }

    // Create neovim config
    let home = dirs::home_dir().context("Could not find home directory")?;
    let nvim_config = home.join(".config").join("nvim");
    fs::create_dir_all(&nvim_config)?;

    let init_lua = nvim_config.join("init.lua");
    if !init_lua.exists() {
        let config = r#"-- Sensible Neovim defaults
vim.opt.number = true
vim.opt.relativenumber = true
vim.opt.mouse = 'a'
vim.opt.ignorecase = true
vim.opt.smartcase = true
vim.opt.hlsearch = false
vim.opt.wrap = false
vim.opt.breakindent = true
vim.opt.tabstop = 4
vim.opt.shiftwidth = 4
vim.opt.expandtab = true
vim.opt.termguicolors = true
vim.opt.signcolumn = 'yes'
vim.opt.updatetime = 250
vim.opt.timeoutlen = 300
vim.opt.splitright = true
vim.opt.splitbelow = true
vim.opt.inccommand = 'split'
vim.opt.cursorline = true
vim.opt.scrolloff = 10
vim.opt.clipboard = 'unnamedplus'
vim.opt.undofile = true

vim.g.mapleader = ' '
vim.g.maplocalleader = ' '

vim.keymap.set('n', '<Esc>', '<cmd>nohlsearch<CR>')
vim.keymap.set('n', '<leader>w', '<cmd>w<CR>', { desc = 'Save' })
vim.keymap.set('n', '<leader>q', '<cmd>q<CR>', { desc = 'Quit' })
vim.keymap.set('n', '<C-h>', '<C-w>h')
vim.keymap.set('n', '<C-j>', '<C-w>j')
vim.keymap.set('n', '<C-k>', '<C-w>k')
vim.keymap.set('n', '<C-l>', '<C-w>l')
"#;
        fs::write(&init_lua, config)?;
    }

    Ok(())
}

/// Install Chromium browser.
pub fn install_chromium() -> Result<()> {
    if which::which("chromium-browser").is_ok() || which::which("chromium").is_ok() {
        return Ok(());
    }

    run_sudo("snap", &["install", "chromium"])?;

    Ok(())
}

/// Install Discord chat client.
pub fn install_discord() -> Result<()> {
    if which::which("discord").is_ok() {
        return Ok(());
    }

    run_sudo("snap", &["install", "discord"])?;

    Ok(())
}

/// Install Obsidian note-taking app.
pub fn install_obsidian() -> Result<()> {
    if which::which("obsidian").is_ok() {
        return Ok(());
    }

    run_sudo("snap", &["install", "obsidian", "--classic"])?;

    Ok(())
}

/// Install Spotify music player.
pub fn install_spotify() -> Result<()> {
    if which::which("spotify").is_ok() {
        return Ok(());
    }

    run_sudo("snap", &["install", "spotify"])?;

    Ok(())
}

/// Install VLC media player.
pub fn install_vlc() -> Result<()> {
    if which::which("vlc").is_ok() {
        return Ok(());
    }

    run_sudo("snap", &["install", "vlc"])?;

    Ok(())
}

/// Install Ghostty terminal emulator.
pub fn install_ghostty() -> Result<()> {
    if which::which("ghostty").is_ok() {
        return Ok(());
    }

    run_sudo("snap", &["install", "ghostty", "--classic"])?;

    Ok(())
}

/// Install Claude Code CLI.
pub fn install_claude_code() -> Result<()> {
    if which::which("claude").is_ok() {
        return Ok(());
    }

    let script = run_command("curl", &["-fsSL", "https://claude.ai/install.sh"])?;
    run_command("sh", &["-c", &script])?;

    Ok(())
}

/// Install tmux plugin manager.
pub fn install_tpm() -> Result<()> {
    let home = dirs::home_dir().context("Could not find home directory")?;
    let tpm_dir = home.join(".tmux").join("plugins").join("tpm");

    if tpm_dir.exists() {
        return Ok(());
    }

    run_command("git", &["clone", "https://github.com/tmux-plugins/tpm", path_to_str(&tpm_dir)?])?;

    let tmux_conf = home.join(".tmux.conf");
    if tmux_conf.exists() {
        let content = fs::read_to_string(&tmux_conf)?;
        if !content.contains("tmux-plugins/tpm") {
            let tpm_config = r#"

# TPM (Tmux Plugin Manager)
set -g @plugin 'tmux-plugins/tpm'
set -g @plugin 'tmux-plugins/tmux-sensible'
set -g @plugin 'tmux-plugins/tmux-resurrect'
set -g @plugin 'tmux-plugins/tmux-continuum'

# Initialize TPM (keep this line at the very bottom)
run '~/.tmux/plugins/tpm/tpm'
"#;
            let mut file = fs::OpenOptions::new().append(true).open(&tmux_conf)?;
            use std::io::Write;
            file.write_all(tpm_config.as_bytes())?;
        }
    }

    Ok(())
}
