use anyhow::{Context, Result};
use std::fs;
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
        // Already installed, just run mise install for .tool-versions
        run_mise_install()?;
        return Ok(());
    }

    // Install mise using the official installer
    let script = run_command("curl", &["-fsSL", "https://mise.run"])?;
    run_command("sh", &["-c", &script])?;

    // Run mise install for .tool-versions
    run_mise_install()?;

    Ok(())
}

fn run_mise_install() -> Result<()> {
    let home = dirs::home_dir().expect("Could not find home directory");
    let tool_versions = home.join(".tool-versions");

    if tool_versions.exists() {
        // Try mise from ~/.local/bin first, then PATH
        let mise_path = home.join(".local").join("bin").join("mise");
        if mise_path.exists() {
            let _ = run_command(mise_path.to_str().unwrap(), &["install"]);
        } else if which::which("mise").is_ok() {
            let _ = run_command("mise", &["install"]);
        }
    }
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
        "netdata",
        "logwatch",
        "fail2ban",
    ];

    run_sudo("apt", &["update"])?;
    run_sudo("apt", &["install", "-y", &packages.join(" ")])?;

    // Configure services
    configure_logwatch()?;
    configure_fail2ban()?;
    configure_netdata()?;

    // Create health check script
    create_health_check_script()?;

    // Add cron job for daily health checks
    add_monitoring_cron()?;

    Ok(())
}

fn configure_logwatch() -> Result<()> {
    // Copy default config if source exists
    let source = "/usr/share/logwatch/default.conf/logwatch.conf";
    let dest = "/etc/logwatch/conf/logwatch.conf";

    if std::path::Path::new(source).exists() {
        let _ = run_sudo("mkdir", &["-p", "/etc/logwatch/conf"]);
        let _ = run_sudo("cp", &[source, dest]);

        let user = std::env::var("USER").unwrap_or_else(|_| "root".to_string());
        let _ = run_sudo(
            "sed",
            &["-i", &format!("s/MailTo = root/MailTo = {}/g", user), dest],
        );
        let _ = run_sudo("sed", &["-i", "s/Detail = Low/Detail = High/g", dest]);
    }
    Ok(())
}

fn configure_fail2ban() -> Result<()> {
    let source = "/etc/fail2ban/jail.conf";
    let dest = "/etc/fail2ban/jail.local";

    if std::path::Path::new(source).exists() && !std::path::Path::new(dest).exists() {
        let _ = run_sudo("cp", &[source, dest]);
    }
    let _ = run_sudo("systemctl", &["enable", "fail2ban"]);
    let _ = run_sudo("systemctl", &["start", "fail2ban"]);
    Ok(())
}

fn configure_netdata() -> Result<()> {
    let _ = run_sudo("systemctl", &["enable", "netdata"]);
    let _ = run_sudo("systemctl", &["start", "netdata"]);
    Ok(())
}

fn create_health_check_script() -> Result<()> {
    let home = dirs::home_dir().expect("Could not find home directory");
    let monitoring_dir = home.join(".monitoring");
    fs::create_dir_all(&monitoring_dir)?;

    let script = r#"#!/bin/bash
log_file="$HOME/.monitoring/health_report.log"

echo "=== System Health Report $(date) ===" > "$log_file"

echo -e "\nDisk Usage:" >> "$log_file"
df -h | grep -v "tmpfs" >> "$log_file"

echo -e "\nMemory Usage:" >> "$log_file"
free -h >> "$log_file"

echo -e "\nCPU Load:" >> "$log_file"
uptime >> "$log_file"

echo -e "\nCritical Services Status:" >> "$log_file"
systemctl status fail2ban netdata 2>/dev/null | grep "Active:" >> "$log_file"

echo -e "\nRecent System Errors:" >> "$log_file"
journalctl -p err -n 20 --no-pager >> "$log_file"
"#;

    // Write to temp file, then sudo move to /usr/local/bin
    let temp_path = "/tmp/check_monitoring.sh";
    fs::write(temp_path, script)?;
    run_sudo("mv", &[temp_path, "/usr/local/bin/check_monitoring.sh"])?;
    run_sudo("chmod", &["+x", "/usr/local/bin/check_monitoring.sh"])?;
    Ok(())
}

fn add_monitoring_cron() -> Result<()> {
    // Add cron job for daily health check at midnight
    let _ = run_command(
        "sh",
        &[
            "-c",
            r#"(crontab -l 2>/dev/null | grep -v check_monitoring; echo "0 0 * * * /usr/local/bin/check_monitoring.sh") | crontab -"#,
        ],
    );
    Ok(())
}

pub fn install_backup() -> Result<()> {
    let packages = ["rsync", "rdiff-backup", "duplicity", "timeshift"];

    run_sudo("apt", &["update"])?;
    run_sudo("apt", &["install", "-y", &packages.join(" ")])?;

    // Create backup directory structure and scripts
    create_backup_structure()?;
    create_backup_config()?;
    create_backup_script()?;
    create_restore_script()?;

    // Add cron job for daily backups
    add_backup_cron()?;

    Ok(())
}

