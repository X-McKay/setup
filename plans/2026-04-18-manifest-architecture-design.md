# Manifest-Driven Architecture + Profiles — Design

**Date:** 2026-04-18
**Scope:** Sub-project 1 of the 2026 refresh. Covers: declarative component manifest, user-override layer, composable profiles, component dependency graph, `--dry-run`, per-component `uninstall` (also used for `--rollback-on-failure`), `setup doctor`, full migration of existing components.
**Out of scope (deferred):** AI tooling installers and `.claude`/`.agents` seed configs (Sub-project 2). Cross-distro package-manager abstraction (future).

---

## 1. Goals

- Make "what components exist, what depends on what, what's in which profile" declarative data, not hardcoded Rust.
- Support machine-shape profiles (`server`, `workstation`, `ai-heavy`, …) that compose via union.
- Give every component a uniform lifecycle contract: install, uninstall, is-installed, verify, dry-run.
- Add a read-only `setup doctor` that reports drift, health issues, and suggested fix commands.
- Make `setup uninstall <component>` a first-class command (also powers optional `--rollback-on-failure`).
- Keep all existing install logic. Only the dispatch/selection layer changes.

## 2. Non-goals

- Parallel component installation. Sequential, topologically ordered. Revisit only with a concrete pain.
- **Persistent install-state file.** The system itself is the source of truth for "what's installed"; each component answers `is_installed()` by probing reality. (Note: *intent* — which profiles the user picked — is a separate concern and is persisted; see §4.5. Install state is probe-derived; intent is user-declared. The two are distinct.)
- A DSL for install steps. Install logic stays in Rust. The manifest describes *what exists and why*, not *how to install it*.
- Cross-distro support. This design assumes Ubuntu but is structured so distro detection can be layered on later without a rewrite.
- Auto-rollback of successful components on a mid-run failure (default is stop-and-leave; `--rollback-on-failure` is opt-in).
- Component extension via user manifest. The user manifest may override default components and profiles, but cannot add a brand-new component whose ID has no Rust implementation. A real plugin mechanism is future work.
- Profile-level uninstall (`setup uninstall --profile X`). Safe set-difference uninstall with composable profiles is non-trivial and deferred to a future round. Single-component `uninstall <id>` is supported.

## 3. Answered design decisions

| Decision | Choice |
|---|---|
| Manifest shape | Catalog with metadata (id, deps, tags, profiles, flags). Install logic stays in Rust. |
| Profile composition | Composable union — multiple `--profile` flags merge component sets. |
| Rollback semantics | Per-component `uninstall()`, stop-and-leave as default; `--rollback-on-failure` opt-in. |
| Doctor scope | Drift + health + suggested fix commands. Read-only. Non-zero exit on any issue. |
| Migration | Full migration of all existing components this round. No permanent two-path system. |
| Execution model | Sequential, topologically ordered. |
| Install-state tracking | None. `is_installed()` probes reality each time. |
| Intent tracking | Persisted in `~/.config/setup/active.toml` (written by `setup install --profile`, read by `doctor` and `uninstall`). |
| Component reversibility | Two orthogonal axes: `uninstall()` (whether removal is *possible*) and `is_reversible()` (whether removal is *safe to automate*). SSH/GPG components typically implement `uninstall()` AND return `is_reversible() = false`: `setup uninstall ssh-keys --force` works; `--rollback-on-failure` skips them. See §4.4. |
| User manifest scope | Override/reshape only. Cannot introduce new component IDs without a Rust implementation. |

## 4. Data model

### 4.1 Manifest schema (TOML)

The repo ships `bootstrap/manifest.toml` as the default catalog. Users may optionally add `~/.config/setup/manifest.toml` to override existing entries (see §4.3). The user manifest is an override/reshaping layer — not an extension mechanism. Every component ID referenced must resolve to a Rust implementation at startup, or loading fails.

