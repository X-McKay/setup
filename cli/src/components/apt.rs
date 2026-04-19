//! `apt` component - core system packages.
//!
//! Installs: curl, wget, git, build-essential, gcc, make, cmake,
//! pkg-config, libssl-dev, libffi-dev, python3-dev, python3-pip,
//! unzip, zip, jq.
//!
//! Uninstall: unsupported. These packages are a shared base layer for
//! many other components, so automated removal is intentionally refused.

use anyhow::Result;

use super::Component;
use crate::system::packages;

pub struct Apt;

impl Component for Apt {
    fn id(&self) -> &str {
        "apt"
    }

    fn is_installed(&self) -> Result<bool> {
        // Probe a representative subset from the package bundle.
        Ok(which::which("curl").is_ok() && which::which("git").is_ok())
    }

    fn install(&self) -> Result<()> {
        packages::install_apt_packages()
    }
}
