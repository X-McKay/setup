use anyhow::{Context, Result};
use clap::Args;
use console::style;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::time::Duration;

use crate::components::registry::Registry;
use crate::manifest::{intent, loader, resolver};
use crate::ui::prompts;

#[derive(Args)]
pub struct InstallArgs {
    /// Component ids to install (positional)
    pub components: Vec<String>,

    /// Profile to install (composable, may be given multiple times)
    #[arg(long = "profile")]
    pub profiles: Vec<String>,

    /// Install every component in the registry
    #[arg(long)]
    pub all: bool,

    /// Preview the plan without installing anything
    #[arg(long)]
    pub dry_run: bool,

    /// Run verify() after each successful install
    #[arg(long)]
    pub verify: bool,

    /// Continue past failures; print summary at end
    #[arg(long = "keep-going", conflicts_with = "rollback_on_failure")]
    pub keep_going: bool,

    /// On mid-run failure, uninstall components installed in this run
    #[arg(long = "rollback-on-failure", conflicts_with = "keep_going")]
    pub rollback_on_failure: bool,

    /// Skip confirmation prompts
    #[arg(short = 'y', long)]
    pub yes: bool,
}

pub fn run(args: InstallArgs) -> Result<()> {
    validate_flag_combination(&args)?;

    let manifest = loader::load().context("loading manifest")?;
    let registry = Registry::build();
    registry
        .validate_against(&manifest)
        .context("manifest/registry drift at install time")?;

    let plan = if args.all {
        let all_ids: Vec<String> = manifest.components.iter().map(|c| c.id.clone()).collect();
        resolver::resolve(&manifest, &[], &all_ids)?
    } else {
        resolver::resolve(&manifest, &args.profiles, &args.components)?
    };

    if plan.ordered.is_empty() {
        println!("{}", style("No components selected.").yellow());
        return Ok(());
    }

    if !plan.auto_pulled.is_empty() {
        println!(
            "{} auto-pulled deps: {}",
            style("ℹ").cyan(),
            plan.auto_pulled.iter().cloned().collect::<Vec<_>>().join(", ")
        );
    }

    if args.dry_run {
        print_dry_run(&registry, &plan.ordered)?;
        return Ok(());
    }

    if !args.yes {
        let display_names: Vec<String> = plan
            .ordered
            .iter()
            .map(|id| spec_display(&manifest, id))
            .collect();
        let display_refs: Vec<&str> = display_names.iter().map(|name| name.as_str()).collect();
        if !prompts::confirm_install(&display_refs)? {
            println!("{}", style("Installation cancelled.").yellow());
            return Ok(());
        }
    }

    let installed_this_run = run_plan(
        &registry,
        &plan.ordered,
        args.keep_going,
        args.rollback_on_failure,
        args.verify,
    )?;

    update_intent_on_success(&args, &installed_this_run, &plan.ordered)?;

    Ok(())
}

fn validate_flag_combination(args: &InstallArgs) -> Result<()> {
    if args.all && (!args.profiles.is_empty() || !args.components.is_empty()) {
        anyhow::bail!("--all is mutually exclusive with --profile and positional components");
    }
    Ok(())
}

fn spec_display(manifest: &crate::manifest::schema::Manifest, id: &str) -> String {
    manifest
        .components
        .iter()
        .find(|component| component.id == id)
        .map(|component| component.display_name.clone())
        .unwrap_or_else(|| id.to_string())
}

fn print_dry_run(registry: &Registry, ordered: &[String]) -> Result<()> {
    println!("{}", style("Dry-run plan:").bold());
    for id in ordered {
        let component = registry.get(id)?;
        println!("  {}", style(format!("• {}", id)).cyan());
        for line in component.dry_run()? {
            println!("      {}", style(line).dim());
        }
    }
    Ok(())
}

fn run_plan(
    registry: &Registry,
    ordered: &[String],
    keep_going: bool,
    rollback_on_failure: bool,
    verify: bool,
) -> Result<Vec<String>> {
    let mp = MultiProgress::new();
    let mut installed: Vec<String> = Vec::new();
    let mut failures: Vec<(String, String)> = Vec::new();
    let mut had_failure = false;
    let mut stopped_at: Option<usize> = None;

    for (idx, id) in ordered.iter().enumerate() {
        let component = registry.get(id)?;
        if component.is_installed().unwrap_or(false) {
            mp.println(format!(
                "{} {} (already installed)",
                style("✓").green().bold(),
                style(id).green()
            ))?;
            continue;
        }

        let spinner = mp.add(ProgressBar::new_spinner());
        spinner.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.cyan} {msg}")
                .unwrap()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
        );
        spinner.set_message(format!("{}...", id));
        spinner.enable_steady_tick(Duration::from_millis(80));

        let outcome = component.install();
        spinner.finish_and_clear();

        match outcome {
            Ok(()) => {
                installed.push(id.clone());
                mp.println(format!("{} {}", style("✓").green().bold(), style(id).green()))?;
                if verify {
                    match component.verify() {
                        Ok(()) => {}
                        Err(err) => {
                            mp.println(format!(
                                "{} verify {}: {}",
                                style("!").yellow().bold(),
                                id,
                                err
                            ))?;
                        }
                    }
                }
            }
            Err(err) => {
                had_failure = true;
                failures.push((id.clone(), err.to_string()));
                mp.println(format!(
                    "{} {} — {}",
                    style("✗").red().bold(),
                    style(id).red(),
                    style(&err).dim()
                ))?;
                if !keep_going {
                    stopped_at = Some(idx);
                    break;
                }
            }
        }
    }

    if had_failure && rollback_on_failure && !installed.is_empty() {
        println!(
            "{} rolling back {} installed component(s)",
            style("↺").yellow().bold(),
            installed.len()
        );
        for id in installed.iter().rev() {
            let component = registry.get(id)?;
            if !component.is_reversible() {
                println!("  {} {} skipped (not reversible)", style("~").yellow(), id);
                continue;
            }
            if let Err(err) = component.uninstall() {
                println!(
                    "  {} {} rollback failed: {}",
                    style("!").red().bold(),
                    id,
                    err
                );
            } else {
                println!("  {} {} rolled back", style("↺").yellow(), id);
            }
        }
    }

    if !failures.is_empty() {
        println!("\n{}", style("Installation summary").bold());
        for (id, err) in &failures {
            println!("  {} {} — {}", style("✗").red(), id, style(err).dim());
        }
        if let Some(idx) = stopped_at {
            let pending = ordered[idx + 1..].join(", ");
            if !pending.is_empty() {
                println!("\n{} still pending: {}", style("→").dim(), pending);
            }
        }
    }

    Ok(installed)
}

fn update_intent_on_success(
    args: &InstallArgs,
    installed: &[String],
    planned: &[String],
) -> Result<()> {
    // Write intent only when the user declared intent via --profile and the
    // run actually reached the end of the plan (either cleanly or via --keep-going).
    if args.all || args.rollback_on_failure || args.profiles.is_empty() {
        return Ok(());
    }

    if !args.keep_going && installed.len() < planned.len() {
        let registry = Registry::build();
        for id in planned {
            let component = registry.get(id)?;
            if !component.is_installed().unwrap_or(false) {
                return Ok(());
            }
        }
    }

    let path = intent::default_path().context("no config dir for intent file")?;
    let mut current = intent::read(&path)?;
    intent::union_add(&mut current, &args.profiles);
    intent::write(&path, &current)?;
    println!(
        "{} recorded intent: active_profiles = {:?}",
        style("ℹ").cyan(),
        current.active_profiles
    );
    Ok(())
}
