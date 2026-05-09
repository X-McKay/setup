---
title: Workstation Maintenance System
date: 2026-05-09
status: design — awaiting user review
---

# Workstation Maintenance System

A layered, AI-provider-agnostic system that scans this workstation for cleanup opportunities, configuration drift, and emerging issues; writes its findings to a versioned journal; and surfaces them through both scheduled (systemd) and on-demand (slash command) entry points.

## Motivation

Computer maintenance currently happens ad hoc, when something is already noticeably wrong. The `setup` CLI already covers managed configs (`setup doctor`, `setup drift`, `setup check`, `setup update`) but does not address: disk cruft, docker/k8s leftovers, journald growth, dead branches and abandoned worktrees, stale Claude/agent artifacts, listening-port drift, or trends over time.

The goal is a small, mostly-bash system whose load-bearing logic lives outside any single AI agent, with thin per-provider glue. Switching to or adding Codex / OpenCode should not require rewriting any of the maintenance work itself.

## Goals

- **Scheduled, unattended scans** that produce a digest readable by humans, AI agents, or `grep`.
- **On-demand entry points** through slash commands for focused work (full audit, disk, dev cruft).
- **Passive surfacing** of unresolved items at the start of each AI session (one line, fast).
- **Provider-agnostic substrate** — the scanners, the journal, and the timer survive any agent change.
- **Report-only by default** — never delete or modify without explicit user action in a session.
- **Leverage `setup`** — call `setup doctor` and `setup drift` rather than re-implementing them.

## Non-goals

- Auto-cleanup of any kind. Even "obviously safe" pruning waits for explicit user action.
- Browser cache / GUI app cruft — too varied per app, not worth the maintenance burden.
- Cross-host sync of the journal or deferred list — per-host state by design.
- A `setup` CLI Rust component for installing the timer (use a bash installer for now; promote to a Rust component if the system proves worth it).
- Trend analysis beyond simple week-over-week deltas (e.g. "Downloads grew 4 GB").
- Per-session anomaly detection. SessionEnd logs only `<timestamp> <cwd>`; the weekly scan does the interpretation.

## Architecture

Three tiers, each with one job:

1. **Substrate** (provider-agnostic): bash scanners, the digest format, the journal, the systemd timer. Lives in `git/setup/bootstrap/scripts/maintenance/` and `git/setup/bootstrap/systemd/`. Per-host state lives at `~/.local/state/maintenance/`.
2. **Shared agent skills** (provider-agnostic markdown): `.agents/skills/maintenance-audit/SKILL.md` and `.agents/skills/maintenance-act/SKILL.md`. These tell *any* agent how to walk a digest with the user and how to safely run a suggested action.
3. **Per-provider shims** (thin): `.claude/commands/<name>.md` files that delegate to the skill. When Codex or OpenCode are adopted, equivalent shims get added under `.codex/` or `.opencode/` without touching the substrate or skills.

```
git/setup/
├── .agents/
│   ├── README.md                            ← explains the .agents convention and shim pattern
│   └── skills/
│       ├── maintenance-audit/SKILL.md
│       └── maintenance-act/SKILL.md
├── .claude/commands/
│   ├── maintenance.md                       ← /maintenance [category]
│   ├── clean-disk.md                        ← /clean-disk
│   └── clean-dev.md                         ← /clean-dev
├── bootstrap/scripts/maintenance/
│   ├── scan.sh                              ← driver: invokes scanners, writes digest
│   ├── show-digest.sh                       ← prints top N items (called by SessionStart)
│   ├── log-session-end.sh                   ← appends timestamp+cwd (called by SessionEnd)
│   ├── install.sh                           ← installs the timer + symlinks the hooks
│   └── scanners/
│       ├── setup-doctor.sh
│       ├── disk.sh
│       ├── docker-k8s.sh
│       ├── system-health.sh
│       ├── dev-cruft.sh
│       └── config-drift.sh
├── bootstrap/systemd/
│   ├── maintenance-scan.service
│   └── maintenance-scan.timer
└── docs/MAINTENANCE.md                      ← user-facing documentation

~/.local/state/maintenance/
├── journal.md                               ← append-only, sectioned by date
├── deferred.md                              ← items the user chose to keep, with expiry
├── last-digest.md                           ← latest scan output (overwritten weekly)
└── sessions.log                             ← <timestamp> <cwd> per AI session

~/.claude/hooks/
├── session-start-nudge.sh                   ← thin shim: exec setup repo's show-digest.sh
└── session-end-cwd-log.sh                   ← thin shim: exec setup repo's log-session-end.sh
```

