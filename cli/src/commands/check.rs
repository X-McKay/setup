use anyhow::Result;
use clap::Args;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

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

    let mut tools_result: Option<(usize, usize)> = None;
    let mut dotfiles_result: Option<(usize, usize)> = None;

    match category {
        CheckCategory::Tools | CheckCategory::All => {
            tools_result = Some(check_tools(args.verbose)?);
        }
        _ => {}
    }

    match category {
        CheckCategory::Dotfiles | CheckCategory::All => {
            dotfiles_result = Some(check_dotfiles()?);
        }
        _ => {}
    }

    match category {
        CheckCategory::System | CheckCategory::All => {
            check_system()?;
        }
        _ => {}
    }

    // Print summary
    println!("\n{}", style("─".repeat(40)).dim());
    println!("{}", style(" Summary").bold());
    println!("{}", style("─".repeat(40)).dim());

    if let Some((installed, total)) = tools_result {
        let status_icon = if installed == total {
            style("✓").green().bold()
        } else if installed > 0 {
            style("◐").yellow().bold()
        } else {
            style("✗").red().bold()
        };
        println!(
            "{} Tools: {}/{} installed",
            status_icon,
            style(installed).bold(),
            total
        );
    }

    if let Some((present, total)) = dotfiles_result {
        let status_icon = if present == total {
            style("✓").green().bold()
        } else if present > 0 {
            style("◐").yellow().bold()
        } else {
            style("✗").red().bold()
        };
        println!(
            "{} Dotfiles: {}/{} present",
            status_icon,
            style(present).bold(),
            total
        );
    }

    // Overall health status
    let all_tools_ok = tools_result.map_or(true, |(i, t)| i == t);
    let all_dotfiles_ok = dotfiles_result.map_or(true, |(p, t)| p == t);

    if all_tools_ok && all_dotfiles_ok {
        println!(
            "\n{}",
            style("System is healthy!").green().bold()
        );
    } else {
        println!(
            "\n{}",
            style("Some components are missing. Run 'setup install' to fix.").yellow()
        );
    }

    Ok(())
}

fn check_tools(verbose: bool) -> Result<(usize, usize)> {
    println!("\n{}", style("Tools").bold());

    let tools = [
        ("git", "Git version control"),
        ("docker", "Docker containers"),
        ("mise", "Version manager"),
        ("lazygit", "Git TUI"),
        ("rg", "Ripgrep search"),
        ("bat|batcat", "Better cat"),      // Ubuntu installs as batcat
        ("eza", "Better ls"),
        ("fd|fdfind", "Better find"),      // Ubuntu installs as fdfind
        ("fzf", "Fuzzy finder"),
        ("delta", "Better diff"),
    ];

    let pb = ProgressBar::new(tools.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("  {bar:20.cyan/blue} {pos}/{len} checking...")
            .unwrap()
            .progress_chars("━━─")
    );

    let mut installed_count = 0;
    let mut results: Vec<(bool, &str, &str, Option<String>)> = Vec::new();

    for (cmd, desc) in &tools {
        // Handle alternative command names (e.g., "bat|batcat")
        let alternatives: Vec<&str> = cmd.split('|').collect();
        let display_name = alternatives[0]; // Use first name for display

        let mut found_status = None;
        for alt in &alternatives {
            let status = health::check_command(alt);
            if status.installed {
                found_status = Some(status);
                break;
            }
        }

        let (is_installed, version) = match found_status {
            Some(status) => (true, status.version),
            None => (false, None),
        };

        if is_installed {
            installed_count += 1;
        }
        results.push((is_installed, display_name, desc, version));
        pb.inc(1);
    }

    pb.finish_and_clear();

    // Print results
    for (is_installed, cmd, desc, version) in results {
        let icon = if is_installed {
            style("✓").green()
        } else {
            style("○").dim()
        };

        if verbose {
            let ver = version.unwrap_or_else(|| "not installed".to_string());
            println!("  {} {} - {} ({})", icon, cmd, desc, style(ver).dim());
        } else {
            println!("  {} {}", icon, cmd);
        }
    }

    Ok((installed_count, tools.len()))
}

fn check_dotfiles() -> Result<(usize, usize)> {
    println!("\n{}", style("Dotfiles").bold());

    let dotfiles = [
        ("~/.bashrc", "Bash config"),
        ("~/.tmux.conf", "Tmux config"),
        ("~/.gitconfig", "Git config"),
        ("~/.config/ghostty/config", "Ghostty config"),
    ];

    let pb = ProgressBar::new(dotfiles.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("  {bar:20.cyan/blue} {pos}/{len} checking...")
            .unwrap()
            .progress_chars("━━─")
    );

    let mut present_count = 0;
    let mut results: Vec<(bool, &str, &str)> = Vec::new();

    for (path, desc) in &dotfiles {
        let expanded = shellexpand::tilde(path);
        let exists = std::path::Path::new(expanded.as_ref()).exists();
        if exists {
            present_count += 1;
        }
        results.push((exists, path, desc));
        pb.inc(1);
    }

    pb.finish_and_clear();

    // Print results
    for (exists, path, desc) in results {
        let icon = if exists {
            style("✓").green()
        } else {
            style("○").dim()
        };
        println!("  {} {} - {}", icon, path, desc);
    }

    Ok((present_count, dotfiles.len()))
}

fn check_system() -> Result<()> {
    println!("\n{}", style("System").bold());

    let spinner_style = ProgressStyle::default_spinner()
        .template("  {spinner:.cyan} {msg}")
        .unwrap()
        .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏");

    let pb = ProgressBar::new_spinner();
    pb.set_style(spinner_style);
    pb.set_message("Gathering system info...");
    pb.enable_steady_tick(Duration::from_millis(80));

    let info = health::get_system_info()?;

    pb.finish_and_clear();

    println!("  {} OS: {}", style("•").cyan(), info.os);
    println!("  {} Shell: {}", style("•").cyan(), info.shell);
    println!("  {} Terminal: {}", style("•").cyan(), info.terminal);
    println!(
        "  {} Disk: {} free",
        style("•").cyan(),
        info.disk_free
    );
    println!(
        "  {} Memory: {} / {}",
        style("•").cyan(),
        info.mem_used,
        info.mem_total
    );

    Ok(())
}
