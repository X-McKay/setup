//! Persistent user intent - which profiles have been selected on this machine.
//!
//! Distinct from install state, which is always probed from the host. See
//! spec section 4.5 for the intent model.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Intent {
    #[serde(default)]
    pub active_profiles: Vec<String>,
}

/// Default intent file location: `~/.config/setup/active.toml`.
/// Overridable via `SETUP_INTENT` for tests.
pub fn default_path() -> Option<PathBuf> {
    if let Ok(env_path) = std::env::var("SETUP_INTENT") {
        return Some(PathBuf::from(env_path));
    }
    dirs::config_dir().map(|d| d.join("setup").join("active.toml"))
}

/// Read intent from `path`. Missing files are treated as empty intent.
pub fn read(path: &Path) -> Result<Intent> {
    if !path.exists() {
        return Ok(Intent::default());
    }
    let text =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    toml::from_str(&text).with_context(|| format!("parsing {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_file_yields_empty_intent() {
        let path = PathBuf::from("/nonexistent-intent-path.toml");
        let intent = read(&path).unwrap();
        assert!(intent.active_profiles.is_empty());
    }

    #[test]
    fn reads_existing_intent() {
        let dir = std::env::temp_dir().join(format!("setup-intent-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("active.toml");
        std::fs::write(&path, "active_profiles = [\"server\", \"ai-heavy\"]\n").unwrap();
        let intent = read(&path).unwrap();
        assert_eq!(intent.active_profiles, vec!["server", "ai-heavy"]);
    }

    #[test]
    fn rejects_unknown_fields() {
        let dir = std::env::temp_dir().join(format!("setup-intent-{}-2", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("active.toml");
        std::fs::write(&path, "garbage = 1\nactive_profiles = []\n").unwrap();
        assert!(read(&path).is_err());
    }
}
