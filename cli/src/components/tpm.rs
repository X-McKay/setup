//! `tpm` component - tmux plugin manager.
//!
//! Install delegates to the legacy installer, which clones the TPM repo and
//! may append TPM configuration to `~/.tmux.conf`.
//!
//! Uninstall removes only the TPM checkout directory and leaves any tmux
//! config edits in place for manual cleanup.

use anyhow::{anyhow, Result};
use std::fs;

use super::Component;
use crate::system::packages;

pub struct Tpm;

impl Component for Tpm {
    fn id(&self) -> &str {
        "tpm"
    }

    fn is_installed(&self) -> Result<bool> {
        Ok(dirs::home_dir()
            .unwrap_or_default()
            .join(".tmux/plugins/tpm")
            .exists())
    }

    fn install(&self) -> Result<()> {
        packages::install_tpm()
    }

    fn uninstall(&self) -> Result<()> {
        let dir = dirs::home_dir()
            .ok_or_else(|| anyhow!("no home dir"))?
            .join(".tmux/plugins/tpm");
        if dir.exists() {
            fs::remove_dir_all(&dir)?;
        }
        Ok(())
    }
}