## Components

### scan.sh (driver)

Single bash script that runs every scanner under `scanners/` with a 60s timeout each. Each scanner emits findings on stdout in the shared format (see Data Formats). `scan.sh` concatenates them, prepends a date heading, writes the result to `~/.local/state/maintenance/last-digest.md`, and appends the same content to `~/.local/state/maintenance/journal.md`.

Flags:
- `--scope=<csv>` — run only the named scanners (default: all).
- `--json` — emit machine-readable JSON instead of markdown (used by the audit skill).
- `--no-write` — print to stdout only; don't update digest or journal (used for dry runs).

Items present in `deferred.md` (and not yet expired) are filtered out before writing.

### Scanners

Each scanner is a self-contained bash script with the same contract: read no arguments, print findings to stdout in the shared format, exit 0 on success even if findings are present. Exit nonzero only on scanner failure (broken environment, missing binary).

| Scanner | What it checks |
|---|---|
| `setup-doctor.sh` | Shells out to `setup doctor` and `setup drift --dry-run`. Captures their summary lines as `info` or `warn` items. |
| `disk.sh` | `~/Downloads` size + week-over-week delta; files >100MB under `$HOME` (excluding `~/.cache`, `~/.local/share`, any `node_modules`, any `.git/objects`); `node_modules` / `target/` / `.venv` directories untouched 90+ days; `/tmp` files older than 30 days. |
| `docker-k8s.sh` | `docker images -f dangling=true`, `docker volume ls -f dangling=true`, total reclaimable space from `docker system df`. For each context in `~/.kube/config`, with `kubectl --request-timeout=5s`: evicted pods, terminating pods stuck >1h. Contexts that time out are reported as a single `info` item rather than failing the scan. |
| `system-health.sh` | `systemctl --user --failed` and `systemctl --failed`; `journalctl -p err --since "7 days ago"` count per unit; `apt list --upgradable` filtered to security updates; new listening ports vs. last week's snapshot (snapshot kept under `~/.local/state/maintenance/ports-prev.txt`). |
| `dev-cruft.sh` | `~/.claude/sessions/` JSONLs >30 days; `~/.claude/plans/` and `~/.claude/todos/` >60 days; isolation-mode worktrees untouched 30+ days that are not the cwd of any session in the last 30 days of `sessions.log`; for each tracked git repo under `~/git/`, local branches fully merged into the repo's default branch (detected via `git rev-parse --abbrev-ref origin/HEAD`); cwds appearing in `sessions.log` (last 30 days) with lingering untracked files per `git status --porcelain`. |
| `config-drift.sh` | Size and entry count of `~/.claude/settings.json` permission lists; flags lists that exceed a threshold (e.g. >150 entries). Future: track per-entry trigger counts. |

Adding a scanner: drop a new script under `scanners/`, no driver changes needed.

### Skills

`.agents/skills/maintenance-audit/SKILL.md` — workflow for walking findings with the user. Steps:

1. Run `bootstrap/scripts/maintenance/scan.sh --scope=<requested> --json` to get a fresh structured list.
2. Group findings by category, present worst severity first (`warn` → `suggest` → `info`).
3. For each item, offer three choices: **act**, **defer**, **ignore**.
4. If **act**: invoke the `maintenance-act` skill with the item's command.
5. If **defer**: append a line to `~/.local/state/maintenance/deferred.md` with date, item key, reason, and expiry (default 90 days).
6. If **ignore**: do nothing — the item will reappear on the next scan.
7. After all items are handled, summarize what was acted on, deferred, ignored.

`.agents/skills/maintenance-act/SKILL.md` — safe execution wrapper. Steps:

1. Echo the proposed command verbatim.
2. Ask the user: run as-is, modify, or skip. If they propose a modified command, echo the modified form back for confirmation before running.
3. Run the chosen command, capture stdout/stderr/exit.
4. Report outcome.
5. If the action was a deletion, suggest re-running `scan.sh --scope=<category> --no-write` to verify the item is gone.

Both skills are plain markdown with no Claude-specific syntax — invokable from any agent.

### Per-provider shims

