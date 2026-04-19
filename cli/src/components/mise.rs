//! `mise` component - version manager.
//!
//! Install delegates to the existing bootstrap script and then lets mise
//! apply any `.tool-versions` file already present in the home directory.
//!
//! Uninstall removes the setup-managed `~/.local/bin/mise` binary. It does
//! not remove installed runtimes, caches, or config that mise may manage.

use anyhow::{anyhow, Result};
use std::fs;

use super::Component;
use crate::system::packages;

pub struct Mise;

impl Component for Mise {
    fn id(&self) -> &str {
        "mise"
    }

    fn is_installed(&self) -> Result<bool> {
        Ok(which::which("mise").is_ok())
    }

    fn install(&self) -> Result<()> {
        packages::install_mise()
    }

    fn uninstall(&self) -> Result<()> {
        let bin = dirs::home_dir()
            .ok_or_else(|| anyhow!("no home dir"))?
            .join(".local/bin/mise");
        if bin.exists() {
            fs::remove_file(&bin)?;
        }
        Ok(())
    }
}
