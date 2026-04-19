//! `just` component - task runner.
//!
//! Install uses the upstream installer to place a standalone binary in
//! `~/.local/bin/just`.
//!
//! Uninstall removes only that setup-managed binary.

use anyhow::{anyhow, Result};
use std::fs;

use super::Component;
use crate::system::packages;

pub struct Just;

impl Component for Just {
    fn id(&self) -> &str {
        "just"
    }

    fn is_installed(&self) -> Result<bool> {
        Ok(which::which("just").is_ok())
    }

    fn install(&self) -> Result<()> {
        packages::install_just()
    }

    fn uninstall(&self) -> Result<()> {
        let bin = dirs::home_dir()
            .ok_or_else(|| anyhow!("no home dir"))?
            .join(".local/bin/just");
        if bin.exists() {
            fs::remove_file(&bin)?;
        }
        Ok(())
    }
}
