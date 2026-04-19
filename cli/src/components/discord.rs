//! `discord` component - Discord desktop client.
//!
//! Install delegates to the legacy installer, which installs Discord via snap.
//!
//! Uninstall removes the snap package.

use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

use super::util::run_sudo;
use super::Component;

pub struct Discord;

impl Component for Discord {
    fn id(&self) -> &str {
        "discord"
    }

    fn is_installed(&self) -> Result<bool> {
        Ok(Path::new("/snap/bin/discord").exists())
    }

    fn install(&self) -> Result<()> {
        install_discord()
    }

    fn uninstall(&self) -> Result<()> {
        if which::which("snap").is_ok() {
            let status = Command::new("sudo")
                .args(["snap", "remove", "discord"])
                .status()
                .context("running sudo snap remove discord")?;
            if !status.success() {
                bail!("sudo snap remove discord failed: {}", status);
            }
        }
        Ok(())
    }
}

fn install_discord() -> Result<()> {
    if which::which("discord").is_ok() {
        return Ok(());
    }

    run_sudo("snap", &["install", "discord"])?;
    Ok(())
}
