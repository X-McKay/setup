//! `obsidian` component - note-taking app.
//!
//! Install delegates to the legacy installer, which installs Obsidian via snap.
//!
//! Uninstall removes the snap package.

use anyhow::{Context, Result, bail};
use std::path::Path;
use std::process::Command;

use super::Component;
use super::util::{brew_install_cask, brew_uninstall_if_present, run_sudo};
use crate::system::platform::Platform;

pub struct Obsidian;

impl Component for Obsidian {
    fn id(&self) -> &str {
        "obsidian"
    }

    fn is_installed(&self) -> Result<bool> {
        Ok(Path::new("/snap/bin/obsidian").exists()
            || Path::new("/Applications/Obsidian.app").exists())
    }

    fn install(&self) -> Result<()> {
        install_obsidian()
    }

    fn uninstall(&self) -> Result<()> {
        brew_uninstall_if_present("obsidian", true)?;
        if which::which("snap").is_ok() {
            let status = Command::new("sudo")
                .args(["snap", "remove", "obsidian"])
                .status()
                .context("running sudo snap remove obsidian")?;
            if !status.success() {
                bail!("sudo snap remove obsidian failed: {}", status);
            }
        }
        Ok(())
    }
}

fn install_obsidian() -> Result<()> {
    if which::which("obsidian").is_ok() {
        return Ok(());
    }

    match Platform::current()? {
        Platform::Linux => run_sudo("snap", &["install", "obsidian", "--classic"]).map(|_| ())?,
        Platform::MacOs => brew_install_cask("obsidian")?,
    }
    Ok(())
}
