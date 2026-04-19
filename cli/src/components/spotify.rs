//! `spotify` component - Spotify desktop client.
//!
//! Install delegates to the legacy installer, which installs Spotify via snap.
//!
//! Uninstall removes the snap package.

use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

use super::Component;
use crate::system::packages;

pub struct Spotify;

impl Component for Spotify {
    fn id(&self) -> &str {
        "spotify"
    }

    fn is_installed(&self) -> Result<bool> {
        Ok(Path::new("/snap/bin/spotify").exists())
    }

    fn install(&self) -> Result<()> {
        packages::install_spotify()
    }

    fn uninstall(&self) -> Result<()> {
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
