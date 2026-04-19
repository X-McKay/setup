//! `setup uninstall` - remove one or more components.

use anyhow::{Context, Result};
use clap::Args;
use console::style;
use std::collections::BTreeSet;

use crate::components::registry::Registry;
use crate::manifest::loader;

#[derive(Args)]
pub struct UninstallArgs {
    /// Component id(s) to uninstall
    #[arg(required = true)]
    pub components: Vec<String>,

    /// Skip the is_reversible refusal and dependency-check refusal
    #[arg(long)]
    pub force: bool,

    /// Also uninstall any components that depend on the target(s)
    #[arg(long)]
    pub cascade: bool,

    /// Skip confirmation prompts
    #[arg(short = 'y', long)]
    pub yes: bool,
}

pub fn run(args: UninstallArgs) -> Result<()> {
    let manifest = loader::load().context("loading manifest")?;
    let registry = Registry::build();
    registry.validate_against(&manifest)?;

    let targets = resolve_targets(&manifest, &registry, &args.components, args.cascade)?;

    if !args.yes {
        println!(
            "Will uninstall: {}",
            targets.iter().cloned().collect::<Vec<_>>().join(", ")
        );
        let confirmed = inquire::Confirm::new("Proceed?")
            .with_default(false)
            .prompt()?;
        if !confirmed {
            println!("{}", style("Cancelled.").yellow());
            return Ok(());
        }
    }

    let ordered = reverse_topo(&manifest, &targets)?;
    for id in ordered {
        let c = registry.get(&id)?;

        if !c.is_installed().unwrap_or(false) {
            println!("{} {} not installed", style("○").dim(), id);
            continue;
        }

        if !c.is_reversible() && !args.force {
            println!(
                "{} {} not reversible - use --force to confirm destructive removal",
                style("✗").red().bold(),
                id
            );
            continue;
        }

        if !args.cascade && !args.force {
            if let Some(blockers) = find_dependents_that_are_installed(&manifest, &registry, &id)? {
                println!(
                    "{} {} has installed dependents: {}. Use --cascade or --force.",
                    style("✗").red().bold(),
                    id,
                    blockers.join(", ")
                );
                continue;
            }
        }

        match c.uninstall() {
            Ok(()) => println!("{} {} uninstalled", style("✓").green().bold(), id),
            Err(e) => println!(
                "{} {} failed: {}",
                style("✗").red().bold(),
                id,
                style(e).dim()
            ),
        }
    }

    Ok(())
}

fn resolve_targets(
    manifest: &crate::manifest::schema::Manifest,
    _registry: &Registry,
    explicit: &[String],
    cascade: bool,
) -> Result<BTreeSet<String>> {
    let mut out: BTreeSet<String> = explicit.iter().cloned().collect();

    for id in explicit {
        if !manifest.components.iter().any(|c| c.id == *id) {
            anyhow::bail!("unknown component: {}", id);
        }
    }

    if cascade {
        let mut frontier: Vec<String> = explicit.to_vec();
        while let Some(id) = frontier.pop() {
            for c in &manifest.components {
                if c.depends_on.iter().any(|d| d == &id) && out.insert(c.id.clone()) {
                    frontier.push(c.id.clone());
                }
            }
        }
    }

    Ok(out)
}

fn reverse_topo(
    manifest: &crate::manifest::schema::Manifest,
    targets: &BTreeSet<String>,
) -> Result<Vec<String>> {
    let mut ordered = crate::manifest::resolver::topo_sort(manifest, targets)?;
    ordered.reverse();
    Ok(ordered)
}

fn find_dependents_that_are_installed(
    manifest: &crate::manifest::schema::Manifest,
    registry: &Registry,
    id: &str,
) -> Result<Option<Vec<String>>> {
    let mut installed_dependents = Vec::new();

    for c in &manifest.components {
        if c.depends_on.iter().any(|d| d == id) {
            let comp = registry.get(&c.id)?;
            if comp.is_installed().unwrap_or(false) {
                installed_dependents.push(c.id.clone());
            }
        }
    }

    if installed_dependents.is_empty() {
        Ok(None)
    } else {
        Ok(Some(installed_dependents))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::schema::{ComponentSpec, Manifest, ProfileSpec};
    use std::collections::BTreeMap;

    fn mk() -> Manifest {
        Manifest {
            components: vec![
                ComponentSpec {
                    id: "apt".into(),
                    display_name: "APT".into(),
                    ..Default::default()
                },
                ComponentSpec {
                    id: "docker".into(),
                    display_name: "Docker".into(),
                    depends_on: vec!["apt".into()],
                    ..Default::default()
                },
            ],
            profiles: BTreeMap::<String, ProfileSpec>::new(),
        }
    }

    #[test]
    fn cascade_pulls_in_dependents() {
        let m = mk();
        let reg = Registry::build();
        let set = resolve_targets(&m, &reg, &["apt".into()], true).unwrap();
        assert!(set.contains("apt"));
        assert!(set.contains("docker"));
    }

    #[test]
    fn no_cascade_keeps_target_only() {
        let m = mk();
        let reg = Registry::build();
        let set = resolve_targets(&m, &reg, &["apt".into()], false).unwrap();
        assert_eq!(set.len(), 1);
        assert!(set.contains("apt"));
    }

    #[test]
    fn unknown_component_errors() {
        let m = mk();
        let reg = Registry::build();
        assert!(resolve_targets(&m, &reg, &["ghost".into()], false).is_err());
    }
}
