//! `yq` component - YAML processor.
//!
//! Install uses the existing download path to place a standalone binary in
//! `~/.local/bin/yq`.
//!
//! Uninstall removes only that setup-managed binary.

use anyhow::{anyhow, Result};
use std::fs;

use super::util::{ensure_bin_dir, get_arch_alt, run_command};
use super::Component;

pub struct Yq;

impl Component for Yq {
    fn id(&self) -> &str {
        "yq"
    }

    fn is_installed(&self) -> Result<bool> {
        Ok(which::which("yq").is_ok())
    }

    fn install(&self) -> Result<()> {
        install_yq()
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

fn install_yq() -> Result<()> {
    if which::which("yq").is_ok() {
        return Ok(());
    }

    let bin_dir = ensure_bin_dir()?;
    let arch = get_arch_alt()?;

    run_command(
        "sh",
        &[
            "-c",
            &format!(
                "curl -Lo {}/yq 'https://github.com/mikefarah/yq/releases/latest/download/yq_linux_{}' && chmod +x {}/yq",
                bin_dir.display(),
                arch,
                bin_dir.display()
            ),
        ],
    )?;

    Ok(())
}
