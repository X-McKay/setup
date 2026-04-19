use clap::{Parser, Subcommand};

pub mod check;
pub mod dotfiles;
pub mod install;
pub mod interactive;
pub mod uninstall;
pub mod update;

#[derive(Parser)]
#[command(name = "setup")]
#[command(author = "Al McKay")]
#[command(version)]
#[command(about = "Personal development environment setup CLI", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Install development tools and packages
    Install(install::InstallArgs),

    /// Manage dotfiles (sync, diff, edit)
    Dotfiles(dotfiles::DotfilesArgs),

    /// Remove components
    Uninstall(uninstall::UninstallArgs),

    /// Check system health and installed tools
    Check(check::CheckArgs),

    /// Update installed tools and configs
    Update(update::UpdateArgs),
}
