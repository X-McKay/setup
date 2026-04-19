# Manifest-Driven Architecture + Profiles — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the hardcoded `Component` enum + match-statement dispatch with a declarative TOML catalog, a `Component` trait, a runtime registry, composable machine-shape profiles, a dependency graph executor, an intent file (`active.toml`), and three new commands: `uninstall`, `doctor`, `list`, plus `profile` subcommands. All 27 existing components are migrated.

**Architecture:** TOML manifest (`bootstrap/manifest.toml`) declares components + profiles. Rust implements the `Component` trait for each id, registered in a `Registry`. At startup, the loader merges the repo manifest with an optional `~/.config/setup/manifest.toml` override, validates every id resolves to a trait impl, expands profiles, topologically sorts by `depends_on`, and dispatches install / uninstall / is-installed / verify / dry-run through the trait. Intent (which profiles the user picked) is persisted in `~/.config/setup/active.toml` and used by `doctor` and `uninstall` but deliberately NOT used to track install state (probing is authoritative for that).

**Tech Stack:** Rust 2021, `clap` 4 (CLI), `serde` + `toml` 0.8 (schema), `anyhow` (errors), `which` (binary probing), `dirs` (home/config paths), existing test harness (Docker + bash).

**Reference spec:** `plans/2026-04-18-manifest-architecture-design.md`. Implementation is authoritative where the two differ, but any such divergence must be a deliberate choice flagged in a commit message.

---

## Phase map

| Phase | Theme | Tasks |
|---|---|---|
| 1 | Manifest schema + parsing | 1.1 – 1.5 |
| 2 | Manifest loader (repo + user override) | 2.1 – 2.3 |
| 3 | Profile + dep graph resolver | 3.1 – 3.4 |
| 4 | Component trait + registry scaffold | 4.1 – 4.2 |
| 5 | `bootstrap/manifest.toml` catalog | 5.1 |
| 6 | Port all 27 components | 6.1 – 6.27 |
| 7 | Swap install dispatch onto registry | 7.1 – 7.3 |
| 8 | Delete old enum + `system/packages/` | 8.1 |
| 9 | Intent file (`active.toml`) I/O | 9.1 – 9.3 |
| 10 | `install` command new flags + intent writes | 10.1 – 10.6 |
| 11 | `uninstall` command | 11.1 – 11.4 |
| 12 | `doctor` command | 12.1 – 12.6 |
| 13 | `list` command | 13.1 |
| 14 | `profile` subcommand | 14.1 – 14.3 |
| 15 | Deprecate `check` command | 15.1 |
| 16 | Docker integration tests | 16.1 – 16.5 |
| 17 | Docs (README + CHANGELOG) | 17.1 – 17.2 |

**Test discipline.** Every phase with a `tests::` or dedicated test file uses TDD: write failing test → verify failure → implement → verify pass → commit. Phases that port existing Rust behavior without a Rust test (e.g. wrapping `install_apt_packages` into a trait impl) lean on the existing Docker integration tests — the test update in Phase 16 is the verification gate for those.

**Branch assumption.** You are on `feature/2026-refresh`. If not, stop and ask.

---

## Phase 1 — Manifest schema + parsing

Produces: `cli/src/manifest/schema.rs`, `cli/src/manifest/mod.rs`, unit tests. No behavior change yet.

### Task 1.1: Create manifest module skeleton

**Files:**
- Create: `cli/src/manifest/mod.rs`
- Modify: `cli/src/main.rs` (add `mod manifest;`)

- [ ] **Step 1: Create the module file**

Write `cli/src/manifest/mod.rs`:

```rust
//! Declarative manifest describing available components and profiles.
//!
//! See `plans/2026-04-18-manifest-architecture-design.md` for the full design.

pub mod schema;
```

- [ ] **Step 2: Register module in main.rs**

In `cli/src/main.rs`, after `mod commands;` add `mod manifest;` so the module is compiled:

```rust
mod commands;
mod config;
mod manifest;
mod system;
mod ui;
```

- [ ] **Step 3: Verify it compiles**

Run: `cd cli && cargo check`
Expected: compiles with a warning about unused module (that's fine).

- [ ] **Step 4: Commit**

```bash
git add cli/src/manifest/ cli/src/main.rs
git commit -m "refactor(manifest): scaffold manifest module"
```

### Task 1.2: Define schema types with failing test

**Files:**
- Create: `cli/src/manifest/schema.rs`
- Test: `cli/src/manifest/schema.rs` (inline `#[cfg(test)]`)

- [ ] **Step 1: Write schema.rs with types and a failing parse test**

Write `cli/src/manifest/schema.rs`:

```rust
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
```

- [ ] **Step 2: Run tests and verify they pass**

Run: `cd cli && cargo test -p setup manifest::schema`
Expected: 4 tests pass. (The fourth test asserts the current permissive behavior; we tighten it in Task 1.3.)

- [ ] **Step 3: Commit**

```bash
git add cli/src/manifest/schema.rs
git commit -m "feat(manifest): add schema types with parsing tests"
```

### Task 1.3: Strict unknown-field rejection for components and profiles

The component/profile schemas should reject typo'd keys (e.g. `requires_sudoo`). Root-level tolerance is kept so users can add forward-compatible metadata; typed sub-tables are strict.

**Files:**
- Modify: `cli/src/manifest/schema.rs`

- [ ] **Step 1: Update test to expect strict rejection**

Replace the `rejects_unknown_top_level_keys` test with a stricter one:

```rust
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
```

- [ ] **Step 2: Run test to confirm it fails**

Run: `cd cli && cargo test -p setup manifest::schema::tests::rejects_unknown_component_field -- --nocapture`
Expected: FAIL — serde accepts the unknown field by default.

- [ ] **Step 3: Add `deny_unknown_fields` to ComponentSpec and ProfileSpec**

Change both struct attributes:

```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentSpec { /* unchanged body */ }
```

```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileSpec { /* unchanged body */ }
```

- [ ] **Step 4: Run tests to verify pass**

Run: `cd cli && cargo test -p setup manifest::schema`
Expected: all tests pass, including the two new rejection tests.

- [ ] **Step 5: Commit**

```bash
git add cli/src/manifest/schema.rs
git commit -m "feat(manifest): reject unknown fields on ComponentSpec and ProfileSpec"
```

### Task 1.4: Component ID validation

IDs must be non-empty, kebab-case ASCII, and unique within a manifest.

**Files:**
- Modify: `cli/src/manifest/schema.rs`

- [ ] **Step 1: Add failing tests for ID validation**

Append to the `tests` module in `schema.rs`:

```rust
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
```

- [ ] **Step 2: Add the `validate` method on `Manifest`**

Add below the `ComponentSpec` impl block:

```rust
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

        Ok(())
    }
}

fn is_kebab_case(s: &str) -> bool {
    !s.is_empty()
        && s.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !s.starts_with('-')
        && !s.ends_with('-')
}
```

- [ ] **Step 3: Run tests**

Run: `cd cli && cargo test -p setup manifest::schema`
Expected: all tests pass including the three new ones.

- [ ] **Step 4: Commit**

```bash
git add cli/src/manifest/schema.rs
git commit -m "feat(manifest): validate ids, uniqueness, and reference integrity"
```

### Task 1.5: Profile extends cycle detection

**Files:**
- Modify: `cli/src/manifest/schema.rs`

- [ ] **Step 1: Add failing test**

Append to `tests`:

```rust
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
```

- [ ] **Step 2: Run test**

Run: `cd cli && cargo test -p setup manifest::schema::tests::rejects_extends_cycle`
Expected: FAIL — validate does not yet check cycles.

- [ ] **Step 3: Add cycle detection to `validate`**

Extend the profile-validation block in `validate` to DFS from each profile, tracking visited. Append inside `validate()`, after the existing profile loop:

```rust
        // Extends cycle detection via DFS.
        for start in self.profiles.keys() {
            let mut stack: Vec<&str> = vec![start.as_str()];
            let mut path: Vec<&str> = Vec::new();
            let mut visited: std::collections::HashSet<&str> = std::collections::HashSet::new();
            while let Some(node) = stack.last().copied() {
                if !visited.insert(node) {
                    if path.contains(&node) {
                        anyhow::bail!(
                            "profile extends cycle detected: {} -> {}",
                            path.join(" -> "),
                            node
                        );
                    }
                    stack.pop();
                    path.pop();
                    continue;
                }
                path.push(node);
                let p = &self.profiles[node];
                let mut pushed = false;
                for ext in &p.extends {
                    if !visited.contains(ext.as_str()) {
                        stack.push(ext.as_str());
                        pushed = true;
                        break;
                    }
                    // If we've already seen it AND it's on the current path, cycle.
                    if path.contains(&ext.as_str()) {
                        anyhow::bail!(
                            "profile extends cycle detected: {} -> {}",
                            path.join(" -> "),
                            ext
                        );
                    }
                }
                if !pushed {
                    stack.pop();
                    path.pop();
                }
            }
        }
```

- [ ] **Step 4: Run tests**

Run: `cd cli && cargo test -p setup manifest`
Expected: all tests pass.

- [ ] **Step 5: Commit**

```bash
git add cli/src/manifest/schema.rs
git commit -m "feat(manifest): detect profile extends cycles"
```

---

## Phase 2 — Manifest loader (repo + user override)

Produces: `cli/src/manifest/loader.rs`. Loads the repo manifest, optionally merges the user override, returns a validated `Manifest`.

### Task 2.1: Loader skeleton + repo-only load

**Files:**
- Create: `cli/src/manifest/loader.rs`
- Modify: `cli/src/manifest/mod.rs` (add `pub mod loader;`)

- [ ] **Step 1: Add failing test**

Create `cli/src/manifest/loader.rs`:

```rust
//! Loads the repo manifest and optionally merges a user override.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use super::schema::Manifest;

/// Default path to the repo manifest, relative to the built binary's parent.
/// Production callers pass an explicit path via `load_from`.
const REPO_MANIFEST_RELATIVE: &str = "bootstrap/manifest.toml";

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
}
```

Register the module in `cli/src/manifest/mod.rs`:

```rust
pub mod loader;
pub mod schema;
```

- [ ] **Step 2: Run test**

Run: `cd cli && cargo test -p setup manifest::loader`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add cli/src/manifest/
git commit -m "feat(manifest): loader with repo-only load path"
```

### Task 2.2: User override merge semantics

**Files:**
- Modify: `cli/src/manifest/loader.rs`

- [ ] **Step 1: Add tests for merge semantics**

Append to `loader.rs` tests:

```rust
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
        // Per spec §4.3: user can't introduce component ids without a Rust impl.
        // At this layer we check reference integrity; the Rust-registry check
        // happens in a later phase when the registry is built.
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
        assert!(err.to_string().contains("unknown component"));
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
```

- [ ] **Step 2: Run tests**

Run: `cd cli && cargo test -p setup manifest::loader`
Expected: all tests pass (the merge function already implements these semantics).

- [ ] **Step 3: Commit**

```bash
git add cli/src/manifest/loader.rs
git commit -m "test(manifest): cover user override merge semantics"
```

### Task 2.3: Resolve the repo manifest path at runtime

Runtime callers need `load()` (no args) that locates `bootstrap/manifest.toml` relative to the binary's distribution layout.

**Files:**
- Modify: `cli/src/manifest/loader.rs`

- [ ] **Step 1: Add function**

At the top of `loader.rs`, add:

```rust
/// Locate the repo-shipped manifest. Tries, in order:
/// 1. `$SETUP_MANIFEST` environment override (used by tests).
/// 2. `./bootstrap/manifest.toml` relative to current working directory
///    (source-tree layout).
/// 3. `<exe_dir>/../share/setup/manifest.toml` (installed layout).
pub fn repo_manifest_path() -> Result<PathBuf> {
    if let Ok(env_path) = std::env::var("SETUP_MANIFEST") {
        return Ok(PathBuf::from(env_path));
    }
    let cwd_rel = PathBuf::from("bootstrap").join("manifest.toml");
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
```

- [ ] **Step 2: Run tests (still using `load_from`)**

Run: `cd cli && cargo test -p setup manifest`
Expected: all existing tests pass.

- [ ] **Step 3: Commit**

```bash
git add cli/src/manifest/loader.rs
git commit -m "feat(manifest): add load() convenience with env/cwd/installed lookup"
```

---

## Phase 3 — Profile + dep graph resolver

Produces: `cli/src/manifest/resolver.rs`. Takes selected profile names + explicit component ids, returns a topologically sorted list of component ids to install.

### Task 3.1: Profile expansion + union

**Files:**
- Create: `cli/src/manifest/resolver.rs`
- Modify: `cli/src/manifest/mod.rs`

- [ ] **Step 1: Write failing tests + function**

Create `cli/src/manifest/resolver.rs`:

```rust
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
```

Make `ComponentSpec` derive `Default` — update `schema.rs`:

```rust
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentSpec {
    pub id: String,
    pub display_name: String,
    /* rest unchanged */
}
```

Register resolver in `cli/src/manifest/mod.rs`:

```rust
pub mod loader;
pub mod resolver;
pub mod schema;
```

- [ ] **Step 2: Run tests**

Run: `cd cli && cargo test -p setup manifest::resolver`
Expected: all 6 tests pass.

- [ ] **Step 3: Commit**

```bash
git add cli/src/manifest/
git commit -m "feat(manifest): profile expansion with extends and union"
```

### Task 3.2: Transitive dependency auto-pull

**Files:**
- Modify: `cli/src/manifest/resolver.rs`

- [ ] **Step 1: Add failing test**

Append to `resolver.rs` tests:

```rust
    #[test]
    fn selection_auto_pulls_transitive_deps() {
        let mut m = mk_manifest();
        // docker depends on apt
        m.components.iter_mut().find(|c| c.id == "docker").unwrap().depends_on =
            vec!["apt".into()];
        // Select docker alone — apt must be auto-pulled.
        let ids = expand_selection(&m, &[], &["docker".into()]).unwrap();
        let plan = pull_in_dependencies(&m, &ids).unwrap();
        assert!(plan.contains("apt"));
        assert!(plan.contains("docker"));
    }
```

- [ ] **Step 2: Run test to confirm it fails**

Run: `cd cli && cargo test -p setup manifest::resolver::tests::selection_auto_pulls_transitive_deps`
Expected: FAIL — `pull_in_dependencies` doesn't exist.

- [ ] **Step 3: Implement `pull_in_dependencies`**

Append to `resolver.rs`:

```rust
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
```

- [ ] **Step 4: Run tests**

Run: `cd cli && cargo test -p setup manifest::resolver`
Expected: all tests pass.

- [ ] **Step 5: Commit**

```bash
git add cli/src/manifest/resolver.rs
git commit -m "feat(manifest): transitive dep auto-pull"
```

### Task 3.3: Topological sort

**Files:**
- Modify: `cli/src/manifest/resolver.rs`

- [ ] **Step 1: Add tests**

Append to `resolver.rs` tests:

```rust
    #[test]
    fn topo_sort_respects_deps() {
        let mut m = mk_manifest();
        m.components.iter_mut().find(|c| c.id == "docker").unwrap().depends_on =
            vec!["apt".into(), "mise".into()];
        let seeds: BTreeSet<String> =
            ["apt", "mise", "docker"].iter().map(|s| s.to_string()).collect();
        let ordered = topo_sort(&m, &seeds).unwrap();
        let pos = |id: &str| ordered.iter().position(|x| x == id).unwrap();
        assert!(pos("apt") < pos("docker"));
        assert!(pos("mise") < pos("docker"));
    }

    #[test]
    fn topo_sort_detects_cycle() {
        let mut m = mk_manifest();
        m.components.iter_mut().find(|c| c.id == "apt").unwrap().depends_on =
            vec!["mise".into()];
        m.components.iter_mut().find(|c| c.id == "mise").unwrap().depends_on =
            vec!["apt".into()];
        let seeds: BTreeSet<String> = ["apt", "mise"].iter().map(|s| s.to_string()).collect();
        let err = topo_sort(&m, &seeds).unwrap_err();
        assert!(err.to_string().contains("cycle"));
    }
```

- [ ] **Step 2: Run tests, confirm failure**

Run: `cd cli && cargo test -p setup manifest::resolver::tests::topo_sort_respects_deps`
Expected: FAIL — `topo_sort` doesn't exist.

- [ ] **Step 3: Implement Kahn's algorithm**

Append to `resolver.rs`:

```rust
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
        .filter(|(_, &d)| d == 0)
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
            .filter(|(_, &d)| d > 0)
            .map(|(k, _)| k.clone())
            .collect();
        anyhow::bail!("dep graph cycle detected among: {}", remaining.join(", "));
    }

    Ok(ordered)
}
```

- [ ] **Step 4: Run tests**

Run: `cd cli && cargo test -p setup manifest::resolver`
Expected: all tests pass.

- [ ] **Step 5: Commit**

```bash
git add cli/src/manifest/resolver.rs
git commit -m "feat(manifest): topological sort with cycle detection"
```

### Task 3.4: End-to-end `resolve` convenience

**Files:**
- Modify: `cli/src/manifest/resolver.rs`

- [ ] **Step 1: Add test + function**

Append:

```rust
/// End-to-end: given a user selection, produce the ordered install plan.
/// Returns the ordered list of component ids, and the set that was
/// auto-pulled as transitive deps (for reporting).
pub struct Plan {
    pub ordered: Vec<String>,
    pub auto_pulled: BTreeSet<String>,
}

