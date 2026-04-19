//! Loads the repo manifest and optionally merges a user override.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use super::schema::Manifest;

/// Default path to the repo manifest, relative to the built binary's parent.
/// Production callers pass an explicit path via `load_from`.
const REPO_MANIFEST_RELATIVE: &str = "bootstrap/manifest.toml";

/// Locate the repo-shipped manifest. Tries, in order:
/// 1. `$SETUP_MANIFEST` environment override (used by tests).
/// 2. `./bootstrap/manifest.toml` relative to current working directory
///    (source-tree layout).
/// 3. `<exe_dir>/../share/setup/manifest.toml` (installed layout).
pub fn repo_manifest_path() -> Result<PathBuf> {
    if let Ok(env_path) = std::env::var("SETUP_MANIFEST") {
        return Ok(PathBuf::from(env_path));
    }
    let cwd_rel = PathBuf::from(REPO_MANIFEST_RELATIVE);
    if cwd_rel.exists() {
        return Ok(cwd_rel);
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let share = parent.join("..").join("share").join("setup").join("manifest.toml");
            if share.exists() {
                return Ok(share);
            }
        }
    }
    anyhow::bail!(
        "could not locate bootstrap/manifest.toml. Set SETUP_MANIFEST to override."
    )
}

/// Convenience: locate both paths and load.
pub fn load() -> Result<Manifest> {
    let repo = repo_manifest_path()?;
    let user = user_manifest_path();
    load_from(&repo, user.as_deref())
}

/// Where the user override lives.
pub fn user_manifest_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("setup").join("manifest.toml"))
}

/// Load a manifest from an explicit path, optionally merging a second file on top.
pub fn load_from(repo: &Path, user: Option<&Path>) -> Result<Manifest> {
    let repo_text =
        std::fs::read_to_string(repo).with_context(|| format!("reading {}", repo.display()))?;
    let mut manifest: Manifest =
        toml::from_str(&repo_text).with_context(|| format!("parsing {}", repo.display()))?;

    if let Some(u) = user {
        if u.exists() {
            let user_text = std::fs::read_to_string(u)
                .with_context(|| format!("reading {}", u.display()))?;
            let user_manifest: Manifest =
                toml::from_str(&user_text).with_context(|| format!("parsing {}", u.display()))?;
            manifest = merge(manifest, user_manifest);
        }
    }

    manifest
        .validate()
        .context("manifest validation failed after merge")?;
    Ok(manifest)
}

/// Merge `user` on top of `default`. Semantics per spec §4.3:
/// - Components merge by id (user wins entirely, no field-level merge).
/// - Profiles merge by name (user wins entirely).
fn merge(mut default: Manifest, user: Manifest) -> Manifest {
    // Components: replace by id.
    let user_ids: std::collections::HashSet<String> =
        user.components.iter().map(|c| c.id.clone()).collect();
    default.components.retain(|c| !user_ids.contains(&c.id));
    default.components.extend(user.components);

    // Profiles: replace by name.
    for (name, p) in user.profiles {
        default.profiles.insert(name, p);
    }

    default
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_tmp(name: &str, content: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("setup-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(name);
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
        path
    }

    #[test]
    fn loads_repo_manifest_only() {
        let repo = write_tmp(
            "repo1.toml",
            r#"
[[components]]
id = "apt"
display_name = "APT"

[profiles.base]
components = ["apt"]
"#,
        );
        let m = load_from(&repo, None).unwrap();
        assert_eq!(m.components.len(), 1);
        assert_eq!(m.components[0].id, "apt");
        assert!(m.profiles.contains_key("base"));
    }

    #[test]
    fn user_overrides_component_by_id() {
        let repo = write_tmp(
            "repo2.toml",
            r#"
[[components]]
id = "apt"
display_name = "APT (repo)"
"#,
        );
        let user = write_tmp(
            "user2.toml",
            r#"
[[components]]
id = "apt"
display_name = "APT (user)"
"#,
        );
        let m = load_from(&repo, Some(&user)).unwrap();
        assert_eq!(m.components.len(), 1);
        assert_eq!(m.components[0].display_name, "APT (user)");
    }

    #[test]
    fn user_adds_new_profile() {
        let repo = write_tmp(
            "repo3.toml",
            r#"
[[components]]
id = "apt"
display_name = "APT"
[profiles.base]
components = ["apt"]
"#,
        );
        let user = write_tmp(
            "user3.toml",
            r#"
[profiles.minimal]
components = ["apt"]
"#,
        );
        let m = load_from(&repo, Some(&user)).unwrap();
        assert!(m.profiles.contains_key("base"));
        assert!(m.profiles.contains_key("minimal"));
    }

    #[test]
    fn user_cannot_reference_unknown_component() {
        let repo = write_tmp(
            "repo4.toml",
            r#"
[[components]]
id = "apt"
display_name = "APT"
"#,
        );
        let user = write_tmp(
            "user4.toml",
            r#"
[profiles.weird]
components = ["nonexistent"]
"#,
        );
        let err = load_from(&repo, Some(&user)).unwrap_err();
        let msg = format!("{:#}", err);
        assert!(
            msg.contains("unknown component"),
            "expected chained error to mention unknown component, got: {msg}"
        );
    }

    #[test]
    fn missing_user_file_is_a_noop() {
        let repo = write_tmp(
            "repo5.toml",
            r#"
[[components]]
id = "apt"
display_name = "APT"
"#,
        );
        let nonexistent = PathBuf::from("/nonexistent-setup-test-path.toml");
        let m = load_from(&repo, Some(&nonexistent)).unwrap();
        assert_eq!(m.components.len(), 1);
    }
}
