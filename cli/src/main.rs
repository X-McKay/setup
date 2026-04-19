use anyhow::Result;
use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use setup::commands::{self, Cli, Commands};
use setup::{components, manifest};

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Validate manifest <-> registry consistency on every run.
    // Keep this as a warning during migration so unrelated flows continue
    // to work even if a future port introduces drift.
    if let (Ok(manifest), registry) = (
        manifest::loader::load(),
        components::registry::Registry::build(),
    ) {
        if let Err(e) = registry.validate_against(&manifest) {
            eprintln!("warning: manifest/registry drift:\n{}\n", e);
        }
    }

    let cli = Cli::parse();

    match cli.command {
        Some(cmd) => run_command(cmd),
        None => commands::interactive::run(),
    }
}

fn run_command(cmd: Commands) -> Result<()> {
    match cmd {
        Commands::Install(args) => commands::install::run(args),
        Commands::List(args) => commands::list::run(args),
        Commands::Profile(args) => commands::profile::run(args),
        Commands::Doctor(args) => commands::doctor::run(args),
        Commands::Drift(args) => commands::drift::run(args),
        Commands::Uninstall(args) => commands::uninstall::run(args),
        Commands::Dotfiles(args) => commands::dotfiles::run(args),
        Commands::Check(args) => commands::check::run(args),
        Commands::Update(args) => commands::update::run(args),
    }
}
