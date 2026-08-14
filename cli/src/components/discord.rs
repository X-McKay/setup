//! `discord` component - Discord desktop client.
//!
//! Install delegates to the legacy installer, which installs Discord via snap.
//!
//! Uninstall removes the snap package.

use anyhow::{Context, Result, bail};
use std::path::Path;
use std::process::Command;

use super::Component;
use super::util::{brew_install_cask, brew_uninstall_if_present, run_sudo};
use crate::system::platform::Platform;

pub struct Discord;

impl Component for Discord {
    fn id(&self) -> &str {
        "discord"
    }

    fn is_installed(&self) -> Result<bool> {
        Ok(Path::new("/snap/bin/discord").exists()
            || Path::new("/Applications/Discord.app").exists())
    }

    fn install(&self) -> Result<()> {
        install_discord()
    }

    fn uninstall(&self) -> Result<()> {
        brew_uninstall_if_present("discord", true)?;
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

    match Platform::current()? {
        Platform::Linux => run_sudo("snap", &["install", "discord"]).map(|_| ())?,
        Platform::MacOs => brew_install_cask("discord")?,
    }
    Ok(())
}
