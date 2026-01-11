use anyhow::{Context, Result};
use std::process::Command;

/// Run a shell command and return its output
fn run_command(cmd: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(cmd)
        .args(args)
        .output()
        .with_context(|| format!("Failed to execute: {} {:?}", cmd, args))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Command failed: {}", stderr)
    }
}

/// Run a shell command with sudo
fn run_sudo(cmd: &str, args: &[&str]) -> Result<String> {
    let mut sudo_args = vec![cmd];
    sudo_args.extend(args);

    run_command("sudo", &sudo_args)
}

// ============================================================================
// Install functions
// ============================================================================

pub fn install_apt_packages() -> Result<()> {
    run_sudo("apt", &["update"])?;

    let packages = [
        "curl",
        "wget",
        "git",
        "build-essential",
        "gcc",
        "make",
        "cmake",
        "pkg-config",
        "libssl-dev",
        "libffi-dev",
        "python3-dev",
        "python3-pip",
        "unzip",
        "zip",
        "jq",
    ];

    run_sudo("apt", &["install", "-y", &packages.join(" ")])?;
    Ok(())
}

pub fn install_extra_tools() -> Result<()> {
    run_sudo("apt", &["update"])?;

    let packages = ["ripgrep", "fd-find", "fzf", "tree", "htop", "ncdu"];

    run_sudo("apt", &["install", "-y", &packages.join(" ")])?;

    // Install eza (replacement for exa)
    install_eza()?;

    // Install bat
    install_bat()?;

    // Install delta
    install_delta()?;

    Ok(())
}

fn install_eza() -> Result<()> {
    // eza is available via cargo or apt on newer Ubuntu
    if which::which("eza").is_ok() {
        return Ok(());
    }

    // Try apt first (Ubuntu 24.04+)
    if run_sudo("apt", &["install", "-y", "eza"]).is_ok() {
        return Ok(());
    }

    // Fall back to cargo
    run_command("cargo", &["install", "eza"])?;
    Ok(())
}

fn install_bat() -> Result<()> {
    if which::which("bat").is_ok() || which::which("batcat").is_ok() {
        return Ok(());
    }

    run_sudo("apt", &["install", "-y", "bat"])?;
    Ok(())
}

fn install_delta() -> Result<()> {
    if which::which("delta").is_ok() {
        return Ok(());
    }

    // Try cargo install
    run_command("cargo", &["install", "git-delta"])?;
    Ok(())
}

pub fn install_mise() -> Result<()> {
    if which::which("mise").is_ok() {
        return Ok(());
    }

    // Install mise using the official installer
    let script = run_command("curl", &["-fsSL", "https://mise.run"])?;
    run_command("sh", &["-c", &script])?;

    Ok(())
}

pub fn install_docker() -> Result<()> {
    if which::which("docker").is_ok() {
        return Ok(());
    }

    // Add Docker's official GPG key and repository
    run_sudo("apt", &["update"])?;
    run_sudo(
        "apt",
        &[
            "install",
            "-y",
            "ca-certificates",
            "curl",
            "gnupg",
            "lsb-release",
        ],
    )?;

    // Install Docker using convenience script
    let script = run_command("curl", &["-fsSL", "https://get.docker.com"])?;
    run_sudo("sh", &["-c", &script])?;

    // Add current user to docker group
    let user = std::env::var("USER").unwrap_or_else(|_| "al".to_string());
    run_sudo("usermod", &["-aG", "docker", &user])?;

    Ok(())
}

pub fn install_monitoring() -> Result<()> {
    let packages = [
        "htop",
        "iotop",
        "nethogs",
        "sysstat",
        "prometheus-node-exporter",
    ];

    run_sudo("apt", &["update"])?;
    run_sudo("apt", &["install", "-y", &packages.join(" ")])?;

    Ok(())
}

pub fn install_backup() -> Result<()> {
    let packages = ["rsync", "rdiff-backup", "duplicity", "timeshift"];

    run_sudo("apt", &["update"])?;
    run_sudo("apt", &["install", "-y", &packages.join(" ")])?;

    Ok(())
}

pub fn install_starship() -> Result<()> {
    if which::which("starship").is_ok() {
        return Ok(());
    }

    // Install using cargo for latest version
    run_command("cargo", &["install", "starship", "--locked"])?;

    Ok(())
}

pub fn install_zoxide() -> Result<()> {
    if which::which("zoxide").is_ok() {
        return Ok(());
    }

    run_command("cargo", &["install", "zoxide", "--locked"])?;

    Ok(())
}

