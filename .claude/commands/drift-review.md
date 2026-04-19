# Review Setup Drift

Inspect managed config drift in this repository and guide the user through reconciliation.

## Arguments
- `$ARGUMENTS` optional extra arguments for `setup drift` such as `--dotfiles`, `--profiles`, or `--profile workstation`

## Instructions

1. Run `setup drift --json $ARGUMENTS` when arguments are provided, otherwise run `setup drift --json`.
2. Summarize dotfile drift and profile drift separately.
3. For each changed managed dotfile, inspect `setup drift diff --name <managed-name>`.
4. Recommend one action for each file: adopt the home version into the repo, sync the repo version back to home, or ignore it for now.
5. Do not mutate files unless the user explicitly asks for adoption or sync.
6. If the user asks to adopt a home change, run `setup drift adopt --name <managed-name>`, review the resulting `git diff`, and commit only if requested.
7. If the user asks to restore the repo version locally, run `setup drift sync --force`.
8. Re-run `setup drift` after any change so the final status is explicit.
