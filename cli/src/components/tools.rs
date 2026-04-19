//! `tools` component - extra CLI tools.
//!
//! Installs a shared command-line bundle through apt plus a few
//! user-space tools handled by the existing legacy installer.
//!
//! Uninstall: unsupported. This package set is intentionally treated as
//! a shared baseline rather than something the tool removes automatically.

use anyhow::Result;

use super::Component;
use crate::system::packages;

pub struct Tools;

impl Component for Tools {
    fn id(&self) -> &str {
        "tools"
    }

    fn is_installed(&self) -> Result<bool> {
        Ok(which::which("rg").is_ok()
            && which::which("fd").is_ok()
            && which::which("bat").is_ok())
    }

    fn install(&self) -> Result<()> {
        packages::install_extra_tools()
    }
}
