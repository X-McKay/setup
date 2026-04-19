//! `docker` component - Docker Engine and Compose.
//!
//! Install delegates to the legacy installer, which also adds the current
//! user to the `docker` group as a side effect.
//!
//! Uninstall: unsupported for now. Reversing the package install and group
//! membership changes safely is deferred to the later uninstall phase work.

use anyhow::Result;

use super::util::{run_command, run_sudo};
use super::Component;

pub struct Docker;

impl Component for Docker {
    fn id(&self) -> &str {
        "docker"
    }

    fn is_installed(&self) -> Result<bool> {
        Ok(which::which("docker").is_ok())
    }

    fn install(&self) -> Result<()> {
        install_docker()
    }
}

fn install_docker() -> Result<()> {
    if which::which("docker").is_ok() {
        return Ok(());
    }

    run_sudo("apt", &["update"])?;
    run_sudo(
        "apt",
        &[
            "install",
            "-y",
            "ca-certificates",
            "curl",
            "gnupg",
            "lsb-release",
        ],
    )?;

    let script = run_command("curl", &["-fsSL", "https://get.docker.com"])?;
    run_sudo("sh", &["-c", &script])?;

    let user = std::env::var("USER").unwrap_or_else(|_| "al".to_string());
    run_sudo("usermod", &["-aG", "docker", &user])?;

    Ok(())
}
