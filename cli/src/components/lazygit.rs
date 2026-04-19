//! `lazygit` component - terminal UI for Git workflows.
//!
//! Install uses the existing GitHub-release download path into
//! `~/.local/bin/lazygit`.
//!
//! Uninstall removes only that setup-managed binary.

use anyhow::{anyhow, Result};
use std::fs;

use super::Component;
use crate::system::packages;

pub struct Lazygit;

impl Component for Lazygit {
    fn id(&self) -> &str {
        "lazygit"
    }

    fn is_installed(&self) -> Result<bool> {
        Ok(which::which("lazygit").is_ok())
    }

    fn install(&self) -> Result<()> {
        packages::install_lazygit()
    }

    fn uninstall(&self) -> Result<()> {
        let bin = dirs::home_dir()
            .ok_or_else(|| anyhow!("no home dir"))?
            .join(".local/bin/lazygit");
        if bin.exists() {
            fs::remove_file(&bin)?;
        }
        Ok(())
    }
}