pub fn resolve(
    manifest: &Manifest,
    profiles: &[String],
    explicit: &[String],
) -> Result<Plan> {
    let seeds = expand_selection(manifest, profiles, explicit)?;
    let full = pull_in_dependencies(manifest, &seeds)?;
    let auto_pulled: BTreeSet<String> = full.difference(&seeds).cloned().collect();
    let ordered = topo_sort(manifest, &full)?;
    Ok(Plan { ordered, auto_pulled })
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
                ComponentSpec { id: "apt".into(), display_name: "APT".into(), ..Default::default() },
                ComponentSpec {
                    id: "docker".into(),
                    display_name: "Docker".into(),
                    depends_on: vec!["apt".into()],
                    ..Default::default()
                },
            ],
            profiles,
        };
        let plan = resolve(&m, &["x".into()], &[]).unwrap();
        assert_eq!(plan.ordered, vec!["apt", "docker"]);
        assert_eq!(plan.auto_pulled, ["apt".to_string()].into_iter().collect());
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cd cli && cargo test -p setup manifest::resolver`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add cli/src/manifest/resolver.rs
git commit -m "feat(manifest): resolve() producing ordered install plan"
```

---

## Phase 4 — Component trait + registry scaffold

Produces: `cli/src/components/mod.rs`, `cli/src/components/registry.rs`. Empty registry; no components registered yet.

### Task 4.1: Component trait

**Files:**
- Create: `cli/src/components/mod.rs`
- Modify: `cli/src/main.rs` (add `mod components;`)

- [ ] **Step 1: Write trait definition**

Create `cli/src/components/mod.rs`:

```rust
//! Component trait and registry.
//!
//! Each component is a unit struct implementing `Component`. Registered
//! in `registry::Registry::build()` and dispatched through the registry
//! by id.

use anyhow::Result;

pub mod registry;

pub trait Component: Send + Sync {
    /// Matches the manifest `id`.
    fn id(&self) -> &str;

    /// Probe whether this component is currently present on the system.
    fn is_installed(&self) -> Result<bool>;

    /// Install the component. MUST be idempotent.
    fn install(&self) -> Result<()>;

    /// Whether it is safe to call `uninstall()` automatically during
    /// `--rollback-on-failure`. Default: true. Override to false for
    /// components that manage user material (SSH keys, GPG keys, etc.)
    /// — these still need `uninstall()` implemented if they support
    /// forced removal; they just won't be called automatically.
    fn is_reversible(&self) -> bool {
        true
    }

    /// Remove the component. Default refuses — this is the safe default
    /// for components that cannot be cleanly uninstalled. Override to
    /// enable `setup uninstall <id>`.
    fn uninstall(&self) -> Result<()> {
        anyhow::bail!(
            "{} does not implement uninstall — not removable by this tool",
            self.id()
        )
    }

    /// Post-install sanity check. Default delegates to `is_installed()`.
    fn verify(&self) -> Result<()> {
        if self.is_installed()? {
            Ok(())
        } else {
            anyhow::bail!("{} not installed", self.id())
        }
    }

    /// Describe what `install()` would do. Called by `--dry-run`.
    fn dry_run(&self) -> Result<Vec<String>> {
        Ok(vec![format!("would install {}", self.id())])
    }
}
```

- [ ] **Step 2: Register the module**

In `cli/src/main.rs`:

```rust
mod commands;
mod components;
mod config;
mod manifest;
mod system;
mod ui;
```

- [ ] **Step 3: Verify compilation**

Run: `cd cli && cargo check`
Expected: compiles with unused-module warning.

- [ ] **Step 4: Commit**

```bash
git add cli/src/components/ cli/src/main.rs
git commit -m "feat(components): add Component trait"
```

### Task 4.2: Registry

**Files:**
- Create: `cli/src/components/registry.rs`

- [ ] **Step 1: Write registry + tests**

Create `cli/src/components/registry.rs`:

```rust
//! Component registry. Populated by `build()` at startup.

use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::Arc;

use super::Component;

pub struct Registry {
    components: HashMap<String, Arc<dyn Component>>,
}

impl Registry {
    /// Build the full registry by wiring every component implementation.
    /// This is the ONE place that knows about every component in the system.
    pub fn build() -> Self {
        let mut r = Self {
            components: HashMap::new(),
        };
        // Components are registered here in Phase 6. For now the registry
        // is empty — this is the seam.
        let _ = &mut r;
        r
    }

    pub fn register(&mut self, c: Arc<dyn Component>) {
        let id = c.id().to_string();
        if self.components.insert(id.clone(), c).is_some() {
            panic!("duplicate component registration: {}", id);
        }
    }

    pub fn get(&self, id: &str) -> Result<Arc<dyn Component>> {
        self.components
            .get(id)
            .cloned()
            .ok_or_else(|| anyhow!("unknown component: {}", id))
    }

    pub fn ids(&self) -> Vec<String> {
        let mut v: Vec<_> = self.components.keys().cloned().collect();
        v.sort();
        v
    }

    /// Validate that every id in the manifest has a registered implementation,
    /// and every registered implementation has a manifest entry.
    pub fn validate_against(&self, manifest: &crate::manifest::schema::Manifest) -> Result<()> {
        use std::collections::HashSet;
        let reg: HashSet<_> = self.components.keys().cloned().collect();
        let man: HashSet<_> = manifest.components.iter().map(|c| c.id.clone()).collect();

        let missing_impls: Vec<_> = man.difference(&reg).cloned().collect();
        let orphan_impls: Vec<_> = reg.difference(&man).cloned().collect();

        if !missing_impls.is_empty() || !orphan_impls.is_empty() {
            let mut msg = String::new();
            if !missing_impls.is_empty() {
                msg.push_str(&format!(
                    "manifest components without Rust impl: {};\n",
                    missing_impls.join(", ")
                ));
            }
            if !orphan_impls.is_empty() {
                msg.push_str(&format!(
                    "Rust impls without manifest entry: {}",
                    orphan_impls.join(", ")
                ));
            }
            anyhow::bail!("registry/manifest mismatch:\n{}", msg);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    struct FakeA;
    impl Component for FakeA {
        fn id(&self) -> &str {
            "a"
        }
        fn is_installed(&self) -> Result<bool> {
            Ok(false)
        }
        fn install(&self) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn register_and_get() {
        let mut r = Registry {
            components: HashMap::new(),
        };
        r.register(Arc::new(FakeA));
        assert!(r.get("a").is_ok());
        assert!(r.get("missing").is_err());
    }

    #[test]
    fn ids_is_sorted() {
        let mut r = Registry {
            components: HashMap::new(),
        };
        r.register(Arc::new(FakeA));
        assert_eq!(r.ids(), vec!["a"]);
    }

    #[test]
    #[should_panic(expected = "duplicate")]
    fn duplicate_registration_panics() {
        let mut r = Registry {
            components: HashMap::new(),
        };
        r.register(Arc::new(FakeA));
        r.register(Arc::new(FakeA));
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cd cli && cargo test -p setup components::registry`
Expected: 3 tests pass.

- [ ] **Step 3: Commit**

```bash
git add cli/src/components/registry.rs
git commit -m "feat(components): registry with id-based dispatch"
```

---

## Phase 5 — `bootstrap/manifest.toml` catalog

Produces: the repo-default manifest declaring all 27 existing components plus four profiles (`base`, `server`, `workstation`, `ai-heavy`).

### Task 5.1: Write the repo manifest

**Files:**
- Create: `bootstrap/manifest.toml`

- [ ] **Step 1: Write manifest**

Create `bootstrap/manifest.toml`:

```toml
# Repo-default component catalog + profiles.
# Users may override entries by creating ~/.config/setup/manifest.toml
# (see docs — override/reshape only, cannot introduce new component ids).

# ---------- components ----------

[[components]]
id = "apt"
display_name = "Basic APT Packages"
description = "Core system packages (curl, git, build-essential, etc.)"
tags = ["core"]
requires_sudo = true

[[components]]
id = "tools"
display_name = "Extra CLI Tools"
description = "ripgrep, fd, fzf, bat, eza, delta"
depends_on = ["apt"]
tags = ["core", "cli"]
requires_sudo = true

[[components]]
id = "mise"
display_name = "Mise Version Manager"
description = "Polyglot version manager (reads ~/.tool-versions)"
depends_on = ["apt"]
tags = ["core", "dev"]

[[components]]
id = "docker"
display_name = "Docker"
description = "Container runtime"
depends_on = ["apt"]
tags = ["container", "dev"]
requires_sudo = true
requires_systemd = true
requires_privileged = true

[[components]]
id = "lazygit"
display_name = "Lazygit"
description = "Terminal UI for git"
tags = ["dev"]

[[components]]
id = "just"
display_name = "Just"
description = "Command runner"
tags = ["dev"]

[[components]]
id = "glow"
display_name = "Glow"
description = "Markdown renderer for terminal"
tags = ["cli"]

[[components]]
id = "bottom"
display_name = "Bottom"
description = "System monitor (btm)"
tags = ["cli"]

[[components]]
id = "gh"
display_name = "GitHub CLI"
description = "GitHub from the terminal"
depends_on = ["apt"]
tags = ["dev"]
requires_sudo = true

[[components]]
id = "hyperfine"
display_name = "Hyperfine"
description = "Command-line benchmarking"
tags = ["cli"]

[[components]]
id = "jq"
display_name = "jq"
description = "JSON processor"
tags = ["cli"]

[[components]]
id = "yq"
display_name = "yq"
description = "YAML processor"
tags = ["cli"]

[[components]]
id = "tldr"
display_name = "tldr"
description = "Simplified man pages"
tags = ["cli"]

[[components]]
id = "chromium"
display_name = "Chromium"
description = "Web browser (snap)"
tags = ["gui"]
requires_sudo = true

[[components]]
id = "discord"
display_name = "Discord"
description = "Chat client (snap)"
tags = ["gui"]
requires_sudo = true

[[components]]
id = "obsidian"
display_name = "Obsidian"
description = "Note-taking app (snap)"
tags = ["gui"]
requires_sudo = true

[[components]]
id = "spotify"
display_name = "Spotify"
description = "Music streaming (snap)"
tags = ["gui"]
requires_sudo = true

[[components]]
id = "vlc"
display_name = "VLC"
description = "Media player (snap)"
tags = ["gui"]
requires_sudo = true

[[components]]
id = "ghostty"
display_name = "Ghostty Terminal"
description = "GPU-accelerated terminal emulator"
tags = ["gui", "dev"]
requires_sudo = true

[[components]]
id = "claude-code"
display_name = "Claude Code"
description = "Anthropic's official CLI for Claude"
depends_on = ["mise"]
tags = ["ai", "dev"]

[[components]]
id = "neovim"
display_name = "Neovim"
description = "Modal editor"
depends_on = ["apt"]
tags = ["dev"]
requires_sudo = true

[[components]]
id = "tpm"
display_name = "Tmux Plugin Manager"
description = "Plugin manager for tmux"
tags = ["dev"]

[[components]]
id = "monitoring"
display_name = "System Monitoring"
description = "htop, netdata, fail2ban, logwatch + daily health cron"
depends_on = ["apt"]
tags = ["ops"]
requires_sudo = true
requires_systemd = true

[[components]]
id = "backup"
display_name = "Backup Utilities"
description = "rsync, timeshift, duplicity + backup cron"
depends_on = ["apt"]
tags = ["ops"]
requires_sudo = true
requires_systemd = true

[[components]]
id = "ssh-keys"
display_name = "SSH Key Generation"
description = "Generate ED25519 SSH keys (interactive)"
tags = ["security"]
interactive = true

[[components]]
id = "gpg"
display_name = "GPG Setup"
description = "Generate GPG keys for commit signing (interactive)"
tags = ["security"]
interactive = true

# ---------- profiles ----------

[profiles.base]
description = "Always present; other profiles extend this"
components = ["apt", "tools", "mise"]

[profiles.server]
description = "Headless server: core + container + ops + dev basics"
extends = ["base"]
components = ["docker", "gh", "neovim", "lazygit", "just", "jq", "yq", "monitoring", "backup"]

[profiles.workstation]
description = "Desktop/laptop development box"
extends = ["base"]
components = [
    "ghostty",
    "docker",
    "lazygit",
    "just",
    "glow",
    "bottom",
    "gh",
    "hyperfine",
    "jq",
    "yq",
    "tldr",
    "neovim",
    "tpm",
    "chromium",
    "obsidian",
]

[profiles.ai-heavy]
description = "AI tooling layer — compose with server or workstation"
components = ["claude-code"]
```

