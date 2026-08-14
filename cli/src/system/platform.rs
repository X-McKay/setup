//! OS platform detection shared by components and commands.

use anyhow::{Result, bail};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Linux,
    MacOs,
}

impl Platform {
    /// Detect the platform this binary is running on.
    pub fn current() -> Result<Platform> {
        match std::env::consts::OS {
            "linux" => Ok(Platform::Linux),
            "macos" => Ok(Platform::MacOs),
            other => bail!("unsupported platform: {} (supported: linux, macos)", other),
        }
    }

    /// The identifier used for this platform in `manifest.toml` `platforms`
    /// lists.
    pub fn manifest_key(&self) -> &'static str {
        match self {
            Platform::Linux => "linux",
            Platform::MacOs => "macos",
        }
    }
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.manifest_key())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_platform_is_supported() {
        // CI and dev machines are always Linux or macOS.
        let p = Platform::current().unwrap();
        assert!(matches!(p, Platform::Linux | Platform::MacOs));
    }

    #[test]
    fn manifest_keys_are_stable() {
        assert_eq!(Platform::Linux.manifest_key(), "linux");
        assert_eq!(Platform::MacOs.manifest_key(), "macos");
    }
}
