use anyhow::Result;
use std::fs;
use std::path::PathBuf;

/// Returns a list of (name, source_path, target_path) for all managed dotfiles
pub fn get_managed_dotfiles() -> Vec<(String, PathBuf, PathBuf)> {
    let home = dirs::home_dir().expect("Could not find home directory");
    let repo_dotfiles = get_repo_dotfiles_dir();

    vec![
        (
            "bashrc".to_string(),
            repo_dotfiles.join("bashrc"),
            home.join(".bashrc"),
        ),
        (
            "bash_profile".to_string(),
            repo_dotfiles.join("bash_profile"),
            home.join(".bash_profile"),
        ),
        (
            "aliases".to_string(),
            repo_dotfiles.join("aliases"),
            home.join(".aliases"),
        ),
        (
            "exports".to_string(),
            repo_dotfiles.join("exports"),
            home.join(".exports"),
        ),
        (
            "util".to_string(),
            repo_dotfiles.join("util"),
            home.join(".util"),
        ),
        (
            "tmux.conf".to_string(),
            repo_dotfiles.join("tmux.conf"),
            home.join(".tmux.conf"),
        ),
        (
            "gitconfig".to_string(),
            repo_dotfiles.join("gitconfig"),
            home.join(".gitconfig"),
        ),
        (
            "tool-versions".to_string(),
            repo_dotfiles.join("tool-versions"),
            home.join(".tool-versions"),
        ),
        (
            "starship.toml".to_string(),
            repo_dotfiles.join("starship.toml"),
            home.join(".config").join("starship.toml"),
        ),
        (
            "ghostty/config".to_string(),
            repo_dotfiles.join("ghostty").join("config"),
            home.join(".config").join("ghostty").join("config"),
        ),
        (
            "lazygit/config.yml".to_string(),
            repo_dotfiles.join("lazygit").join("config.yml"),
            home.join(".config").join("lazygit").join("config.yml"),
        ),
    ]
}

fn get_repo_dotfiles_dir() -> PathBuf {
    // Try to find the repo directory
    // First check if we're running from within the repo
    let exe_path = std::env::current_exe().ok();

    if let Some(path) = exe_path {
        // Check if we're in the cli/target directory
        if let Some(repo_root) = path
            .ancestors()
            .find(|p| p.join("bootstrap").join("dotfiles").exists())
        {
            return repo_root.join("bootstrap").join("dotfiles");
        }
    }

    // Fall back to checking common locations
    let home = dirs::home_dir().expect("Could not find home directory");
    let candidates = [
        home.join("git").join("setup").join("bootstrap").join("dotfiles"),
        home.join(".setup").join("bootstrap").join("dotfiles"),
        PathBuf::from("/home/al/git/setup/bootstrap/dotfiles"),
    ];

    for candidate in &candidates {
        if candidate.exists() {
            return candidate.clone();
        }
    }

    // Default to the expected location
    home.join("git").join("setup").join("bootstrap").join("dotfiles")
}

pub fn copy_dotfile(source: &PathBuf, target: &PathBuf) -> Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::copy(source, target)?;
    Ok(())
}

pub fn diff_files(source: &PathBuf, target: &PathBuf) -> Result<Option<String>> {
    if !target.exists() {
        return Ok(Some(format!("Target does not exist: {}", target.display())));
    }

    let source_content = fs::read_to_string(source)?;
    let target_content = fs::read_to_string(target)?;

    if source_content == target_content {
        return Ok(None);
    }

    // Simple line-by-line diff
    let mut diff_output = String::new();
    for (i, (s, t)) in source_content.lines().zip(target_content.lines()).enumerate() {
        if s != t {
            diff_output.push_str(&format!("Line {}: \n  repo:  {}\n  home:  {}\n", i + 1, s, t));
        }
    }

    if diff_output.is_empty() && source_content.lines().count() != target_content.lines().count() {
        diff_output = "Files have different number of lines".to_string();
    }

    Ok(Some(diff_output))
}

pub fn files_match(source: &PathBuf, target: &PathBuf) -> Result<bool> {
    if !source.exists() || !target.exists() {
        return Ok(false);
    }

    let source_content = fs::read_to_string(source)?;
    let target_content = fs::read_to_string(target)?;

    Ok(source_content == target_content)
}

pub fn create_backup() -> Result<PathBuf> {
    let home = dirs::home_dir().expect("Could not find home directory");
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let backup_dir = home.join(".dotfiles_backup").join(timestamp.to_string());

    fs::create_dir_all(&backup_dir)?;

    let dotfiles = get_managed_dotfiles();
    for (name, _, target) in dotfiles {
        if target.exists() {
            let backup_path = backup_dir.join(&name);
            if let Some(parent) = backup_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&target, &backup_path)?;
        }
    }

    Ok(backup_dir)
}
