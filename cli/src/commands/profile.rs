//! `setup profile` - inspect profiles and manage active.toml.

use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use console::style;

use crate::manifest::{intent, loader};

#[derive(Args)]
pub struct ProfileArgs {
    #[command(subcommand)]
    pub command: ProfileCmd,
}

#[derive(Subcommand)]
pub enum ProfileCmd {
    /// List known profiles
    List,
    /// Show resolved components for a profile
    Show { name: String },
    /// Add a profile to ~/.config/setup/active.toml (no install)
    Activate { name: String },
    /// Remove a profile from ~/.config/setup/active.toml (no uninstall)
    Deactivate { name: String },
}

pub fn run(args: ProfileArgs) -> Result<()> {
    let manifest = loader::load().context("loading manifest")?;

    match args.command {
        ProfileCmd::List => {
            println!("{}", style("Profiles:").bold());
            for (name, p) in &manifest.profiles {
                let desc = if p.description.is_empty() {
                    String::new()
                } else {
                    format!(" - {}", p.description)
                };
                println!("  {}{}", style(name).cyan(), style(desc).dim());
            }
        }
        ProfileCmd::Show { name } => {
            let set = crate::manifest::resolver::expand_selection(
                &manifest,
                std::slice::from_ref(&name),
                &[],
            )?;
            println!("{}", style(format!("Components in profile {}:", name)).bold());
            for id in &set {
                println!("  {}", id);
            }
        }
        ProfileCmd::Activate { name } => {
            if !manifest.profiles.contains_key(&name) {
                anyhow::bail!("unknown profile: {}", name);
            }
            let path = intent::default_path().context("no config dir")?;
            let mut i = intent::read(&path)?;
            intent::union_add(&mut i, std::slice::from_ref(&name));
            intent::write(&path, &i)?;
            println!(
                "{} active_profiles = {:?}",
                style("✓").green().bold(),
                i.active_profiles
            );
        }
        ProfileCmd::Deactivate { name } => {
            let path = intent::default_path().context("no config dir")?;
            let mut i = intent::read(&path)?;
            intent::remove(&mut i, &name);
            intent::write(&path, &i)?;
            println!(
                "{} active_profiles = {:?}",
                style("✓").green().bold(),
                i.active_profiles
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn tmp_intent() -> PathBuf {
        let p =
            std::env::temp_dir().join(format!("setup-profile-test-{}.toml", std::process::id()));
        if p.exists() {
            std::fs::remove_file(&p).unwrap();
        }
        // Test-only process-local env override used before any worker threads exist.
        unsafe {
            std::env::set_var("SETUP_INTENT", &p);
        }
        p
    }

    #[test]
    fn activate_then_deactivate_roundtrip() {
        let p = tmp_intent();

        run(ProfileArgs {
            command: ProfileCmd::Activate {
                name: "server".into(),
            },
        })
        .unwrap();
        let i = intent::read(&p).unwrap();
        assert!(i.active_profiles.contains(&"server".to_string()));

        run(ProfileArgs {
            command: ProfileCmd::Deactivate {
                name: "server".into(),
            },
        })
        .unwrap();
        let i = intent::read(&p).unwrap();
        assert!(!i.active_profiles.contains(&"server".to_string()));
    }

    #[test]
    fn activate_unknown_profile_errors() {
        tmp_intent();
        let err = run(ProfileArgs {
            command: ProfileCmd::Activate {
                name: "ghost".into(),
            },
        })
        .unwrap_err();
        assert!(err.to_string().contains("unknown profile"));
    }
}