`.claude/commands/maintenance.md` (and the two siblings) are each ~5 lines of markdown that say:

```markdown
# Maintenance Audit
Use the `maintenance-audit` skill with scope=`$ARGUMENTS or "all"`.
```

When other providers are adopted, their command directories get analogous shims pointing at the same skill.

### Hooks

`~/.claude/hooks/session-start-nudge.sh`:

```bash
#!/usr/bin/env bash
exec ~/git/setup/bootstrap/scripts/maintenance/show-digest.sh 3
```

`~/.claude/hooks/session-end-cwd-log.sh`:

```bash
#!/usr/bin/env bash
exec ~/git/setup/bootstrap/scripts/maintenance/log-session-end.sh
```

`show-digest.sh N` reads `~/.local/state/maintenance/last-digest.md` and prints up to N items, warn-first. If the digest is missing or older than 14 days, prints a one-line reminder pointing at `systemctl --user status maintenance-scan.timer`. Must complete in <50ms (target <20ms).

`log-session-end.sh` appends `<ISO-8601-timestamp> <cwd>` to `~/.local/state/maintenance/sessions.log`. No interpretation — the weekly scan reads this log later.

Hooks registered in `~/.claude/settings.json` under existing `hooks` block:

```json
{
  "hooks": {
    "SessionStart": [{ "hooks": [{ "type": "command", "command": "bash ~/.claude/hooks/session-start-nudge.sh", "timeout": 2 }] }],
    "SessionEnd": [{ "hooks": [{ "type": "command", "command": "bash ~/.claude/hooks/session-end-cwd-log.sh", "timeout": 2 }] }]
  }
}
```

These are added alongside existing hook entries; no existing hook is modified.

### Systemd timer

`bootstrap/systemd/maintenance-scan.timer`:

```
[Unit]
Description=Weekly workstation maintenance scan

[Timer]
OnCalendar=Sun 09:00
RandomizedDelaySec=2h
Persistent=true

[Install]
WantedBy=timers.target
```

`bootstrap/systemd/maintenance-scan.service`:

```
[Unit]
Description=Run workstation maintenance scan

[Service]
Type=oneshot
ExecStart=%h/git/setup/bootstrap/scripts/maintenance/scan.sh
TimeoutStartSec=10min
```

User-level systemd (`systemctl --user`), no root. `Persistent=true` ensures missed runs (laptop closed Sunday morning) catch up on next boot.

### install.sh

Installs the timer and symlinks the hooks. Idempotent. Single command:

```bash
~/git/setup/bootstrap/scripts/maintenance/install.sh
```

