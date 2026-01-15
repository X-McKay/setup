//! Shared utilities for package installation.

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;

/// Default fallback versions when GitHub API is unavailable
pub mod fallback_versions {
    pub const LAZYGIT: &str = "0.44.1";
    pub const GLOW: &str = "2.0.0";
    pub const BOTTOM: &str = "0.10.2";
    pub const HYPERFINE: &str = "1.18.0";
}

/// Fetch the latest version from GitHub releases API.
/// Returns the version string or a fallback with a warning.
pub fn fetch_github_version(repo: &str, fallback: &str) -> String {
    let cmd = format!(
        "curl -sf https://api.github.com/repos/{}/releases/latest | grep -o '\"tag_name\": \"v\\?[^\"]*\"' | sed 's/.*\"v\\?\\([^\"]*\\)\"/\\1/'",
        repo
    );

    match run_command("sh", &["-c", &cmd]) {
        Ok(version) => {
            let v = version.trim();
            if v.is_empty() {
                eprintln!(
                    "Warning: Could not fetch latest {} version, using fallback {}",
                    repo, fallback
                );
                fallback.to_string()
            } else {
                v.to_string()
            }
        }
        Err(_) => {
            eprintln!(
                "Warning: GitHub API unavailable for {}, using fallback version {}",
                repo, fallback
            );
            fallback.to_string()
        }
    }
}

/// Convert a Path to &str with proper error handling.
pub fn path_to_str(path: &std::path::Path) -> Result<&str> {
    path.to_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid UTF-8 in path: {:?}", path))
}

/// Run a shell command and return its output.
pub fn run_command(cmd: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(cmd)
        .args(args)
        .output()
        .with_context(|| format!("Failed to execute: {} {:?}", cmd, args))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Command failed: {}", stderr)
    }
}

/// Run a shell command with sudo.
pub fn run_sudo(cmd: &str, args: &[&str]) -> Result<String> {
    let mut sudo_args = vec![cmd];
    sudo_args.extend(args);
    run_command("sudo", &sudo_args)
}

/// Install apt packages.
pub fn apt_install(packages: &[&str]) -> Result<()> {
    let mut args = vec!["install", "-y"];
    args.extend(packages);
    run_sudo("apt", &args)?;
    Ok(())
}

/// Get the user's bin directory (~/.local/bin), creating it if needed.
pub fn ensure_bin_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not find home directory")?;
    let bin_dir = home.join(".local").join("bin");
    std::fs::create_dir_all(&bin_dir)?;
    Ok(bin_dir)
}

/// Get current system architecture for binary downloads.
pub fn get_arch() -> Result<&'static str> {
    match std::env::consts::ARCH {
        "x86_64" => Ok("x86_64"),
        "aarch64" => Ok("aarch64"),
        _ => anyhow::bail!("Unsupported architecture: {}", std::env::consts::ARCH),
    }
}

/// Get architecture string for GitHub releases (some use different naming).
pub fn get_arch_alt() -> Result<&'static str> {
    match std::env::consts::ARCH {
        "x86_64" => Ok("amd64"),
        "aarch64" => Ok("arm64"),
        _ => anyhow::bail!("Unsupported architecture: {}", std::env::consts::ARCH),
    }
}
