//! `just` component - task runner.
//!
//! Install uses the upstream installer to place a standalone binary in
//! `~/.local/bin/just`.
//!
//! Uninstall removes only that setup-managed binary.

use anyhow::{anyhow, Result};
use std::fs;

use super::util::{ensure_bin_dir, run_command};
use super::Component;

pub struct Just;

impl Component for Just {
    fn id(&self) -> &str {
        "just"
    }

    fn is_installed(&self) -> Result<bool> {
        Ok(which::which("just").is_ok())
    }

    fn install(&self) -> Result<()> {
        install_just()
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

fn install_just() -> Result<()> {
    if which::which("just").is_ok() {
        return Ok(());
    }

    let bin_dir = ensure_bin_dir()?;
    run_command(
        "sh",
        &[
            "-c",
            &format!(
                "curl --proto '=https' --tlsv1.2 -sSf https://just.systems/install.sh | bash -s -- --to {}",
                bin_dir.display()
            ),
        ],
    )?;

    Ok(())
}