```toml
# bootstrap/manifest.toml — repo default

[[components]]
id = "apt"
display_name = "Basic APT Packages"
description = "Core system packages (curl, git, build-essential)"
depends_on = []
tags = ["core"]
requires_sudo = true

[[components]]
id = "mise"
display_name = "Mise Version Manager"
depends_on = ["apt"]
tags = ["dev"]

[[components]]
id = "claude-code"
display_name = "Claude Code"
depends_on = ["mise"]       # needs node runtime
tags = ["ai", "dev"]
interactive = true          # may prompt for auth; suppressed from --all non-interactive runs

[profiles.base]
description = "Always present; everything else extends this"
components = ["apt", "tools", "mise"]

[profiles.server]
description = "Headless server"
extends = ["base"]
components = ["docker", "gh", "neovim", "monitoring", "backup"]

[profiles.workstation]
description = "Desktop/laptop dev box"
extends = ["base"]
components = ["ghostty", "docker", "lazygit", "tpm", "neovim", "gh", "chromium", "obsidian"]

[profiles.ai-heavy]
description = "AI tooling layer; compose with server or workstation"
components = ["claude-code"]   # codex, gemini, aider, ollama added in Sub-project 2
```

**`[[components]]` fields:**

| Field | Type | Required | Meaning |
|---|---|---|---|
| `id` | string | yes | Unique ID. Matches Rust registry key. kebab-case. |
| `display_name` | string | yes | Human label for UI. |
| `description` | string | no | Longer blurb for `setup list`. |
| `depends_on` | list<string> | no | IDs that must be installed first. Feeds dep graph. |
| `tags` | list<string> | no | Free-form labels (`core`, `dev`, `ai`, `container`, …). Future filter selector. |
| `requires_sudo` | bool | no, default false | Component needs root. |
| `requires_systemd` | bool | no, default false | Component needs systemd (fails in Docker test harness). |
| `requires_privileged` | bool | no, default false | Component needs privileged container or host access. |
| `interactive` | bool | no, default false | Component prompts for user input. |

**Derived predicate — `docker_testable`:** A component is considered docker-testable iff `!requires_systemd && !requires_privileged && !interactive`. This predicate is computed once at manifest load and used consistently by:
- `--all` filtering under Docker (skips non-docker-testable components).
- The Docker integration test harness (installs only the docker-testable subset).
- The component contract test (runs only on components that are both docker-testable and reversible).

Keeping `docker_testable` as a derivation of the capability flags — rather than a separately-stored field — eliminates the drift risk where a stored flag disagrees with the underlying capabilities.

**Derived predicate — `non_interactive_installable`:** A component is installable in `--all --yes` runs iff `!interactive`. Interactive components must be named explicitly. (Kept separate from `docker_testable` because `interactive` is also a user-UX concern, not just a test-harness concern.)

**`[profiles.<name>]` fields:**

| Field | Type | Required | Meaning |
|---|---|---|---|
| `description` | string | no | Shown by `setup profile list`. |
| `extends` | list<string> | no | Transitively include another profile's components. |
| `components` | list<string> | yes | Component IDs this profile contributes. May be empty when `extends` supplies content. |

### 4.2 Profile resolution

1. Collect `--profile X --profile Y` flags from CLI. Also accept explicit `<component>` args.
2. For each profile name, transitively expand `extends` (cycle-detected).
3. Union resulting component ID sets. Dedup.
4. Validate every ID exists in the registry — unknown IDs fail fast with a usage hint.

### 4.3 User-override merge

If `~/.config/setup/manifest.toml` exists, it's loaded and merged on top of the repo default. This is an **override/reshaping layer**, not an extension mechanism:

- **Components merge by `id`.** If the user defines a component with an ID already present in the default, the user's entry replaces the default entry entirely (no field-level merge — simplest semantic, least surprise). If the user defines a component with an ID that has no Rust implementation, loading fails with a clear error. Adding genuinely new components requires code.
- **Profiles merge by name.** Same rule: user's profile replaces the default one entirely. To modify an existing profile, user copies and edits. Users may freely define new profiles as long as every referenced component ID resolves.
- Anything not redefined in the user file comes from the default.

Finer-grained merge (e.g. `components.add`, `components.exclude`) is a future extension if copy-paste friction becomes real.

### 4.4 Component trait (Rust)

