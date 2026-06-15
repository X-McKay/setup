# Manifest and Profiles

The setup CLI uses a TOML manifest to describe what components exist, how they depend on each other, and which components belong to each machine profile. Install logic still lives in Rust; the manifest is the catalog and selection layer.

## Manifest Location

The repo catalog lives at `bootstrap/manifest.toml`.

Users may optionally create `~/.config/setup/manifest.toml` to replace existing component or profile entries. This is an override layer, not a plugin mechanism: every referenced component ID must already have a Rust implementation.

## Components

Each `[[components]]` entry describes one installable component:

```toml
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
```

Common fields:

| Field | Meaning |
|---|---|
| `id` | Unique kebab-case ID. Must match a Rust component in `cli/src/components/`. |
| `display_name` | Human-readable name for list and prompt output. |
| `description` | Longer description shown by `setup list`. |
| `depends_on` | Component IDs that must be installed first. |
| `tags` | Free-form labels used for filtering and grouping. |
| `requires_sudo` | Component needs root privileges. |
| `requires_systemd` | Component needs systemd and is skipped in the Docker test harness. |
| `requires_privileged` | Component needs host or privileged-container access. |
| `interactive` | Component may prompt or require auth, so it is not included in non-interactive `--all` installs. |

Docker testability is derived from the capability flags. A component is Docker-testable when it does not require systemd, privileged access, or interactivity.

## Profiles

Profiles define reusable machine shapes. They can extend other profiles and compose by union when multiple `--profile` flags are passed.

```toml
[profiles.workstation]
description = "Desktop/laptop dev box"
extends = ["base"]
components = ["ghostty", "docker", "lazygit", "tpm", "neovim", "gh", "chromium", "obsidian"]
```

Profile resolution:

1. Expand every requested profile, including transitive `extends`.
2. Add explicitly named components, if any.
3. Pull in transitive component dependencies from `depends_on`.
4. Validate that every ID exists in both the manifest and Rust registry.
5. Install in deterministic topological order.

## User Overrides

If `~/.config/setup/manifest.toml` exists, it is loaded on top of the repo manifest.

Components merge by `id`: redefining an existing component replaces the repo entry completely.

Profiles merge by name: redefining a profile replaces the repo profile completely. Users may define new profiles as long as all referenced component IDs exist in the Rust registry.

Unknown component IDs, invalid dependency references, and profile cycles fail fast during manifest loading.

## Active Profile Intent

Profile intent is stored separately from install state:

```toml
# ~/.config/setup/active.toml
active_profiles = ["server", "ai-heavy"]
```

The system does not keep a persistent record of what is installed. Components answer that by probing the machine through `is_installed()`. The intent file only records what profiles the user says this machine should have.

Intent is updated by profile-oriented commands:

| Command | Effect |
|---|---|
| `setup install --profile <name>` | Adds the profile after a successful install. |
| `setup install --profile <name> --keep-going` | Adds the profile even if some components failed, so `setup doctor` can report what remains missing. |
| `setup profile activate <name>` | Adds profile intent without installing anything. |
| `setup profile deactivate <name>` | Removes profile intent without uninstalling anything. |
| `setup install <component>` | Leaves profile intent unchanged. |
| `setup install --all` | Leaves profile intent unchanged. |

## Doctor Behavior

`setup doctor` runs machine-health checks every time. These include PATH sanity, dotfile drift, broken symlinks, and optional component verification.

Profile-drift checks require an active set. The active set comes from explicit `--profile` flags first, then `~/.config/setup/active.toml`. If neither is present, doctor skips profile-drift checks and prints a note.

Useful commands:

```bash
setup doctor
setup doctor --profile workstation
setup doctor --verify
setup doctor --warn-only
```

## Component Lifecycle

Each Rust component implements the lifecycle contract in `cli/src/components/mod.rs`:

| Method | Purpose |
|---|---|
| `is_installed()` | Probe real machine state. |
| `install()` | Idempotently install the component. |
| `verify()` | Optional deeper post-install check. |
| `dry_run()` | Explain what install would do without side effects. |
| `uninstall()` | Remove the component when removal is supported. |
| `is_reversible()` | Declare whether automatic rollback may uninstall it safely. |

`setup uninstall <component>` calls the component's uninstall implementation. Components that manage user material, such as SSH or GPG material, can be uninstallable but non-reversible; those require `--force` for manual uninstall and are skipped by automatic rollback.

Historical design notes are kept in [`plans/manifest-architecture-spec.md`](../plans/manifest-architecture-spec.md).
