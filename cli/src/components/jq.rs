//! `jq` component - JSON processor.
//!
//! Install delegates to the legacy installer, which prefers `apt` and
//! falls back to a downloaded standalone binary.
//!
//! Uninstall: unsupported. The manifest treats `jq` as baseline system
//! tooling rather than something the tool removes automatically.

use anyhow::Result;

use super::Component;
use super::util::{brew_install, ensure_bin_dir, get_arch_alt, run_command, run_sudo};
use crate::system::platform::Platform;

pub struct Jq;

impl Component for Jq {
    fn id(&self) -> &str {
        "jq"
    }

    fn is_installed(&self) -> Result<bool> {
        Ok(which::which("jq").is_ok())
    }

    fn install(&self) -> Result<()> {
        install_jq()
    }
}

fn install_jq() -> Result<()> {
    if which::which("jq").is_ok() {
        return Ok(());
    }

    if Platform::current()? == Platform::MacOs {
        return brew_install(&["jq"]);
    }

    if run_sudo("apt", &["install", "-y", "jq"]).is_ok() {
        return Ok(());
    }

    let bin_dir = ensure_bin_dir()?;
    let arch = get_arch_alt()?;

    run_command(
        "sh",
        &[
            "-c",
            &format!(
                "curl -Lo {}/jq 'https://github.com/jqlang/jq/releases/latest/download/jq-linux-{}' && chmod +x {}/jq",
                bin_dir.display(),
                arch,
                bin_dir.display()
            ),
        ],
    )?;

    Ok(())
}