```rust
pub trait Component: Send + Sync {
    /// Matches the manifest `id`.
    fn id(&self) -> &str;

    /// Is this component currently present on the system? Probes reality.
    /// Used by doctor and the installer's skip-if-already-present check.
    fn is_installed(&self) -> anyhow::Result<bool>;

    /// Install the component. MUST be idempotent — safe to re-run.
    fn install(&self) -> anyhow::Result<()>;

    /// Whether this component is safe for AUTOMATIC uninstall (i.e., by
    /// `--rollback-on-failure`). Orthogonal to whether `uninstall()` is implemented.
    ///
    /// - true (default): `--rollback-on-failure` may call `uninstall()` without extra confirmation.
    /// - false: `--rollback-on-failure` SKIPS this component with a printed notice.
    ///   The component may still implement `uninstall()` — it just won't be called automatically.
    ///   Override to false for components that manage user material (SSH keys, GPG keys, etc.)
    ///   where destruction on rollback would be a footgun.
    fn is_reversible(&self) -> bool { true }

    /// Remove the component. Best-effort; documented scope limits per component.
    ///
    /// Default implementation refuses — this is the safe default for components that
    /// truly cannot be cleanly uninstalled. Components that want `setup uninstall <id>`
    /// to work must override this.
    ///
    /// Called by:
    /// - `setup uninstall <id>` — always calls `uninstall()` if implemented (subject to
    ///   the `--force` check for non-reversible components below).
    /// - `--rollback-on-failure` — calls `uninstall()` only when `is_reversible()` is true.
    ///
    /// Components are expected to implement `uninstall()` whenever removal is *possible*,
    /// regardless of `is_reversible()`. The two axes are independent:
    ///
    /// | is_reversible | uninstall() overridden | `setup uninstall <id>`      | --rollback-on-failure |
    /// |---------------|------------------------|------------------------------|------------------------|
    /// | true          | yes                    | works                        | called                 |
    /// | false         | yes                    | requires --force             | skipped (with notice)  |
    /// | either        | no (default refuses)   | fails with "not removable"   | skipped (with notice)  |
    fn uninstall(&self) -> anyhow::Result<()> {
        anyhow::bail!(
            "{} does not implement uninstall — it cannot be removed by this tool",
            self.id()
        )
    }

    /// Optional: post-install sanity check (run --version, ping daemon, …).
    /// Called only when --verify is passed to `install` or `doctor`.
    fn verify(&self) -> anyhow::Result<()> {
        if self.is_installed()? {
            Ok(())
        } else {
            anyhow::bail!("{} not installed", self.id())
        }
    }

    /// Optional: describe what install WOULD do, for --dry-run.
    fn dry_run(&self) -> anyhow::Result<Vec<String>> {
        Ok(vec![format!("would install {}", self.id())])
    }
}
```

Each component lives at `cli/src/components/<id>.rs` as a unit struct implementing this trait. A `Registry::build()` function wires them into a `HashMap<String, Arc<dyn Component>>` at startup.

Adding a new component:
1. Add one `[[components]]` entry to `bootstrap/manifest.toml`.
2. Create `cli/src/components/<id>.rs` with a struct implementing `Component`.
   - Implement `id`, `is_installed`, `install`.
   - **Override `uninstall()`** if removal is technically possible. Skip only if the component genuinely cannot be uninstalled.
   - **Override `is_reversible()` to `false`** if automatic rollback would destroy user material (SSH keys, GPG keys, generated secrets). This is independent of whether `uninstall()` is implemented — ssh-keys will typically have both `uninstall()` implemented *and* `is_reversible() = false`, so `setup uninstall ssh-keys --force` works but `--rollback-on-failure` skips it.
   - Override `verify()` and `dry_run()` where meaningful.
3. Add one `registry.register(Arc::new(...))` line to `registry.rs`.

### 4.5 Intent persistence (`active.toml`)

Profile *intent* — which profiles the user selected for this machine — is persisted separately from install state. This is distinct from tracking "what's installed" (which remains probe-derived and non-persistent per §2).

File: `~/.config/setup/active.toml`

```toml
# Written by `setup install --profile X [--profile Y]`. Human-editable.
active_profiles = ["server", "ai-heavy"]
```

**Write semantics.** `active.toml` is updated by these paths with precise rules:

| Trigger | Behavior |
|---|---|
| `setup install --profile X [--profile Y]` completes **without error** (all selected components installed or already present) | **Union-add** X, Y to existing `active_profiles`. Existing entries preserved. |
| `setup install --profile X` completes **with `--keep-going`**, some components failed | **Union-add** X. Intent was declared by the user; doctor surfaces the still-missing components. Partial install ≠ invalid intent. |
| `setup install --profile X` **stops on failure** (no `--keep-going`) | **No change** to `active.toml`. Intent wasn't realized; user should fix and re-run. |
| `setup install --profile X --rollback-on-failure` triggers rollback | **No change** to `active.toml`. |
| `setup install <components>` (no `--profile`) | **No change** to `active.toml`. Ad-hoc installs don't alter declared intent. |
| `setup install --all` | **No change** to `active.toml`. `--all` is an override, not a declaration of long-term intent. |
| `setup profile activate <name>` | Add `<name>` to `active_profiles` (no install). |
| `setup profile deactivate <name>` | Remove `<name>` from `active_profiles` (no uninstall). |
| User edits the file directly | Honored as-is on next read. Invalid profile names produce a warning but don't fail the run (see §11). |

