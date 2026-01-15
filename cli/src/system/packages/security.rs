//! Security setup: SSH keys and GPG configuration.

use anyhow::{Context, Result};
use std::fs;
use super::utils::{apt_install, path_to_str, run_command};

/// Generate SSH keys.
pub fn setup_ssh_keys() -> Result<()> {
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
            run_command("git", &["config", "--global", "user.email"])
                .map(|s| s.trim().to_string())
        })
        .unwrap_or_else(|_| "user@localhost".to_string());

    eprintln!("Note: SSH key generated without passphrase. Add one with: ssh-keygen -p -f ~/.ssh/id_ed25519");

    run_command("ssh-keygen", &[
        "-t", "ed25519",
        "-C", &email,
        "-f", path_to_str(&key_path)?,
        "-N", "",
    ])?;

    run_command("chmod", &["600", path_to_str(&key_path)?])?;
    run_command("chmod", &["644", &format!("{}.pub", key_path.display())])?;

    println!("\nSSH public key generated. Add this to GitHub/GitLab:");
    println!("----------------------------------------");
    let pub_key = fs::read_to_string(format!("{}.pub", key_path.display()))?;
    println!("{}", pub_key);
    println!("----------------------------------------");

    Ok(())
}

/// Generate GPG keys and configure git signing.
pub fn setup_gpg() -> Result<()> {
    if !which::which("gpg").is_ok() {
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
        eprintln!("\nNote: Configuring git to sign commits with GPG key: {}", key_id);
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
