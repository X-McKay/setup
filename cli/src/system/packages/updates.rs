//! System and tool update functions.

use anyhow::Result;
use super::utils::{run_command, run_sudo};

/// Update system packages via apt.
pub fn update_system() -> Result<()> {
    run_sudo("apt", &["update"])?;
    run_sudo("apt", &["upgrade", "-y"])?;
    run_sudo("apt", &["autoremove", "-y"])?;
    Ok(())
}

/// Update mise version manager.
pub fn update_mise() -> Result<()> {
    run_command("mise", &["self-update"])?;
    run_command("mise", &["upgrade"])?;
    Ok(())
}

/// Update Rust-based tools via cargo.
pub fn update_rust_tools() -> Result<()> {
    run_command("rustup", &["update"])?;

    let tools = ["starship", "zoxide", "eza", "git-delta", "bat"];
    for tool in &tools {
        let _ = run_command("cargo", &["install", tool, "--locked"]);
    }

    Ok(())
}

/// Sync dotfiles from repo to home directory.
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
