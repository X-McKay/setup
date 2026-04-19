//! `glow` component - markdown renderer.
//!
//! Install delegates to the legacy installer, which prefers `apt` and
//! falls back to a standalone binary in `~/.local/bin/glow`.
//!
//! Uninstall removes the apt package when available and also deletes the
//! local fallback binary if it exists.

use anyhow::{anyhow, bail, Context, Result};
use std::fs;
use std::process::Command;

use super::util::{ensure_bin_dir, fallback_versions, fetch_github_version, run_command, run_sudo};
use super::Component;

pub struct Glow;

impl Component for Glow {
    fn id(&self) -> &str {
        "glow"
    }

    fn is_installed(&self) -> Result<bool> {
        Ok(which::which("glow").is_ok())
    }

    fn install(&self) -> Result<()> {
        install_glow()
    }

    fn uninstall(&self) -> Result<()> {
        if which::which("apt").is_ok() {
            let status = Command::new("sudo")
                .args(["apt", "remove", "-y", "glow"])
                .status()
                .context("running sudo apt remove -y glow")?;
            if !status.success() {
                bail!("sudo apt remove -y glow failed: {}", status);
            }
        }

        let bin = dirs::home_dir()
            .ok_or_else(|| anyhow!("no home dir"))?
            .join(".local/bin/glow");
        if bin.exists() {
            fs::remove_file(&bin)?;
        }
        Ok(())
    }
}

fn install_glow() -> Result<()> {
    if which::which("glow").is_ok() {
        return Ok(());
    }

    if run_sudo("apt", &["install", "-y", "glow"]).is_ok() {
        return Ok(());
    }

    let bin_dir = ensure_bin_dir()?;
    let version = fetch_github_version("charmbracelet/glow", fallback_versions::GLOW);

    let arch = match std::env::consts::ARCH {
        "x86_64" => "x86_64",
        "aarch64" => "arm64",
        _ => return Err(anyhow!("Unsupported architecture")),
    };

    let url = format!(
        "https://github.com/charmbracelet/glow/releases/download/v{}/glow_{}_Linux_{}.tar.gz",
        version, version, arch
    );

    run_command(
        "sh",
        &[
            "-c",
            &format!(
                "curl -Lo /tmp/glow.tar.gz '{}' && tar xf /tmp/glow.tar.gz -C /tmp && mv /tmp/glow_*/glow {}",
                url,
                bin_dir.display()
            ),
        ],
    )?;

    Ok(())
}
