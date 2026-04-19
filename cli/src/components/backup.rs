//! `backup` component - backup tooling and scripts.
//!
//! Install preserves the legacy behavior: install backup packages, create
//! `~/.backup`, and write helper scripts/config plus a cron entry.
//!
//! Uninstall: unsupported for now because cleanup spans packages and user
//! data under `~/.backup`.

use anyhow::{Context, Result};
use std::fs;

use super::util::{apt_install, path_to_str, run_command, run_sudo};
use super::Component;

pub struct Backup;

impl Component for Backup {
    fn id(&self) -> &str {
        "backup"
    }

    fn is_installed(&self) -> Result<bool> {
        Ok(which::which("rsync").is_ok()
            && dirs::home_dir()
                .unwrap_or_default()
                .join(".backup")
                .exists())
    }

    fn install(&self) -> Result<()> {
        install_backup()
    }
}

fn install_backup() -> Result<()> {
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
    let config_path = home
        .join(".backup")
        .join("configs")
        .join("backup_config.sh");

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