Who reads it:
- `setup doctor` with no `--profile` flag uses `active_profiles` as the active set for the profile-dependent checks (declared-but-missing, installed-but-not-declared). See §6 for precise fallback behavior — critically, profile-independent checks (PATH, dotfile drift, broken symlinks) are **not** affected by the absence of intent.
- `setup uninstall <id>` checks `active_profiles` to refine the "other components depend on this" warning.

If `--profile` is passed explicitly to `doctor`, the intent file is ignored for that run.

Rationale: Distinguishing user-declared *intent* (persisted) from system *state* (probed) lets doctor produce meaningful results on both declared-intent machines (server, workstation, etc.) and ad-hoc machines (no profile declared) without reintroducing the "state file lies" problem.

## 5. Runtime behavior

### 5.1 Dependency graph

1. Build a DAG from the selected components' `depends_on` edges.
2. **Transitive auto-pull:** if you select `docker` and `docker` depends on `apt`, `apt` is added automatically (with a printed note). Auto-pulled components are treated identically to explicitly selected ones for the rest of the run — they count toward `--rollback-on-failure` and toward the "installed this run" summary.
3. Cycle detection → fail fast with the cycle path.
4. Topological sort → deterministic, sequential install order.

### 5.2 Execution

For each node in topo order:
- Call `is_installed()`. If true, skip with `✓ already installed`.
- Else call `install()`.
- If `--verify` is set, call `verify()` after install.
- On failure: **stop.** Print the failed component, the error tail, and the still-pending list so the user can fix and resume. No auto-rollback.

Flags:

| Flag | Effect |
|---|---|
| `--dry-run` | Resolve + print plan + call each component's `dry_run()`. Zero side effects. Exits 0 if plan is valid. |
| `--rollback-on-failure` | On install failure, iterate components that completed in this run, reverse topo order, call `uninstall()` best-effort. **Skips components where `is_reversible()` returns false, with a printed notice.** Opt-in. |
| `--keep-going` | Continue past failures; print a summary at the end instead of stopping. |
| `--verify` | Call `verify()` after each successful install. Slower; not default. |
| `--yes` | Skip confirmation. Existing behavior. |

**Flag mutual exclusion:** `--keep-going` and `--rollback-on-failure` are mutually exclusive (parse-time error). Their failure models are incompatible — "continue through failures" and "undo on failure" describe opposite intents. A future design could combine them (e.g. "keep going, then rollback everything if any failed") but that's deferred.

### 5.3 `setup uninstall`

`setup uninstall <id>` calls the component's `uninstall()` method. Control flow:

1. **Does the component implement `uninstall()`?** (i.e., override the trait default.) If not, fail with `"<id>: not removable by this tool"` — no flag can force it, because there is no implementation to call. This is the case for components that truly can't be cleanly removed.
2. **Is `is_reversible()` false?** (ssh-keys, gpg, etc.) If yes, refuse unless `--force` is passed. The force flag acknowledges destruction of user material explicitly.
3. **Do other installed components declare `depends_on = [<id>]`?** If yes, refuse unless `--force` (ignore dependents) or `--cascade` (uninstall dependents first, reverse topo order).
4. Call `uninstall()`. Component's `uninstall()` docstring documents its own scope limits (apt transitive deps stay; `gh auth` state stays; etc.).

Matrix (summarizes the trait docs in §4.4):

| component shape | `setup uninstall <id>` | `setup uninstall <id> --force` | `--rollback-on-failure` |
|---|---|---|---|
| reversible, `uninstall()` implemented | works | works | called |
| non-reversible, `uninstall()` implemented (ssh-keys, gpg) | refuses | works | skipped (notice) |
| `uninstall()` not implemented | fails with "not removable" | fails with "not removable" | skipped (notice) |

**No `setup uninstall --profile` in v1.** With composable profiles and shared base dependencies, safe set-difference uninstall requires design work that's out of scope for this round (see §12). Uninstall one component at a time, or remove components explicitly.

