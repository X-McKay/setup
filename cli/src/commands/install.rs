use anyhow::Result;
use clap::{Args, ValueEnum};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};

use crate::system::packages;
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
    /// Starship prompt
    Starship,
    /// Zoxide directory jumper
    Zoxide,
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
}

impl Component {
    pub fn all() -> Vec<Component> {
        vec![
            Component::Apt,
            Component::Tools,
            Component::Mise,
            Component::Docker,
            Component::Starship,
            Component::Zoxide,
            Component::Lazygit,
            Component::Just,
            Component::Glow,
            Component::Bottom,
            Component::Gh,
            Component::Hyperfine,
            Component::Jq,
            Component::Yq,
            Component::Tldr,
            Component::Monitoring,
            Component::Backup,
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
            Component::Starship => "Starship Prompt",
            Component::Zoxide => "Zoxide",
            Component::Lazygit => "Lazygit",
            Component::Just => "Just Task Runner",
            Component::Glow => "Glow Markdown Renderer",
            Component::Bottom => "Bottom System Monitor",
            Component::Gh => "GitHub CLI",
            Component::Hyperfine => "Hyperfine Benchmarking",
            Component::Jq => "jq JSON Processor",
            Component::Yq => "yq YAML Processor",
            Component::Tldr => "tldr Man Pages",
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

    // Install each component
    for component in &components {
        install_component(component)?;
    }

    println!(
        "\n{}",
        style("All components installed successfully!").green().bold()
    );

    Ok(())
}

fn install_component(component: &Component) -> Result<()> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.blue} {msg}")
            .unwrap(),
    );
    pb.set_message(format!("Installing {}...", component.display_name()));
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    let result = match component {
        Component::Apt => packages::install_apt_packages(),
        Component::Tools => packages::install_extra_tools(),
        Component::Mise => packages::install_mise(),
        Component::Docker => packages::install_docker(),
        Component::Monitoring => packages::install_monitoring(),
        Component::Backup => packages::install_backup(),
        Component::Starship => packages::install_starship(),
        Component::Zoxide => packages::install_zoxide(),
        Component::Lazygit => packages::install_lazygit(),
        Component::Just => packages::install_just(),
        Component::Glow => packages::install_glow(),
        Component::Bottom => packages::install_bottom(),
        Component::Gh => packages::install_gh(),
        Component::Hyperfine => packages::install_hyperfine(),
        Component::Jq => packages::install_jq(),
        Component::Yq => packages::install_yq(),
        Component::Tldr => packages::install_tldr(),
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
