use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct CheckArgs {
    /// Check a specific category
    #[arg(short, long)]
    pub category: Option<CheckCategory>,

    /// Show verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Clone, clap::ValueEnum)]
pub enum CheckCategory {
    /// Check installed tools
    Tools,
    /// Check dotfiles sync status
    Dotfiles,
    /// Check system resources
    System,
    /// Check all categories
    All,
}

pub fn run(_args: CheckArgs) -> Result<()> {
    eprintln!(
        "{} `setup check` is deprecated - use `setup doctor`. Forwarding.",
        console::style("warn:").yellow()
    );
    super::doctor::run(super::doctor::DoctorArgs {
        profiles: vec![],
        verify: false,
        warn_only: false,
    })
}
