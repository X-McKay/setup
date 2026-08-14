//! `spotify` component - Spotify desktop client.
//!
//! Install delegates to the legacy installer, which installs Spotify via snap.
//!
//! Uninstall removes the snap package.

use anyhow::{Context, Result, bail};
use std::path::Path;
use std::process::Command;

use super::Component;
use super::util::{brew_install_cask, brew_uninstall_if_present, run_sudo};
use crate::system::platform::Platform;

pub struct Spotify;

impl Component for Spotify {
    fn id(&self) -> &str {
        "spotify"
    }

    fn is_installed(&self) -> Result<bool> {
        Ok(Path::new("/snap/bin/spotify").exists()
            || Path::new("/Applications/Spotify.app").exists())
    }

    fn install(&self) -> Result<()> {
        install_spotify()
    }

    fn uninstall(&self) -> Result<()> {
        brew_uninstall_if_present("spotify", true)?;
        if which::which("snap").is_ok() {
            let status = Command::new("sudo")
                .args(["snap", "remove", "spotify"])
                .status()
                .context("running sudo snap remove spotify")?;
            if !status.success() {
                bail!("sudo snap remove spotify failed: {}", status);
            }
        }
        Ok(())
    }
}

fn install_spotify() -> Result<()> {
    if which::which("spotify").is_ok() {
        return Ok(());
    }

    match Platform::current()? {
        Platform::Linux => run_sudo("snap", &["install", "spotify"]).map(|_| ())?,
        Platform::MacOs => brew_install_cask("spotify")?,
    }
    Ok(())
}
