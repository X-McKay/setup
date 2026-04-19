//! `lazygit` component - terminal UI for Git workflows.
//!
//! Install uses the existing GitHub-release download path into
//! `~/.local/bin/lazygit`.
//!
//! Uninstall removes only that setup-managed binary.

use anyhow::{anyhow, Result};
use std::fs;

use super::util::{ensure_bin_dir, fallback_versions, fetch_github_version, run_command};
use super::Component;

pub struct Lazygit;

impl Component for Lazygit {
    fn id(&self) -> &str {
        "lazygit"
    }

    fn is_installed(&self) -> Result<bool> {
        Ok(which::which("lazygit").is_ok())
    }

    fn install(&self) -> Result<()> {
        install_lazygit()
    }

    fn uninstall(&self) -> Result<()> {
        let bin = dirs::home_dir()
            .ok_or_else(|| anyhow!("no home dir"))?
            .join(".local/bin/lazygit");
        if bin.exists() {
            fs::remove_file(&bin)?;
        }
        Ok(())
    }
}

fn install_lazygit() -> Result<()> {
    if which::which("lazygit").is_ok() {
        return Ok(());
    }

    let bin_dir = ensure_bin_dir()?;
    let version = fetch_github_version("jesseduffield/lazygit", fallback_versions::LAZYGIT);

    let arch = match std::env::consts::ARCH {
        "x86_64" => "x86_64",
        "aarch64" => "arm64",
        _ => return Err(anyhow!("Unsupported architecture")),
    };

    let url = format!(
        "https://github.com/jesseduffield/lazygit/releases/download/v{}/lazygit_{}_Linux_{}.tar.gz",
        version, version, arch
    );

    run_command(
        "sh",
        &[
            "-c",
            &format!(
                "curl -Lo /tmp/lazygit.tar.gz '{}' && tar xf /tmp/lazygit.tar.gz -C {} lazygit",
                url,
                bin_dir.display()
            ),
        ],
    )?;

    Ok(())
}
