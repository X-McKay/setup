//! Resolve a user selection (profile names + explicit component ids)
//! into an ordered install plan.

use anyhow::{Result, bail};
use std::collections::{BTreeSet, HashSet};

use super::schema::Manifest;
use crate::system::platform::Platform;

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

/// Walk the dep graph from `seeds` and return a set including all transitive
/// dependencies. `seeds` is typically the output of `expand_selection`.
pub fn pull_in_dependencies(
    manifest: &Manifest,
    seeds: &BTreeSet<String>,
) -> Result<BTreeSet<String>> {
    let mut out = seeds.clone();
    let mut frontier: Vec<String> = seeds.iter().cloned().collect();
    while let Some(id) = frontier.pop() {
        let spec = manifest
            .components
            .iter()
            .find(|c| c.id == id)
            .ok_or_else(|| anyhow::anyhow!("unknown component: {}", id))?;
        for dep in &spec.depends_on {
            if out.insert(dep.clone()) {
                frontier.push(dep.clone());
            }
        }
    }
    Ok(out)
}

/// Topologically sort the given component ids using their `depends_on` edges.
/// Uses Kahn's algorithm. Order is deterministic: ties broken alphabetically.
pub fn topo_sort(manifest: &Manifest, ids: &BTreeSet<String>) -> Result<Vec<String>> {
    use std::collections::{BTreeMap, VecDeque};

    let in_set = |id: &str| ids.contains(id);

    let mut in_degree: BTreeMap<String, usize> = ids.iter().map(|id| (id.clone(), 0)).collect();
    let mut adj: BTreeMap<String, Vec<String>> =
        ids.iter().map(|id| (id.clone(), Vec::new())).collect();

    for id in ids {
        let spec = manifest
            .components
            .iter()
            .find(|c| c.id == *id)
            .ok_or_else(|| anyhow::anyhow!("component {:?} not in manifest", id))?;
        for dep in &spec.depends_on {
            if in_set(dep) {
                adj.get_mut(dep).unwrap().push(id.clone());
                *in_degree.get_mut(id).unwrap() += 1;
            }
        }
    }

    let mut ready: VecDeque<String> = in_degree
        .iter()
        .filter(|&(_, &d)| d == 0)
        .map(|(k, _)| k.clone())
        .collect();

    let mut ordered: Vec<String> = Vec::with_capacity(ids.len());
    while let Some(node) = ready.pop_front() {
        ordered.push(node.clone());
        for next in &adj[&node] {
            let d = in_degree.get_mut(next).unwrap();
            *d -= 1;
            if *d == 0 {
                ready.push_back(next.clone());
            }
        }
    }

    if ordered.len() != ids.len() {
        let remaining: Vec<String> = in_degree
            .iter()
            .filter(|&(_, &d)| d > 0)
            .map(|(k, _)| k.clone())
            .collect();
        anyhow::bail!("dep graph cycle detected among: {}", remaining.join(", "));
    }

    Ok(ordered)
}

/// End-to-end: given a user selection, produce the ordered install plan.
/// Returns the ordered list of component ids, the set that was auto-pulled
/// as transitive deps, and profile components skipped because they do not
/// support the current platform (all for reporting).
#[derive(Debug)]
pub struct Plan {
    pub ordered: Vec<String>,
    pub auto_pulled: BTreeSet<String>,
    pub skipped_platform: BTreeSet<String>,
}

