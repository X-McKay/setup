//! `setup list` - print the component catalog.

use anyhow::{Context, Result};
use clap::Args;
use console::style;
use std::collections::BTreeSet;

use crate::manifest::loader;

#[derive(Args)]
pub struct ListArgs {
    /// Show only components in this profile
    #[arg(long)]
    pub profile: Option<String>,

    /// Show only components with this tag
    #[arg(long)]
    pub tag: Option<String>,
}

pub fn run(args: ListArgs) -> Result<()> {
    let manifest = loader::load().context("loading manifest")?;

    let in_profile: Option<BTreeSet<String>> = if let Some(p) = &args.profile {
        let set = crate::manifest::resolver::expand_selection(&manifest, std::slice::from_ref(p), &[])?;
        Some(set)
    } else {
        None
    };

    println!("{}", style("Components:").bold());
    for c in &manifest.components {
        if let Some(ref set) = in_profile {
            if !set.contains(&c.id) {
                continue;
            }
        }
        if let Some(ref t) = args.tag {
            if !c.tags.contains(t) {
                continue;
            }
        }

        println!(
            "  {} {} {}",
            style(&c.id).cyan(),
            style(format!("({})", c.display_name)).dim(),
            if c.tags.is_empty() {
                String::new()
            } else {
                style(format!("[{}]", c.tags.join(","))).dim().to_string()
            }
        );
    }

    Ok(())
}
