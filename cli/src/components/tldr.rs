//! `tldr` component - simplified command help pages.
//!
//! Install delegates to the legacy installer, which prefers `apt` and
//! falls back to a standalone `tealdeer` binary.
//!
//! Uninstall: unsupported. The plan keeps `tldr` on the default refuse path.

use anyhow::Result;

use super::Component;
use crate::system::packages;

pub struct Tldr;

impl Component for Tldr {
    fn id(&self) -> &str {
        "tldr"
    }

    fn is_installed(&self) -> Result<bool> {
        Ok(which::which("tldr").is_ok())
    }

    fn install(&self) -> Result<()> {
        packages::install_tldr()
    }
}
