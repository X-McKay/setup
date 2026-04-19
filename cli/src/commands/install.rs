use anyhow::Result;
use clap::{Args, ValueEnum};
use console::style;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::time::Duration;

use crate::ui::prompts;

#[derive(Args)]
pub struct InstallArgs {
    /// Component to install (if not specified, shows interactive menu)
    #[arg(value_enum)]
    pub component: Option<Component>,

    /// Install all components
    #[arg(short, long)]
    pub all: bool,

    /// Skip confirmation prompts
    #[arg(short = 'y', long)]
    pub yes: bool,
}

#[derive(Clone, ValueEnum, Debug, PartialEq)]
pub enum Component {
    /// Basic apt packages (curl, git, build-essential, etc.)
    Apt,
    /// Extra CLI tools (ripgrep, bat, fd, etc.)
    Tools,
    /// Mise version manager
    Mise,
    /// Docker and Docker Compose
    Docker,
    /// System monitoring tools
    Monitoring,
    /// Backup utilities
    Backup,
    /// Lazygit terminal UI
    Lazygit,
    /// Just task runner
    Just,
    /// Glow markdown renderer
    Glow,
    /// Bottom system monitor
    Bottom,
    /// GitHub CLI
    Gh,
    /// Hyperfine command benchmarking
    Hyperfine,
    /// jq JSON processor
    Jq,
    /// yq YAML processor
    Yq,
    /// tldr simplified man pages
    Tldr,
    /// Chromium web browser
    Chromium,
    /// Discord chat client
    Discord,
    /// Obsidian note-taking app
    Obsidian,
    /// Spotify music player
    Spotify,
    /// VLC media player
    Vlc,
    /// Ghostty terminal emulator
    Ghostty,
    /// Claude Code AI coding assistant
    ClaudeCode,
    /// Neovim editor with sensible defaults
    Neovim,
    /// Tmux plugin manager
    Tpm,
    /// Generate SSH keys
    SshKeys,
    /// Setup GPG keys
    Gpg,
}

impl Component {
    pub fn all() -> Vec<Component> {
        vec![
            Component::Apt,
            Component::Tools,
            Component::Mise,
            Component::Docker,
            Component::Lazygit,
            Component::Just,
            Component::Glow,
            Component::Bottom,
            Component::Gh,
            Component::Hyperfine,
            Component::Jq,
            Component::Yq,
            Component::Tldr,
            Component::Chromium,
            Component::Discord,
            Component::Obsidian,
            Component::Spotify,
            Component::Vlc,
            Component::Ghostty,
            Component::ClaudeCode,
            Component::Neovim,
            Component::Tpm,
            Component::Monitoring,
            Component::Backup,
            // Note: SshKeys and Gpg are not in --all as they require user input
        ]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Component::Apt => "Basic APT Packages",
            Component::Tools => "Extra CLI Tools",
            Component::Mise => "Mise Version Manager",
            Component::Docker => "Docker",
            Component::Monitoring => "Monitoring Tools",
            Component::Backup => "Backup Utilities",
            Component::Lazygit => "Lazygit",
            Component::Just => "Just Task Runner",
            Component::Glow => "Glow Markdown Renderer",
            Component::Bottom => "Bottom System Monitor",
            Component::Gh => "GitHub CLI",
            Component::Hyperfine => "Hyperfine Benchmarking",
            Component::Jq => "jq JSON Processor",
            Component::Yq => "yq YAML Processor",
            Component::Tldr => "tldr Man Pages",
            Component::Chromium => "Chromium Browser",
            Component::Discord => "Discord",
            Component::Obsidian => "Obsidian",
            Component::Spotify => "Spotify",
            Component::Vlc => "VLC Media Player",
            Component::Ghostty => "Ghostty Terminal",
            Component::ClaudeCode => "Claude Code",
            Component::Neovim => "Neovim Editor",
            Component::Tpm => "Tmux Plugin Manager",
            Component::SshKeys => "SSH Key Generation",
            Component::Gpg => "GPG Key Setup",
        }
    }
}

pub fn run(args: InstallArgs) -> Result<()> {
    let components = if args.all {
        Component::all()
    } else if let Some(component) = args.component {
        vec![component]
    } else {
        // Interactive selection
        prompts::select_components()?
    };

    if components.is_empty() {
        println!("{}", style("No components selected.").yellow());
        return Ok(());
    }

    // Confirm installation
    if !args.yes {
        let names: Vec<_> = components.iter().map(|c| c.display_name()).collect();
        if !prompts::confirm_install(&names)? {
            println!("{}", style("Installation cancelled.").yellow());
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
    overall_pb.set_prefix("Installing");

    // Track results for summary
    let mut successes: Vec<&str> = Vec::new();
    let mut failures: Vec<(&str, String)> = Vec::new();

    // Install each component
    for (idx, component) in components.iter().enumerate() {
        overall_pb.set_message(format!("{}", component.display_name()));

        match install_component_with_progress(&mp, component) {
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
    println!("{}", style(" Installation Summary").bold());
    println!("{}\n", style("─".repeat(50)).dim());

    if !successes.is_empty() {
        println!(
            "{} {} component(s) installed successfully",
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
            style("All components installed successfully!").green().bold()
        );
        Ok(())
    } else {
        println!(
            "\n{}",
            style("Some components failed to install.").yellow()
        );
        // Return Ok to avoid double error message, failures are shown in summary
        Ok(())
    }
}

fn install_component_with_progress(mp: &MultiProgress, component: &Component) -> Result<()> {
    let id = match component {
        Component::Apt => "apt",
        Component::Tools => "tools",
        Component::Mise => "mise",
        Component::Docker => "docker",
        Component::Monitoring => "monitoring",
        Component::Backup => "backup",
        Component::Lazygit => "lazygit",
        Component::Just => "just",
        Component::Glow => "glow",
        Component::Bottom => "bottom",
        Component::Gh => "gh",
        Component::Hyperfine => "hyperfine",
        Component::Jq => "jq",
        Component::Yq => "yq",
        Component::Tldr => "tldr",
        Component::Chromium => "chromium",
        Component::Discord => "discord",
        Component::Obsidian => "obsidian",
        Component::Spotify => "spotify",
        Component::Vlc => "vlc",
        Component::Ghostty => "ghostty",
        Component::ClaudeCode => "claude-code",
        Component::Neovim => "neovim",
        Component::Tpm => "tpm",
        Component::SshKeys => "ssh-keys",
        Component::Gpg => "gpg",
    };
    install_via_registry(mp, id)
}

fn install_via_registry(mp: &MultiProgress, id: &str) -> Result<()> {
    use crate::components::registry::Registry;

    let spinner_style = ProgressStyle::default_spinner()
        .template("{spinner:.cyan} {msg}")
        .unwrap()
        .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏");

    let pb = mp.add(ProgressBar::new_spinner());
    pb.set_style(spinner_style);
    pb.set_message(format!("{}...", style(id).cyan()));
    pb.enable_steady_tick(Duration::from_millis(80));

    let registry = Registry::build();
    let component = registry.get(id)?;
    let result = component.install();

    pb.finish_and_clear();
    result
}
