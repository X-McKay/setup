//! TOML schema for the manifest.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Root manifest document. Both the repo default and the user override
/// deserialize into this type.
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Manifest {
    #[serde(default)]
    pub components: Vec<ComponentSpec>,
    #[serde(default)]
    pub profiles: BTreeMap<String, ProfileSpec>,
}

/// Metadata describing a single component. Install/uninstall logic lives
/// in Rust; this type is the "what exists and why" side.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ComponentSpec {
    pub id: String,
    pub display_name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub requires_sudo: bool,
    #[serde(default)]
    pub requires_systemd: bool,
    #[serde(default)]
    pub requires_privileged: bool,
    #[serde(default)]
    pub interactive: bool,
}

impl ComponentSpec {
    /// Derived predicate — can this component be installed in the Docker test
    /// harness? True iff none of the capability flags that block Docker are set.
    pub fn docker_testable(&self) -> bool {
        !self.requires_systemd && !self.requires_privileged && !self.interactive
    }
}

/// A named, composable machine shape. Extends transitively pulls another
/// profile's components.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProfileSpec {
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub extends: Vec<String>,
    #[serde(default)]
    pub components: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_component() {
        let input = r#"
[[components]]
id = "apt"
display_name = "Basic APT Packages"
"#;
        let m: Manifest = toml::from_str(input).unwrap();
        assert_eq!(m.components.len(), 1);
        let c = &m.components[0];
        assert_eq!(c.id, "apt");
        assert_eq!(c.display_name, "Basic APT Packages");
        assert!(c.depends_on.is_empty());
        assert!(!c.requires_sudo);
        assert!(c.docker_testable());
    }

    #[test]
    fn parses_component_with_all_fields() {
        let input = r#"
[[components]]
id = "docker"
display_name = "Docker"
description = "Container runtime"
depends_on = ["apt"]
tags = ["container", "dev"]
requires_sudo = true
requires_systemd = true
requires_privileged = true
interactive = false
"#;
        let m: Manifest = toml::from_str(input).unwrap();
        let c = &m.components[0];
        assert_eq!(c.depends_on, vec!["apt"]);
        assert_eq!(c.tags, vec!["container", "dev"]);
        assert!(c.requires_sudo);
        assert!(c.requires_systemd);
        assert!(!c.docker_testable());
    }

    #[test]
    fn parses_profile_with_extends() {
        let input = r#"
[profiles.server]
description = "Headless server"
extends = ["base"]
components = ["docker", "gh"]
"#;
        let m: Manifest = toml::from_str(input).unwrap();
        let p = &m.profiles["server"];
        assert_eq!(p.extends, vec!["base"]);
        assert_eq!(p.components, vec!["docker", "gh"]);
    }

    #[test]
    fn rejects_unknown_top_level_keys() {
        // serde by default tolerates unknown fields. We want strict mode so
        // typos surface immediately. This test will be made to pass in the
        // next task by adding deny_unknown_fields or a validation pass.
        // For now, confirm the permissive baseline.
        let input = r#"
garbage = "hi"
[[components]]
id = "x"
display_name = "X"
"#;
        let m: Manifest = toml::from_str(input).unwrap();
        assert_eq!(m.components.len(), 1);
    }
}
