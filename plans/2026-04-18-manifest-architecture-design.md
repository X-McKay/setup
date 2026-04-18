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
- Persistent install-state file. The system itself is the source of truth for "what's installed"; each component answers `is_installed()` by probing reality.
- A DSL for install steps. Install logic stays in Rust. The manifest describes *what exists and why*, not *how to install it*.
- Cross-distro support. This design assumes Ubuntu but is structured so distro detection can be layered on later without a rewrite.
- Auto-rollback of successful components on a mid-run failure (default is stop-and-leave; `--rollback-on-failure` is opt-in).

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

## 4. Data model

### 4.1 Manifest schema (TOML)

The repo ships `bootstrap/manifest.toml` as the default catalog. Users may optionally add `~/.config/setup/manifest.toml` to override or extend it.

```toml
# bootstrap/manifest.toml — repo default

[[components]]
id = "apt"
display_name = "Basic APT Packages"
description = "Core system packages (curl, git, build-essential)"
depends_on = []
tags = ["core"]
requires_sudo = true
skip_in_docker = false

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
| `skip_in_docker` | bool | no, default false | Omit from `--all` when running under Docker (e.g. components requiring systemd). |
| `interactive` | bool | no, default false | Component prompts for user input. Suppressed from non-interactive `--all` unless explicitly named. |

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

If `~/.config/setup/manifest.toml` exists, it's loaded and merged on top of the repo default:

- **Components merge by `id`.** If the user defines a component with an ID already present in the default, the user's entry replaces the default entry entirely (no field-level merge — simplest semantic, least surprise).
- **Profiles merge by name.** Same rule: user's profile replaces the default one entirely. To modify an existing profile, user copies and edits.
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

    /// Remove the component. Best-effort; documented scope limits per component.
    /// Used by `setup uninstall` and by `--rollback-on-failure`.
    fn uninstall(&self) -> anyhow::Result<()>;

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
3. Add one `registry.register(Arc::new(...))` line to `registry.rs`.

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
| `--rollback-on-failure` | On install failure, iterate components that completed in this run, reverse topo order, call `uninstall()` best-effort. Opt-in. |
| `--keep-going` | Continue past failures; print a summary at the end instead of stopping. |
| `--verify` | Call `verify()` after each successful install. Slower; not default. |
| `--yes` | Skip confirmation. Existing behavior. |

### 5.3 `setup uninstall`

- `setup uninstall <id>` — calls `uninstall()` on the target. If other installed components declare `depends_on = [<id>]`, refuse unless `--force` (ignore dependents) or `--cascade` (also uninstall dependents in reverse topo order).
- `setup uninstall --profile workstation` — uninstalls every component in that profile, reverse topo order, confirmation required.
- Each component's `uninstall()` docs its own scope limits (e.g. apt transitive deps stay; `gh auth` state stays).

## 6. Doctor

`setup doctor [--profile P]... [--verify] [--warn-only]`

When no `--profile` is passed, the "active set" is the union of every component that appears in any profile in the merged manifest. This makes doctor useful out-of-the-box without requiring the user to remember which profiles they installed.

Checks (in order):
1. **PATH sanity** — is `~/.local/bin` on `$PATH`? Flag duplicates or shadowing.
2. **Declared-but-missing** — for each component in the active profile(s), `is_installed()` must be true. If false, flag with a fix command.
3. **Installed-but-not-declared** — for each component in the registry not in the active profile(s), if `is_installed()` is true, flag informationally.
4. **Dotfile drift** — diff repo version vs installed. Reuses existing `dotfiles diff` logic.
5. **Broken symlinks** — scan `~/.local/bin` for dangling links.
6. **Optional `verify()`** — run each installed component's `verify()` when `--verify` is passed.

Output example:

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

Exit codes:
- `0` — no `✗` or `!` findings.
- `1` — any `✗` (missing) or `!` (broken).
- `~` and `?` are informational, do not fail.
- `--warn-only` forces `0`.

Doctor never modifies state. A future `doctor --fix` mode is an additive, separate design.

## 7. CLI surface

```
setup install [<component>...] [--profile P]... [--all] [--dry-run] [--rollback-on-failure] [--keep-going] [--verify] [--yes]
setup uninstall <component>... [--profile P] [--force] [--cascade]
setup doctor [--profile P]... [--verify] [--warn-only]
setup list [--profile P] [--tag T]                # lists registry contents
setup profile list                                # lists known profiles
setup profile show <name>                         # resolves + prints component list
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
- `setup install --dry-run --profile server` — exits 0, no filesystem changes (verified via `diff` of container state before/after).
- `setup install --profile server` inside fresh container — each component's `is_installed()` returns true afterward.
- `setup doctor` on fresh container → reports all declared-missing, exits 1.
- `setup doctor` after full install → exits 0.
- `setup uninstall <id>` — `is_installed()` returns false after.
- User-override manifest — mount a test `~/.config/setup/manifest.toml` that redefines an existing profile (e.g. moves `gh` out of `workstation` and into a new user-defined `minimal` profile). Confirm `setup profile show minimal` returns the expected component list and `setup install --profile workstation` no longer includes `gh`. (Override cannot add components whose IDs have no Rust implementation — that's a startup validation failure per §11.)

### Component contract test
A shared Rust test that, for every registered component, asserts the install-contract invariant: `install()` followed by `is_installed()` → true; `uninstall()` followed by `is_installed()` → false. Runs only inside Docker (destructive), gated by an env flag (`SETUP_CONTRACT_TESTS=1`).

## 11. Risks and mitigations

| Risk | Mitigation |
|---|---|
| Manifest ID drift from Rust registry key | Startup validation: every manifest `id` must resolve to a registered component; every registered component must have a manifest entry. Fail loudly on mismatch. |
| `is_installed()` probes are slow on full doctor runs | Probes are cheap (single `which` call or file existence) by default. Heavier checks gated behind `--verify`. |
| Dep graph auto-pull surprises user | Print each auto-pulled component with its triggering dependent before starting the install. |
| `uninstall()` leaves side effects (apt transitive deps, group membership, auth state) | Documented per-component in the component's Rust file. `setup uninstall` prints scope limits before confirmation. |
| Partial migration leaves two dispatch paths | Full migration is scoped for this sub-project. No component ships with both paths. |
| User manifest shadows a default component in a broken way | Every override passes the same validation as the default (dep references resolve, no cycles). Invalid override fails fast at load time. |

## 12. Follow-ups (deferred, out of scope for this design)

- **Sub-project 2:** AI tooling installers + `.claude` / `.agents` seed configs. Plugs into this architecture as new components.
- Cross-distro support (distro detection, per-distro install bodies).
- `doctor --fix` interactive remediation.
- Finer-grained user-override merge (`components.add`, `components.exclude`).
- Parallel execution of independent DAG nodes.
- Persistent state file tracking install history and versions.
- Tag-based component selection (`setup install --tag ai`).
