//! `gpg` component - GPG key setup and git signing config.
//!
//! Install delegates to the legacy setup flow, which may install `gnupg`,
//! generate a key, and configure git commit signing.
//!
//! Uninstall remains manual-only. The tool refuses to automate GPG key
//! deletion because it is destructive user material.

use anyhow::{bail, Result};
use std::fs;
use std::process::Command;

use super::util::{apt_install, run_command};
use super::Component;

pub struct Gpg;

impl Component for Gpg {
    fn id(&self) -> &str {
        "gpg"
    }

    fn is_installed(&self) -> Result<bool> {
        Ok(Command::new("gpg")
            .args(["--list-secret-keys"])
            .output()
            .map(|output| !output.stdout.is_empty())
            .unwrap_or(false))
    }

    fn install(&self) -> Result<()> {
        setup_gpg()
    }

    fn is_reversible(&self) -> bool {
        false
    }

    fn uninstall(&self) -> Result<()> {
        bail!(
            "gpg uninstall requires manual action: run `gpg --delete-secret-keys <keyid>` then \
             `gpg --delete-keys <keyid>`. This tool will not automate GPG key deletion."
        )
    }
}

fn setup_gpg() -> Result<()> {
    if which::which("gpg").is_err() {
        apt_install(&["gnupg"])?;
    }

    let existing = run_command("gpg", &["--list-secret-keys", "--keyid-format=long"]);
    if let Ok(ref output) = existing {
        if !output.trim().is_empty() {
            println!("GPG key already exists");
            return Ok(());
        }
    }

    let name = run_command("git", &["config", "--global", "user.name"])
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "User".to_string());

    let email = run_command("git", &["config", "--global", "user.email"])
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "user@localhost".to_string());

    let batch_content = format!(
        r#"Key-Type: eddsa
Key-Curve: ed25519
Key-Usage: sign
Subkey-Type: ecdh
Subkey-Curve: cv25519
Subkey-Usage: encrypt
Name-Real: {}
Name-Email: {}
Expire-Date: 2y
%no-protection
%commit
"#,
        name, email
    );

    let batch_path = "/tmp/gpg_batch";
    fs::write(batch_path, batch_content)?;

    run_command("gpg", &["--batch", "--generate-key", batch_path])?;
    fs::remove_file(batch_path)?;

    let keys_output = run_command("gpg", &["--list-secret-keys", "--keyid-format=long"])?;
    println!("\nGPG key generated:");
    println!("{}", keys_output);

    if let Some(key_id) = extract_gpg_key_id(&keys_output) {
        eprintln!(
            "\nNote: Configuring git to sign commits with GPG key: {}",
            key_id
        );
        eprintln!("This will set git config: user.signingkey and commit.gpgsign=true");

        if let Err(e) = run_command("git", &["config", "--global", "user.signingkey", &key_id]) {
            eprintln!("Warning: Could not set git signing key: {}", e);
        }
        if let Err(e) = run_command("git", &["config", "--global", "commit.gpgsign", "true"]) {
            eprintln!("Warning: Could not enable git commit signing: {}", e);
        }
        println!("\nGit configured to sign commits with key: {}", key_id);

        let public_key = run_command("gpg", &["--armor", "--export", &key_id])?;
        println!("\nAdd this GPG public key to GitHub:");
        println!("----------------------------------------");
        println!("{}", public_key);
        println!("----------------------------------------");
    }

    Ok(())
}

fn extract_gpg_key_id(output: &str) -> Option<String> {
    for line in output.lines() {
        if line.starts_with("sec") {
            if let Some(key_part) = line.split('/').nth(1) {
                if let Some(key_id) = key_part.split_whitespace().next() {
                    return Some(key_id.to_string());
                }
            }
        }
    }
    None
}
