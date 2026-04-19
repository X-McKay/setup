//! `bottom` component - system monitor.
//!
//! Install delegates to the legacy installer, which prefers `apt` and
//! falls back to a standalone binary in `~/.local/bin/btm`.
//!
//! Uninstall removes the apt package when available and also deletes the
//! local fallback binary if it exists.

use anyhow::{anyhow, bail, Context, Result};
use std::fs;
use std::process::Command;

use super::Component;
use crate::system::packages;

pub struct Bottom;

impl Component for Bottom {
    fn id(&self) -> &str {
        "bottom"
    }

    fn is_installed(&self) -> Result<bool> {
        Ok(which::which("btm").is_ok())
    }

    fn install(&self) -> Result<()> {
        packages::install_bottom()
    }

    fn uninstall(&self) -> Result<()> {
        if which::which("apt").is_ok() {
            let status = Command::new("sudo")
                .args(["apt", "remove", "-y", "bottom"])
                .status()
                .context("running sudo apt remove -y bottom")?;
            if !status.success() {
                bail!("sudo apt remove -y bottom failed: {}", status);
            }
        }

        let bin = dirs::home_dir()
            .ok_or_else(|| anyhow!("no home dir"))?
            .join(".local/bin/btm");
        if bin.exists() {
            fs::remove_file(&bin)?;
        }
        Ok(())
    }
}
