//! `mise` component - version manager.
//!
//! Install delegates to the existing bootstrap script and then lets mise
//! apply any `.tool-versions` file already present in the home directory.
//!
//! Uninstall removes the setup-managed `~/.local/bin/mise` binary. It does
//! not remove installed runtimes, caches, or config that mise may manage.

use anyhow::{anyhow, Context, Result};
use std::fs;

use super::util::{path_to_str, run_command};
use super::Component;

pub struct Mise;

impl Component for Mise {
    fn id(&self) -> &str {
        "mise"
    }

    fn is_installed(&self) -> Result<bool> {
        Ok(which::which("mise").is_ok())
    }

    fn install(&self) -> Result<()> {
        install_mise()
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

fn install_mise() -> Result<()> {
    if which::which("mise").is_ok() {
        run_mise_install()?;
        return Ok(());
    }

    let script = run_command("curl", &["-fsSL", "https://mise.run"])?;
    run_command("sh", &["-c", &script])?;
    run_mise_install()?;

    Ok(())
}

fn run_mise_install() -> Result<()> {
    let home = dirs::home_dir().context("Could not find home directory")?;
    let tool_versions = home.join(".tool-versions");

    if tool_versions.exists() {
        let mise_path = home.join(".local").join("bin").join("mise");
        if mise_path.exists() {
            let _ = run_command(path_to_str(&mise_path)?, &["install"]);
        } else if which::which("mise").is_ok() {
            let _ = run_command("mise", &["install"]);
        }
    }
    Ok(())
}
