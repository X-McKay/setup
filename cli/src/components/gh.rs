//! `gh` component - GitHub CLI.
//!
//! Install delegates to the legacy apt-based installer, which adds the
//! GitHub CLI package repository and installs `gh`.
//!
//! Uninstall: unsupported for now. Authentication state and apt source
//! cleanup are deferred to the later uninstall phase work.

use anyhow::Result;

use super::util::run_command;
use super::Component;

pub struct Gh;

impl Component for Gh {
    fn id(&self) -> &str {
        "gh"
    }

    fn is_installed(&self) -> Result<bool> {
        Ok(which::which("gh").is_ok())
    }

    fn install(&self) -> Result<()> {
        install_gh()
    }
}

fn install_gh() -> Result<()> {
    if which::which("gh").is_ok() {
        return Ok(());
    }

    run_command(
        "sh",
        &[
            "-c",
            r#"(type -p wget >/dev/null || (sudo apt update && sudo apt install wget -y)) \
        && sudo mkdir -p -m 755 /etc/apt/keyrings \
        && out=$(mktemp) && wget -nv -O$out https://cli.github.com/packages/githubcli-archive-keyring.gpg \
        && cat $out | sudo tee /etc/apt/keyrings/githubcli-archive-keyring.gpg > /dev/null \
        && sudo chmod go+r /etc/apt/keyrings/githubcli-archive-keyring.gpg \
        && echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | sudo tee /etc/apt/sources.list.d/github-cli.list > /dev/null \
        && sudo apt update \
        && sudo apt install gh -y"#,
        ],
    )?;

    Ok(())
}
