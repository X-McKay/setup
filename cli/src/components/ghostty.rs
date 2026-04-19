//! `ghostty` component - Ghostty terminal emulator.
//!
//! Install delegates to the legacy installer.
//!
//! Uninstall: unsupported for now. The plan keeps Ghostty on the default
//! refuse path until the later uninstall phase settles its cleanup scope.

use anyhow::Result;

use super::util::run_sudo;
use super::Component;

pub struct Ghostty;

impl Component for Ghostty {
    fn id(&self) -> &str {
        "ghostty"
    }

    fn is_installed(&self) -> Result<bool> {
        Ok(which::which("ghostty").is_ok())
    }

    fn install(&self) -> Result<()> {
        install_ghostty()
    }
}

fn install_ghostty() -> Result<()> {
    if which::which("ghostty").is_ok() {
        return Ok(());
    }

    run_sudo("snap", &["install", "ghostty", "--classic"])?;
    Ok(())
}
