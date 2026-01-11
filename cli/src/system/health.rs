use anyhow::Result;
use std::process::Command;

pub struct CommandStatus {
    pub installed: bool,
    pub version: Option<String>,
}

pub fn check_command(cmd: &str) -> CommandStatus {
    let installed = which::which(cmd).is_ok();

    let version = if installed {
        get_version(cmd)
    } else {
        None
    };

    CommandStatus { installed, version }
}

fn get_version(cmd: &str) -> Option<String> {
    // Different commands have different version flags
    let version_args: &[&str] = match cmd {
        "git" => &["--version"],
        "docker" => &["--version"],
        "mise" => &["--version"],
        "starship" => &["--version"],
        "zoxide" => &["--version"],
        "lazygit" => &["--version"],
        "rg" => &["--version"],
        "bat" | "batcat" => &["--version"],
        "eza" => &["--version"],
        "fd" | "fdfind" => &["--version"],
        "fzf" => &["--version"],
        "delta" => &["--version"],
        _ => &["--version"],
    };

    let output = Command::new(cmd).args(version_args).output().ok()?;

    if output.status.success() {
        let version_str = String::from_utf8_lossy(&output.stdout);
        // Extract just the version number from the first line
        let first_line = version_str.lines().next()?;
        Some(extract_version(first_line))
    } else {
        None
    }
}

fn extract_version(line: &str) -> String {
    // Try to extract version number from common formats
    // "git version 2.34.1" -> "2.34.1"
    // "starship 1.18.0" -> "1.18.0"
    // "ripgrep 13.0.0" -> "13.0.0"

    let words: Vec<&str> = line.split_whitespace().collect();

    for word in &words {
        // Check if word looks like a version number
        if word
            .chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
        {
            // Remove any trailing characters like newlines
            return word.trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '.').to_string();
        }
    }

    // Fall back to returning the whole line
    line.trim().to_string()
}

pub struct SystemInfo {
    pub os: String,
    pub shell: String,
    pub terminal: String,
    pub disk_free: String,
    pub mem_total: String,
    pub mem_used: String,
}

pub fn get_system_info() -> Result<SystemInfo> {
    let os = get_os_info();
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "unknown".to_string());
    let terminal = std::env::var("TERM").unwrap_or_else(|_| "unknown".to_string());
    let (disk_free, mem_total, mem_used) = get_resource_info();

    Ok(SystemInfo {
        os,
        shell,
        terminal,
        disk_free,
        mem_total,
        mem_used,
    })
}

fn get_os_info() -> String {
    // Try to read /etc/os-release
    if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            if line.starts_with("PRETTY_NAME=") {
                return line
                    .trim_start_matches("PRETTY_NAME=")
                    .trim_matches('"')
                    .to_string();
            }
        }
    }

    "Linux".to_string()
}

fn get_resource_info() -> (String, String, String) {
    // Get disk free space
    let disk_free = Command::new("df")
        .args(["-h", "/", "--output=avail"])
        .output()
        .ok()
        .and_then(|o| {
            let s = String::from_utf8_lossy(&o.stdout);
            s.lines().nth(1).map(|l| l.trim().to_string())
        })
        .unwrap_or_else(|| "unknown".to_string());

    // Get memory info
    let (mem_total, mem_used) = Command::new("free")
        .args(["-h"])
        .output()
        .ok()
        .and_then(|o| {
            let s = String::from_utf8_lossy(&o.stdout);
            for line in s.lines() {
                if line.starts_with("Mem:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 3 {
                        return Some((parts[1].to_string(), parts[2].to_string()));
                    }
                }
            }
            None
        })
        .unwrap_or_else(|| ("unknown".to_string(), "unknown".to_string()));

    (disk_free, mem_total, mem_used)
}
