//! `obsidian` component - note-taking app.
//!
//! Install delegates to the legacy installer, which installs Obsidian via snap.
//!
//! Uninstall removes the snap package.

use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

use super::Component;
use crate::system::packages;

pub struct Obsidian;

impl Component for Obsidian {
    fn id(&self) -> &str {
        "obsidian"
    }

    fn is_installed(&self) -> Result<bool> {
        Ok(Path::new("/snap/bin/obsidian").exists())
    }

    fn install(&self) -> Result<()> {
        packages::install_obsidian()
    }

    fn uninstall(&self) -> Result<()> {
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