pub fn resolve(
    manifest: &Manifest,
    profiles: &[String],
    explicit: &[String],
    platform: Platform,
) -> Result<Plan> {
    let seeds = expand_selection(manifest, profiles, explicit)?;

    // Components the user named directly must support this platform;
    // profile-derived ones are silently dropped (and reported).
    let supports = |id: &String| {
        manifest
            .components
            .iter()
            .find(|c| c.id == *id)
            .is_some_and(|c| c.supports(platform))
    };
    for id in explicit {
        if !supports(id) {
            bail!("component {:?} is not supported on {}", id, platform);
        }
    }
    let (seeds, skipped_platform): (BTreeSet<String>, BTreeSet<String>) =
        seeds.into_iter().partition(supports);

    let full = pull_in_dependencies(manifest, &seeds)?;
    for id in &full {
        if !supports(id) {
            bail!(
                "dependency {:?} of the selected components is not supported on {}",
                id,
                platform
            );
        }
    }

    let auto_pulled: BTreeSet<String> = full.difference(&seeds).cloned().collect();
    let ordered = topo_sort(manifest, &full)?;
    Ok(Plan {
        ordered,
        auto_pulled,
        skipped_platform,
    })
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
                ComponentSpec {
                    id: "apt".into(),
                    display_name: "APT".into(),
                    ..Default::default()
                },
                ComponentSpec {
                    id: "mise".into(),
                    display_name: "Mise".into(),
                    ..Default::default()
                },
                ComponentSpec {
                    id: "docker".into(),
                    display_name: "Docker".into(),
                    ..Default::default()
                },
                ComponentSpec {
                    id: "claude-code".into(),
                    display_name: "Claude Code".into(),
                    ..Default::default()
                },
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
            ["apt", "mise", "docker"]
                .iter()
                .map(|s| s.to_string())
                .collect()
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
            ["apt", "mise", "docker"]
                .iter()
                .map(|s| s.to_string())
                .collect()
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

    #[test]
    fn selection_auto_pulls_transitive_deps() {
        let mut m = mk_manifest();
        // docker depends on apt
        m.components
            .iter_mut()
            .find(|c| c.id == "docker")
            .unwrap()
            .depends_on = vec!["apt".into()];
        // Select docker alone — apt must be auto-pulled.
        let ids = expand_selection(&m, &[], &["docker".into()]).unwrap();
        let plan = pull_in_dependencies(&m, &ids).unwrap();
        assert!(plan.contains("apt"));
        assert!(plan.contains("docker"));
    }

    #[test]
    fn topo_sort_respects_deps() {
        let mut m = mk_manifest();
        m.components
            .iter_mut()
            .find(|c| c.id == "docker")
            .unwrap()
            .depends_on = vec!["apt".into(), "mise".into()];
        let seeds: BTreeSet<String> = ["apt", "mise", "docker"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let ordered = topo_sort(&m, &seeds).unwrap();
        let pos = |id: &str| ordered.iter().position(|x| x == id).unwrap();
        assert!(pos("apt") < pos("docker"));
        assert!(pos("mise") < pos("docker"));
    }

    #[test]
    fn topo_sort_detects_cycle() {
        let mut m = mk_manifest();
        m.components
            .iter_mut()
            .find(|c| c.id == "apt")
            .unwrap()
            .depends_on = vec!["mise".into()];
        m.components
            .iter_mut()
            .find(|c| c.id == "mise")
            .unwrap()
            .depends_on = vec!["apt".into()];
        let seeds: BTreeSet<String> = ["apt", "mise"].iter().map(|s| s.to_string()).collect();
        let err = topo_sort(&m, &seeds).unwrap_err();
        assert!(err.to_string().contains("cycle"));
    }
}

#[cfg(test)]
mod resolve_tests {
    use super::*;
    use crate::manifest::schema::{ComponentSpec, ProfileSpec};
    use std::collections::BTreeMap;

    #[test]
    fn resolve_reports_auto_pulled() {
        let mut profiles = BTreeMap::new();
        profiles.insert(
            "x".into(),
            ProfileSpec {
                description: String::new(),
                extends: vec![],
                components: vec!["docker".into()],
            },
        );
        let m = Manifest {
            components: vec![
                ComponentSpec {
                    id: "apt".into(),
                    display_name: "APT".into(),
                    ..Default::default()
                },
                ComponentSpec {
                    id: "docker".into(),
                    display_name: "Docker".into(),
                    depends_on: vec!["apt".into()],
                    ..Default::default()
                },
            ],
            profiles,
        };
        let plan = resolve(&m, &["x".into()], &[], Platform::Linux).unwrap();
        assert_eq!(plan.ordered, vec!["apt", "docker"]);
        assert_eq!(plan.auto_pulled, ["apt".to_string()].into_iter().collect());
        assert!(plan.skipped_platform.is_empty());
    }

    fn platform_manifest() -> Manifest {
        let mut profiles = BTreeMap::new();
        profiles.insert(
            "srv".into(),
            ProfileSpec {
                description: String::new(),
                extends: vec![],
                components: vec!["portable".into(), "linux-only".into()],
            },
        );
        Manifest {
            components: vec![
                ComponentSpec {
                    id: "portable".into(),
                    display_name: "Portable".into(),
                    ..Default::default()
                },
                ComponentSpec {
                    id: "linux-only".into(),
                    display_name: "Linux Only".into(),
                    platforms: vec!["linux".into()],
                    ..Default::default()
                },
            ],
            profiles,
        }
    }

    #[test]
    fn profile_skips_unsupported_platform_components() {
        let m = platform_manifest();
        let plan = resolve(&m, &["srv".into()], &[], Platform::MacOs).unwrap();
        assert_eq!(plan.ordered, vec!["portable"]);
        assert_eq!(
            plan.skipped_platform,
            ["linux-only".to_string()].into_iter().collect()
        );
    }

    #[test]
    fn explicit_unsupported_component_fails() {
        let m = platform_manifest();
        let err = resolve(&m, &[], &["linux-only".into()], Platform::MacOs).unwrap_err();
        assert!(err.to_string().contains("not supported on macos"));
    }

    #[test]
    fn unsupported_dependency_fails() {
        let mut m = platform_manifest();
        m.components
            .iter_mut()
            .find(|c| c.id == "portable")
            .unwrap()
            .depends_on = vec!["linux-only".into()];
        let err = resolve(&m, &[], &["portable".into()], Platform::MacOs).unwrap_err();
        assert!(err.to_string().contains("not supported on macos"));
    }

    #[test]
    fn linux_keeps_everything() {
        let m = platform_manifest();
        let plan = resolve(&m, &["srv".into()], &[], Platform::Linux).unwrap();
        assert_eq!(plan.ordered, vec!["linux-only", "portable"]);
        assert!(plan.skipped_platform.is_empty());
    }
}
