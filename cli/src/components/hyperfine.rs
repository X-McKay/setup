//! `hyperfine` component - benchmarking CLI.
//!
//! Install delegates to the legacy installer, which prefers `apt` and
//! falls back to a standalone binary in `~/.local/bin/hyperfine`.
//!
//! Uninstall removes the apt package when available and also deletes the
//! local fallback binary if it exists.

use anyhow::{anyhow, bail, Context, Result};
use std::fs;
use std::process::Command;

use super::Component;
use crate::system::packages;

pub struct Hyperfine;

impl Component for Hyperfine {
    fn id(&self) -> &str {
        "hyperfine"
    }

    fn is_installed(&self) -> Result<bool> {
        Ok(which::which("hyperfine").is_ok())
    }

    fn install(&self) -> Result<()> {
        packages::install_hyperfine()
    }

    fn uninstall(&self) -> Result<()> {
        if which::which("apt").is_ok() {
            let status = Command::new("sudo")
                .args(["apt", "remove", "-y", "hyperfine"])
                .status()
                .context("running sudo apt remove -y hyperfine")?;
            if !status.success() {
                bail!("sudo apt remove -y hyperfine failed: {}", status);
            }
        }

        let bin = dirs::home_dir()
            .ok_or_else(|| anyhow!("no home dir"))?
            .join(".local/bin/hyperfine");
        if bin.exists() {
            fs::remove_file(&bin)?;
        }
        Ok(())
    }
}