pub fn install_lazygit() -> Result<()> {
    if which::which("lazygit").is_ok() {
        return Ok(());
    }

    // Install via go or download binary
    let arch = std::env::consts::ARCH;
    let os = "Linux";

    let url = format!(
        "https://github.com/jesseduffield/lazygit/releases/latest/download/lazygit_{}_{}_{}.tar.gz",
        "0.44.1", os, arch
    );

    let home = dirs::home_dir().expect("Could not find home directory");
    let bin_dir = home.join(".local").join("bin");
    std::fs::create_dir_all(&bin_dir)?;

    run_command(
        "sh",
        &[
            "-c",
            &format!(
                "curl -Lo /tmp/lazygit.tar.gz '{}' && tar xf /tmp/lazygit.tar.gz -C {} lazygit",
                url,
                bin_dir.display()
            ),
        ],
    )?;

    Ok(())
}

pub fn install_just() -> Result<()> {
    if which::which("just").is_ok() {
        return Ok(());
    }

    let home = dirs::home_dir().expect("Could not find home directory");
    let bin_dir = home.join(".local").join("bin");
    std::fs::create_dir_all(&bin_dir)?;

    run_command(
        "sh",
        &[
            "-c",
            &format!(
                "curl --proto '=https' --tlsv1.2 -sSf https://just.systems/install.sh | bash -s -- --to {}",
                bin_dir.display()
            ),
        ],
    )?;

    Ok(())
}

pub fn install_glow() -> Result<()> {
    if which::which("glow").is_ok() {
        return Ok(());
    }

    // Try apt first
    if run_sudo("apt", &["install", "-y", "glow"]).is_ok() {
        return Ok(());
    }

    // Fall back to binary download
    let home = dirs::home_dir().expect("Could not find home directory");
    let bin_dir = home.join(".local").join("bin");
    std::fs::create_dir_all(&bin_dir)?;

    let arch = match std::env::consts::ARCH {
        "x86_64" => "x86_64",
        "aarch64" => "arm64",
        _ => return Err(anyhow::anyhow!("Unsupported architecture")),
    };

    run_command(
        "sh",
        &[
            "-c",
            &format!(
                "curl -Lo /tmp/glow.tar.gz 'https://github.com/charmbracelet/glow/releases/latest/download/glow_Linux_{}.tar.gz' && tar xf /tmp/glow.tar.gz -C /tmp && mv /tmp/glow {}",
                arch,
                bin_dir.display()
            ),
        ],
    )?;

    Ok(())
}

pub fn install_bottom() -> Result<()> {
    if which::which("btm").is_ok() {
        return Ok(());
    }

    // Try apt first
    if run_sudo("apt", &["install", "-y", "bottom"]).is_ok() {
        return Ok(());
    }

    // Fall back to binary download
    let home = dirs::home_dir().expect("Could not find home directory");
    let bin_dir = home.join(".local").join("bin");
    std::fs::create_dir_all(&bin_dir)?;

    let arch = match std::env::consts::ARCH {
        "x86_64" => "x86_64",
        "aarch64" => "aarch64",
        _ => return Err(anyhow::anyhow!("Unsupported architecture")),
    };

    run_command(
        "sh",
        &[
            "-c",
            &format!(
                "curl -Lo /tmp/bottom.tar.gz 'https://github.com/ClementTsang/bottom/releases/latest/download/bottom_{}-unknown-linux-gnu.tar.gz' && tar xf /tmp/bottom.tar.gz -C {} btm",
                arch,
                bin_dir.display()
            ),
        ],
    )?;

    Ok(())
}

pub fn install_gh() -> Result<()> {
    if which::which("gh").is_ok() {
        return Ok(());
    }

    // Install via official apt repository
    run_command(
        "sh",
        &[
            "-c",
            r#"(type -p wget >/dev/null || (sudo apt update && sudo apt install wget -y)) \
            && sudo mkdir -p -m 755 /etc/apt/keyrings \
            && out=$(mktemp) && wget -nv -O$out https://cli.github.com/packages/githubcli-archive-keyring.gpg \
            && cat $out | sudo tee /etc/apt/keyrings/githubcli-archive-keyring.gpg > /dev/null \
            && sudo chmod go+r /etc/apt/keyrings/githubcli-archive-keyring.gpg \
            && echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | sudo tee /etc/apt/sources.list.d/github-cli.list > /dev/null \
            && sudo apt update \
            && sudo apt install gh -y"#,
        ],
    )?;

    Ok(())
}

