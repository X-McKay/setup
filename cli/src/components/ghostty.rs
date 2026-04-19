//! `ghostty` component - Ghostty terminal emulator.
//!
//! Install delegates to the legacy installer.
//!
//! Uninstall: unsupported for now. The plan keeps Ghostty on the default
//! refuse path until the later uninstall phase settles its cleanup scope.

use anyhow::Result;

use super::Component;
use crate::system::packages;

pub struct Ghostty;

impl Component for Ghostty {
    fn id(&self) -> &str {
        "ghostty"
    }

    fn is_installed(&self) -> Result<bool> {
        Ok(which::which("ghostty").is_ok())
    }

    fn install(&self) -> Result<()> {
        packages::install_ghostty()
    }
}
