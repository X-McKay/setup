//! `neovim` component - Neovim editor.
//!
//! Install delegates to the legacy installer, which also seeds a default
//! config under `~/.config/nvim`.
//!
//! Uninstall: unsupported for now because the install path can mix package
//! manager state with user config that should not be deleted automatically.

use anyhow::Result;

use super::Component;
use crate::system::packages;

pub struct Neovim;

impl Component for Neovim {
    fn id(&self) -> &str {
        "neovim"
    }

    fn is_installed(&self) -> Result<bool> {
        Ok(which::which("nvim").is_ok())
    }

    fn install(&self) -> Result<()> {
        packages::install_neovim()
    }
}
