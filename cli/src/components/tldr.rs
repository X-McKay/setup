//! `tldr` component - simplified command help pages.
//!
//! Install delegates to the legacy installer, which prefers `apt` and
//! falls back to a standalone `tealdeer` binary.
//!
//! Uninstall: unsupported. The plan keeps `tldr` on the default refuse path.

use anyhow::Result;

use super::util::{ensure_bin_dir, get_arch, run_command, run_sudo};
use super::Component;

pub struct Tldr;

impl Component for Tldr {
    fn id(&self) -> &str {
        "tldr"
    }

    fn is_installed(&self) -> Result<bool> {
        Ok(which::which("tldr").is_ok())
    }

    fn install(&self) -> Result<()> {
        install_tldr()
    }
}

fn install_tldr() -> Result<()> {
    if which::which("tldr").is_ok() {
        return Ok(());
    }

    if run_sudo("apt", &["install", "-y", "tldr"]).is_ok() {
        let _ = run_command("tldr", &["--update"]);
        return Ok(());
    }

    let bin_dir = ensure_bin_dir()?;
    let arch = get_arch()?;

    run_command(
        "sh",
        &[
            "-c",
            &format!(
                "curl -Lo {}/tldr 'https://github.com/dbrgn/tealdeer/releases/latest/download/tealdeer-linux-{}-musl' && chmod +x {}/tldr && {}/tldr --update",
                bin_dir.display(),
                arch,
                bin_dir.display(),
                bin_dir.display()
            ),
        ],
    )?;

    Ok(())
}
