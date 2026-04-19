//! `chromium` component - Chromium browser.
//!
//! Install delegates to the legacy installer, which installs Chromium via snap.
//!
//! Uninstall removes the snap package.

use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

use super::util::run_sudo;
use super::Component;

pub struct Chromium;

impl Component for Chromium {
    fn id(&self) -> &str {
        "chromium"
    }

    fn is_installed(&self) -> Result<bool> {
        Ok(which::which("chromium").is_ok() || Path::new("/snap/bin/chromium").exists())
    }

    fn install(&self) -> Result<()> {
        install_chromium()
    }

    fn uninstall(&self) -> Result<()> {
        if which::which("snap").is_ok() {
            let status = Command::new("sudo")
                .args(["snap", "remove", "chromium"])
                .status()
                .context("running sudo snap remove chromium")?;
            if !status.success() {
                bail!("sudo snap remove chromium failed: {}", status);
            }
        }
        Ok(())
    }
}

fn install_chromium() -> Result<()> {
    if which::which("chromium-browser").is_ok() || which::which("chromium").is_ok() {
        return Ok(());
    }

    run_sudo("snap", &["install", "chromium"])?;
    Ok(())
}
