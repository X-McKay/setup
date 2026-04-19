---
name: setup-drift
description: Review and reconcile drift between this repo's managed dotfiles/profile intent and the current machine state. Use whenever the user wants to compare home-directory config changes against the setup repo, adopt edits from home back into the repo, sync repo versions back to home, or audit managed configs such as Ghostty, mise, lazygit, and shell dotfiles over time.
---

# Setup Drift

Use `setup drift` as the main entrypoint. Prefer it over stitching together ad hoc `diff`,
`doctor`, and dotfiles commands.

## Workflow

1. Run `setup drift --json` for a machine-readable summary.
2. Report dotfile drift separately from profile drift.
3. For each changed managed dotfile, inspect `setup drift diff --name <managed-name>`.
4. Recommend one action per file: `adopt`, `sync`, or `ignore`.
5. Do not mutate files unless the user explicitly asked to adopt or sync.
6. For home-to-repo updates, run `setup drift adopt --name <managed-name>`, review the resulting `git diff`, then run any relevant verification before committing.
7. For repo-to-home updates, run `setup drift sync --force` when the user wants the repo version restored.
8. Re-run `setup drift` after changes to confirm the state is clean or to show residual drift.

## Notes

- `setup doctor` is still the broader health check. Use it when the user wants PATH, broken
  symlink, or component verification coverage beyond drift review.
- `setup drift --json` is the preferred entrypoint for agent workflows because it includes repo
  and home paths for each managed file.
- If the built-in diff is not detailed enough, use the paths from `setup drift --json` and inspect
  the file contents or `git diff` directly before changing anything.