## 6. Doctor

`setup doctor [--profile P]... [--verify] [--warn-only]`

### Check categories

Doctor's checks fall into two categories that are treated independently:

- **Machine-health checks** — profile-independent. Run always at full severity regardless of whether intent is declared.
- **Profile-drift checks** — profile-dependent. Require an active set to be meaningful; downgraded or skipped when no intent is declared.

| # | Check | Category |
|---|---|---|
| 1 | PATH sanity — is `~/.local/bin` on `$PATH`? Flag duplicates or shadowing. | Machine-health |
| 2 | Declared-but-missing — for each component in active set, `is_installed()` must be true. | Profile-drift |
| 3 | Installed-but-not-declared — for each component in the registry not in active set, if `is_installed()` is true, note informationally. | Profile-drift |
| 4 | Dotfile drift — diff repo version vs installed. | Machine-health |
| 5 | Broken symlinks — scan `~/.local/bin` for dangling links. | Machine-health |
| 6 | Optional `verify()` — run each installed component's `verify()` when `--verify` is passed. | Machine-health |

### Active-set resolution for profile-drift checks

1. If `--profile` is passed (one or more times), use the union of those profiles. Ignore the intent file.
2. Else, if `~/.config/setup/active.toml` exists and has non-empty `active_profiles`, use that.
3. Else, **skip profile-drift checks** (do not run checks 2 and 3 at all). Output a one-line note: `info: no active profiles — skipping profile-drift checks. Run 'setup install --profile <name>' or 'setup profile activate <name>' to declare intent.`

Machine-health checks **always run** regardless of intent declaration. A broken `PATH` or a dangling symlink is a real problem whether or not the user has declared a profile. Their individual severity mapping (below) is independent of intent.

### Severity mapping and exit codes

Each finding is tagged with a severity symbol:

| Symbol | Meaning | Contributes to exit 1? |
|---|---|---|
| `✓` | OK | no |
| `✗` | declared but missing | **yes** (profile-drift only) |
| `!` | broken / structural problem (PATH not set, dangling symlinks, `verify()` failed) | **yes** |
| `~` | drift that might be intentional (dotfile edited locally) | no |
| `?` | informational (installed but not in active profile) | no |

`--warn-only` forces exit `0` regardless.

### Output example (intent declared)

```
setup doctor --profile workstation

✓ apt           installed
✓ mise          installed
✗ claude-code   declared, not installed       → setup install claude-code
~ .bashrc       differs from repo             → setup dotfiles sync
! ~/.local/bin  not in PATH                   → add to .bashrc manually
? ollama        installed, not in active profile (informational)

Issues: 1 missing, 1 dotfile drift, 1 PATH warning
```

### Output example (no intent declared)

```
setup doctor

info: no active profiles — skipping profile-drift checks. Run 'setup install --profile <name>' or 'setup profile activate <name>' to declare intent.

! ~/.local/bin  not in PATH                   → add to .bashrc manually
~ .bashrc       differs from repo             → setup dotfiles sync

Issues: 1 PATH warning, 1 dotfile drift  (profile-drift checks skipped)
```

Note the second example still exits `1` because of the `!` PATH warning — machine health is evaluated independently of intent.

Doctor never modifies state. A future `doctor --fix` mode is an additive, separate design.

## 7. CLI surface

```
setup install [<component>...] [--profile P]... [--all] [--dry-run] [--rollback-on-failure] [--keep-going] [--verify] [--yes]
setup uninstall <component>... [--force] [--cascade]
setup doctor [--profile P]... [--verify] [--warn-only]
setup list [--profile P] [--tag T]                # lists registry contents
setup profile list                                # lists known profiles
setup profile show <name>                         # resolves + prints component list
setup profile activate <name>                     # add to ~/.config/setup/active.toml
setup profile deactivate <name>                   # remove from ~/.config/setup/active.toml
setup dotfiles {sync|diff|list|backup}            # unchanged
setup update [target]                             # unchanged
setup check ...                                   # DEPRECATED alias → doctor; keep for one release
```

`list`, `profile`, `uninstall`, and `doctor` are new. `install` gains new flags but keeps backward-compatible behavior when called with a single component name.

