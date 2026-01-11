use anyhow::Result;
use console::style;
use inquire::Select;

use super::{check, dotfiles, install, update};

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
                install::run(install::InstallArgs {
                    component: None,
                    all: false,
                    yes: false,
                })?;
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
