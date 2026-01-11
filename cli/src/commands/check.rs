use anyhow::Result;
use clap::Args;
use console::style;

use crate::system::health;

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

pub fn run(args: CheckArgs) -> Result<()> {
    let category = args.category.unwrap_or(CheckCategory::All);

    println!("{}", style("System Health Check").cyan().bold());
    println!("{}", style("═".repeat(40)).dim());

    match category {
        CheckCategory::Tools | CheckCategory::All => {
            check_tools(args.verbose)?;
        }
        _ => {}
    }

    match category {
        CheckCategory::Dotfiles | CheckCategory::All => {
            check_dotfiles()?;
        }
        _ => {}
    }

    match category {
        CheckCategory::System | CheckCategory::All => {
            check_system()?;
        }
        _ => {}
    }

    Ok(())
}

fn check_tools(verbose: bool) -> Result<()> {
    println!("\n{}", style("Tools").bold());

    let tools = [
        ("git", "Git version control"),
        ("docker", "Docker containers"),
        ("mise", "Version manager"),
        ("starship", "Shell prompt"),
        ("zoxide", "Directory jumper"),
        ("lazygit", "Git TUI"),
        ("rg", "Ripgrep search"),
        ("bat", "Better cat"),
        ("eza", "Better ls"),
        ("fd", "Better find"),
        ("fzf", "Fuzzy finder"),
        ("delta", "Better diff"),
    ];

    for (cmd, desc) in &tools {
        let status = health::check_command(cmd);
        let icon = if status.installed {
            style("✓").green()
        } else {
            style("○").dim()
        };

        if verbose {
            let version = status.version.unwrap_or_else(|| "not installed".to_string());
            println!("  {} {} - {} ({})", icon, cmd, desc, style(version).dim());
        } else {
            println!("  {} {}", icon, cmd);
        }
    }

    Ok(())
}

fn check_dotfiles() -> Result<()> {
    println!("\n{}", style("Dotfiles").bold());

    let dotfiles = [
        ("~/.bashrc", "Bash config"),
        ("~/.tmux.conf", "Tmux config"),
        ("~/.gitconfig", "Git config"),
        ("~/.config/ghostty/config", "Ghostty config"),
        ("~/.config/starship.toml", "Starship config"),
    ];

    for (path, desc) in &dotfiles {
        let expanded = shellexpand::tilde(path);
        let exists = std::path::Path::new(expanded.as_ref()).exists();
        let icon = if exists {
            style("✓").green()
        } else {
            style("○").dim()
        };
        println!("  {} {} - {}", icon, path, desc);
    }

    Ok(())
}

fn check_system() -> Result<()> {
    println!("\n{}", style("System").bold());

    let info = health::get_system_info()?;

    println!("  {} OS: {}", style("•").blue(), info.os);
    println!("  {} Shell: {}", style("•").blue(), info.shell);
    println!("  {} Terminal: {}", style("•").blue(), info.terminal);
    println!(
        "  {} Disk: {} free",
        style("•").blue(),
        info.disk_free
    );
    println!(
        "  {} Memory: {} / {}",
        style("•").blue(),
        info.mem_used,
        info.mem_total
    );

    Ok(())
}
