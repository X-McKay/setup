//! `yq` component - YAML processor.
//!
//! Install uses the existing download path to place a standalone binary in
//! `~/.local/bin/yq`.
//!
//! Uninstall removes only that setup-managed binary.

use anyhow::{anyhow, Result};
use std::fs;

use super::Component;
use crate::system::packages;

pub struct Yq;

impl Component for Yq {
    fn id(&self) -> &str {
        "yq"
    }

    fn is_installed(&self) -> Result<bool> {
        Ok(which::which("yq").is_ok())
    }

    fn install(&self) -> Result<()> {
        packages::install_yq()
    }

    fn uninstall(&self) -> Result<()> {
        let bin = dirs::home_dir()
            .ok_or_else(|| anyhow!("no home dir"))?
            .join(".local/bin/yq");
        if bin.exists() {
            fs::remove_file(&bin)?;
        }
        Ok(())
    }
}