- [ ] **Step 2: Add a loader test that reads the real manifest**

Append to `cli/src/manifest/loader.rs` tests:

```rust
    #[test]
    fn real_repo_manifest_parses_and_validates() {
        // This test assumes cargo test runs with CWD at the crate root (cli/)
        // or the workspace root. Try both.
        let candidates = [
            PathBuf::from("../bootstrap/manifest.toml"),
            PathBuf::from("bootstrap/manifest.toml"),
        ];
        let found = candidates.iter().find(|p| p.exists()).expect(
            "bootstrap/manifest.toml not found; run from repo root or cli/",
        );
        let m = load_from(found, None).expect("repo manifest should load");
        assert!(!m.components.is_empty());
        assert!(m.profiles.contains_key("base"));
        assert!(m.profiles.contains_key("server"));
        assert!(m.profiles.contains_key("workstation"));
        assert!(m.profiles.contains_key("ai-heavy"));
    }
```

- [ ] **Step 3: Run test**

Run: `cd cli && cargo test -p setup manifest::loader::tests::real_repo_manifest_parses_and_validates`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add bootstrap/manifest.toml cli/src/manifest/loader.rs
git commit -m "feat(manifest): add bootstrap/manifest.toml with 27 components + 4 profiles"
```

---

## Phase 6 — Port all 27 components

Each component becomes a struct in `cli/src/components/<id>.rs` implementing `Component`, registered in `Registry::build()`. The existing function in `cli/src/system/packages/` is kept intact for now — the struct simply calls it. This keeps each port a tiny, reversible commit.

**Template (Task 6.1 for `apt`). Every subsequent component task follows this same shape — read this template carefully.**

### Task 6.1: Port `apt` — template for all following ports

**Files:**
- Create: `cli/src/components/apt.rs`
- Modify: `cli/src/components/registry.rs` (register)

- [ ] **Step 1: Create the component file**

Create `cli/src/components/apt.rs`:

```rust
//! `apt` component — core system packages.
//!
//! Installs: curl, wget, git, build-essential, gcc, make, cmake,
//! pkg-config, libssl-dev, libffi-dev, python3-dev, python3-pip,
//! unzip, zip, jq.
//!
//! Uninstall: unsupported. Removing these packages on a live system
//! would break countless downstream dependencies. Users who want to
//! remove them must do so by hand via apt.

use anyhow::Result;

use super::Component;
use crate::system::packages;

pub struct Apt;

impl Component for Apt {
    fn id(&self) -> &str {
        "apt"
    }

    fn is_installed(&self) -> Result<bool> {
        // Probe a representative binary that comes from this package set.
        Ok(which::which("curl").is_ok() && which::which("git").is_ok())
    }

    fn install(&self) -> Result<()> {
        packages::install_apt_packages()
    }

    // uninstall: use default (refuses). See module doc for rationale.
    // is_reversible: default true (does not affect anything because uninstall refuses).
}
```

- [ ] **Step 2: Register in `registry.rs`**

Modify `cli/src/components/registry.rs`. Replace the existing `build()` method with:

```rust
    pub fn build() -> Self {
        let mut r = Self {
            components: HashMap::new(),
        };
        r.register(Arc::new(super::apt::Apt));
        r
    }
```

Add a `mod` declaration at the top of `cli/src/components/mod.rs` (above `pub mod registry;`):

```rust
pub mod apt;
pub mod registry;
```

- [ ] **Step 3: Verify it compiles**

Run: `cd cli && cargo check`
Expected: compiles cleanly.

- [ ] **Step 4: Add a smoke test**

Append to `cli/src/components/registry.rs` tests:

```rust
    #[test]
    fn apt_is_registered() {
        let r = Registry::build();
        let c = r.get("apt").unwrap();
        assert_eq!(c.id(), "apt");
    }
```

Run: `cd cli && cargo test -p setup components::registry`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add cli/src/components/
git commit -m "feat(components): port apt"
```

---

### Tasks 6.2 – 6.27 — Remaining components

Each task below follows the same five-step template as Task 6.1:

1. Create `cli/src/components/<file>.rs` with a struct implementing `Component` (id, `is_installed`, `install`, plus `uninstall` / `is_reversible` / `verify` / `dry_run` where listed).
2. Add `pub mod <file>;` to `cli/src/components/mod.rs`.
3. Add `r.register(Arc::new(super::<file>::<Struct>));` inside `Registry::build()`.
4. Run `cargo check` then `cargo test -p setup components::registry`.
5. Commit with message `feat(components): port <id>`.

The table below specifies the particulars for each component. `install_fn` is the existing function in `cli/src/system/packages/`; wrap it verbatim. `probe` is the `is_installed()` body. `uninstall` tells you whether to override. `is_reversible` tells you whether to override to false.

| # | id | file | Struct | install_fn | probe (`is_installed`) | uninstall override | is_reversible override |
|---|---|---|---|---|---|---|---|
| 6.2 | `tools` | `tools.rs` | `Tools` | `packages::install_extra_tools()` | `which::which("rg").is_ok() && which::which("fd").is_ok() && which::which("bat").is_ok()` | **no** (apt set — refuse default) | no |
| 6.3 | `mise` | `mise.rs` | `Mise` | `packages::install_mise()` | `which::which("mise").is_ok()` | **yes** — see below | no |
| 6.4 | `docker` | `docker.rs` | `Docker` | `packages::install_docker()` | `which::which("docker").is_ok()` | no | no |
| 6.5 | `lazygit` | `lazygit.rs` | `Lazygit` | `packages::install_lazygit()` | `which::which("lazygit").is_ok()` | **yes** — see below | no |
| 6.6 | `just` | `just.rs` | `Just` | `packages::install_just()` | `which::which("just").is_ok()` | **yes** — single binary in `~/.local/bin` | no |
| 6.7 | `glow` | `glow.rs` | `Glow` | `packages::install_glow()` | `which::which("glow").is_ok()` | **yes** — single binary | no |
| 6.8 | `bottom` | `bottom.rs` | `Bottom` | `packages::install_bottom()` | `which::which("btm").is_ok()` | **yes** — single binary | no |
| 6.9 | `gh` | `gh.rs` | `Gh` | `packages::install_gh()` | `which::which("gh").is_ok()` | no (apt-installed; auth state is separate concern) | no |
| 6.10 | `hyperfine` | `hyperfine.rs` | `Hyperfine` | `packages::install_hyperfine()` | `which::which("hyperfine").is_ok()` | **yes** — single binary | no |
| 6.11 | `jq` | `jq.rs` | `Jq` | `packages::install_jq()` | `which::which("jq").is_ok()` | no | no |
| 6.12 | `yq` | `yq.rs` | `Yq` | `packages::install_yq()` | `which::which("yq").is_ok()` | **yes** — single binary | no |
| 6.13 | `tldr` | `tldr.rs` | `Tldr` | `packages::install_tldr()` | `which::which("tldr").is_ok()` | no | no |
| 6.14 | `chromium` | `chromium.rs` | `Chromium` | `packages::install_chromium()` | `which::which("chromium").is_ok() \|\| std::path::Path::new("/snap/bin/chromium").exists()` | **yes** — `sudo snap remove chromium` | no |
| 6.15 | `discord` | `discord.rs` | `Discord` | `packages::install_discord()` | snap: `std::path::Path::new("/snap/bin/discord").exists()` | **yes** — `sudo snap remove discord` | no |
| 6.16 | `obsidian` | `obsidian.rs` | `Obsidian` | `packages::install_obsidian()` | `std::path::Path::new("/snap/bin/obsidian").exists()` | **yes** — `sudo snap remove obsidian` | no |
| 6.17 | `spotify` | `spotify.rs` | `Spotify` | `packages::install_spotify()` | `std::path::Path::new("/snap/bin/spotify").exists()` | **yes** — `sudo snap remove spotify` | no |
| 6.18 | `vlc` | `vlc.rs` | `Vlc` | `packages::install_vlc()` | `which::which("vlc").is_ok()` | **yes** — `sudo snap remove vlc` | no |
| 6.19 | `ghostty` | `ghostty.rs` | `Ghostty` | `packages::install_ghostty()` | `which::which("ghostty").is_ok()` | no (built from source — messy to uninstall cleanly) | no |
| 6.20 | `claude-code` | `claude_code.rs` | `ClaudeCode` | `packages::install_claude_code()` | `which::which("claude").is_ok() \|\| which::which("claude-code").is_ok()` | **yes** — `npm uninstall -g @anthropic-ai/claude-code` | no |
| 6.21 | `neovim` | `neovim.rs` | `Neovim` | `packages::install_neovim()` | `which::which("nvim").is_ok()` | no | no |
| 6.22 | `tpm` | `tpm.rs` | `Tpm` | `packages::install_tpm()` | `std::path::Path::new(&dirs::home_dir().unwrap_or_default()).join(".tmux/plugins/tpm").exists()` | **yes** — `rm -rf ~/.tmux/plugins/tpm` | no |
| 6.23 | `monitoring` | `monitoring.rs` | `Monitoring` | `packages::install_monitoring()` | `which::which("htop").is_ok() && which::which("netdata").is_ok()` | no | no |
| 6.24 | `backup` | `backup.rs` | `Backup` | `packages::install_backup()` | `which::which("rsync").is_ok() && std::path::Path::new(&dirs::home_dir().unwrap_or_default()).join(".backup").exists()` | no | no |
| 6.25 | `ssh-keys` | `ssh_keys.rs` | `SshKeys` | `packages::setup_ssh_keys()` | `std::path::Path::new(&dirs::home_dir().unwrap_or_default()).join(".ssh/id_ed25519").exists()` | **yes** — see below | **yes → false** |
| 6.26 | `gpg` | `gpg.rs` | `Gpg` | `packages::setup_gpg()` | `std::process::Command::new("gpg").args(["--list-secret-keys"]).output().map(\|o\| !o.stdout.is_empty()).unwrap_or(false)` | **yes** — see below | **yes → false** |

### Uninstall implementations to override

For components where "uninstall override" is **yes** above, the `uninstall()` method body is one of:

**Single binary in `~/.local/bin`** (mise, just, glow, bottom, hyperfine, yq):

```rust
    fn uninstall(&self) -> Result<()> {
        let bin = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("no home dir"))?
            .join(".local/bin/<BINARY>");   // replace <BINARY> with actual name: mise, just, glow, btm, hyperfine, yq
        if bin.exists() {
            std::fs::remove_file(&bin)?;
        }
        Ok(())
    }
```

> Note for `bottom`: the binary is `btm`, not `bottom`.
> Note for `lazygit`: binary is `lazygit`.

**Snap removal** (chromium, discord, obsidian, spotify, vlc):

```rust
    fn uninstall(&self) -> Result<()> {
        let status = std::process::Command::new("sudo")
            .args(["snap", "remove", "<ID>"])   // replace <ID>: chromium / discord / obsidian / spotify / vlc
            .status()?;
        if !status.success() {
            anyhow::bail!("snap remove <ID> failed");
        }
        Ok(())
    }
```

**npm global uninstall** (claude-code):

```rust
    fn uninstall(&self) -> Result<()> {
        let status = std::process::Command::new("npm")
            .args(["uninstall", "-g", "@anthropic-ai/claude-code"])
            .status()?;
        if !status.success() {
            anyhow::bail!("npm uninstall -g @anthropic-ai/claude-code failed");
        }
        Ok(())
    }
```

**Directory removal** (tpm):

```rust
    fn uninstall(&self) -> Result<()> {
        let dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("no home dir"))?
            .join(".tmux/plugins/tpm");
        if dir.exists() {
            std::fs::remove_dir_all(&dir)?;
        }
        Ok(())
    }
```

**SSH keys** (ssh-keys) — removes the generated keys. Destructive of user material, hence `is_reversible = false`:

```rust
    fn is_reversible(&self) -> bool {
        false
    }

    fn uninstall(&self) -> Result<()> {
        let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("no home dir"))?;
        for name in &["id_ed25519", "id_ed25519.pub"] {
            let p = home.join(".ssh").join(name);
            if p.exists() {
                std::fs::remove_file(&p)?;
            }
        }
        Ok(())
    }
```

**GPG** (gpg) — prints guidance rather than performing destructive deletion, even with `--force`. The `--force` path accepts being told "do it yourself":

```rust
    fn is_reversible(&self) -> bool {
        false
    }

    fn uninstall(&self) -> Result<()> {
        anyhow::bail!(
            "gpg uninstall requires manual action: run `gpg --delete-secret-keys <keyid>` \
             then `gpg --delete-keys <keyid>`. This tool will not automate GPG key deletion."
        )
    }
```

> Note: because this component's `uninstall()` always errors, `setup uninstall gpg --force` will surface the guidance message and exit non-zero. That's the intended behavior — the user performs the step manually.

---

## Phase 7 — Swap install dispatch onto registry

Replaces the enum match in `cli/src/commands/install.rs` with registry dispatch. At this point the old enum is still present and `--all` still works; we change only the single dispatch function.

### Task 7.1: Introduce a registry-backed installer helper

**Files:**
- Modify: `cli/src/commands/install.rs`

- [ ] **Step 1: Add a helper function that dispatches via registry**

Near the bottom of `cli/src/commands/install.rs`, above the existing `install_component_with_progress`, add:

```rust
fn install_via_registry(mp: &MultiProgress, id: &str) -> Result<()> {
    use crate::components::registry::Registry;

    let spinner_style = ProgressStyle::default_spinner()
        .template("{spinner:.cyan} {msg}")
        .unwrap()
        .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏");

    let pb = mp.add(ProgressBar::new_spinner());
    pb.set_style(spinner_style);
    pb.set_message(format!("{}...", style(id).cyan()));
    pb.enable_steady_tick(Duration::from_millis(80));

    let registry = Registry::build();
    let component = registry.get(id)?;
    let result = component.install();

    pb.finish_and_clear();
    result
}
```

- [ ] **Step 2: Verify compile**