pub fn install_hyperfine() -> Result<()> {
    if which::which("hyperfine").is_ok() {
        return Ok(());
    }

    // Try apt first
    if run_sudo("apt", &["install", "-y", "hyperfine"]).is_ok() {
        return Ok(());
    }

    // Fall back to binary download
    let home = dirs::home_dir().expect("Could not find home directory");
    let bin_dir = home.join(".local").join("bin");
    std::fs::create_dir_all(&bin_dir)?;

    let arch = match std::env::consts::ARCH {
        "x86_64" => "x86_64",
        "aarch64" => "aarch64",
        _ => return Err(anyhow::anyhow!("Unsupported architecture")),
    };

    run_command(
        "sh",
        &[
            "-c",
            &format!(
                "curl -Lo /tmp/hyperfine.tar.gz 'https://github.com/sharkdp/hyperfine/releases/latest/download/hyperfine-v1.18.0-{}-unknown-linux-musl.tar.gz' && tar xf /tmp/hyperfine.tar.gz -C /tmp && mv /tmp/hyperfine-*/hyperfine {}",
                arch,
                bin_dir.display()
            ),
        ],
    )?;

    Ok(())
}

pub fn install_jq() -> Result<()> {
    if which::which("jq").is_ok() {
        return Ok(());
    }

    // Try apt first (usually available)
    if run_sudo("apt", &["install", "-y", "jq"]).is_ok() {
        return Ok(());
    }

    // Fall back to binary download
    let home = dirs::home_dir().expect("Could not find home directory");
    let bin_dir = home.join(".local").join("bin");
    std::fs::create_dir_all(&bin_dir)?;

    let arch = match std::env::consts::ARCH {
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        _ => return Err(anyhow::anyhow!("Unsupported architecture")),
    };

    run_command(
        "sh",
        &[
            "-c",
            &format!(
                "curl -Lo {}/jq 'https://github.com/jqlang/jq/releases/latest/download/jq-linux-{}' && chmod +x {}/jq",
                bin_dir.display(),
                arch,
                bin_dir.display()
            ),
        ],
    )?;

    Ok(())
}

pub fn install_yq() -> Result<()> {
    if which::which("yq").is_ok() {
        return Ok(());
    }

    let home = dirs::home_dir().expect("Could not find home directory");
    let bin_dir = home.join(".local").join("bin");
    std::fs::create_dir_all(&bin_dir)?;

    let arch = match std::env::consts::ARCH {
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        _ => return Err(anyhow::anyhow!("Unsupported architecture")),
    };

    run_command(
        "sh",
        &[
            "-c",
            &format!(
                "curl -Lo {}/yq 'https://github.com/mikefarah/yq/releases/latest/download/yq_linux_{}' && chmod +x {}/yq",
                bin_dir.display(),
                arch,
                bin_dir.display()
            ),
        ],
    )?;

    Ok(())
}

pub fn install_tldr() -> Result<()> {
    if which::which("tldr").is_ok() {
        return Ok(());
    }

    // Try apt first
    if run_sudo("apt", &["install", "-y", "tldr"]).is_ok() {
        let _ = run_command("tldr", &["--update"]);
        return Ok(());
    }

    // Fall back to tealdeer (Rust implementation)
    let home = dirs::home_dir().expect("Could not find home directory");
    let bin_dir = home.join(".local").join("bin");
    std::fs::create_dir_all(&bin_dir)?;

    let arch = match std::env::consts::ARCH {
        "x86_64" => "x86_64",
        "aarch64" => "aarch64",
        _ => return Err(anyhow::anyhow!("Unsupported architecture")),
    };

    run_command(
        "sh",
        &[
            "-c",
            &format!(
                "curl -Lo {}/tldr 'https://github.com/dbrgn/tealdeer/releases/latest/download/tealdeer-linux-{}-musl' && chmod +x {}/tldr && {}/tldr --update",
                bin_dir.display(),
                arch,
                bin_dir.display(),
                bin_dir.display()
            ),
        ],
    )?;

    Ok(())
}

// ============================================================================
// Update functions
// ============================================================================

pub fn update_system() -> Result<()> {
    run_sudo("apt", &["update"])?;
    run_sudo("apt", &["upgrade", "-y"])?;
    run_sudo("apt", &["autoremove", "-y"])?;
    Ok(())
}

pub fn update_mise() -> Result<()> {
    run_command("mise", &["self-update"])?;
    run_command("mise", &["upgrade"])?;
    Ok(())
}

pub fn update_rust_tools() -> Result<()> {
    // Update rustup and cargo
    run_command("rustup", &["update"])?;

    // Update installed cargo binaries
    let tools = ["starship", "zoxide", "eza", "git-delta", "bat"];

    for tool in &tools {
        let _ = run_command("cargo", &["install", tool, "--locked"]);
    }

    Ok(())
}

pub fn sync_dotfiles() -> Result<()> {
    use crate::config::dotfiles;

    let managed = dotfiles::get_managed_dotfiles();
    for (_, source, target) in managed {
        if source.exists() {
            dotfiles::copy_dotfile(&source, &target)?;
        }
    }

    Ok(())
}
