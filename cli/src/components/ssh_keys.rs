//! `ssh-keys` component - SSH key generation.
//!
//! Install delegates to the legacy setup flow, which generates a new
//! `~/.ssh/id_ed25519` keypair and prints the public key for the user.
//!
//! Uninstall deletes the generated keypair. This is destructive user
//! material, so the component is not automatically reversible.

use anyhow::{anyhow, Context, Result};
use std::fs;

use super::util::{path_to_str, run_command};
use super::Component;

pub struct SshKeys;

impl Component for SshKeys {
    fn id(&self) -> &str {
        "ssh-keys"
    }

    fn is_installed(&self) -> Result<bool> {
        Ok(dirs::home_dir()
            .unwrap_or_default()
            .join(".ssh/id_ed25519")
            .exists())
    }

    fn install(&self) -> Result<()> {
        setup_ssh_keys()
    }

    fn is_reversible(&self) -> bool {
        false
    }

    fn uninstall(&self) -> Result<()> {
        let home = dirs::home_dir().ok_or_else(|| anyhow!("no home dir"))?;
        for name in ["id_ed25519", "id_ed25519.pub"] {
            let path = home.join(".ssh").join(name);
            if path.exists() {
                fs::remove_file(&path)?;
            }
        }
        Ok(())
    }
}

fn setup_ssh_keys() -> Result<()> {
    let home = dirs::home_dir().context("Could not find home directory")?;
    let ssh_dir = home.join(".ssh");
    let key_path = ssh_dir.join("id_ed25519");

    if key_path.exists() {
        println!("SSH key already exists at {}", key_path.display());
        return Ok(());
    }

    fs::create_dir_all(&ssh_dir)?;
    run_command("chmod", &["700", path_to_str(&ssh_dir)?])?;

    let email = std::env::var("EMAIL")
        .or_else(|_| {
            run_command("git", &["config", "--global", "user.email"]).map(|s| s.trim().to_string())
        })
        .unwrap_or_else(|_| "user@localhost".to_string());

    eprintln!(
        "Note: SSH key generated without passphrase. Add one with: ssh-keygen -p -f ~/.ssh/id_ed25519"
    );

    run_command(
        "ssh-keygen",
        &[
            "-t",
            "ed25519",
            "-C",
            &email,
            "-f",
            path_to_str(&key_path)?,
            "-N",
            "",
        ],
    )?;

    run_command("chmod", &["600", path_to_str(&key_path)?])?;
    run_command("chmod", &["644", &format!("{}.pub", key_path.display())])?;

    println!("\nSSH public key generated. Add this to GitHub/GitLab:");
    println!("----------------------------------------");
    let pub_key = fs::read_to_string(format!("{}.pub", key_path.display()))?;
    println!("{}", pub_key);
    println!("----------------------------------------");

    Ok(())
}
