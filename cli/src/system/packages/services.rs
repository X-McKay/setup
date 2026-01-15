//! System services: monitoring, backup, and scheduled tasks.

use anyhow::{Context, Result};
use std::fs;
use super::utils::{apt_install, path_to_str, run_command, run_sudo};

/// Install system monitoring tools.
pub fn install_monitoring() -> Result<()> {
    let packages = [
        "htop", "iotop", "nethogs", "sysstat", "netdata", "logwatch", "fail2ban",
    ];

    run_sudo("apt", &["update"])?;
    apt_install(&packages)?;

    configure_logwatch()?;
    configure_fail2ban()?;
    configure_netdata()?;
    create_health_check_script()?;
    add_monitoring_cron()?;

    Ok(())
}

fn configure_logwatch() -> Result<()> {
    let source = "/usr/share/logwatch/default.conf/logwatch.conf";
    let dest = "/etc/logwatch/conf/logwatch.conf";

    if std::path::Path::new(source).exists() {
        let _ = run_sudo("mkdir", &["-p", "/etc/logwatch/conf"]);
        let _ = run_sudo("cp", &[source, dest]);

        let user = std::env::var("USER").unwrap_or_else(|_| "root".to_string());
        let _ = run_sudo("sed", &["-i", &format!("s/MailTo = root/MailTo = {}/g", user), dest]);
        let _ = run_sudo("sed", &["-i", "s/Detail = Low/Detail = High/g", dest]);
    }
    Ok(())
}

fn configure_fail2ban() -> Result<()> {
    let source = "/etc/fail2ban/jail.conf";
    let dest = "/etc/fail2ban/jail.local";

    if std::path::Path::new(source).exists() && !std::path::Path::new(dest).exists() {
        run_sudo("cp", &[source, dest])?;
    }

    if let Err(e) = run_sudo("systemctl", &["enable", "fail2ban"]) {
        eprintln!("Warning: Could not enable fail2ban service: {}", e);
    }
    if let Err(e) = run_sudo("systemctl", &["start", "fail2ban"]) {
        eprintln!("Warning: Could not start fail2ban service: {}", e);
    }
    Ok(())
}

fn configure_netdata() -> Result<()> {
    if let Err(e) = run_sudo("systemctl", &["enable", "netdata"]) {
        eprintln!("Warning: Could not enable netdata service: {}", e);
    }
    if let Err(e) = run_sudo("systemctl", &["start", "netdata"]) {
        eprintln!("Warning: Could not start netdata service: {}", e);
    }
    Ok(())
}

fn create_health_check_script() -> Result<()> {
    let home = dirs::home_dir().context("Could not find home directory")?;
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

    let temp_path = "/tmp/check_monitoring.sh";
    fs::write(temp_path, script)?;
    run_sudo("mv", &[temp_path, "/usr/local/bin/check_monitoring.sh"])?;
    run_sudo("chmod", &["+x", "/usr/local/bin/check_monitoring.sh"])?;
    Ok(())
}

fn add_monitoring_cron() -> Result<()> {
    let _ = run_command("sh", &[
        "-c",
        r#"(crontab -l 2>/dev/null | grep -v check_monitoring; echo "0 0 * * * /usr/local/bin/check_monitoring.sh") | crontab -"#,
    ]);
    Ok(())
}

/// Install backup tools and scripts.
pub fn install_backup() -> Result<()> {
    let packages = ["rsync", "rdiff-backup", "duplicity", "timeshift"];

    run_sudo("apt", &["update"])?;
    apt_install(&packages)?;

    create_backup_structure()?;
    create_backup_config()?;
    create_backup_script()?;
    create_restore_script()?;
    add_backup_cron()?;

    Ok(())
}

fn create_backup_structure() -> Result<()> {
    let home = dirs::home_dir().context("Could not find home directory")?;
    let backup_root = home.join(".backup");

    fs::create_dir_all(backup_root.join("configs"))?;
    fs::create_dir_all(backup_root.join("data"))?;
    fs::create_dir_all(backup_root.join("system"))?;

    Ok(())
}

fn create_backup_config() -> Result<()> {
    let home = dirs::home_dir().context("Could not find home directory")?;
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
    run_command("chmod", &["+x", path_to_str(&config_path)?])?;
    Ok(())
}

fn create_backup_script() -> Result<()> {
    let home = dirs::home_dir().context("Could not find home directory")?;
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
    run_command("chmod", &["+x", path_to_str(&script_path)?])?;
    Ok(())
}

fn create_restore_script() -> Result<()> {
    let home = dirs::home_dir().context("Could not find home directory")?;
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
    run_command("chmod", &["+x", path_to_str(&script_path)?])?;
    Ok(())
}

fn add_backup_cron() -> Result<()> {
    let home = dirs::home_dir().context("Could not find home directory")?;
    let backup_script = home.join(".backup").join("backup.sh");

    let _ = run_command("sh", &[
        "-c",
        &format!(
            r#"(crontab -l 2>/dev/null | grep -v backup.sh; echo "0 2 * * * {}") | crontab -"#,
            backup_script.display()
        ),
    ]);
    Ok(())
}
