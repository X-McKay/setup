//! `docker` component - Docker Engine and Compose.
//!
//! Install delegates to the legacy installer, which also adds the current
//! user to the `docker` group as a side effect.
//!
//! Uninstall: unsupported for now. Reversing the package install and group
//! membership changes safely is deferred to the later uninstall phase work.

use anyhow::Result;

use super::Component;
use crate::system::packages;

pub struct Docker;

impl Component for Docker {
    fn id(&self) -> &str {
        "docker"
    }

    fn is_installed(&self) -> Result<bool> {
        Ok(which::which("docker").is_ok())
    }

    fn install(&self) -> Result<()> {
        packages::install_docker()
    }
}
