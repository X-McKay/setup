use anyhow::Result;
use clap::Args;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};

use crate::system::packages;
use crate::ui::prompts;

#[derive(Args)]
pub struct UpdateArgs {
    /// Update a specific component
    #[arg(short, long)]
    pub component: Option<UpdateComponent>,

    /// Update all components
    #[arg(short, long)]
    pub all: bool,

    /// Skip confirmation prompts
    #[arg(short = 'y', long)]
    pub yes: bool,
}

#[derive(Clone, clap::ValueEnum)]
pub enum UpdateComponent {
    /// Update system packages
    System,
    /// Update mise and managed runtimes
    Mise,
    /// Update Rust-based tools
    Rust,
    /// Sync dotfiles from repo
    Dotfiles,
}

impl UpdateComponent {
    pub fn all() -> Vec<UpdateComponent> {
        vec![
            UpdateComponent::System,
            UpdateComponent::Mise,
            UpdateComponent::Rust,
            UpdateComponent::Dotfiles,
        ]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            UpdateComponent::System => "System Packages",
            UpdateComponent::Mise => "Mise Runtimes",
            UpdateComponent::Rust => "Rust Tools",
            UpdateComponent::Dotfiles => "Dotfiles",
        }
    }
}

pub fn run(args: UpdateArgs) -> Result<()> {
    let components = if args.all {
        UpdateComponent::all()
    } else if let Some(component) = args.component {
        vec![component]
    } else {
        prompts::select_update_components()?
    };

    if components.is_empty() {
        println!("{}", style("No components selected.").yellow());
        return Ok(());
    }

    if !args.yes {
        let names: Vec<_> = components.iter().map(|c| c.display_name()).collect();
        if !prompts::confirm_update(&names)? {
            println!("{}", style("Update cancelled.").yellow());
            return Ok(());
        }
    }

    for component in &components {
        update_component(component)?;
    }

    println!(
        "\n{}",
        style("All components updated successfully!").green().bold()
    );

    Ok(())
}

fn update_component(component: &UpdateComponent) -> Result<()> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.blue} {msg}")
            .unwrap(),
    );
    pb.set_message(format!("Updating {}...", component.display_name()));
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    let result = match component {
        UpdateComponent::System => packages::update_system(),
        UpdateComponent::Mise => packages::update_mise(),
        UpdateComponent::Rust => packages::update_rust_tools(),
        UpdateComponent::Dotfiles => packages::sync_dotfiles(),
    };

    pb.finish_and_clear();

    match result {
        Ok(_) => {
            println!(
                "{} {}",
                style("✓").green().bold(),
                component.display_name()
            );
            Ok(())
        }
        Err(e) => {
            println!(
                "{} {} - {}",
                style("✗").red().bold(),
                component.display_name(),
                e
            );
            Err(e)
        }
    }
}