Run: `cd cli && cargo check`
Expected: clean.

- [ ] **Step 3: Commit**

```bash
git add cli/src/commands/install.rs
git commit -m "refactor(install): add registry-backed install helper"
```

### Task 7.2: Route enum dispatch through the new helper

**Files:**
- Modify: `cli/src/commands/install.rs`

- [ ] **Step 1: Replace the match arms in `install_component_with_progress`**

Currently the function matches on every `Component` variant and calls `packages::install_X()` directly. Replace that match with a mapping to component id + call to `install_via_registry`:

```rust
fn install_component_with_progress(mp: &MultiProgress, component: &Component) -> Result<()> {
    let id = match component {
        Component::Apt => "apt",
        Component::Tools => "tools",
        Component::Mise => "mise",
        Component::Docker => "docker",
        Component::Monitoring => "monitoring",
        Component::Backup => "backup",
        Component::Lazygit => "lazygit",
        Component::Just => "just",
        Component::Glow => "glow",
        Component::Bottom => "bottom",
        Component::Gh => "gh",
        Component::Hyperfine => "hyperfine",
        Component::Jq => "jq",
        Component::Yq => "yq",
        Component::Tldr => "tldr",
        Component::Chromium => "chromium",
        Component::Discord => "discord",
        Component::Obsidian => "obsidian",
        Component::Spotify => "spotify",
        Component::Vlc => "vlc",
        Component::Ghostty => "ghostty",
        Component::ClaudeCode => "claude-code",
        Component::Neovim => "neovim",
        Component::Tpm => "tpm",
        Component::SshKeys => "ssh-keys",
        Component::Gpg => "gpg",
    };
    install_via_registry(mp, id)
}
```

- [ ] **Step 2: Build and run existing tests**

Run: `cd cli && cargo test -p setup`
Expected: all existing unit tests pass. Integration-test behavior is unchanged because each id goes through `registry.get().install()`, which wraps the same `packages::install_*()` function.

- [ ] **Step 3: Commit**

```bash
git add cli/src/commands/install.rs
git commit -m "refactor(install): dispatch through Component registry"
```

### Task 7.3: Add registry/manifest consistency check at startup

**Files:**
- Modify: `cli/src/main.rs`

- [ ] **Step 1: Validate at startup**

In `cli/src/main.rs`, add a validation call before `run_command`:

```rust
fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Validate manifest <-> registry consistency on every run.
    // Deliberately non-fatal for commands that don't depend on it
    // (dotfiles, update) — we only enforce for install/uninstall/doctor/list/profile.
    //
    // For now, run the check unconditionally but treat errors as
    // diagnostic warnings so the migration doesn't break existing flows.
    if let (Ok(manifest), registry) = (
        manifest::loader::load(),
        components::registry::Registry::build(),
    ) {
        if let Err(e) = registry.validate_against(&manifest) {
            eprintln!("warning: manifest/registry drift:\n{}\n", e);
        }
    }

    let cli = Cli::parse();
    match cli.command {
        Some(cmd) => run_command(cmd),
        None => commands::interactive::run(),
    }
}
```

- [ ] **Step 2: Run**

Run: `cd cli && cargo run -- --help`
Expected: help prints; no "manifest/registry drift" warning (because we migrated all 27).

- [ ] **Step 3: Commit**

```bash
git add cli/src/main.rs
git commit -m "feat(main): validate manifest/registry consistency at startup"
```

---

## Phase 8 — Delete old enum + `system/packages/`

Only run this phase after Phase 7 is complete and the Docker integration tests still pass. This is the point of no return for the dual-path architecture.

### Task 8.1: Remove the old Component enum, the dispatch match, and `system/packages/`

**Files:**
- Modify: `cli/src/commands/install.rs` (rewrite — see Phase 10 for the full new version)
- Modify: `cli/src/system/mod.rs`
- Delete: `cli/src/system/packages/` (entire directory)
- Modify: `cli/src/commands/interactive.rs` (stop using the enum)

> Note: Because Phase 10 rewrites `install.rs` heavily for the new flags, defer the enum deletion to be handled there, not as a standalone task. Mark Phase 8 as "integrated into Phase 10." This avoids a transient broken state.

- [ ] **Step 1: Add a tombstone note**

Add to `cli/src/system/packages/mod.rs` (top of file):

