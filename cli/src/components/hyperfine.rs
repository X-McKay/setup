//! `hyperfine` component - benchmarking CLI.
//!
//! Install delegates to the legacy installer, which prefers `apt` and
//! falls back to a standalone binary in `~/.local/bin/hyperfine`.
//!
//! Uninstall removes the apt package when available and also deletes the
//! local fallback binary if it exists.

use anyhow::{anyhow, bail, Context, Result};
use std::fs;
use std::process::Command;

use super::util::{
    ensure_bin_dir, fallback_versions, fetch_github_version, get_arch, run_command, run_sudo,
};
use super::Component;

pub struct Hyperfine;

impl Component for Hyperfine {
    fn id(&self) -> &str {
        "hyperfine"
    }

    fn is_installed(&self) -> Result<bool> {
        Ok(which::which("hyperfine").is_ok())
    }

    fn install(&self) -> Result<()> {
        install_hyperfine()
    }

    fn uninstall(&self) -> Result<()> {
        if which::which("apt").is_ok() {
            let status = Command::new("sudo")
                .args(["apt", "remove", "-y", "hyperfine"])
                .status()
                .context("running sudo apt remove -y hyperfine")?;
            if !status.success() {
                bail!("sudo apt remove -y hyperfine failed: {}", status);
            }
        }

        let bin = dirs::home_dir()
            .ok_or_else(|| anyhow!("no home dir"))?
            .join(".local/bin/hyperfine");
        if bin.exists() {
            fs::remove_file(&bin)?;
        }
        Ok(())
    }
}

fn install_hyperfine() -> Result<()> {
    if which::which("hyperfine").is_ok() {
        return Ok(());
    }

    if run_sudo("apt", &["install", "-y", "hyperfine"]).is_ok() {
        return Ok(());
    }

    let bin_dir = ensure_bin_dir()?;
    let version = fetch_github_version("sharkdp/hyperfine", fallback_versions::HYPERFINE);
    let arch = get_arch()?;

    let url = format!(
        "https://github.com/sharkdp/hyperfine/releases/download/v{}/hyperfine-v{}-{}-unknown-linux-musl.tar.gz",
        version, version, arch
    );

    run_command(
        "sh",
        &[
            "-c",
            &format!(
                "curl -Lo /tmp/hyperfine.tar.gz '{}' && tar xf /tmp/hyperfine.tar.gz -C /tmp && mv /tmp/hyperfine-*/hyperfine {}",
                url,
                bin_dir.display()
            ),
        ],
    )?;

    Ok(())
}
