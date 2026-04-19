//! `tools` component - extra CLI tools.
//!
//! Installs a shared command-line bundle through apt plus a few
//! user-space tools handled by the existing legacy installer.
//!
//! Uninstall: unsupported. This package set is intentionally treated as
//! a shared baseline rather than something the tool removes automatically.

use anyhow::Result;

use super::util::{apt_install, ensure_bin_dir, path_to_str, run_command, run_sudo};
use super::Component;

pub struct Tools;

impl Component for Tools {
    fn id(&self) -> &str {
        "tools"
    }

    fn is_installed(&self) -> Result<bool> {
        Ok(which::which("rg").is_ok()
            && (which::which("fd").is_ok() || which::which("fdfind").is_ok())
            && which::which("fzf").is_ok()
            && (which::which("bat").is_ok() || which::which("batcat").is_ok())
            && which::which("eza").is_ok()
            && which::which("delta").is_ok())
    }

    fn install(&self) -> Result<()> {
        install_extra_tools()
    }
}

fn install_extra_tools() -> Result<()> {
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

    if run_sudo("apt", &["install", "-y", "eza"]).is_ok() {
        return Ok(());
    }

    cargo_install_local("eza")?;
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

    cargo_install_local("git-delta")?;
    Ok(())
}

fn cargo_install_local(crate_name: &str) -> Result<()> {
    let bin_dir = ensure_bin_dir()?;
    let root = bin_dir
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Could not determine ~/.local from {:?}", bin_dir))?;

    run_command(
        "cargo",
        &["install", "--root", path_to_str(root)?, crate_name],
    )?;
    Ok(())
}