Flag combination rules for `install`:
- `--all` is mutually exclusive with `--profile` and explicit `<component>` args. Combining them is a parse-time error.
- Multiple `--profile` flags union (composable, per design).
- Explicit `<component>` args may be combined with `--profile` flags; the resulting selection set is the union.

## 8. Rust directory layout

```
cli/src/
├── main.rs
├── commands/
│   ├── install.rs         # uses registry + resolver
│   ├── uninstall.rs       # NEW
│   ├── doctor.rs          # NEW (supersedes most of check.rs)
│   ├── list.rs            # NEW
│   ├── profile.rs         # NEW
│   ├── dotfiles.rs        # unchanged
│   ├── update.rs          # unchanged
│   ├── interactive.rs     # updated to use registry
│   └── mod.rs
├── components/            # NEW — one file per component
│   ├── mod.rs             # Component trait
│   ├── registry.rs        # Registry::build() wires every component
│   ├── apt.rs
│   ├── tools.rs
│   ├── mise.rs
│   ├── docker.rs
│   ├── claude_code.rs
│   └── ... (27 total, one per existing component)
├── manifest/              # NEW
│   ├── mod.rs
│   ├── schema.rs          # serde structs for TOML
│   ├── loader.rs          # load repo manifest + merge user override
│   └── resolver.rs        # profile expand, dep graph, topo sort, cycle detect
├── config/                # existing
├── system/
│   └── health.rs          # existing, stays
└── ui/
    └── prompts.rs         # updated to work with registry IDs
```

The existing `cli/src/system/packages/` module is removed; its logic is redistributed into `cli/src/components/<id>.rs` files (one per component).

## 9. Migration path

Full migration in small, independently reviewable commits. Code compiles and tests pass after each step.

1. **Skeleton:** add `components/` with trait + empty registry; add `manifest/` with schema types + loader + resolver; add `bootstrap/manifest.toml` declaring all 27 existing components and three profiles (`base`, `server`, `workstation`). No behavior change yet.
2. **Port components one-by-one**, starting with `apt` (simplest). Each port = new `components/<id>.rs` wrapping the existing function as a struct method, then register it in the new registry. The old enum + match dispatch path remains the one actually used by `setup install` at this stage; the new registry is populated but inert. This lets each port be a tiny, test-verifiable commit.
3. **Swap dispatch** in `install.rs` — replace the enum match with `registry.get(id)`. Clap switches from `ValueEnum` to `String`, validated at runtime.
4. **Delete** the old `Component` enum and `cli/src/system/packages/` module once all components are ported.
5. **Replace `check.rs` with `doctor.rs`.** Keep `setup check` as a deprecated alias that prints a one-line notice and forwards to doctor.
6. **Add `uninstall` command** and implement `Component::uninstall` for each component. Document per-component scope limits.
7. **Add `list` + `profile` commands.**
8. **Update Docker tests** to exercise `install --profile`, `doctor`, `uninstall`, `--dry-run`, and user-manifest override.
9. **Update README + CHANGELOG.**

## 10. Testing strategy

### Unit tests (Rust, hermetic — no system side effects)
- Manifest parsing: valid fixtures; missing required fields; unknown fields tolerated with warning; invalid TOML fails with line/column.
- Profile resolution: `extends` transitive expansion, multi-profile union, dedup, unknown profile name fails fast, cycles in `extends` detected.
- Dep graph: transitive pull-in, cycle detection with cycle path in error, unknown `depends_on` ID fails fast, topo sort determinism.
- User override merge: component replace by ID, profile replace by name, missing user manifest is a no-op, empty user manifest is a no-op.

### Integration tests (Docker, existing test harness)

The existing test container runs without systemd or privileged mode, so the test harness operates on the `docker_testable` subset defined in §4.1. Profiles are filtered to that subset before installation; skipped components are reported by name in test output.

