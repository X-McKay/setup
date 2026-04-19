//! `monitoring` component - system monitoring tooling.
//!
//! Install delegates to the legacy installer, which installs packages and
//! configures services, scripts, and cron jobs.
//!
//! Uninstall: unsupported for now because cleanup spans packages, service
//! state, system config, and user-owned artifacts.

use anyhow::Result;

use super::Component;
use crate::system::packages;

pub struct Monitoring;

impl Component for Monitoring {
    fn id(&self) -> &str {
        "monitoring"
    }

    fn is_installed(&self) -> Result<bool> {
        Ok(which::which("htop").is_ok() && which::which("netdata").is_ok())
    }

    fn install(&self) -> Result<()> {
        packages::install_monitoring()
    }
}