```rust
//! DEPRECATED: these functions are wrapped by trait impls in
//! `cli/src/components/*.rs`. This module is deleted in Phase 10
//! after `install.rs` is fully rewritten onto the manifest.
```

- [ ] **Step 2: Commit the note**

```bash
git add cli/src/system/packages/mod.rs
git commit -m "chore(packages): mark deprecated pending install.rs rewrite"
```

The actual deletion happens in Task 10.6.

---

## Phase 9 — Intent file (`active.toml`) I/O

### Task 9.1: Intent schema + read

**Files:**
- Create: `cli/src/manifest/intent.rs`
- Modify: `cli/src/manifest/mod.rs`

- [ ] **Step 1: Write module with tests**

Create `cli/src/manifest/intent.rs`:

```rust
//! Persistent user intent — which profiles have been selected on this machine.
//!
//! Distinct from install state (which is always probed). See spec §4.5.

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
/// Overridable via `SETUP_INTENT` env var (used by tests).
pub fn default_path() -> Option<PathBuf> {
    if let Ok(env_path) = std::env::var("SETUP_INTENT") {
        return Some(PathBuf::from(env_path));
    }
    dirs::config_dir().map(|d| d.join("setup").join("active.toml"))
}

/// Read intent from the given path. Returns Intent::default() if the file
/// doesn't exist — absence is a valid state (no intent declared).
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
        let p = PathBuf::from("/nonexistent-intent-path.toml");
        let i = read(&p).unwrap();
        assert!(i.active_profiles.is_empty());
    }

    #[test]
    fn reads_existing_intent() {
        let dir = std::env::temp_dir().join(format!("setup-intent-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("active.toml");
        std::fs::write(&p, "active_profiles = [\"server\", \"ai-heavy\"]\n").unwrap();
        let i = read(&p).unwrap();
        assert_eq!(i.active_profiles, vec!["server", "ai-heavy"]);
    }

    #[test]
    fn rejects_unknown_fields() {
        let dir = std::env::temp_dir().join(format!("setup-intent-{}-2", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("active.toml");
        std::fs::write(&p, "garbage = 1\nactive_profiles = []\n").unwrap();
        assert!(read(&p).is_err());
    }
}
```

Register in `cli/src/manifest/mod.rs`:

```rust
pub mod intent;
pub mod loader;
pub mod resolver;
pub mod schema;
```

- [ ] **Step 2: Run tests**

Run: `cd cli && cargo test -p setup manifest::intent`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add cli/src/manifest/
git commit -m "feat(intent): read ~/.config/setup/active.toml"
```

### Task 9.2: Write intent

**Files:**
- Modify: `cli/src/manifest/intent.rs`

- [ ] **Step 1: Add tests**

Append to `intent.rs` tests:

```rust
    #[test]
    fn write_creates_parent_dirs() {
        let dir = std::env::temp_dir().join(format!("setup-intent-{}-w", std::process::id()));
        let p = dir.join("nested").join("active.toml");
        let i = Intent {
            active_profiles: vec!["server".into()],
        };
        write(&p, &i).unwrap();
        let round = read(&p).unwrap();
        assert_eq!(round, i);
    }

    #[test]
    fn union_add_is_idempotent() {
        let mut i = Intent {
            active_profiles: vec!["a".into(), "b".into()],
        };
        union_add(&mut i, &["b".into(), "c".into()]);
        assert_eq!(i.active_profiles, vec!["a", "b", "c"]);
    }

    #[test]
    fn remove_preserves_order() {
        let mut i = Intent {
            active_profiles: vec!["a".into(), "b".into(), "c".into()],
        };
        remove(&mut i, "b");
        assert_eq!(i.active_profiles, vec!["a", "c"]);
    }
```

- [ ] **Step 2: Run tests, confirm failure**

Run: `cd cli && cargo test -p setup manifest::intent`
Expected: FAIL — `write`, `union_add`, `remove` don't exist.

- [ ] **Step 3: Implement**

Append to `intent.rs` (above the `#[cfg(test)]`):

```rust
/// Write intent to the given path. Creates parent directories if needed.
pub fn write(path: &Path, intent: &Intent) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating {}", parent.display()))?;
    }
    let text = toml::to_string_pretty(intent).context("serializing intent")?;
    std::fs::write(path, text).with_context(|| format!("writing {}", path.display()))
}

/// Add `profiles` to `intent.active_profiles`, preserving existing order,
/// deduplicating. New entries appended in the order given.
pub fn union_add(intent: &mut Intent, profiles: &[String]) {
    for p in profiles {
        if !intent.active_profiles.iter().any(|x| x == p) {
            intent.active_profiles.push(p.clone());
        }
    }
}

/// Remove `name` from `intent.active_profiles` if present. Preserves order.
pub fn remove(intent: &mut Intent, name: &str) {
    intent.active_profiles.retain(|p| p != name);
}
```

- [ ] **Step 4: Run tests**

Run: `cd cli && cargo test -p setup manifest::intent`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add cli/src/manifest/intent.rs
git commit -m "feat(intent): write, union_add, remove"
```

### Task 9.3: Validation against manifest

Stale `active_profiles` entries (profile removed from manifest) should produce a warning, not a failure.

**Files:**
- Modify: `cli/src/manifest/intent.rs`

- [ ] **Step 1: Add test**

```rust
    #[test]
    fn warn_drift_filters_unknown_profiles() {
        use crate::manifest::schema::{Manifest, ProfileSpec};
        use std::collections::BTreeMap;

        let mut profiles = BTreeMap::new();
        profiles.insert(
            "server".into(),
            ProfileSpec {
                description: String::new(),
                extends: vec![],
                components: vec![],
            },
        );
        let manifest = Manifest {
            components: vec![],
            profiles,
        };
        let intent = Intent {
            active_profiles: vec!["server".into(), "ghost".into()],
        };
        let (valid, warnings) = validated(&intent, &manifest);
        assert_eq!(valid, vec!["server"]);
        assert_eq!(warnings, vec!["ghost"]);
    }
```

- [ ] **Step 2: Implement**

Append to `intent.rs`:

```rust
/// Filter `intent.active_profiles` to the subset that exists in `manifest`.
/// Returns `(valid_profiles, unknown_profiles_for_warning)`.
pub fn validated(intent: &Intent, manifest: &crate::manifest::schema::Manifest) -> (Vec<String>, Vec<String>) {
    let mut valid = Vec::new();
    let mut unknown = Vec::new();
    for p in &intent.active_profiles {
        if manifest.profiles.contains_key(p) {
            valid.push(p.clone());
        } else {
            unknown.push(p.clone());
        }
    }
    (valid, unknown)
}
```

- [ ] **Step 3: Run tests**

Run: `cd cli && cargo test -p setup manifest::intent`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add cli/src/manifest/intent.rs
git commit -m "feat(intent): validated() filters unknown profiles with warnings"
```

---

## Phase 10 — `install` command new flags + intent writes + deletion of old enum

Rewrites `cli/src/commands/install.rs` to use the new flags and the resolver. Deletes the old `Component` enum and the legacy `packages` module.

### Task 10.1: New `InstallArgs` shape

**Files:**
- Modify: `cli/src/commands/install.rs`

- [ ] **Step 1: Replace the InstallArgs struct and Component enum**

At the top of `cli/src/commands/install.rs`, replace the existing `InstallArgs` and `Component` enum with:

```rust
use anyhow::{Context, Result};
use clap::Args;
use console::style;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::time::Duration;

use crate::components::registry::Registry;
use crate::manifest::{intent, loader, resolver};

#[derive(Args)]
pub struct InstallArgs {
    /// Component ids to install (positional)
    pub components: Vec<String>,

    /// Profile to install (composable, may be given multiple times)
    #[arg(long = "profile")]
    pub profiles: Vec<String>,

    /// Install every component in the registry (mutually exclusive with --profile and positional ids)
    #[arg(long)]
    pub all: bool,

    /// Preview the plan without installing anything
    #[arg(long)]
    pub dry_run: bool,

    /// Run verify() after each successful install (may be slow)
    #[arg(long)]
    pub verify: bool,

    /// Continue past failures; print summary at end
    #[arg(long = "keep-going", conflicts_with = "rollback_on_failure")]
    pub keep_going: bool,

    /// On mid-run failure, uninstall components installed in this run (skips non-reversible)
    #[arg(long = "rollback-on-failure", conflicts_with = "keep_going")]
    pub rollback_on_failure: bool,

    /// Skip confirmation prompts
    #[arg(short = 'y', long)]
    pub yes: bool,
}
```

Remove the entire `#[derive(Clone, ValueEnum, Debug, PartialEq)] pub enum Component { ... }` block and its `impl` block. Keep the `InstallArgs` definition above. We'll rebuild `run()` in the next task.

- [ ] **Step 2: Temporarily stub `run()` so the crate compiles**

Replace the existing `pub fn run(args: InstallArgs) -> Result<()>` with a stub that will be filled in 10.2:

```rust
pub fn run(_args: InstallArgs) -> Result<()> {
    anyhow::bail!("install: not yet re-implemented in Phase 10.2")
}
```

Also delete everything below `run` (the helper functions that referenced the old enum).

- [ ] **Step 3: Verify compile**

Run: `cd cli && cargo check`
Expected: compiles.

- [ ] **Step 4: Commit**

```bash
git add cli/src/commands/install.rs
git commit -m "refactor(install): new InstallArgs with profile/dry-run/rollback flags (stub run)"
```

### Task 10.2: Re-implement `run()` using the resolver

**Files:**
- Modify: `cli/src/commands/install.rs`

- [ ] **Step 1: Write `run()`**

Replace the stub:

```rust
pub fn run(args: InstallArgs) -> Result<()> {
    validate_flag_combination(&args)?;

    let manifest = loader::load().context("loading manifest")?;
    let registry = Registry::build();
    registry
        .validate_against(&manifest)
        .context("manifest/registry drift at install time")?;

    let plan = if args.all {
        let all_ids: Vec<String> = manifest.components.iter().map(|c| c.id.clone()).collect();
        resolver::resolve(&manifest, &[], &all_ids)?
    } else {
        resolver::resolve(&manifest, &args.profiles, &args.components)?
    };

    if plan.ordered.is_empty() {
        println!("{}", style("No components selected.").yellow());
        return Ok(());
    }

    if !plan.auto_pulled.is_empty() {
        println!(
            "{} auto-pulled deps: {}",
            style("ℹ").cyan(),
            plan.auto_pulled
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    if args.dry_run {
        print_dry_run(&manifest, &registry, &plan.ordered)?;
        return Ok(());
    }

    if !args.yes {
        let display_names: Vec<String> = plan
            .ordered
            .iter()
            .map(|id| spec_display(&manifest, id))
            .collect();
        if !crate::ui::prompts::confirm_install(&display_names.iter().map(|s| s.as_str()).collect::<Vec<_>>())? {
            println!("{}", style("Installation cancelled.").yellow());
            return Ok(());
        }
    }

    let installed_this_run = run_plan(&registry, &plan.ordered, args.keep_going, args.rollback_on_failure, args.verify)?;

    update_intent_on_success(&args, &installed_this_run, &plan.ordered)?;

    Ok(())
}

fn validate_flag_combination(args: &InstallArgs) -> Result<()> {
    if args.all && (!args.profiles.is_empty() || !args.components.is_empty()) {
        anyhow::bail!("--all is mutually exclusive with --profile and positional components");
    }
    Ok(())
}

fn spec_display(m: &crate::manifest::schema::Manifest, id: &str) -> String {
    m.components
        .iter()
        .find(|c| c.id == id)
        .map(|c| c.display_name.clone())
        .unwrap_or_else(|| id.to_string())
}

fn print_dry_run(
    _manifest: &crate::manifest::schema::Manifest,
    registry: &Registry,
    ordered: &[String],
) -> Result<()> {
    println!("{}", style("Dry-run plan:").bold());
    for id in ordered {
        let c = registry.get(id)?;
        println!("  {}", style(format!("• {}", id)).cyan());
        for line in c.dry_run()? {
            println!("      {}", style(line).dim());
        }
    }
    Ok(())
}

fn run_plan(
    registry: &Registry,
    ordered: &[String],
    keep_going: bool,
    rollback_on_failure: bool,
    verify: bool,
) -> Result<Vec<String>> {
    let mp = MultiProgress::new();
    let mut installed: Vec<String> = Vec::new();
    let mut failures: Vec<(String, String)> = Vec::new();
    let mut had_failure = false;

    for id in ordered {
        let c = registry.get(id)?;
        if c.is_installed().unwrap_or(false) {
            mp.println(format!(
                "{} {} (already installed)",
                style("✓").green().bold(),
                style(id).green()
            ))?;
            continue;
        }

        let spinner = mp.add(ProgressBar::new_spinner());
        spinner.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.cyan} {msg}")
                .unwrap()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
        );
        spinner.set_message(format!("{}...", id));
        spinner.enable_steady_tick(Duration::from_millis(80));

        let outcome = c.install();
        spinner.finish_and_clear();

        match outcome {
            Ok(()) => {
                installed.push(id.clone());
                mp.println(format!("{} {}", style("✓").green().bold(), style(id).green()))?;
                if verify {
                    match c.verify() {
                        Ok(()) => {}
                        Err(e) => {
                            mp.println(format!(
                                "{} verify {}: {}",
                                style("!").yellow().bold(),
                                id,
                                e
                            ))?;
                        }
                    }
                }
            }
            Err(e) => {
                had_failure = true;
                failures.push((id.clone(), e.to_string()));
                mp.println(format!(
                    "{} {} — {}",
                    style("✗").red().bold(),
                    style(id).red(),
                    style(&e).dim()
                ))?;
                if !keep_going {
                    break;
                }
            }
        }
    }

    if had_failure && rollback_on_failure && !installed.is_empty() {
        println!(
            "{} rolling back {} installed component(s)",
            style("↺").yellow().bold(),
            installed.len()
        );
        for id in installed.iter().rev() {
            let c = registry.get(id)?;
            if !c.is_reversible() {
                println!("  {} {} skipped (not reversible)", style("~").yellow(), id);
                continue;
            }
            if let Err(e) = c.uninstall() {
                println!(
                    "  {} {} rollback failed: {}",
                    style("!").red().bold(),
                    id,
                    e
                );
            } else {
                println!("  {} {} rolled back", style("↺").yellow(), id);
            }
        }
    }

    if !failures.is_empty() {
        println!("\n{}", style("Installation summary").bold());
        for (id, err) in &failures {
            println!("  {} {} — {}", style("✗").red(), id, style(err).dim());
        }
        if had_failure && !rollback_on_failure {
            let remaining: Vec<_> = ordered
                .iter()
                .skip_while(|id| installed.iter().any(|i| i == *id))
                .filter(|id| !installed.contains(id))
                .collect();
            if !remaining.is_empty() {
                println!(
                    "\n{} still pending: {}",
                    style("→").dim(),
                    remaining
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
        }
    }

    Ok(installed)
}

fn update_intent_on_success(
    args: &InstallArgs,
    installed: &[String],
    planned: &[String],
) -> Result<()> {
    // Write intent only when the user declared intent via --profile and the
    // run actually reached the end of the plan (either cleanly or via --keep-going).
    // --all, stop-on-failure, and rollback-on-failure never update intent.
    if args.all || args.rollback_on_failure {
        return Ok(());
    }
    if args.profiles.is_empty() {
        return Ok(());
    }
    // stop-on-failure: install stopped early → installed.len() < planned.len() AND not keep_going
    if !args.keep_going && installed.len() < planned.len() {
        // Check: did any planned component neither install nor was already installed?
        // If so, this was a stop-on-failure. Be conservative and don't write intent.
        // Heuristic: if `run_plan` returned before processing every id, we stopped early.
        // We can infer this from the installed list vs planned list only if we also
        // tracked "skipped because already-installed." For simplicity: if keep_going
        // is false and any planned id is absent from installed and not already-installed,
        // we treat it as stop-on-failure.
        // Re-probe each planned id; if any is not installed, we stopped early.
        let registry = Registry::build();
        for id in planned {
            let c = registry.get(id)?;
            if !c.is_installed().unwrap_or(false) {
                // Run did not complete; don't update intent.
                return Ok(());
            }
        }
    }

    let path = intent::default_path().context("no config dir for intent file")?;
    let mut i = intent::read(&path)?;
    intent::union_add(&mut i, &args.profiles);
    intent::write(&path, &i)?;
    println!(
        "{} recorded intent: active_profiles = {:?}",
        style("ℹ").cyan(),
        i.active_profiles
    );
    Ok(())
}
```

- [ ] **Step 2: Verify compile**

Run: `cd cli && cargo check`
Expected: compiles (may warn about unused import — fine).

- [ ] **Step 3: Commit**

```bash
git add cli/src/commands/install.rs
git commit -m "feat(install): profile resolution, dry-run, rollback, intent writes"
```

### Task 10.3: Unit tests for flag validation and intent-write gating

**Files:**
- Modify: `cli/src/commands/install.rs`

- [ ] **Step 1: Add tests at the bottom of `install.rs`**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn args(
        components: Vec<&str>,
        profiles: Vec<&str>,
        all: bool,
    ) -> InstallArgs {
        InstallArgs {
            components: components.into_iter().map(String::from).collect(),
            profiles: profiles.into_iter().map(String::from).collect(),
            all,
            dry_run: false,
            verify: false,
            keep_going: false,
            rollback_on_failure: false,
            yes: true,
        }
    }

    #[test]
    fn all_conflicts_with_profile() {
        let a = args(vec![], vec!["server"], true);
        assert!(validate_flag_combination(&a).is_err());
    }

    #[test]
    fn all_conflicts_with_explicit_components() {
        let a = args(vec!["apt"], vec![], true);
        assert!(validate_flag_combination(&a).is_err());
    }

    #[test]
    fn profile_plus_explicit_is_ok() {
        let a = args(vec!["apt"], vec!["server"], false);
        assert!(validate_flag_combination(&a).is_ok());
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cd cli && cargo test -p setup commands::install`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add cli/src/commands/install.rs
git commit -m "test(install): flag combination validation"
```

### Task 10.4: Update `interactive.rs` to use registry ids

**Files:**
- Modify: `cli/src/commands/interactive.rs`
- Modify: `cli/src/ui/prompts.rs` (if it references the old enum)

- [ ] **Step 1: Read current interactive.rs**

Run: `cat cli/src/commands/interactive.rs`
The file currently dispatches via the old `Component` enum. Rewrite it to show a picker over `Registry::build().ids()` and pass the selected ids to `install::run` via a constructed `InstallArgs`.

- [ ] **Step 2: Write replacement**

Replace `cli/src/commands/interactive.rs` with:

```rust
//! Interactive top-level menu shown when `setup` is invoked with no subcommand.

use anyhow::Result;
use inquire::{MultiSelect, Select};

use crate::commands::install::{self, InstallArgs};
use crate::components::registry::Registry;
use crate::manifest::loader;

pub fn run() -> Result<()> {
    let manifest = loader::load()?;
    let registry = Registry::build();

    let top = Select::new(
        "What would you like to do?",
        vec![
            "Install components",
            "Install by profile",
            "Exit",
        ],
    )
    .prompt()?;

    match top {
        "Install components" => {
            let ids = registry.ids();
            let picked = MultiSelect::new("Select components:", ids).prompt()?;
            install::run(InstallArgs {
                components: picked,
                profiles: vec![],
                all: false,
                dry_run: false,
                verify: false,
                keep_going: false,
                rollback_on_failure: false,
                yes: false,
            })
        }
        "Install by profile" => {
            let names: Vec<String> = manifest.profiles.keys().cloned().collect();
            let picked = MultiSelect::new("Select profiles:", names).prompt()?;
            install::run(InstallArgs {
                components: vec![],
                profiles: picked,
                all: false,
                dry_run: false,
                verify: false,
                keep_going: false,
                rollback_on_failure: false,
                yes: false,
            })
        }
        _ => Ok(()),
    }
}
```

- [ ] **Step 3: Check `ui/prompts.rs`**

Run: `grep -n "Component::" cli/src/ui/prompts.rs`. If there are references to the old enum, rewrite them to take `&[&str]` (display names). The `confirm_install(&[&str]) -> Result<bool>` signature is already used by install.rs — keep that. If `select_components()` still references the old enum, remove it (interactive.rs no longer calls it).

- [ ] **Step 4: Build and run**

Run: `cd cli && cargo build`
Expected: compiles.

- [ ] **Step 5: Commit**

```bash
git add cli/src/commands/interactive.rs cli/src/ui/prompts.rs
git commit -m "refactor(interactive): drive from registry + manifest"
```

### Task 10.5: Delete the `packages/` module and stop calling it

**Files:**
- Modify: `cli/src/system/mod.rs` (remove `pub mod packages;`)
- Delete: `cli/src/system/packages/` (whole tree)

- [ ] **Step 1: Confirm no references remain**

Run: `grep -rn "system::packages" cli/src/ || echo OK`
Expected: `OK` — nothing references it.

If references remain, they're in component files. Those component files import their install logic from `super::super::system::packages::X`. That import chain needs to be broken: inline the install logic into the component file, or move the helper functions into `cli/src/components/util/` (new module). **Prefer inlining** for single-use helpers; **move to `components/util/`** for multi-use (apt install helpers, GitHub release fetcher).

- [ ] **Step 2: Move shared util to `components/util/`**

Copy the *entire content* of `cli/src/system/packages/utils.rs` into a new file `cli/src/components/util.rs`. The file has exactly these public items and they all get called from multiple component files — keep them all:

- `pub mod fallback_versions` (constants LAZYGIT, GLOW, BOTTOM, HYPERFINE)
- `pub fn fetch_github_version(repo, fallback) -> String`
- `pub fn path_to_str(path) -> Result<&str>`
- `pub fn run_command(cmd, args) -> Result<String>`
- `pub fn run_sudo(cmd, args) -> Result<String>`
- `pub fn apt_install(packages) -> Result<()>`
- `pub fn ensure_bin_dir() -> Result<PathBuf>`
- `pub fn get_arch() -> Result<&'static str>`
- `pub fn get_arch_alt() -> Result<&'static str>`

Add to `cli/src/components/mod.rs`:

```rust
pub mod util;
```

Then, in every component file created in Phase 6 that previously imported from `crate::system::packages::utils`, rewrite the import. For example in `cli/src/components/lazygit.rs`:

```rust
// before
use crate::system::packages::utils::{fetch_github_version, ensure_bin_dir, fallback_versions, get_arch};
// after
use super::util::{fetch_github_version, ensure_bin_dir, fallback_versions, get_arch};
```

Do a full search to find every callsite to update:

```bash
grep -rn "system::packages::utils" cli/src/components/
```

- [ ] **Step 3: Delete the old directory**

Run:
```bash
git rm -r cli/src/system/packages/
```

Remove the `pub mod packages;` line from `cli/src/system/mod.rs`. The file should now read:

```rust
pub mod health;
```

- [ ] **Step 4: Build + test**

Run: `cd cli && cargo test -p setup`
Expected: all tests pass.

- [ ] **Step 5: Commit**

```bash
git add cli/src/ 
git commit -m "refactor: delete legacy system/packages/ module (migrated to components/)"
```

### Task 10.6: Install smoke test

**Files:**
- No code change; verification only.

- [ ] **Step 1: Build + dry-run server profile**

Run: `cd cli && cargo run -- install --profile server --dry-run`
Expected: prints plan, no side effects, exits 0.

- [ ] **Step 2: Commit nothing**

No commit — this is a manual smoke test.

---

## Phase 11 — `uninstall` command

### Task 11.1: Command skeleton and CLI wiring

**Files:**
- Create: `cli/src/commands/uninstall.rs`
- Modify: `cli/src/commands/mod.rs`
- Modify: `cli/src/main.rs`

- [ ] **Step 1: Write uninstall.rs**

Create `cli/src/commands/uninstall.rs`:

```rust
//! `setup uninstall` — remove a component (or multiple).

use anyhow::{Context, Result};
use clap::Args;
use console::style;

use crate::components::registry::Registry;
use crate::manifest::loader;

#[derive(Args)]
pub struct UninstallArgs {
    /// Component id(s) to uninstall
    #[arg(required = true)]
    pub components: Vec<String>,

    /// Skip the is_reversible refusal and dependency-check refusal
    #[arg(long)]
    pub force: bool,

    /// Also uninstall any installed components that depend on the target(s)
    #[arg(long)]
    pub cascade: bool,

    /// Skip confirmation prompts
    #[arg(short = 'y', long)]
    pub yes: bool,
}

pub fn run(args: UninstallArgs) -> Result<()> {
    let manifest = loader::load().context("loading manifest")?;
    let registry = Registry::build();
    registry.validate_against(&manifest)?;

    // Resolve target set, applying --cascade if requested.
    let targets = resolve_targets(&manifest, &registry, &args.components, args.cascade)?;

    if !args.yes {
        println!(
            "Will uninstall: {}",
            targets.iter().cloned().collect::<Vec<_>>().join(", ")
        );
        let confirmed = inquire::Confirm::new("Proceed?").with_default(false).prompt()?;
        if !confirmed {
            println!("{}", style("Cancelled.").yellow());
            return Ok(());
        }
    }

    // Uninstall in reverse topo order.
    let ordered = reverse_topo(&manifest, &targets)?;
    for id in ordered {
        let c = registry.get(&id)?;
        // is_reversible check
        if !c.is_reversible() && !args.force {
            println!(
                "{} {} not reversible — use --force to confirm destructive removal",
                style("✗").red().bold(),
                id
            );
            continue;
        }
        // dep-blocking check (unless cascade already handled it, or --force)
        if !args.cascade && !args.force {
            if let Some(blockers) = find_dependents_that_are_installed(&manifest, &registry, &id)? {
                println!(
                    "{} {} has installed dependents: {}. Use --cascade or --force.",
                    style("✗").red().bold(),
                    id,
                    blockers.join(", ")
                );
                continue;
            }
        }
        match c.uninstall() {
            Ok(()) => println!("{} {} uninstalled", style("✓").green().bold(), id),
            Err(e) => println!(
                "{} {} failed: {}",
                style("✗").red().bold(),
                id,
                style(e).dim()
            ),
        }
    }

    Ok(())
}

fn resolve_targets(
    manifest: &crate::manifest::schema::Manifest,
    _registry: &Registry,
    explicit: &[String],
    cascade: bool,
) -> Result<std::collections::BTreeSet<String>> {
    use std::collections::BTreeSet;
    let mut out: BTreeSet<String> = explicit.iter().cloned().collect();
    for id in explicit {
        if !manifest.components.iter().any(|c| c.id == *id) {
            anyhow::bail!("unknown component: {}", id);
        }
    }
    if cascade {
        // Pull in all installed dependents, transitively.
        let mut frontier: Vec<String> = explicit.to_vec();
        while let Some(id) = frontier.pop() {
            for c in &manifest.components {
                if c.depends_on.iter().any(|d| d == &id) && out.insert(c.id.clone()) {
                    frontier.push(c.id.clone());
                }
            }
        }
    }
    Ok(out)
}

fn reverse_topo(
    manifest: &crate::manifest::schema::Manifest,
    targets: &std::collections::BTreeSet<String>,
) -> Result<Vec<String>> {
    let mut ordered = crate::manifest::resolver::topo_sort(manifest, targets)?;
    ordered.reverse();
    Ok(ordered)
}

fn find_dependents_that_are_installed(
    manifest: &crate::manifest::schema::Manifest,
    registry: &Registry,
    id: &str,
) -> Result<Option<Vec<String>>> {
    let mut installed_dependents = Vec::new();
    for c in &manifest.components {
        if c.depends_on.iter().any(|d| d == id) {
            let comp = registry.get(&c.id)?;
            if comp.is_installed().unwrap_or(false) {
                installed_dependents.push(c.id.clone());
            }
        }
    }
    if installed_dependents.is_empty() {
        Ok(None)
    } else {
        Ok(Some(installed_dependents))
    }
}
```

- [ ] **Step 2: Wire into CLI**

Modify `cli/src/commands/mod.rs`:

```rust
pub mod check;
pub mod dotfiles;
pub mod install;
pub mod interactive;
pub mod uninstall;
pub mod update;
```

Add to the `Commands` enum:

```rust
    /// Remove components
    Uninstall(uninstall::UninstallArgs),
```

Modify `cli/src/main.rs` `run_command`:

```rust
fn run_command(cmd: Commands) -> Result<()> {
    match cmd {
        Commands::Install(args) => commands::install::run(args),
        Commands::Uninstall(args) => commands::uninstall::run(args),
        Commands::Dotfiles(args) => commands::dotfiles::run(args),
        Commands::Check(args) => commands::check::run(args),
        Commands::Update(args) => commands::update::run(args),
    }
}
```

- [ ] **Step 3: Build**

Run: `cd cli && cargo build`
Expected: compiles.

- [ ] **Step 4: Commit**

```bash
git add cli/src/commands/ cli/src/main.rs
git commit -m "feat(uninstall): new command with is_reversible + cascade handling"
```

### Task 11.2: Unit tests for `resolve_targets` and `find_dependents_that_are_installed`

**Files:**
- Modify: `cli/src/commands/uninstall.rs`

- [ ] **Step 1: Add tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::schema::{ComponentSpec, Manifest, ProfileSpec};
    use std::collections::BTreeMap;

    fn mk() -> Manifest {
        Manifest {
            components: vec![
                ComponentSpec { id: "apt".into(), display_name: "APT".into(), ..Default::default() },
                ComponentSpec {
                    id: "docker".into(),
                    display_name: "Docker".into(),
                    depends_on: vec!["apt".into()],
                    ..Default::default()
                },
            ],
            profiles: BTreeMap::new(),
        }
    }

    #[test]
    fn cascade_pulls_in_dependents() {
        let m = mk();
        let reg = Registry::build();
        let set = resolve_targets(&m, &reg, &["apt".into()], true).unwrap();
        assert!(set.contains("apt"));
        assert!(set.contains("docker"));
    }

    #[test]
    fn no_cascade_keeps_target_only() {
        let m = mk();
        let reg = Registry::build();
        let set = resolve_targets(&m, &reg, &["apt".into()], false).unwrap();
        assert_eq!(set.len(), 1);
        assert!(set.contains("apt"));
    }

    #[test]
    fn unknown_component_errors() {
        let m = mk();
        let reg = Registry::build();
        assert!(resolve_targets(&m, &reg, &["ghost".into()], false).is_err());
    }
}
```

- [ ] **Step 2: Run**

Run: `cd cli && cargo test -p setup commands::uninstall`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add cli/src/commands/uninstall.rs
git commit -m "test(uninstall): cover resolve_targets cascade"
```

### Task 11.3: Smoke test

- [ ] **Step 1: Dry-run via `--help` to check wiring**

Run: `cd cli && cargo run -- uninstall --help`
Expected: help text shows `--force`, `--cascade`, `-y`.

No commit.

---

## Phase 12 — `doctor` command

### Task 12.1: Command skeleton

**Files:**
- Create: `cli/src/commands/doctor.rs`
- Modify: `cli/src/commands/mod.rs`
- Modify: `cli/src/main.rs`

- [ ] **Step 1: Write skeleton**

Create `cli/src/commands/doctor.rs`:

```rust
//! `setup doctor` — read-only health and drift report.

use anyhow::{Context, Result};
use clap::Args;
use console::style;
use std::collections::BTreeSet;

use crate::components::registry::Registry;
use crate::manifest::{intent, loader};

#[derive(Args)]
pub struct DoctorArgs {
    /// Active profiles to check against (overrides ~/.config/setup/active.toml)
    #[arg(long = "profile")]
    pub profiles: Vec<String>,

    /// Run each installed component's verify() method
    #[arg(long)]
    pub verify: bool,

    /// Force exit 0 even when issues are found
    #[arg(long = "warn-only")]
    pub warn_only: bool,
}

#[derive(Debug, Default)]
struct Report {
    /// PATH, symlinks, dotfiles, verify — independent of profile intent.
    machine_findings: Vec<Finding>,
    /// declared-missing, installed-not-declared — depend on active set.
    drift_findings: Vec<Finding>,
    /// True when no active set was resolvable; drift_findings is empty in this case.
    drift_skipped: bool,
}

#[derive(Debug, Clone)]
struct Finding {
    severity: Severity,
    subject: String,
    message: String,
    fix_hint: Option<String>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Severity {
    Ok,          // ✓
    Missing,     // ✗ (fails exit)
    Broken,      // ! (fails exit)
    Drift,       // ~ (informational)
    Info,        // ? (informational)
}

pub fn run(args: DoctorArgs) -> Result<()> {
    let manifest = loader::load().context("loading manifest")?;
    let registry = Registry::build();
    registry.validate_against(&manifest)?;

    let active_set = resolve_active_set(&manifest, &args.profiles)?;

    let mut report = Report::default();
    check_path(&mut report);
    check_dotfile_drift(&mut report);
    check_broken_symlinks(&mut report);

    match active_set {
        Some(set) => {
            check_declared_missing(&registry, &manifest, &set, &mut report);
            check_installed_not_declared(&registry, &manifest, &set, &mut report);
        }
        None => {
            report.drift_skipped = true;
        }
    }

    if args.verify {
        check_verify_installed(&registry, &manifest, &mut report);
    }

    print_report(&report);
    let exit_code = compute_exit(&report, args.warn_only);
    std::process::exit(exit_code);
}

fn resolve_active_set(
    manifest: &crate::manifest::schema::Manifest,
    explicit: &[String],
) -> Result<Option<BTreeSet<String>>> {
    if !explicit.is_empty() {
        let seeds = crate::manifest::resolver::expand_selection(manifest, explicit, &[])?;
        return Ok(Some(seeds));
    }
    let path = intent::default_path().context("no config dir")?;
    let i = intent::read(&path)?;
    let (valid, unknown) = intent::validated(&i, manifest);
    for u in &unknown {
        eprintln!(
            "{} intent file references unknown profile {:?} — ignoring",
            style("warn:").yellow(),
            u
        );
    }
    if valid.is_empty() {
        return Ok(None);
    }
    let seeds = crate::manifest::resolver::expand_selection(manifest, &valid, &[])?;
    Ok(Some(seeds))
}

fn check_path(report: &mut Report) {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return,
    };
    let local_bin = home.join(".local").join("bin");
    let path_var = std::env::var("PATH").unwrap_or_default();
    let found = path_var
        .split(':')
        .any(|seg| std::path::Path::new(seg) == local_bin);
    if !found {
        report.machine_findings.push(Finding {
            severity: Severity::Broken,
            subject: "~/.local/bin".into(),
            message: "not in PATH".into(),
            fix_hint: Some("add to .bashrc manually".into()),
        });
    } else {
        report.machine_findings.push(Finding {
            severity: Severity::Ok,
            subject: "PATH".into(),
            message: "ok".into(),
            fix_hint: None,
        });
    }
}

fn check_dotfile_drift(report: &mut Report) {
    // Reuses the existing dotfiles-diff logic. For the first pass, we call
    // into cli/src/commands/dotfiles.rs::diff_summary() (to be added).
    match crate::commands::dotfiles::diff_summary() {
        Ok(list) => {
            for (name, differs) in list {
                if differs {
                    report.machine_findings.push(Finding {
                        severity: Severity::Drift,
                        subject: format!(".{}", name),
                        message: "differs from repo".into(),
                        fix_hint: Some("setup dotfiles sync".into()),
                    });
                }
            }
        }
        Err(e) => {
            report.machine_findings.push(Finding {
                severity: Severity::Broken,
                subject: "dotfile scan".into(),
                message: format!("failed: {}", e),
                fix_hint: None,
            });
        }
    }
}

fn check_broken_symlinks(report: &mut Report) {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return,
    };
    let bin_dir = home.join(".local").join("bin");
    if !bin_dir.exists() {
        return;
    }
    let entries = match std::fs::read_dir(&bin_dir) {
        Ok(r) => r,
        Err(_) => return,
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_symlink() {
            // p.exists() follows the link; if it returns false, the link is broken.
            if !p.exists() {
                report.machine_findings.push(Finding {
                    severity: Severity::Broken,
                    subject: p.display().to_string(),
                    message: "dangling symlink".into(),
                    fix_hint: Some(format!("rm {}", p.display())),
                });
            }
        }
    }
}

fn check_declared_missing(
    registry: &Registry,
    manifest: &crate::manifest::schema::Manifest,
    active: &BTreeSet<String>,
    report: &mut Report,
) {
    for id in active {
        if let Ok(c) = registry.get(id) {
            let installed = c.is_installed().unwrap_or(false);
            if installed {
                let display = manifest
                    .components
                    .iter()
                    .find(|cs| cs.id == *id)
                    .map(|cs| cs.display_name.clone())
                    .unwrap_or_else(|| id.clone());
                report.drift_findings.push(Finding {
                    severity: Severity::Ok,
                    subject: id.clone(),
                    message: format!("{} installed", display),
                    fix_hint: None,
                });
            } else {
                report.drift_findings.push(Finding {
                    severity: Severity::Missing,
                    subject: id.clone(),
                    message: "declared, not installed".into(),
                    fix_hint: Some(format!("setup install {}", id)),
                });
            }
        }
    }
}

fn check_installed_not_declared(
    registry: &Registry,
    manifest: &crate::manifest::schema::Manifest,
    active: &BTreeSet<String>,
    report: &mut Report,
) {
    for cs in &manifest.components {
        if active.contains(&cs.id) {
            continue;
        }
        if let Ok(c) = registry.get(&cs.id) {
            if c.is_installed().unwrap_or(false) {
                report.drift_findings.push(Finding {
                    severity: Severity::Info,
                    subject: cs.id.clone(),
                    message: "installed, not in active profile".into(),
                    fix_hint: None,
                });
            }
        }
    }
}

fn check_verify_installed(
    registry: &Registry,
    manifest: &crate::manifest::schema::Manifest,
    report: &mut Report,
) {
    for cs in &manifest.components {
        if let Ok(c) = registry.get(&cs.id) {
            if c.is_installed().unwrap_or(false) {
                if let Err(e) = c.verify() {
                    report.machine_findings.push(Finding {
                        severity: Severity::Broken,
                        subject: cs.id.clone(),
                        message: format!("verify failed: {}", e),
                        fix_hint: None,
                    });
                }
            }
        }
    }
}

fn sev_symbol(s: Severity) -> &'static str {
    match s {
        Severity::Ok => "✓",
        Severity::Missing => "✗",
        Severity::Broken => "!",
        Severity::Drift => "~",
        Severity::Info => "?",
    }
}

fn print_report(r: &Report) {
    if r.drift_skipped {
        println!(
            "{} no active profiles — skipping profile-drift checks. Run 'setup install --profile <name>' or 'setup profile activate <name>' to declare intent.",
            style("info:").dim()
        );
    }
    for f in r.drift_findings.iter().chain(r.machine_findings.iter()) {
        let line = format!(
            "{} {:15} {}{}",
            style(sev_symbol(f.severity)),
            f.subject,
            f.message,
            f.fix_hint
                .as_ref()
                .map(|h| format!("       → {}", h))
                .unwrap_or_default()
        );
        println!("{}", line);
    }
}

fn compute_exit(r: &Report, warn_only: bool) -> i32 {
    if warn_only {
        return 0;
    }
    let any_fail = r
        .drift_findings
        .iter()
        .chain(r.machine_findings.iter())
        .any(|f| f.severity == Severity::Missing || f.severity == Severity::Broken);
    if any_fail {
        1
    } else {
        0
    }
}
```

- [ ] **Step 2: Wire into CLI**

Modify `cli/src/commands/mod.rs`:

```rust
pub mod check;
pub mod doctor;
pub mod dotfiles;
pub mod install;
pub mod interactive;
pub mod uninstall;
pub mod update;
```

Add to `Commands`:

```rust
    /// System health + drift report
    Doctor(doctor::DoctorArgs),
```

Add to `run_command`:

```rust
        Commands::Doctor(args) => commands::doctor::run(args),
```

- [ ] **Step 3: Build — `diff_summary` doesn't exist yet, expect a compile error**

Run: `cd cli && cargo build`
Expected: compile error about `dotfiles::diff_summary`. Phase 12.2 fixes it.

- [ ] **Step 4: (defer commit until 12.2 passes)**

### Task 12.2: Add `dotfiles::diff_summary` helper for doctor

**Files:**
- Modify: `cli/src/commands/dotfiles.rs`

- [ ] **Step 1: Find the existing diff-producing code**

Run: `grep -n "fn " cli/src/commands/dotfiles.rs`
Locate the function that implements `setup dotfiles diff`. Extract its core (without prints) into a helper:

```rust
/// Return a list of (dotfile_name, differs) pairs, reusing the internal
/// diff logic. Does not print anything.
pub fn diff_summary() -> anyhow::Result<Vec<(String, bool)>> {
    // Reuse the existing per-file diff function. Pseudocode — adapt to
    // the actual variable names in the current file.
    let managed = list_managed_dotfiles()?;  // already exists or extract
    let mut out = Vec::new();
    for name in managed {
        let differs = dotfile_differs_from_repo(&name)?;
        out.push((name, differs));
    }
    Ok(out)
}
```

If those inner functions don't exist yet, extract them from the existing `run` handler's code paths into private helpers in the same file.

- [ ] **Step 2: Build**

Run: `cd cli && cargo build`
Expected: compiles.

- [ ] **Step 3: Smoke test doctor**

Run: `cd cli && cargo run -- doctor --warn-only`
Expected: prints a report, exits 0 (warn-only). No crash.

- [ ] **Step 4: Commit 12.1 + 12.2 together**

```bash
git add cli/src/commands/
git commit -m "feat(doctor): health + drift report with severity split"
```

### Task 12.3: Doctor exit code integration test helper

**Files:**
- Modify: `cli/src/commands/doctor.rs`

- [ ] **Step 1: Split the exit out of `run` for testability**

Refactor `run` so it returns the `Report` instead of calling `std::process::exit`. Add a thin wrapper at the module level that calls `std::process::exit(compute_exit(&report, warn_only))`.

```rust
pub fn run(args: DoctorArgs) -> Result<()> {
    let report = build_report(&args)?;
    print_report(&report);
    let code = compute_exit(&report, args.warn_only);
    std::process::exit(code);
}

pub(crate) fn build_report(args: &DoctorArgs) -> Result<Report> {
    // ... the body of the old run(), minus print/exit
}
```

- [ ] **Step 2: Add unit test for `compute_exit`**

```rust
#[cfg(test)]
mod exit_tests {
    use super::*;

    #[test]
    fn ok_report_exits_zero() {
        let r = Report::default();
        assert_eq!(compute_exit(&r, false), 0);
    }

    #[test]
    fn missing_fails_exit() {
        let r = Report {
            drift_findings: vec![Finding {
                severity: Severity::Missing,
                subject: "x".into(),
                message: "m".into(),
                fix_hint: None,
            }],
            ..Default::default()
        };
        assert_eq!(compute_exit(&r, false), 1);
    }

    #[test]
    fn warn_only_forces_zero() {
        let r = Report {
            machine_findings: vec![Finding {
                severity: Severity::Broken,
                subject: "x".into(),
                message: "m".into(),
                fix_hint: None,
            }],
            ..Default::default()
        };
        assert_eq!(compute_exit(&r, true), 0);
    }

    #[test]
    fn drift_skipped_does_not_fail_on_its_own() {
        let r = Report {
            drift_skipped: true,
            ..Default::default()
        };
        assert_eq!(compute_exit(&r, false), 0);
    }
}
```

- [ ] **Step 3: Run**

Run: `cd cli && cargo test -p setup commands::doctor`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add cli/src/commands/doctor.rs
git commit -m "test(doctor): exit-code mapping covers missing, warn-only, drift-skipped"
```

---

## Phase 13 — `list` command

### Task 13.1: `list` command

**Files:**
- Create: `cli/src/commands/list.rs`
- Modify: `cli/src/commands/mod.rs`, `cli/src/main.rs`

- [ ] **Step 1: Implement**

Create `cli/src/commands/list.rs`:

```rust
//! `setup list` — print the component catalog (optionally filtered).

use anyhow::{Context, Result};
use clap::Args;
use console::style;

use crate::manifest::loader;

#[derive(Args)]
pub struct ListArgs {
    /// Show only components in this profile
    #[arg(long)]
    pub profile: Option<String>,

    /// Show only components with this tag
    #[arg(long)]
    pub tag: Option<String>,
}

pub fn run(args: ListArgs) -> Result<()> {
    let manifest = loader::load().context("loading manifest")?;

    let in_profile: Option<std::collections::BTreeSet<String>> = if let Some(p) = &args.profile {
        let set = crate::manifest::resolver::expand_selection(&manifest, &[p.clone()], &[])?;
        Some(set)
    } else {
        None
    };

    println!("{}", style("Components:").bold());
    for c in &manifest.components {
        if let Some(ref set) = in_profile {
            if !set.contains(&c.id) {
                continue;
            }
        }
        if let Some(ref t) = args.tag {
            if !c.tags.contains(t) {
                continue;
            }
        }
        println!(
            "  {} {} {}",
            style(&c.id).cyan(),
            style(format!("({})", c.display_name)).dim(),
            if c.tags.is_empty() {
                String::new()
            } else {
                style(format!("[{}]", c.tags.join(","))).dim().to_string()
            }
        );
    }

    Ok(())
}
```

Wire in `mod.rs` + `main.rs` (same pattern as uninstall/doctor).

- [ ] **Step 2: Build + smoke test**

Run: `cd cli && cargo run -- list --profile server`
Expected: prints the server profile's components.

- [ ] **Step 3: Commit**

```bash
git add cli/src/commands/ cli/src/main.rs
git commit -m "feat(list): print component catalog with --profile / --tag filters"
```

---

## Phase 14 — `profile` subcommand

### Task 14.1: `profile` command skeleton

**Files:**
- Create: `cli/src/commands/profile.rs`
- Modify: `cli/src/commands/mod.rs`, `cli/src/main.rs`

- [ ] **Step 1: Implement**

Create `cli/src/commands/profile.rs`:

```rust
//! `setup profile` — inspect profiles and manage active.toml.

use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use console::style;

use crate::manifest::{intent, loader};

#[derive(Args)]
pub struct ProfileArgs {
    #[command(subcommand)]
    pub command: ProfileCmd,
}

#[derive(Subcommand)]
pub enum ProfileCmd {
    /// List known profiles
    List,
    /// Show resolved components for a profile
    Show { name: String },
    /// Add a profile to ~/.config/setup/active.toml (no install)
    Activate { name: String },
    /// Remove a profile from ~/.config/setup/active.toml (no uninstall)
    Deactivate { name: String },
}

pub fn run(args: ProfileArgs) -> Result<()> {
    let manifest = loader::load().context("loading manifest")?;

    match args.command {
        ProfileCmd::List => {
            println!("{}", style("Profiles:").bold());
            for (name, p) in &manifest.profiles {
                let desc = if p.description.is_empty() {
                    String::new()
                } else {
                    format!(" — {}", p.description)
                };
                println!("  {}{}", style(name).cyan(), style(desc).dim());
            }
        }
        ProfileCmd::Show { name } => {
            let set = crate::manifest::resolver::expand_selection(&manifest, &[name.clone()], &[])?;
            println!("{}", style(format!("Components in profile {}:", name)).bold());
            for id in &set {
                println!("  {}", id);
            }
        }
        ProfileCmd::Activate { name } => {
            if !manifest.profiles.contains_key(&name) {
                anyhow::bail!("unknown profile: {}", name);
            }
            let path = intent::default_path().context("no config dir")?;
            let mut i = intent::read(&path)?;
            intent::union_add(&mut i, &[name.clone()]);
            intent::write(&path, &i)?;
            println!("{} active_profiles = {:?}", style("✓").green().bold(), i.active_profiles);
        }
        ProfileCmd::Deactivate { name } => {
            let path = intent::default_path().context("no config dir")?;
            let mut i = intent::read(&path)?;
            intent::remove(&mut i, &name);
            intent::write(&path, &i)?;
            println!("{} active_profiles = {:?}", style("✓").green().bold(), i.active_profiles);
        }
    }

    Ok(())
}
```

Wire in `mod.rs` + `main.rs`.

- [ ] **Step 2: Smoke tests**

Run:
```
cd cli && cargo run -- profile list
cd cli && cargo run -- profile show server
```
Expected: lists / resolves without error.

- [ ] **Step 3: Commit**

```bash
git add cli/src/commands/ cli/src/main.rs
git commit -m "feat(profile): list, show, activate, deactivate"
```

### Task 14.2: Activate-deactivate integration test

**Files:**
- Modify: `cli/src/commands/profile.rs`

- [ ] **Step 1: Add test using SETUP_INTENT env override**

Append to `profile.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn tmp_intent() -> PathBuf {
        let p = std::env::temp_dir().join(format!("setup-profile-test-{}.toml", std::process::id()));
        if p.exists() {
            std::fs::remove_file(&p).unwrap();
        }
        std::env::set_var("SETUP_INTENT", &p);
        p
    }

    #[test]
    fn activate_then_deactivate_roundtrip() {
        let p = tmp_intent();
        // Activate
        run(ProfileArgs {
            command: ProfileCmd::Activate { name: "server".into() },
        })
        .unwrap();
        let i = intent::read(&p).unwrap();
        assert!(i.active_profiles.contains(&"server".to_string()));
        // Deactivate
        run(ProfileArgs {
            command: ProfileCmd::Deactivate { name: "server".into() },
        })
        .unwrap();
        let i = intent::read(&p).unwrap();
        assert!(!i.active_profiles.contains(&"server".to_string()));
    }
}
```

- [ ] **Step 2: Run**

Run: `cd cli && cargo test -p setup commands::profile -- --test-threads=1`
(Serial needed because the test mutates a process-wide env var.)
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add cli/src/commands/profile.rs
git commit -m "test(profile): activate/deactivate roundtrip"
```

### Task 14.3: Unknown-profile error path

- [ ] **Step 1: Add test**

Append:

```rust
    #[test]
    fn activate_unknown_profile_errors() {
        tmp_intent();
        let err = run(ProfileArgs {
            command: ProfileCmd::Activate { name: "ghost".into() },
        })
        .unwrap_err();
        assert!(err.to_string().contains("unknown profile"));
    }
```

Run: `cd cli && cargo test -p setup commands::profile -- --test-threads=1`
Expected: PASS.

```bash
git add cli/src/commands/profile.rs
git commit -m "test(profile): activate refuses unknown profile"
```

---

## Phase 15 — Deprecate `check` command

### Task 15.1: Forward `check` to `doctor` with a deprecation notice

**Files:**
- Modify: `cli/src/commands/check.rs`

- [ ] **Step 1: Replace `run` body**

Replace the body of `check::run` with:

```rust
pub fn run(_args: CheckArgs) -> Result<()> {
    eprintln!(
        "{} `setup check` is deprecated — use `setup doctor`. Forwarding.",
        console::style("warn:").yellow()
    );
    super::doctor::run(super::doctor::DoctorArgs {
        profiles: vec![],
        verify: false,
        warn_only: false,
    })
}
```

You may delete the rest of the check.rs body (the old check_tools/check_dotfiles helpers) if nothing else references them. Confirm with `grep`.

- [ ] **Step 2: Build + run**

Run: `cd cli && cargo run -- check`
Expected: prints deprecation notice, then doctor output.

- [ ] **Step 3: Commit**

```bash
git add cli/src/commands/check.rs
git commit -m "refactor(check): deprecate and forward to doctor"
```

---

## Phase 16 — Docker integration tests

### Task 16.1: Add profile-aware test script invocation

**Files:**
- Modify: `tests/docker/test_installs.sh`

- [ ] **Step 1: Read existing script**

Run: `cat tests/docker/test_installs.sh`
Note the structure — it currently calls individual install commands.

- [ ] **Step 2: Add a new test block after the existing tool checks**

Insert near the end of the script, before the "Skipped Tests" block:

```bash
# ---------- new: profile-based install (docker-testable subset) ----------
echo ""
echo "--- Test: install --profile server --dry-run ---"
$SETUP_BIN install --profile server --dry-run
check_exit $? "install --profile server --dry-run exits 0"

echo ""
echo "--- Test: doctor --warn-only (no intent) ---"
$SETUP_BIN doctor --warn-only
check_exit $? "doctor --warn-only exits 0"

echo ""
echo "--- Test: profile activate / deactivate ---"
SETUP_INTENT="$HOME/.config/setup/active.toml"
rm -f "$SETUP_INTENT"
$SETUP_BIN profile activate server
grep -q 'server' "$SETUP_INTENT" && echo -e "${GREEN}[PASS]${NC} activate wrote server" || { echo -e "${RED}[FAIL]${NC} activate did not write server"; FAILED=$((FAILED+1)); }
$SETUP_BIN profile deactivate server
! grep -q 'server' "$SETUP_INTENT" && echo -e "${GREEN}[PASS]${NC} deactivate removed server" || { echo -e "${RED}[FAIL]${NC} deactivate did not remove"; FAILED=$((FAILED+1)); }

echo ""
echo "--- Test: doctor --profile server (fresh, expect failures for docker/monitoring/backup) ---"
$SETUP_BIN doctor --profile server || true  # expected to exit 1
```

Add a `check_exit()` helper near the top of the script if not present:

```bash
check_exit() {
  if [ "$1" -eq 0 ]; then
    echo -e "${GREEN}[PASS]${NC} $2"
    PASSED=$((PASSED+1))
  else
    echo -e "${RED}[FAIL]${NC} $2 (exit $1)"
    FAILED=$((FAILED+1))
  fi
}
```

- [ ] **Step 3: Build Docker image locally and run**

Run:
```
cd /home/al/git/setup
./tests/docker/run_tests.sh
```
Expected: existing tests still pass; the new tests pass.

- [ ] **Step 4: Commit**

```bash
git add tests/docker/test_installs.sh
git commit -m "test(docker): cover profile install, doctor, and profile activate/deactivate"
```

### Task 16.2: User-manifest override test

**Files:**
- Create: `tests/docker/fixtures/user-manifest.toml`
- Modify: `tests/docker/test_installs.sh`

- [ ] **Step 1: Add fixture**

Create `tests/docker/fixtures/user-manifest.toml`:

```toml
# Test fixture: redefines workstation to exclude `gh`, adds a new profile.

[profiles.workstation]
description = "Workstation (test-override)"
extends = ["base"]
components = ["lazygit", "just", "jq"]

[profiles.minimal]
components = ["apt"]
```

- [ ] **Step 2: Add test to the script**

Before the "Skipped Tests" block:

```bash
echo ""
echo "--- Test: user-manifest override ---"
mkdir -p "$HOME/.config/setup"
cp /setup/tests/docker/fixtures/user-manifest.toml "$HOME/.config/setup/manifest.toml"
$SETUP_BIN profile show workstation | grep -q 'lazygit' && echo -e "${GREEN}[PASS]${NC} override workstation has lazygit" || { echo -e "${RED}[FAIL]${NC} override workstation missing lazygit"; FAILED=$((FAILED+1)); }
$SETUP_BIN profile show workstation | grep -q 'gh' && { echo -e "${RED}[FAIL]${NC} override workstation still has gh"; FAILED=$((FAILED+1)); } || echo -e "${GREEN}[PASS]${NC} override workstation excludes gh"
$SETUP_BIN profile show minimal | grep -q 'apt' && echo -e "${GREEN}[PASS]${NC} new minimal profile works" || { echo -e "${RED}[FAIL]${NC} new minimal profile missing"; FAILED=$((FAILED+1)); }
rm -f "$HOME/.config/setup/manifest.toml"
```

Ensure the Dockerfile copies `tests/docker/fixtures/` into the container at `/setup/tests/docker/fixtures/`. Inspect `tests/docker/Dockerfile`:

```bash
grep -n COPY tests/docker/Dockerfile
```

If fixtures are missing, add near the existing COPY lines:

```dockerfile
COPY tests/docker/fixtures/ /setup/tests/docker/fixtures/
```

- [ ] **Step 3: Run**

Run: `./tests/docker/run_tests.sh`
Expected: all pass including the new override assertions.

- [ ] **Step 4: Commit**

```bash
git add tests/docker/
git commit -m "test(docker): user-manifest override covers add + redefine"
```

### Task 16.3: Contract test toggle

**Files:**
- Create: `cli/tests/contract.rs`

- [ ] **Step 1: Write the integration test**

Create `cli/tests/contract.rs`:

```rust
//! Component contract test: for the subset of components that are both
//! docker-testable and reversible, assert install → is_installed true,
//! uninstall → is_installed false. Runs only when SETUP_CONTRACT_TESTS=1.

#[test]
fn contract_install_uninstall_roundtrip() {
    if std::env::var("SETUP_CONTRACT_TESTS").ok().as_deref() != Some("1") {
        eprintln!("SETUP_CONTRACT_TESTS != 1 — skipping.");
        return;
    }

    use setup::components::registry::Registry;
    use setup::manifest::loader;

    let manifest = loader::load().expect("manifest");
    let registry = Registry::build();

    for spec in &manifest.components {
        if !spec.docker_testable() {
            eprintln!("[skip] {} (not docker_testable)", spec.id);
            continue;
        }
        let c = registry.get(&spec.id).expect("registered");
        if !c.is_reversible() {
            eprintln!("[skip] {} (not reversible)", spec.id);
            continue;
        }
        // Only meaningful if we can actually get to a clean state first.
        // In the Docker harness this test runs on a fresh container.
        c.install().unwrap_or_else(|e| panic!("{}: install failed: {}", spec.id, e));
        assert!(
            c.is_installed().unwrap_or(false),
            "{}: is_installed false after install",
            spec.id
        );
        c.uninstall().unwrap_or_else(|e| panic!("{}: uninstall failed: {}", spec.id, e));
        assert!(
            !c.is_installed().unwrap_or(true),
            "{}: still installed after uninstall",
            spec.id
        );
    }
}
```

For the `setup` crate to be importable from `tests/`, it must expose a library target. Split `src/main.rs` into a thin binary + a library:

1. Create `cli/src/lib.rs`:

```rust
//! setup — Personal development environment setup CLI (library entry point).
//! The binary `main.rs` delegates here so integration tests can reach internals.

pub mod commands;
pub mod components;
pub mod config;
pub mod manifest;
pub mod system;
pub mod ui;
```

2. Replace `cli/src/main.rs` with:

```rust
use anyhow::Result;
use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use setup::commands::{self, Cli, Commands};
use setup::{components, manifest};

fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    if let (Ok(m), r) = (manifest::loader::load(), components::registry::Registry::build()) {
        if let Err(e) = r.validate_against(&m) {
            eprintln!("warning: manifest/registry drift:\n{}\n", e);
        }
    }

    let cli = Cli::parse();
    match cli.command {
        Some(cmd) => run_command(cmd),
        None => commands::interactive::run(),
    }
}

fn run_command(cmd: Commands) -> Result<()> {
    match cmd {
        Commands::Install(args) => commands::install::run(args),
        Commands::Uninstall(args) => commands::uninstall::run(args),
        Commands::Dotfiles(args) => commands::dotfiles::run(args),
        Commands::Doctor(args) => commands::doctor::run(args),
        Commands::List(args) => commands::list::run(args),
        Commands::Profile(args) => commands::profile::run(args),
        Commands::Check(args) => commands::check::run(args),
        Commands::Update(args) => commands::update::run(args),
    }
}
```

3. Add `[lib]` to `cli/Cargo.toml` (between `[package]` and `[dependencies]`):

```toml
[lib]
name = "setup"
path = "src/lib.rs"

[[bin]]
name = "setup"
path = "src/main.rs"
```

This is a one-time refactor; subsequent tasks that reference modules can use either `crate::...` (from within the lib) or `setup::...` (from tests).

- [ ] **Step 2: Build**

Run: `cd cli && cargo build --tests`
Expected: compiles.

- [ ] **Step 3: Run test without the flag — should skip**

Run: `cd cli && cargo test -p setup --test contract`
Expected: PASS with skip message.

- [ ] **Step 4: Gate the contract test into Docker runs**

Modify `tests/docker/test_installs.sh` — add near the top of the install block:

```bash
echo ""
echo "--- Test: component contract suite ---"
SETUP_CONTRACT_TESTS=1 $CARGO_TEST_BIN || { echo -e "${RED}[FAIL]${NC} contract"; FAILED=$((FAILED+1)); }
```

Where `$CARGO_TEST_BIN` is either: (a) pre-built test binary copied into the image, or (b) a `cargo test` call if the container has Rust. The existing test harness uses `$SETUP_BIN` which is the release binary — it doesn't have Rust. Two options:

- Option A: Build the test binary in CI/Dockerfile and copy it in.
- Option B: Run the contract test from the *host*, not the container, pointing at the container's `bootstrap/manifest.toml` via `SETUP_MANIFEST`.

For this plan, pick **Option B** (simpler): run the contract test from the host as part of `run_tests.sh`, before/after the container tests. Update `run_tests.sh`:

```bash
# Run component contract suite (host-side, exercises Rust impls directly).
echo "--- Host: component contract suite ---"
(cd cli && SETUP_CONTRACT_TESTS=1 SETUP_MANIFEST="$PWD/../bootstrap/manifest.toml" cargo test --test contract -- --nocapture)
```

- [ ] **Step 5: Run**

Run: `./tests/docker/run_tests.sh`
Expected: host-side contract suite runs; container tests still pass.

- [ ] **Step 6: Commit**

```bash
git add cli/ tests/docker/
git commit -m "test(contract): cross-component install/uninstall roundtrip (gated)"
```

### Task 16.4: Update `tests/docker/run_tests.sh` summary

No-op if the script already summarizes PASS/FAIL — verify and adjust.

- [ ] **Step 1: Verify exit code behavior**

Run: `./tests/docker/run_tests.sh && echo "script returned 0"`
Expected: exit 0 on success, non-zero on failure. If not, tighten the summary block.

No commit unless adjustments were needed.

### Task 16.5: Update `tests/docker/Dockerfile` if needed

- [ ] **Step 1: Verify all new file dependencies are present**

Run: `grep -n COPY tests/docker/Dockerfile`
Ensure: `bootstrap/manifest.toml` is in the image (the Dockerfile already copies the whole bootstrap tree; verify).

If the Dockerfile copies only specific files, add:

```dockerfile
COPY bootstrap/manifest.toml /setup/bootstrap/manifest.toml
```

- [ ] **Step 2: Rebuild image and re-run tests**

Run: `./tests/docker/run_tests.sh`
Expected: all pass.

- [ ] **Step 3: Commit if Dockerfile changed**

```bash
git add tests/docker/Dockerfile
git commit -m "chore(docker): ensure bootstrap/manifest.toml is in the test image"
```

---

## Phase 17 — Docs

### Task 17.1: Update README

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Rewrite the Usage and Available Components sections**

Replace the "Available Components" table with a reference to `setup list`, and the "Install Components" section with the new flag set:

```markdown
### Install Components

```bash
# Install a profile
setup install --profile server
setup install --profile workstation --profile ai-heavy    # compose

# Install specific components
setup install docker mise claude-code

# Preview without installing
setup install --profile server --dry-run

# Continue past failures
setup install --profile workstation --keep-going

# Roll back on first failure
setup install --profile server --rollback-on-failure

# Everything (non-interactive only)
setup install --all -y
```

### See what's available

```bash
setup list                              # all components
setup list --profile ai-heavy           # components in a profile
setup list --tag dev                    # components with a tag
setup profile list                      # known profiles
setup profile show server               # resolved components for a profile
```

### Health and drift

```bash
setup doctor                            # check declared profiles vs reality
setup doctor --profile workstation      # check a specific profile
setup doctor --verify                   # also run post-install verify()
setup doctor --warn-only                # never exit non-zero
```

### Remove components

```bash
setup uninstall claude-code
setup uninstall ssh-keys --force        # required for non-reversible components
setup uninstall docker --cascade        # also removes components that depend on docker
```

### Profile intent

`~/.config/setup/active.toml` records which profiles you've selected on this machine.
Doctor reads this when `--profile` isn't passed, so it knows what "should be here."

```bash
setup profile activate server           # add to active.toml (no install)
setup profile deactivate server         # remove from active.toml (no uninstall)
```
```

Also update the "Available Components" table row-style description to a short paragraph:

```markdown
## Available Components

The full catalog lives in [`bootstrap/manifest.toml`](bootstrap/manifest.toml).
Run `setup list` to see it with your local profile/tag filters applied.
See [plans/2026-04-18-manifest-architecture-design.md](plans/2026-04-18-manifest-architecture-design.md)
for the architecture.
```

- [ ] **Step 2: Commit**

```bash
git add README.md
git commit -m "docs(readme): describe profile-based install, doctor, uninstall, intent"
```

### Task 17.2: Update CHANGELOG

**Files:**
- Modify: `CHANGELOG.md`

- [ ] **Step 1: Add entry at the top**

Add a section under `## [Unreleased]` (create the heading if absent):

```markdown
## [Unreleased]

### Added

- Declarative component manifest (`bootstrap/manifest.toml`) — source of truth for components and profiles.
- Composable machine-shape profiles: `base`, `server`, `workstation`, `ai-heavy`. Use `--profile X --profile Y` to union them.
- Persistent intent file at `~/.config/setup/active.toml` — records which profiles you've activated so `setup doctor` knows what "should be here" without having to remember.
- Component dependency graph with topological install order and transitive auto-pull.
- `setup install --dry-run` previews a plan without side effects.
- `setup install --rollback-on-failure` uninstalls components that completed in this run when a later one fails (skips non-reversible).
- `setup install --keep-going` to continue past failures.
- `setup install --verify` runs a post-install sanity probe on each component.
- `setup uninstall` — first-class removal command, with `--force` for non-reversible components and `--cascade` for dependents.
- `setup doctor` — read-only health + drift report. Distinguishes machine-health checks (PATH, dotfiles, symlinks — always run) from profile-drift checks (declared vs installed — require intent).
- `setup list [--profile X] [--tag T]` — browse the catalog.
- `setup profile list / show / activate / deactivate` — inspect and manage active intent.
- Optional `~/.config/setup/manifest.toml` user override — reshape profiles or override component metadata (cannot introduce brand-new components without a Rust impl).

### Changed

- Components are now Rust structs implementing a `Component` trait, not enum variants. Dispatch is by id via `Registry::build()`.
- `setup check` is deprecated; it forwards to `setup doctor` with a notice.

### Removed

- `cli/src/system/packages/` module (install logic moved to `cli/src/components/<id>.rs`).
- Hardcoded `Component` enum in `cli/src/commands/install.rs`.

### Migration notes

Existing users: no action required. Your `setup install <component>` invocations still work. Consider:
- `setup profile activate workstation` (or `server`) so `setup doctor` has intent to compare against.
- `setup doctor` to see any drift.
```

- [ ] **Step 2: Commit**

```bash
git add CHANGELOG.md
git commit -m "docs(changelog): document manifest + profiles release"
```

---

## Self-review checklist

Run through this before declaring the plan ready to execute:

- [ ] **Spec coverage:** every numbered requirement in `plans/2026-04-18-manifest-architecture-design.md` §§1–12 maps to a task above.
- [ ] **Placeholders:** grep the plan for "TBD", "TODO", "fill in", "similar to". Fix any hits.
- [ ] **Type consistency:** `ComponentSpec` field names match between schema.rs, the manifest TOML, and the resolver references (id, display_name, depends_on, tags, requires_*, interactive).
- [ ] **Command consistency:** `setup install`, `setup uninstall`, `setup doctor`, `setup list`, `setup profile`, `setup check` all match the design's CLI surface (§7).
- [ ] **File paths:** every `Create:` / `Modify:` path is absolute from the repo root and exists (for modify) or is in a dir that will be created (for create).
- [ ] **Commands:** every `Run: ...` command can be pasted into a shell in the repo root without adjustment.

---

## Execution handoff

Plan complete and saved to `plans/2026-04-18-manifest-architecture-plan.md`. Two execution options:

1. **Subagent-Driven (recommended)** — a fresh subagent per task, two-stage review between tasks, fast iteration.
2. **Inline Execution** — work the plan in this session, batch checkpoints for review.

Which approach?
