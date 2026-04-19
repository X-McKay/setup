//! `vlc` component - media player.
//!
//! Install delegates to the legacy installer, which installs VLC via snap.
//!
//! Uninstall removes the snap package.

use anyhow::{bail, Context, Result};
use std::process::Command;

use super::Component;
use crate::system::packages;

pub struct Vlc;

impl Component for Vlc {
    fn id(&self) -> &str {
        "vlc"
    }

    fn is_installed(&self) -> Result<bool> {
        Ok(which::which("vlc").is_ok())
    }

    fn install(&self) -> Result<()> {
        packages::install_vlc()
    }

    fn uninstall(&self) -> Result<()> {
        if which::which("snap").is_ok() {
            let status = Command::new("sudo")
                .args(["snap", "remove", "vlc"])
                .status()
                .context("running sudo snap remove vlc")?;
            if !status.success() {
                bail!("sudo snap remove vlc failed: {}", status);
            }
        }
        Ok(())
    }
}
