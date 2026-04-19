//! `claude-code` component - Claude Code CLI.
//!
//! Install delegates to the legacy installer script.
//!
//! Uninstall removes the global npm package used by the current installer.

use anyhow::{bail, Context, Result};
use std::process::Command;

use super::Component;
use crate::system::packages;

pub struct ClaudeCode;

impl Component for ClaudeCode {
    fn id(&self) -> &str {
        "claude-code"
    }

    fn is_installed(&self) -> Result<bool> {
        Ok(which::which("claude").is_ok() || which::which("claude-code").is_ok())
    }

    fn install(&self) -> Result<()> {
        packages::install_claude_code()
    }

    fn uninstall(&self) -> Result<()> {
        let status = Command::new("npm")
            .args(["uninstall", "-g", "@anthropic-ai/claude-code"])
            .status()
            .context("running npm uninstall -g @anthropic-ai/claude-code")?;
        if !status.success() {
            bail!("npm uninstall -g @anthropic-ai/claude-code failed: {}", status);
        }
        Ok(())
    }
}
