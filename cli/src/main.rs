use anyhow::Result;
use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod commands;
mod config;
mod system;
mod ui;

use commands::{Cli, Commands};

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Some(cmd) => run_command(cmd),
        None => commands::interactive::run(),
    }
}

fn run_command(cmd: Commands) -> Result<()> {
    match cmd {
        Commands::Install(args) => commands::install::run(args),
        Commands::Dotfiles(args) => commands::dotfiles::run(args),
        Commands::Check(args) => commands::check::run(args),
        Commands::Update(args) => commands::update::run(args),
    }
}