Steps it performs:
1. `mkdir -p ~/.local/state/maintenance ~/.config/systemd/user ~/.claude/hooks`.
2. Symlink the systemd unit files into `~/.config/systemd/user/`.
3. Write the two hook shims to `~/.claude/hooks/` (overwriting if present).
4. Print a manual instruction for the user to add the two `~/.claude/settings.json` hook entries (we don't auto-edit settings.json — too easy to corrupt).
5. `systemctl --user daemon-reload && systemctl --user enable --now maintenance-scan.timer`.
6. Run `scan.sh` once immediately to populate the first digest.

## Data Formats

### Digest / journal entry

```markdown
## 2026-05-09 (weekly scan)
### setup
- [info] setup doctor: all green
- [warn] setup drift: 2 dotfiles changed in home :: review and reconcile :: `setup drift`

### disk
- [warn] ~/Downloads: 12.4 GB (up from 8.1 GB last week) :: review and prune :: `du -sh ~/Downloads/* | sort -h`
- [suggest] ~/projects/old-thing/node_modules: 1.2 GB, untouched 187d :: rm if dead :: `rm -rf ~/projects/old-thing/node_modules`

### docker-k8s
- [suggest] 8 dangling images, ~2.4 GB reclaimable :: prune :: `docker image prune -f`
```

Format spec, per item:

```
- [<severity>] <item-description> :: <suggested-action> :: `<command>`
```

`severity ∈ {info, suggest, warn}`. The `:: <command>` segment is optional for items with no obvious one-liner.

### JSON form (for skill consumption)

```json
{
  "scan_date": "2026-05-09T09:00:00+02:00",
  "items": [
    {
      "category": "disk",
      "severity": "warn",
      "key": "downloads-size",
      "description": "~/Downloads: 12.4 GB (up from 8.1 GB last week)",
      "action": "review and prune",
      "command": "du -sh ~/Downloads/* | sort -h"
    }
  ]
}
```

`key` is a stable identifier per scanner+item, used to match against `deferred.md`.

### deferred.md

```markdown
- 2026-05-02 :: dev-cruft :: ~/projects/quarterly-thing/node_modules :: kept — used quarterly :: expires 2026-08-02
- 2026-04-15 :: disk :: ~/Downloads/old-installer.iso :: keep until next migration :: expires 2026-07-15
```

Plain text, manually editable. `scan.sh` reads it and filters items whose `category::key` matches an unexpired entry.

### sessions.log

```
2026-05-09T14:32:11+02:00 /home/al/git/setup
2026-05-09T15:01:44+02:00 /home/al/projects/foo
```

Append-only. The `dev-cruft.sh` scanner reads this to surface "cwds you worked in recently that have lingering untracked files."

## Documentation Deliverables

- `docs/MAINTENANCE.md` (new) — user-facing: what gets scanned, how to read the digest, how to run on-demand commands, how to defer items, how to disable the timer, how to add a new scanner. Style matches existing `docs/GHOSTTY.md` and `docs/TMUX.md`.
- `README.md` — new "Maintenance" subsection under Features pointing at `docs/MAINTENANCE.md`.
- `CHANGELOG.md` — new entry under unreleased: `feat(maintenance): add provider-agnostic workstation maintenance system`.
- `.agents/README.md` (new) — short note on the `.agents/` convention and the per-provider shim pattern; references the `maintenance-audit` and `maintenance-act` skills as canonical examples.
- `.agents/skills/maintenance-audit/SKILL.md` and `.agents/skills/maintenance-act/SKILL.md` — agent-facing instructions; not user docs but should be readable by a human.

## Risks and Open Questions

- **SessionStart latency.** Hook adds wall time to every Claude session start. Target <20ms; the `tail`-based shim should beat that, but worth measuring on real hardware before committing.
- **`setup doctor` hang risk.** If `setup doctor` ever blocks on network or a missing binary, the systemd timer's `TimeoutStartSec=10min` cap saves us, but the resulting digest will be partial. The driver should print which scanners timed out.
- **Reachable kube contexts.** `docker-k8s.sh` scans every context in `~/.kube/config`. If a context points at an unreachable cluster, it'll hang on connect. Solution: per-context timeout via `kubectl --request-timeout=5s`.
- **`apt` on systems without it.** `system-health.sh` assumes Ubuntu. Acceptable per the setup repo's stated Ubuntu 22.04/24.04 scope, but the scanner should detect-and-skip rather than error on non-apt systems.
- **`deferred.md` growth.** Without expiry enforcement, the deferred list grows forever. Expiry is per-entry, but the driver should warn when entries are past their expiry so the user can re-decide rather than silently re-surfacing them.
- **Promotion to a `setup` Rust component.** If the system proves valuable, the install step becomes a `setup` component (`cli/src/components/maintenance.rs`) so it gets profile-aware install/uninstall. Tracked as future work, not in this spec.

## Phased Delivery

The implementation plan should sequence work so partial delivery still produces value:

1. **Phase 1 — substrate.** `scan.sh` driver, three highest-value scanners (`setup-doctor.sh`, `disk.sh`, `dev-cruft.sh`), digest/journal format, `install.sh`, systemd units. No agent integration yet — the digest is readable directly.
2. **Phase 2 — hooks.** `show-digest.sh`, `log-session-end.sh`, the two thin Claude hook shims, settings.json entries. SessionStart nudge becomes the primary "you have unresolved items" surface.
3. **Phase 3 — skills and commands.** `maintenance-audit` and `maintenance-act` skills, three Claude command shims. On-demand `/maintenance`, `/clean-disk`, `/clean-dev` work end-to-end.
4. **Phase 4 — remaining scanners.** `docker-k8s.sh`, `system-health.sh`, `config-drift.sh`. Each is independent and can land when the relevant trade-off (e.g. kubectl timeout handling) is sorted.
5. **Phase 5 — docs.** `docs/MAINTENANCE.md`, README section, CHANGELOG, `.agents/README.md`. Done last so the docs reflect what shipped, not what was planned.

After Phase 1 the system already produces value: a weekly digest you can read manually. Each subsequent phase adds an interaction layer on top.