fn create_backup_structure() -> Result<()> {
    let home = dirs::home_dir().expect("Could not find home directory");
    let backup_root = home.join(".backup");

    fs::create_dir_all(backup_root.join("configs"))?;
    fs::create_dir_all(backup_root.join("data"))?;
    fs::create_dir_all(backup_root.join("system"))?;

    Ok(())
}

fn create_backup_config() -> Result<()> {
    let home = dirs::home_dir().expect("Could not find home directory");
    let config_path = home.join(".backup").join("configs").join("backup_config.sh");

    let config = r#"#!/bin/bash
BACKUP_ROOT="$HOME/.backup"
CONFIGS_DIR="$BACKUP_ROOT/configs"
DATA_DIR="$BACKUP_ROOT/data"
SYSTEM_DIR="$BACKUP_ROOT/system"

IMPORTANT_DIRS=(
    "$HOME/.config"
    "$HOME/.local"
    "$HOME/Documents"
    "$HOME/Pictures"
    "$HOME/.ssh"
)

SYSTEM_FILES=(
    "/etc/fstab"
    "/etc/hosts"
    "/etc/apt/sources.list"
    "/etc/apt/sources.list.d"
)

create_backup() {
    local backup_type=$1
    local backup_dir="$BACKUP_ROOT/$backup_type"

    case $backup_type in
        "configs")
            for dir in "${IMPORTANT_DIRS[@]}"; do
                if [ -d "$dir" ]; then
                    rsync -av --delete "$dir" "$backup_dir/"
                fi
            done
            ;;
        "system")
            for file in "${SYSTEM_FILES[@]}"; do
                if [ -e "$file" ]; then
                    sudo rsync -av "$file" "$backup_dir/"
                fi
            done
            ;;
        "data")
            # Timeshift handled by its built-in scheduler
            ;;
    esac
}

restore_backup() {
    local backup_type=$1
    local backup_date=$2

    case $backup_type in
        "configs")
            rsync -av "$BACKUP_ROOT/configs/$backup_date/" "$HOME/"
            ;;
        "system")
            sudo rsync -av "$BACKUP_ROOT/system/$backup_date/" "/"
            ;;
        "data")
            sudo timeshift --restore --snapshot "$backup_date"
            ;;
    esac
}
"#;

    fs::write(&config_path, config)?;
    run_command("chmod", &["+x", config_path.to_str().unwrap()])?;
    Ok(())
}

fn create_backup_script() -> Result<()> {
    let home = dirs::home_dir().expect("Could not find home directory");
    let script_path = home.join(".backup").join("backup.sh");

    let script = r#"#!/bin/bash
source "$HOME/.backup/configs/backup_config.sh"

timestamp=$(date +%Y%m%d_%H%M%S)

mkdir -p "$CONFIGS_DIR/$timestamp"
mkdir -p "$SYSTEM_DIR/$timestamp"

create_backup "configs"
create_backup "system"
create_backup "data"

find "$CONFIGS_DIR" -type d -mtime +7 -exec rm -rf {} \; 2>/dev/null
find "$SYSTEM_DIR" -type d -mtime +7 -exec rm -rf {} \; 2>/dev/null

echo "Backup completed at $(date)" >> "$BACKUP_ROOT/backup.log"
"#;

    fs::write(&script_path, script)?;
    run_command("chmod", &["+x", script_path.to_str().unwrap()])?;
    Ok(())
}

fn create_restore_script() -> Result<()> {
    let home = dirs::home_dir().expect("Could not find home directory");
    let script_path = home.join(".backup").join("restore.sh");

    let script = r#"#!/bin/bash
source "$HOME/.backup/configs/backup_config.sh"

if [ $# -ne 2 ]; then
    echo "Usage: $0 <backup_type> <backup_date>"
    echo "Example: $0 configs 20240315_120000"
    exit 1
fi

backup_type=$1
backup_date=$2

if [ ! -d "$BACKUP_ROOT/$backup_type/$backup_date" ]; then
    echo "Backup not found: $BACKUP_ROOT/$backup_type/$backup_date"
    exit 1
fi

read -p "Are you sure you want to restore from $backup_type backup dated $backup_date? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Restore cancelled"
    exit 1
fi

restore_backup "$backup_type" "$backup_date"

echo "Restore completed at $(date)" >> "$BACKUP_ROOT/restore.log"
"#;

    fs::write(&script_path, script)?;
    run_command("chmod", &["+x", script_path.to_str().unwrap()])?;
    Ok(())
}

fn add_backup_cron() -> Result<()> {
    let home = dirs::home_dir().expect("Could not find home directory");
    let backup_script = home.join(".backup").join("backup.sh");

    // Add cron job for daily backup at 2 AM, removing any existing backup.sh entry first
    let _ = run_command(
        "sh",
        &[
            "-c",
            &format!(
                r#"(crontab -l 2>/dev/null | grep -v backup.sh; echo "0 2 * * * {}") | crontab -"#,
                backup_script.display()
            ),
        ],
    );
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
