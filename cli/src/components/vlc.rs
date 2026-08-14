//! `vlc` component - media player.
//!
//! Install delegates to the legacy installer, which installs VLC via snap.
//!
//! Uninstall removes the snap package.

use anyhow::{Context, Result, bail};
use std::path::Path;
use std::process::Command;

use super::Component;
use super::util::{brew_install_cask, brew_uninstall_if_present, run_sudo};
use crate::system::platform::Platform;

pub struct Vlc;

impl Component for Vlc {
    fn id(&self) -> &str {
        "vlc"
    }

    fn is_installed(&self) -> Result<bool> {
        Ok(which::which("vlc").is_ok() || Path::new("/Applications/VLC.app").exists())
    }

    fn install(&self) -> Result<()> {
        install_vlc()
    }

    fn uninstall(&self) -> Result<()> {
        brew_uninstall_if_present("vlc", true)?;
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

    match Platform::current()? {
        Platform::Linux => run_sudo("snap", &["install", "vlc"]).map(|_| ())?,
        Platform::MacOs => brew_install_cask("vlc")?,
    }
    Ok(())
}
