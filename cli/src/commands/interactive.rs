use anyhow::Result;
use console::style;
use inquire::{MultiSelect, Select};

use super::{check, dotfiles, install, update};
use crate::components::registry::Registry;
use crate::manifest::loader;

pub fn run() -> Result<()> {
    println!();
    println!(
        "{}",
        style("  ╔═══════════════════════════════════════╗").cyan()
    );
    println!(
        "{}",
        style("  ║     Development Environment Setup     ║").cyan()
    );
    println!(
        "{}",
        style("  ╚═══════════════════════════════════════╝").cyan()
    );
    println!();

    loop {
        let options = vec![
            "Install components",
            "Manage dotfiles",
            "Health check",
            "Update tools",
            "Exit",
        ];

        let selection = Select::new("What would you like to do?", options)
            .with_help_message("Use arrow keys to navigate, Enter to select")
            .prompt()?;

        match selection {
            "Install components" => {
                let install_mode = Select::new(
                    "How would you like to install?",
                    vec!["Install all", "Select components", "Install by profile"],
                )
                .with_help_message("Profiles and direct ids both resolve through the manifest")
                .prompt()?;

                match install_mode {
                    "Install all" => {
                        install::run(install::InstallArgs {
                            components: vec![],
                            profiles: vec![],
                            all: true,
                            dry_run: false,
                            verify: false,
                            keep_going: false,
                            rollback_on_failure: false,
                            yes: false,
                        })?;
                    }
                    "Select components" => {
                        let ids = Registry::build().ids();
                        let picked = MultiSelect::new("Select components:", ids)
                            .with_help_message("Space to select, Enter to confirm")
                            .prompt()?;
                        install::run(install::InstallArgs {
                            components: picked,
                            profiles: vec![],
                            all: false,
                            dry_run: false,
                            verify: false,
                            keep_going: false,
                            rollback_on_failure: false,
                            yes: false,
                        })?;
                    }
                    "Install by profile" => {
                        let manifest = loader::load()?;
                        let names: Vec<String> = manifest.profiles.keys().cloned().collect();
                        let picked = MultiSelect::new("Select profiles:", names)
                            .with_help_message("Space to select, Enter to confirm")
                            .prompt()?;
                        install::run(install::InstallArgs {
                            components: vec![],
                            profiles: picked,
                            all: false,
                            dry_run: false,
                            verify: false,
                            keep_going: false,
                            rollback_on_failure: false,
                            yes: false,
                        })?;
                    }
                    _ => unreachable!(),
                }
            }
            "Manage dotfiles" => {
                dotfiles::run(dotfiles::DotfilesArgs { action: None })?;
            }
            "Health check" => {
                check::run(check::CheckArgs {
                    category: None,
                    verbose: true,
                })?;
            }
            "Update tools" => {
                update::run(update::UpdateArgs {
                    component: None,
                    all: false,
                    yes: false,
                })?;
            }
            "Exit" => {
                println!("{}", style("Goodbye!").green());
                break;
            }
            _ => unreachable!(),
        }

        println!();
    }

    Ok(())
}
