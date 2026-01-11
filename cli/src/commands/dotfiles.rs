use anyhow::Result;
use clap::{Args, Subcommand};
use console::style;

use crate::config::dotfiles as dotfiles_config;
use crate::ui::prompts;

#[derive(Args)]
pub struct DotfilesArgs {
    #[command(subcommand)]
    pub action: Option<DotfilesAction>,
}

#[derive(Subcommand)]
pub enum DotfilesAction {
    /// Sync dotfiles to home directory
    Sync {
        /// Overwrite existing files without prompting
        #[arg(short, long)]
        force: bool,
    },
    /// Show diff between repo and installed dotfiles
    Diff,
    /// List all managed dotfiles
    List,
    /// Edit a specific dotfile
    Edit {
        /// Name of the dotfile to edit (e.g., "bashrc", "tmux")
        name: Option<String>,
    },
    /// Backup current dotfiles before syncing
    Backup,
}

pub fn run(args: DotfilesArgs) -> Result<()> {
    let action = match args.action {
        Some(a) => a,
        None => prompts::select_dotfiles_action()?,
    };

    match action {
        DotfilesAction::Sync { force } => sync_dotfiles(force),
        DotfilesAction::Diff => show_diff(),
        DotfilesAction::List => list_dotfiles(),
        DotfilesAction::Edit { name } => edit_dotfile(name),
        DotfilesAction::Backup => backup_dotfiles(),
    }
}

fn sync_dotfiles(force: bool) -> Result<()> {
    let dotfiles = dotfiles_config::get_managed_dotfiles();

    println!(
        "{}",
        style("Syncing dotfiles to home directory...").cyan().bold()
    );

    for (name, source, target) in &dotfiles {
        if target.exists() && !force {
            if !prompts::confirm_overwrite(name)? {
                println!("  {} {} (skipped)", style("→").yellow(), name);
                continue;
            }
        }

        dotfiles_config::copy_dotfile(source, target)?;
        println!("  {} {}", style("✓").green(), name);
    }

    println!("\n{}", style("Dotfiles synced successfully!").green().bold());
    Ok(())
}

fn show_diff() -> Result<()> {
    let dotfiles = dotfiles_config::get_managed_dotfiles();

    println!("{}", style("Comparing dotfiles...").cyan().bold());

    let mut has_diff = false;
    for (name, source, target) in &dotfiles {
        if let Some(diff) = dotfiles_config::diff_files(source, target)? {
            has_diff = true;
            println!("\n{} {}:", style("─").dim(), style(name).bold());
            println!("{}", diff);
        }
    }

    if !has_diff {
        println!("{}", style("All dotfiles are in sync!").green());
    }

    Ok(())
}

fn list_dotfiles() -> Result<()> {
    let dotfiles = dotfiles_config::get_managed_dotfiles();

    println!("{}", style("Managed dotfiles:").cyan().bold());
    println!();

    for (name, source, target) in &dotfiles {
        let status = if target.exists() {
            if dotfiles_config::files_match(source, target)? {
                style("✓ synced").green()
            } else {
                style("⚠ differs").yellow()
            }
        } else {
            style("○ not installed").dim()
        };

        println!("  {} {} → {}", status, name, target.display());
    }

    Ok(())
}

fn edit_dotfile(name: Option<String>) -> Result<()> {
    let dotfile_name = match name {
        Some(n) => n,
        None => prompts::select_dotfile_to_edit()?,
    };

    let dotfiles = dotfiles_config::get_managed_dotfiles();
    let (_, source, _) = dotfiles
        .iter()
        .find(|(n, _, _)| n == &dotfile_name)
        .ok_or_else(|| anyhow::anyhow!("Unknown dotfile: {}", dotfile_name))?;

    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
    std::process::Command::new(&editor)
        .arg(source)
        .status()?;

    Ok(())
}

fn backup_dotfiles() -> Result<()> {
    let backup_dir = dotfiles_config::create_backup()?;
    println!(
        "{} Dotfiles backed up to: {}",
        style("✓").green().bold(),
        backup_dir.display()
    );
    Ok(())
}
