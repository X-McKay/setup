use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Settings {
    #[serde(default)]
    pub installed: InstalledComponents,

    #[serde(default)]
    pub preferences: Preferences,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct InstalledComponents {
    pub apt_packages: bool,
    pub extra_tools: bool,
    pub mise: bool,
    pub docker: bool,
    pub monitoring: bool,
    pub backup: bool,
    pub starship: bool,
    pub zoxide: bool,
    pub lazygit: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Preferences {
    pub theme: String,
    pub shell: String,
    pub editor: String,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            theme: "github-dark-high-contrast".to_string(),
            shell: "bash".to_string(),
            editor: "vim".to_string(),
        }
    }
}

impl Settings {
    pub fn load() -> Result<Self> {
        let path = Self::config_path();

        if path.exists() {
            let content = fs::read_to_string(&path)?;
            Ok(toml::from_str(&content)?)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path();

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        fs::write(&path, content)?;

        Ok(())
    }

    fn config_path() -> PathBuf {
        let home = dirs::home_dir().expect("Could not find home directory");
        home.join(".config").join("setup").join("config.toml")
    }
}
