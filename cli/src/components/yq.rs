//! `yq` component - YAML processor.
//!
//! Install uses the existing download path to place a standalone binary in
//! `~/.local/bin/yq`.
//!
//! Uninstall removes only that setup-managed binary.

use anyhow::{Result, anyhow};
use std::fs;

use super::Component;
use super::util::{
    brew_install, brew_uninstall_if_present, ensure_bin_dir, get_arch_alt, run_command,
};
use crate::system::platform::Platform;

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
        brew_uninstall_if_present("yq", false)?;
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

    if Platform::current()? == Platform::MacOs {
        return brew_install(&["yq"]);
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
