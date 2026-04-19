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
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
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

impl Manifest {
    /// Validate structural invariants: non-empty kebab-case ids, unique ids,
    /// every depends_on / profile component reference resolves inside this manifest.
    /// Called after merging repo + user manifests.
    pub fn validate(&self) -> anyhow::Result<()> {
        use std::collections::HashSet;

        let mut seen = HashSet::new();
        for c in &self.components {
            if c.id.is_empty() {
                anyhow::bail!("component has empty id");
            }
            if !is_kebab_case(&c.id) {
                anyhow::bail!("component has invalid id: {:?} (must be lowercase kebab-case)", c.id);
            }
            if !seen.insert(c.id.clone()) {
                anyhow::bail!("duplicate component id: {:?}", c.id);
            }
        }

        // Reference integrity — every depends_on id must exist in the manifest.
        let ids: HashSet<&str> = self.components.iter().map(|c| c.id.as_str()).collect();
        for c in &self.components {
            for d in &c.depends_on {
                if !ids.contains(d.as_str()) {
                    anyhow::bail!("component {:?} depends on unknown component {:?}", c.id, d);
                }
            }
        }

        // Profile component references must also resolve.
        for (name, p) in &self.profiles {
            for cid in &p.components {
                if !ids.contains(cid.as_str()) {
                    anyhow::bail!("profile {:?} references unknown component {:?}", name, cid);
                }
            }
            for ext in &p.extends {
                if !self.profiles.contains_key(ext) {
                    anyhow::bail!("profile {:?} extends unknown profile {:?}", name, ext);
                }
            }
        }

        // Extends cycle detection via recursive DFS (three-color marking).
        #[derive(Clone, Copy, PartialEq)]
        enum Mark {
            Unvisited,
            OnStack,
            Done,
        }
        let mut marks: std::collections::HashMap<&str, Mark> =
            self.profiles.keys().map(|k| (k.as_str(), Mark::Unvisited)).collect();

        fn dfs<'a>(
            node: &'a str,
            profiles: &'a BTreeMap<String, ProfileSpec>,
            marks: &mut std::collections::HashMap<&'a str, Mark>,
            path: &mut Vec<&'a str>,
        ) -> anyhow::Result<()> {
            marks.insert(node, Mark::OnStack);
            path.push(node);
            let p = &profiles[node];
            for ext in &p.extends {
                let ext_str = ext.as_str();
                // Resolve &str keyed into the map by looking up the profile key.
                let ext_key = profiles
                    .keys()
                    .find(|k| k.as_str() == ext_str)
                    .map(|k| k.as_str())
                    .unwrap_or(ext_str);
                match marks.get(ext_key).copied().unwrap_or(Mark::Unvisited) {
                    Mark::OnStack => {
                        anyhow::bail!(
                            "profile extends cycle detected: {} -> {}",
                            path.join(" -> "),
                            ext_key
                        );
                    }
                    Mark::Done => continue,
                    Mark::Unvisited => {
                        dfs(ext_key, profiles, marks, path)?;
                    }
                }
            }
            path.pop();
            marks.insert(node, Mark::Done);
            Ok(())
        }

        for start in self.profiles.keys() {
            if marks.get(start.as_str()).copied().unwrap_or(Mark::Unvisited) == Mark::Unvisited {
                let mut path: Vec<&str> = Vec::new();
                dfs(start.as_str(), &self.profiles, &mut marks, &mut path)?;
            }
        }

        Ok(())
    }
}

fn is_kebab_case(s: &str) -> bool {
    !s.is_empty()
        && s.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !s.starts_with('-')
        && !s.ends_with('-')
}

/// A named, composable machine shape. Extends transitively pulls another
/// profile's components.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
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
    fn rejects_unknown_component_field() {
        let input = r#"
[[components]]
id = "x"
display_name = "X"
typo_field = "oops"
"#;
        let err = toml::from_str::<Manifest>(input).unwrap_err();
        assert!(
            err.to_string().contains("typo_field") || err.to_string().contains("unknown field"),
            "expected unknown-field error, got: {}",
            err
        );
    }

    #[test]
    fn rejects_unknown_profile_field() {
        let input = r#"
[profiles.x]
typo = 1
components = []
"#;
        let err = toml::from_str::<Manifest>(input).unwrap_err();
        assert!(
            err.to_string().contains("typo") || err.to_string().contains("unknown field"),
            "expected unknown-field error, got: {}",
            err
        );
    }

    #[test]
    fn rejects_empty_id() {
        let input = r#"
[[components]]
id = ""
display_name = "X"
"#;
        let m: Manifest = toml::from_str(input).unwrap();
        let err = m.validate().unwrap_err();
        assert!(err.to_string().contains("empty id"));
    }

    #[test]
    fn rejects_invalid_id_chars() {
        let input = r#"
[[components]]
id = "Apt_Packages"
display_name = "X"
"#;
        let m: Manifest = toml::from_str(input).unwrap();
        let err = m.validate().unwrap_err();
        assert!(err.to_string().contains("invalid id"));
    }

    #[test]
    fn rejects_duplicate_ids() {
        let input = r#"
[[components]]
id = "apt"
display_name = "X"
[[components]]
id = "apt"
display_name = "Y"
"#;
        let m: Manifest = toml::from_str(input).unwrap();
        let err = m.validate().unwrap_err();
        assert!(err.to_string().contains("duplicate"));
    }

    #[test]
    fn rejects_extends_cycle() {
        let input = r#"
[profiles.a]
extends = ["b"]
components = []
[profiles.b]
extends = ["a"]
components = []
"#;
        let m: Manifest = toml::from_str(input).unwrap();
        let err = m.validate().unwrap_err();
        assert!(err.to_string().contains("cycle"));
    }
}
