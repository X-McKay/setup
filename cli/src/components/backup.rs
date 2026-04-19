//! `backup` component - backup tooling and scripts.
//!
//! Install delegates to the legacy installer, which installs packages and
//! creates backup directories, scripts, config, and cron entries.
//!
//! Uninstall: unsupported for now because cleanup spans packages and user
//! data under `~/.backup`.

use anyhow::Result;

use super::Component;
use crate::system::packages;

pub struct Backup;

impl Component for Backup {
    fn id(&self) -> &str {
        "backup"
    }

    fn is_installed(&self) -> Result<bool> {
        Ok(which::which("rsync").is_ok()
            && dirs::home_dir()
                .unwrap_or_default()
                .join(".backup")
                .exists())
    }

    fn install(&self) -> Result<()> {
        packages::install_backup()
    }
}
