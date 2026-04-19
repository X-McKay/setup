//! `gh` component - GitHub CLI.
//!
//! Install delegates to the legacy apt-based installer, which adds the
//! GitHub CLI package repository and installs `gh`.
//!
//! Uninstall: unsupported for now. Authentication state and apt source
//! cleanup are deferred to the later uninstall phase work.

use anyhow::Result;

use super::Component;
use crate::system::packages;

pub struct Gh;

impl Component for Gh {
    fn id(&self) -> &str {
        "gh"
    }

    fn is_installed(&self) -> Result<bool> {
        Ok(which::which("gh").is_ok())
    }

    fn install(&self) -> Result<()> {
        packages::install_gh()
    }
}
