//! `jq` component - JSON processor.
//!
//! Install delegates to the legacy installer, which prefers `apt` and
//! falls back to a downloaded standalone binary.
//!
//! Uninstall: unsupported. The manifest treats `jq` as baseline system
//! tooling rather than something the tool removes automatically.

use anyhow::Result;

use super::Component;
use crate::system::packages;

pub struct Jq;

impl Component for Jq {
    fn id(&self) -> &str {
        "jq"
    }

    fn is_installed(&self) -> Result<bool> {
        Ok(which::which("jq").is_ok())
    }

    fn install(&self) -> Result<()> {
        packages::install_jq()
    }
}
