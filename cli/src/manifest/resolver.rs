//! Resolve a user selection (profile names + explicit component ids)
//! into an ordered install plan.

use anyhow::{bail, Result};
use std::collections::{BTreeSet, HashSet};

use super::schema::Manifest;

/// Parse CLI selection and return the deduplicated set of component ids.
///
/// `profile_names` is the list of profiles the user passed via `--profile`.
/// `explicit` is the list of component ids passed positionally.
/// The result is the union of (each profile's transitive components) and `explicit`.
pub fn expand_selection(
    manifest: &Manifest,
    profile_names: &[String],
    explicit: &[String],
) -> Result<BTreeSet<String>> {
    let mut out = BTreeSet::new();

    for name in profile_names {
        if !manifest.profiles.contains_key(name) {
            bail!("unknown profile: {:?}", name);
        }
        expand_profile(manifest, name, &mut out, &mut HashSet::new())?;
    }

    for id in explicit {
        if !manifest.components.iter().any(|c| c.id == *id) {
            bail!("unknown component: {:?}", id);
        }
        out.insert(id.clone());
    }

    Ok(out)
}

fn expand_profile(
    manifest: &Manifest,
    name: &str,
    out: &mut BTreeSet<String>,
    seen: &mut HashSet<String>,
) -> Result<()> {
    if !seen.insert(name.to_string()) {
        return Ok(()); // already expanded this profile in this traversal
    }
    let profile = &manifest.profiles[name];
    for ext in &profile.extends {
        expand_profile(manifest, ext, out, seen)?;
    }
    for cid in &profile.components {
        out.insert(cid.clone());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::schema::{ComponentSpec, ProfileSpec};
    use std::collections::BTreeMap;

    fn mk_manifest() -> Manifest {
        let mut profiles = BTreeMap::new();
        profiles.insert(
            "base".to_string(),
            ProfileSpec {
                description: String::new(),
                extends: vec![],
                components: vec!["apt".to_string(), "mise".to_string()],
            },
        );
        profiles.insert(
            "server".to_string(),
            ProfileSpec {
                description: String::new(),
                extends: vec!["base".to_string()],
                components: vec!["docker".to_string()],
            },
        );
        profiles.insert(
            "ai".to_string(),
            ProfileSpec {
                description: String::new(),
                extends: vec![],
                components: vec!["claude-code".to_string()],
            },
        );
        Manifest {
            components: vec![
                ComponentSpec { id: "apt".into(), display_name: "APT".into(), ..Default::default() },
                ComponentSpec { id: "mise".into(), display_name: "Mise".into(), ..Default::default() },
                ComponentSpec { id: "docker".into(), display_name: "Docker".into(), ..Default::default() },
                ComponentSpec { id: "claude-code".into(), display_name: "Claude Code".into(), ..Default::default() },
            ],
            profiles,
        }
    }

    #[test]
    fn single_profile_expands_to_its_components() {
        let m = mk_manifest();
        let got = expand_selection(&m, &["base".into()], &[]).unwrap();
        assert_eq!(got, ["apt", "mise"].iter().map(|s| s.to_string()).collect());
    }

    #[test]
    fn extends_is_transitive() {
        let m = mk_manifest();
        let got = expand_selection(&m, &["server".into()], &[]).unwrap();
        assert_eq!(
            got,
            ["apt", "mise", "docker"].iter().map(|s| s.to_string()).collect()
        );
    }

    #[test]
    fn two_profiles_union() {
        let m = mk_manifest();
        let got = expand_selection(&m, &["server".into(), "ai".into()], &[]).unwrap();
        assert_eq!(
            got,
            ["apt", "mise", "docker", "claude-code"]
                .iter()
                .map(|s| s.to_string())
                .collect()
        );
    }

    #[test]
    fn explicit_and_profile_merge() {
        let m = mk_manifest();
        let got = expand_selection(&m, &["base".into()], &["docker".into()]).unwrap();
        assert_eq!(
            got,
            ["apt", "mise", "docker"].iter().map(|s| s.to_string()).collect()
        );
    }

    #[test]
    fn unknown_profile_fails() {
        let m = mk_manifest();
        let err = expand_selection(&m, &["bogus".into()], &[]).unwrap_err();
        assert!(err.to_string().contains("unknown profile"));
    }

    #[test]
    fn unknown_explicit_component_fails() {
        let m = mk_manifest();
        let err = expand_selection(&m, &[], &["bogus".into()]).unwrap_err();
        assert!(err.to_string().contains("unknown component"));
    }
}
