//! `apt` component - core system packages.
//!
//! On Linux installs via apt: curl, wget, git, build-essential, gcc, make,
//! cmake, pkg-config, libssl-dev, libffi-dev, python3-dev, python3-pip,
//! unzip, zip, jq. On macOS the compiler toolchain comes from the Xcode
//! Command Line Tools and the remainder from Homebrew.
//!
//! Uninstall: unsupported. These packages are a shared base layer for
//! many other components, so automated removal is intentionally refused.

use anyhow::Result;

use super::Component;
use super::util::{apt_install, brew_install, run_command, run_sudo};
use crate::system::platform::Platform;

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
        match Platform::current()? {
            Platform::Linux => install_apt_packages(),
            Platform::MacOs => install_macos_packages(),
        }
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

fn install_macos_packages() -> Result<()> {
    // gcc/make/git come from the Xcode Command Line Tools; everything else
    // that apt provides on Linux comes from Homebrew. zip/unzip ship with
    // macOS.
    if run_command("xcode-select", &["-p"]).is_err() {
        anyhow::bail!("Xcode Command Line Tools are required — run `xcode-select --install` first");
    }

    brew_install(&["curl", "wget", "cmake", "pkg-config", "python3", "jq"])?;
    Ok(())
}