Tests:
- `setup install --dry-run --profile server` — exits 0, no filesystem changes (dry-run is safe on the full profile; it performs no install steps).
- `setup install --profile server` inside fresh container — installs only the `docker_testable` subset; each installed component's `is_installed()` returns true afterward.
- `setup doctor` with no intent file and a healthy system → exits 0; profile-drift checks skipped with hint; machine-health checks run clean.
- `setup doctor` with no intent file and a broken PATH → exits 1 (PATH is machine-health, runs regardless of intent).
- `setup doctor --profile server` on fresh container → reports all declared-missing, exits 1.
- `setup doctor --profile server` after install → reports the docker-skipped components as `✗` (declared, not installed). Test asserts the expected-skipped set matches.
- **active.toml write on successful install:** `setup install --profile server` on fresh container → `~/.config/setup/active.toml` exists and contains `active_profiles = ["server"]`. Subsequent `setup doctor` (no args) uses it and produces the same profile-drift output as `setup doctor --profile server`.
- **active.toml via profile activate:** starting from no active.toml, `setup profile activate server` (no install) writes `active_profiles = ["server"]` and does not install anything. Verifies the standalone command path.
- **active.toml union-add:** starting from `active_profiles = ["server"]`, run `setup install --profile ai-heavy` → file now contains both, order preserved.
- **active.toml not written on stop-on-failure:** with no `--keep-going`, inject a failure mid-install for `--profile server`; file is not modified.
- `setup uninstall <id>` for a reversible docker-testable component — `is_installed()` returns false after.
- `setup uninstall <ssh-keys>` — refused (non-reversible), exit non-zero; with `--force` succeeds.
- User-override manifest — mount a test `~/.config/setup/manifest.toml` that redefines an existing profile (e.g. moves `gh` out of `workstation` and into a new user-defined `minimal` profile). Confirm `setup profile show minimal` returns the expected component list. (Override cannot add components whose IDs have no Rust implementation — startup validation failure per §11.)

### Component contract test
A shared Rust test that iterates all registered components and, for the subset that is both `docker_testable` (per §4.1) and `is_reversible()`, asserts: `install()` → `is_installed()` true; `uninstall()` → `is_installed()` false. Components that fail either capability are skipped with an informative note (not failed). Runs only inside Docker, gated by env flag (`SETUP_CONTRACT_TESTS=1`).

## 11. Risks and mitigations

| Risk | Mitigation |
|---|---|
| Manifest ID drift from Rust registry key | Startup validation: every manifest `id` must resolve to a registered component; every registered component must have a manifest entry. Fail loudly on mismatch. |
| `is_installed()` probes are slow on full doctor runs | Probes are cheap (single `which` call or file existence) by default. Heavier checks gated behind `--verify`. |
| Dep graph auto-pull surprises user | Print each auto-pulled component with its triggering dependent before starting the install. |
| `uninstall()` leaves side effects (apt transitive deps, group membership, auth state) | Documented per-component in the component's Rust file. `setup uninstall` prints scope limits before confirmation. |
| Partial migration leaves two dispatch paths | Full migration is scoped for this sub-project. No component ships with both paths. |
| User manifest shadows a default component in a broken way | Every override passes the same validation as the default (dep references resolve, no cycles). Invalid override fails fast at load time. |
| `active.toml` intent file goes stale (profile removed from manifest) | At load time, validate every name in `active_profiles` resolves; unknown profile name produces a warning and is ignored for the current run. Doctor surfaces the mismatch. |
| Non-reversible components auto-uninstalled via `--rollback-on-failure` | `is_reversible()` defaults to true but must be explicitly set to false for components generating user material. Code review checklist for new components must include "review is_reversible choice." Rollback skips non-reversible components with a printed notice so the user knows manual cleanup may be needed. |
| Capability-filtering means docker tests don't cover everything | Docker subset is documented. A host-integration test (run on a real Ubuntu VM during release) covers the full set. VM harness is out of scope for this design but tracked as follow-up. |

## 12. Follow-ups (deferred, out of scope for this design)

- **Sub-project 2:** AI tooling installers + `.claude` / `.agents` seed configs. Plugs into this architecture as new components.
- Cross-distro support (distro detection, per-distro install bodies).
- `doctor --fix` interactive remediation.
- Finer-grained user-override merge (`components.add`, `components.exclude`).
- Parallel execution of independent DAG nodes.
- Persistent *install-state* file tracking install history and versions (separate from the intent file in §4.5).
- Tag-based component selection (`setup install --tag ai`).
- `setup uninstall --profile <name>` with safe set-difference semantics: compute the components uniquely in the named profile (not shared with any other currently-active profile) and uninstall only those. Requires careful UX around shared deps and `is_reversible` interactions.
- Combined `--keep-going --rollback-on-failure` (collect failures, rollback all successes if any failed).
- Plugin mechanism: allow user manifest to point at external Rust modules / shell scripts, expanding user override from reshaping into true extension.
- Host-integration test harness (real Ubuntu VM) to cover components that can't run in Docker.
