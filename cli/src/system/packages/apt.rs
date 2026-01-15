//! APT package installation functions.

use anyhow::Result;
use super::utils::{apt_install, run_command, run_sudo};

/// Install base system packages via apt.
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

    apt_install(&packages)?;
    Ok(())
}

/// Install extra CLI tools via apt.
pub fn install_extra_tools() -> Result<()> {
    run_sudo("apt", &["update"])?;

    let packages = ["ripgrep", "fd-find", "fzf", "tree", "htop", "ncdu"];
    apt_install(&packages)?;

    install_eza()?;
    install_bat()?;
    install_delta()?;

    Ok(())
}

fn install_eza() -> Result<()> {
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

    run_command("cargo", &["install", "git-delta"])?;
    Ok(())
}
