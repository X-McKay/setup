//! `ghostty` component - Ghostty terminal emulator.
//!
//! Install delegates to the legacy installer.
//!
//! Uninstall: unsupported for now. The plan keeps Ghostty on the default
//! refuse path until the later uninstall phase settles its cleanup scope.

use anyhow::Result;

use super::Component;
use super::util::{brew_install_cask, brew_uninstall_if_present, run_sudo};
use crate::system::platform::Platform;
use std::path::Path;

pub struct Ghostty;

impl Component for Ghostty {
    fn id(&self) -> &str {
        "ghostty"
    }

    fn is_installed(&self) -> Result<bool> {
        Ok(which::which("ghostty").is_ok() || Path::new("/Applications/Ghostty.app").exists())
    }

    fn install(&self) -> Result<()> {
        install_ghostty()
    }

    fn uninstall(&self) -> Result<()> {
        if Platform::current()? == Platform::MacOs {
            brew_uninstall_if_present("ghostty", true)?;
            return Ok(());
        }
        anyhow::bail!("ghostty does not implement uninstall on Linux — not removable by this tool")
    }
}

fn install_ghostty() -> Result<()> {
    if which::which("ghostty").is_ok() {
        return Ok(());
    }

    match Platform::current()? {
        Platform::Linux => run_sudo("snap", &["install", "ghostty", "--classic"]).map(|_| ())?,
        Platform::MacOs => brew_install_cask("ghostty")?,
    }
    Ok(())
}
