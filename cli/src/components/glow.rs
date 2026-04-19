//! `glow` component - markdown renderer.
//!
//! Install delegates to the legacy installer, which prefers `apt` and
//! falls back to a standalone binary in `~/.local/bin/glow`.
//!
//! Uninstall removes the apt package when available and also deletes the
//! local fallback binary if it exists.

use anyhow::{anyhow, bail, Context, Result};
use std::fs;
use std::process::Command;

use super::Component;
use crate::system::packages;

pub struct Glow;

impl Component for Glow {
    fn id(&self) -> &str {
        "glow"
    }

    fn is_installed(&self) -> Result<bool> {
        Ok(which::which("glow").is_ok())
    }

    fn install(&self) -> Result<()> {
        packages::install_glow()
    }

    fn uninstall(&self) -> Result<()> {
        if which::which("apt").is_ok() {
            let status = Command::new("sudo")
                .args(["apt", "remove", "-y", "glow"])
                .status()
                .context("running sudo apt remove -y glow")?;
            if !status.success() {
                bail!("sudo apt remove -y glow failed: {}", status);
            }
        }

        let bin = dirs::home_dir()
            .ok_or_else(|| anyhow!("no home dir"))?
            .join(".local/bin/glow");
        if bin.exists() {
            fs::remove_file(&bin)?;
        }
        Ok(())
    }
}
