//! `tpm` component - tmux plugin manager.
//!
//! Install delegates to the legacy installer, which clones the TPM repo and
//! may append TPM configuration to `~/.tmux.conf`.
//!
//! Uninstall removes only the TPM checkout directory and leaves any tmux
//! config edits in place for manual cleanup.

use anyhow::{anyhow, Context, Result};
use std::fs;

use super::util::{path_to_str, run_command};
use super::Component;

pub struct Tpm;

impl Component for Tpm {
    fn id(&self) -> &str {
        "tpm"
    }

    fn is_installed(&self) -> Result<bool> {
        Ok(dirs::home_dir()
            .unwrap_or_default()
            .join(".tmux/plugins/tpm")
            .exists())
    }

    fn install(&self) -> Result<()> {
        install_tpm()
    }

    fn uninstall(&self) -> Result<()> {
        let dir = dirs::home_dir()
            .ok_or_else(|| anyhow!("no home dir"))?
            .join(".tmux/plugins/tpm");
        if dir.exists() {
            fs::remove_dir_all(&dir)?;
        }
        Ok(())
    }
}

fn install_tpm() -> Result<()> {
    let home = dirs::home_dir().context("Could not find home directory")?;
    let tpm_dir = home.join(".tmux").join("plugins").join("tpm");

    if tpm_dir.exists() {
        return Ok(());
    }

    run_command(
        "git",
        &[
            "clone",
            "https://github.com/tmux-plugins/tpm",
            path_to_str(&tpm_dir)?,
        ],
    )?;

    let tmux_conf = home.join(".tmux.conf");
    if tmux_conf.exists() {
        let content = fs::read_to_string(&tmux_conf)?;
        if !content.contains("tmux-plugins/tpm") {
            let tpm_config = r#"

# TPM (Tmux Plugin Manager)
set -g @plugin 'tmux-plugins/tpm'
set -g @plugin 'tmux-plugins/tmux-sensible'
set -g @plugin 'tmux-plugins/tmux-resurrect'
set -g @plugin 'tmux-plugins/tmux-continuum'

# Initialize TPM (keep this line at the very bottom)
run '~/.tmux/plugins/tpm/tpm'
"#;
            let mut file = fs::OpenOptions::new().append(true).open(&tmux_conf)?;
            use std::io::Write;
            file.write_all(tpm_config.as_bytes())?;
        }
    }

    Ok(())
}
