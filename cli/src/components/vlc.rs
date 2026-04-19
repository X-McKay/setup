//! `vlc` component - media player.
//!
//! Install delegates to the legacy installer, which installs VLC via snap.
//!
//! Uninstall removes the snap package.

use anyhow::{bail, Context, Result};
use std::process::Command;

use super::util::run_sudo;
use super::Component;

pub struct Vlc;

impl Component for Vlc {
    fn id(&self) -> &str {
        "vlc"
    }

    fn is_installed(&self) -> Result<bool> {
        Ok(which::which("vlc").is_ok())
    }

    fn install(&self) -> Result<()> {
        install_vlc()
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

fn install_vlc() -> Result<()> {
    if which::which("vlc").is_ok() {
        return Ok(());
    }

    run_sudo("snap", &["install", "vlc"])?;
    Ok(())
}
