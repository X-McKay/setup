//! `apt` component - core system packages.
//!
//! Installs: curl, wget, git, build-essential, gcc, make, cmake,
//! pkg-config, libssl-dev, libffi-dev, python3-dev, python3-pip,
//! unzip, zip, jq.
//!
//! Uninstall: unsupported. These packages are a shared base layer for
//! many other components, so automated removal is intentionally refused.

use anyhow::Result;

use super::util::{apt_install, run_sudo};
use super::Component;

pub struct Apt;

impl Component for Apt {
    fn id(&self) -> &str {
        "apt"
    }

    fn is_installed(&self) -> Result<bool> {
        // Probe the command-bearing packages so preinstalled build deps do not
        // cause us to skip the rest of the base bundle.
        let required_commands = [
            "curl",
            "wget",
            "git",
            "gcc",
            "make",
            "cmake",
            "pkg-config",
            "python3",
            "pip3",
            "unzip",
            "zip",
            "jq",
        ];

        Ok(required_commands
            .iter()
            .all(|command| which::which(command).is_ok()))
    }

    fn install(&self) -> Result<()> {
        install_apt_packages()
    }
}

fn install_apt_packages() -> Result<()> {
    run_sudo("apt", &["update"])?;

    let packages = [
        "curl",
        "wget",
        "git",
        "build-essential",
        "gcc",
        "make",
        "cmake",
        "pkg-config",
        "libssl-dev",
        "libffi-dev",
        "python3-dev",
        "python3-pip",
        "unzip",
        "zip",
        "jq",
    ];

    apt_install(&packages)?;
    Ok(())
}
