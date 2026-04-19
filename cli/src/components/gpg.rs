//! `gpg` component - GPG key setup and git signing config.
//!
//! Install delegates to the legacy setup flow, which may install `gnupg`,
//! generate a key, and configure git commit signing.
//!
//! Uninstall remains manual-only. The tool refuses to automate GPG key
//! deletion because it is destructive user material.

use anyhow::{bail, Result};
use std::process::Command;

use super::Component;
use crate::system::packages;

pub struct Gpg;

impl Component for Gpg {
    fn id(&self) -> &str {
        "gpg"
    }

    fn is_installed(&self) -> Result<bool> {
        Ok(Command::new("gpg")
            .args(["--list-secret-keys"])
            .output()
            .map(|output| !output.stdout.is_empty())
            .unwrap_or(false))
    }

    fn install(&self) -> Result<()> {
        packages::setup_gpg()
    }

    fn is_reversible(&self) -> bool {
        false
    }

    fn uninstall(&self) -> Result<()> {
        bail!(
            "gpg uninstall requires manual action: run `gpg --delete-secret-keys <keyid>` then \
             `gpg --delete-keys <keyid>`. This tool will not automate GPG key deletion."
        )
    }
}
