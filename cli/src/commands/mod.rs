use clap::{Parser, Subcommand};

pub mod check;
pub mod doctor;
pub mod dotfiles;
pub mod install;
pub mod interactive;
pub mod list;
pub mod profile;
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

    /// Print the component catalog
    List(list::ListArgs),

    /// Inspect and manage manifest profiles
    Profile(profile::ProfileArgs),

    /// System health + drift report
    Doctor(doctor::DoctorArgs),

    /// Remove components
    Uninstall(uninstall::UninstallArgs),

    /// Check system health and installed tools
    Check(check::CheckArgs),

    /// Update installed tools and configs
    Update(update::UpdateArgs),
}
