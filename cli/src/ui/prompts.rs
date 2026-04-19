use anyhow::Result;
use inquire::{Confirm, MultiSelect, Select};

use crate::commands::dotfiles::DotfilesAction;
use crate::commands::update::UpdateComponent;

pub fn confirm_install(components: &[&str]) -> Result<bool> {
    let msg = format!("Install {} component(s)?", components.len());
    Ok(Confirm::new(&msg)
        .with_default(true)
        .with_help_message(&components.join(", "))
        .prompt()?)
}

pub fn select_dotfiles_action() -> Result<DotfilesAction> {
    let options = vec![
        "Sync dotfiles to home",
        "Show diff",
        "List managed dotfiles",
        "Edit a dotfile",
        "Backup current dotfiles",
    ];

    let selection = Select::new("What would you like to do?", options).prompt()?;

    Ok(match selection {
        "Sync dotfiles to home" => DotfilesAction::Sync { force: false },
        "Show diff" => DotfilesAction::Diff,
        "List managed dotfiles" => DotfilesAction::List,
        "Edit a dotfile" => DotfilesAction::Edit { name: None },
        "Backup current dotfiles" => DotfilesAction::Backup,
        _ => unreachable!(),
    })
}

pub fn confirm_overwrite(name: &str) -> Result<bool> {
    Ok(Confirm::new(&format!("Overwrite existing {}?", name))
        .with_default(false)
        .prompt()?)
}

pub fn select_dotfile_to_edit() -> Result<String> {
    let options = vec![
        "bashrc",
        "bash_profile",
        "aliases",
        "exports",
        "tmux.conf",
        "gitconfig",
        "ghostty/config",
    ];

    Ok(Select::new("Select dotfile to edit:", options)
        .prompt()?
        .to_string())
}

pub fn select_update_components() -> Result<Vec<UpdateComponent>> {
    // First, ask if user wants to update all
    let update_all = Select::new(
        "How would you like to update?",
        vec!["Update All (recommended)", "Select individual components"],
    )
    .with_help_message("Update All refreshes system packages, mise, rust tools, and dotfiles")
    .prompt()?;

    if update_all == "Update All (recommended)" {
        return Ok(UpdateComponent::all());
    }

    let options = vec![
        ("System Packages", UpdateComponent::System),
        ("Mise Runtimes", UpdateComponent::Mise),
        ("Rust Tools", UpdateComponent::Rust),
        ("Dotfiles", UpdateComponent::Dotfiles),
    ];

    let labels: Vec<&str> = options.iter().map(|(l, _)| *l).collect();

    let selected = MultiSelect::new("Select components to update:", labels)
        .with_help_message("Space to select, Enter to confirm")
        .prompt()?;

    let components = options
        .into_iter()
        .filter(|(l, _)| selected.contains(l))
        .map(|(_, c)| c)
        .collect();

    Ok(components)
}

pub fn confirm_update(components: &[&str]) -> Result<bool> {
    let msg = format!("Update {} component(s)?", components.len());
    Ok(Confirm::new(&msg)
        .with_default(true)
        .with_help_message(&components.join(", "))
        .prompt()?)
}
