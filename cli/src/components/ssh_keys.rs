//! `ssh-keys` component - SSH key generation.
//!
//! Install delegates to the legacy setup flow, which generates a new
//! `~/.ssh/id_ed25519` keypair and prints the public key for the user.
//!
//! Uninstall deletes the generated keypair. This is destructive user
//! material, so the component is not automatically reversible.

use anyhow::{anyhow, Result};
use std::fs;

use super::Component;
use crate::system::packages;

pub struct SshKeys;

impl Component for SshKeys {
    fn id(&self) -> &str {
        "ssh-keys"
    }

    fn is_installed(&self) -> Result<bool> {
        Ok(dirs::home_dir()
            .unwrap_or_default()
            .join(".ssh/id_ed25519")
            .exists())
    }

    fn install(&self) -> Result<()> {
        packages::setup_ssh_keys()
    }

    fn is_reversible(&self) -> bool {
        false
    }

    fn uninstall(&self) -> Result<()> {
        let home = dirs::home_dir().ok_or_else(|| anyhow!("no home dir"))?;
        for name in ["id_ed25519", "id_ed25519.pub"] {
            let path = home.join(".ssh").join(name);
            if path.exists() {
                fs::remove_file(&path)?;
            }
        }
        Ok(())
    }
}
