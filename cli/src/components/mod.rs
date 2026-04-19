//! Component trait and registry.
//!
//! Each component is a unit struct implementing `Component`. Registered
//! in `registry::Registry::build()` and dispatched through the registry
//! by id.

use anyhow::Result;

pub mod apt;
pub mod docker;
pub mod glow;
pub mod just;
pub mod lazygit;
pub mod mise;
pub mod registry;
pub mod tools;

pub trait Component: Send + Sync {
    /// Matches the manifest `id`.
    fn id(&self) -> &str;

    /// Probe whether this component is currently present on the system.
    fn is_installed(&self) -> Result<bool>;

    /// Install the component. MUST be idempotent.
    fn install(&self) -> Result<()>;

    /// Whether it is safe to call `uninstall()` automatically during
    /// `--rollback-on-failure`. Default: true. Override to false for
    /// components that manage user material (SSH keys, GPG keys, etc.)
    /// — these still need `uninstall()` implemented if they support
    /// forced removal; they just won't be called automatically.
    fn is_reversible(&self) -> bool {
        true
    }

    /// Remove the component. Default refuses — this is the safe default
    /// for components that cannot be cleanly uninstalled. Override to
    /// enable `setup uninstall <id>`.
    fn uninstall(&self) -> Result<()> {
        anyhow::bail!(
            "{} does not implement uninstall — not removable by this tool",
            self.id()
        )
    }

    /// Post-install sanity check. Default delegates to `is_installed()`.
    fn verify(&self) -> Result<()> {
        if self.is_installed()? {
            Ok(())
        } else {
            anyhow::bail!("{} not installed", self.id())
        }
    }

    /// Describe what `install()` would do. Called by `--dry-run`.
    fn dry_run(&self) -> Result<Vec<String>> {
        Ok(vec![format!("would install {}", self.id())])
    }
}
