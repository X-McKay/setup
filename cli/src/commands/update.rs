use anyhow::Result;
use clap::Args;
use console::style;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::time::Duration;

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

    // Setup progress display
    let total = components.len();
    let mp = MultiProgress::new();

    // Overall progress bar
    let overall_style = ProgressStyle::default_bar()
        .template("{prefix:.bold.dim} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
        .unwrap()
        .progress_chars("━━─");

    let overall_pb = mp.add(ProgressBar::new(total as u64));
    overall_pb.set_style(overall_style);
    overall_pb.set_prefix("Updating");

    // Track results for summary
    let mut successes: Vec<&str> = Vec::new();
    let mut failures: Vec<(&str, String)> = Vec::new();

    // Update each component
    for (idx, component) in components.iter().enumerate() {
        overall_pb.set_message(format!("{}", component.display_name()));

        match update_component_with_progress(&mp, component) {
            Ok(_) => {
                successes.push(component.display_name());
                mp.println(format!(
                    "{} {} {}",
                    style("✓").green().bold(),
                    style(component.display_name()).green(),
                    style(format!("({}/{})", idx + 1, total)).dim()
                ))?;
            }
            Err(e) => {
                let err_msg = e.to_string();
                failures.push((component.display_name(), err_msg.clone()));
                mp.println(format!(
                    "{} {} {} - {}",
                    style("✗").red().bold(),
                    style(component.display_name()).red(),
                    style(format!("({}/{})", idx + 1, total)).dim(),
                    style(&err_msg).dim()
                ))?;
            }
        }

        overall_pb.inc(1);
    }

    overall_pb.finish_and_clear();

    // Print summary
    println!("\n{}", style("─".repeat(50)).dim());
    println!("{}", style(" Update Summary").bold());
    println!("{}\n", style("─".repeat(50)).dim());

    if !successes.is_empty() {
        println!(
            "{} {} component(s) updated successfully",
            style("✓").green().bold(),
            successes.len()
        );
    }

    if !failures.is_empty() {
        println!(
            "{} {} component(s) failed:",
            style("✗").red().bold(),
            failures.len()
        );
        for (name, err) in &failures {
            println!("  {} {} - {}", style("•").dim(), name, style(err).dim());
        }
    }

    if failures.is_empty() {
        println!(
            "\n{}",
            style("All components updated successfully!").green().bold()
        );
    } else {
        println!(
            "\n{}",
            style("Some components failed to update.").yellow()
        );
    }

    Ok(())
}

fn update_component_with_progress(mp: &MultiProgress, component: &UpdateComponent) -> Result<()> {
    let spinner_style = ProgressStyle::default_spinner()
        .template("{spinner:.cyan} {msg}")
        .unwrap()
        .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏");

    let pb = mp.add(ProgressBar::new_spinner());
    pb.set_style(spinner_style);
    pb.set_message(format!(
        "{}...",
        style(component.display_name()).cyan()
    ));
    pb.enable_steady_tick(Duration::from_millis(80));

    let result = match component {
        UpdateComponent::System => packages::update_system(),
        UpdateComponent::Mise => packages::update_mise(),
        UpdateComponent::Rust => packages::update_rust_tools(),
        UpdateComponent::Dotfiles => packages::sync_dotfiles(),
    };

    pb.finish_and_clear();
    result
}
