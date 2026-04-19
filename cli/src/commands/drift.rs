use anyhow::{anyhow, Context, Result};
use clap::{Args, Subcommand};
use console::style;
use serde::Serialize;
use std::collections::BTreeSet;
use std::path::Path;

use crate::components::registry::Registry;
use crate::config::dotfiles as dotfiles_config;
use crate::manifest::{intent, loader};

#[derive(Args)]
pub struct DriftArgs {
    #[command(subcommand)]
    pub action: Option<DriftAction>,

    /// Emit JSON for the default summary
    #[arg(long)]
    pub json: bool,

    /// Limit the default summary to managed dotfiles
    #[arg(long)]
    pub dotfiles: bool,

    /// Limit the default summary to profile drift
    #[arg(long)]
    pub profiles: bool,

    /// Active profiles to check against (overrides ~/.config/setup/active.toml)
    #[arg(long = "profile")]
    pub active_profiles: Vec<String>,
}

#[derive(Subcommand)]
pub enum DriftAction {
    /// Show managed dotfile diffs
    Diff {
        /// Limit output to one managed dotfile name (for example: ghostty/config)
        #[arg(long)]
        name: Option<String>,

        /// Emit JSON
        #[arg(long)]
        json: bool,
    },

    /// Sync managed dotfiles from repo to home
    Sync {
        /// Overwrite existing files without prompting
        #[arg(short, long)]
        force: bool,
    },

    /// Adopt the installed home-directory version back into the repo
    Adopt {
        /// Managed dotfile name (for example: ghostty/config)
        #[arg(long)]
        name: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SummaryScope {
    dotfiles: bool,
    profiles: bool,
}

#[derive(Debug, Serialize)]
struct DriftReport {
    dotfiles: Option<Vec<DotfileStatusEntry>>,
    profiles: Option<ProfileDriftSection>,
}

#[derive(Debug, Serialize)]
struct DotfileStatusEntry {
    name: String,
    repo_path: String,
    home_path: String,
    status: DotfileStatus,
}

#[derive(Debug, Serialize)]
struct DotfileDiffEntry {
    name: String,
    repo_path: String,
    home_path: String,
    status: DotfileStatus,
    diff: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum DotfileStatus {
    Synced,
    Differs,
    MissingHome,
    MissingRepo,
}

#[derive(Debug, Serialize)]
struct ProfileDriftSection {
    skipped: bool,
    active_profiles: Vec<String>,
    findings: Vec<ProfileFinding>,
}

#[derive(Debug, Serialize)]
struct ProfileFinding {
    severity: ProfileSeverity,
    component: String,
    message: String,
    fix_hint: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum ProfileSeverity {
    Ok,
    Missing,
    Extra,
}

pub fn run(args: DriftArgs) -> Result<()> {
    match args.action {
        Some(DriftAction::Diff { name, json }) => show_diff(name.as_deref(), json),
        Some(DriftAction::Sync { force }) => {
            crate::commands::dotfiles::run(crate::commands::dotfiles::DotfilesArgs {
                action: Some(crate::commands::dotfiles::DotfilesAction::Sync { force }),
            })
        }
        Some(DriftAction::Adopt { name }) => adopt_home_change(&name),
        None => {
            let report = build_summary(
                resolve_scope(args.dotfiles, args.profiles),
                &args.active_profiles,
            )?;
            if args.json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                print_summary(&report);
            }
            Ok(())
        }
    }
}

fn resolve_scope(dotfiles: bool, profiles: bool) -> SummaryScope {
    if !dotfiles && !profiles {
        SummaryScope {
            dotfiles: true,
            profiles: true,
        }
    } else {
        SummaryScope { dotfiles, profiles }
    }
}

fn build_summary(scope: SummaryScope, explicit_profiles: &[String]) -> Result<DriftReport> {
    Ok(DriftReport {
        dotfiles: if scope.dotfiles {
            Some(collect_dotfile_status_entries(None)?)
        } else {
            None
        },
        profiles: if scope.profiles {
            Some(collect_profile_drift(explicit_profiles)?)
        } else {
            None
        },
    })
}

fn collect_dotfile_status_entries(name: Option<&str>) -> Result<Vec<DotfileStatusEntry>> {
    let mut out = Vec::new();
    for (managed_name, source, target) in select_managed_dotfiles(name)? {
        out.push(DotfileStatusEntry {
            name: managed_name,
            repo_path: source.display().to_string(),
            home_path: target.display().to_string(),
            status: classify_dotfile(&source, &target)?,
        });
    }
    Ok(out)
}

fn collect_dotfile_diff_entries(name: Option<&str>) -> Result<Vec<DotfileDiffEntry>> {
    let mut out = Vec::new();
    for (managed_name, source, target) in select_managed_dotfiles(name)? {
        let status = classify_dotfile(&source, &target)?;
        let diff = match status {
            DotfileStatus::Synced => None,
            DotfileStatus::MissingRepo => {
                Some(format!("Source does not exist: {}", source.display()))
            }
            _ => dotfiles_config::diff_files(&source, &target)?,
        };

        out.push(DotfileDiffEntry {
            name: managed_name,
            repo_path: source.display().to_string(),
            home_path: target.display().to_string(),
            status,
            diff,
        });
    }
    Ok(out)
}

fn classify_dotfile(source: &Path, target: &Path) -> Result<DotfileStatus> {
    if !source.exists() {
        return Ok(DotfileStatus::MissingRepo);
    }
    if !target.exists() {
        return Ok(DotfileStatus::MissingHome);
    }
    if dotfiles_config::files_match(&source.to_path_buf(), &target.to_path_buf())? {
        Ok(DotfileStatus::Synced)
    } else {
        Ok(DotfileStatus::Differs)
    }
}

fn select_managed_dotfiles(
    name: Option<&str>,
) -> Result<Vec<(String, std::path::PathBuf, std::path::PathBuf)>> {
    if let Some(name) = name {
        let entry = dotfiles_config::get_managed_dotfile(name).ok_or_else(|| {
            let known = dotfiles_config::get_managed_dotfiles()
                .into_iter()
                .map(|(managed_name, _, _)| managed_name)
                .collect::<Vec<_>>()
                .join(", ");
            anyhow!("Unknown managed dotfile: {}. Known names: {}", name, known)
        })?;
        Ok(vec![entry])
    } else {
        Ok(dotfiles_config::get_managed_dotfiles())
    }
}

fn collect_profile_drift(explicit_profiles: &[String]) -> Result<ProfileDriftSection> {
    let manifest = loader::load().context("loading manifest")?;
    let registry = Registry::build();
    registry.validate_against(&manifest)?;

    let (active_profiles, active_set) = resolve_active_profiles(&manifest, explicit_profiles)?;
    if let Some(active_set) = active_set {
        let mut findings = Vec::new();
        collect_declared_missing(&registry, &manifest, &active_set, &mut findings);
        collect_installed_not_declared(&registry, &manifest, &active_set, &mut findings);
        Ok(ProfileDriftSection {
            skipped: false,
            active_profiles,
            findings,
        })
    } else {
        Ok(ProfileDriftSection {
            skipped: true,
            active_profiles,
            findings: Vec::new(),
        })
    }
}

fn resolve_active_profiles(
    manifest: &crate::manifest::schema::Manifest,
    explicit_profiles: &[String],
) -> Result<(Vec<String>, Option<BTreeSet<String>>)> {
    if !explicit_profiles.is_empty() {
        let active_set =
            crate::manifest::resolver::expand_selection(manifest, explicit_profiles, &[])?;
        return Ok((explicit_profiles.to_vec(), Some(active_set)));
    }

    let path = intent::default_path().context("no config dir")?;
    let declared = intent::read(&path)?;
    let (valid, unknown) = intent::validated(&declared, manifest);
    for profile in &unknown {
        eprintln!(
            "{} intent file references unknown profile {:?} - ignoring",
            style("warn:").yellow(),
            profile
        );
    }
    if valid.is_empty() {
        return Ok((Vec::new(), None));
    }

    let active_set = crate::manifest::resolver::expand_selection(manifest, &valid, &[])?;
    Ok((valid, Some(active_set)))
}

fn collect_declared_missing(
    registry: &Registry,
    manifest: &crate::manifest::schema::Manifest,
    active: &BTreeSet<String>,
    findings: &mut Vec<ProfileFinding>,
) {
    for id in active {
        if let Ok(component) = registry.get(id) {
            let installed = component.is_installed().unwrap_or(false);
            let display_name = manifest
                .components
                .iter()
                .find(|spec| spec.id == *id)
                .map(|spec| spec.display_name.clone())
                .unwrap_or_else(|| id.clone());

            if installed {
                findings.push(ProfileFinding {
                    severity: ProfileSeverity::Ok,
                    component: id.clone(),
                    message: format!("{} installed", display_name),
                    fix_hint: None,
                });
            } else {
                findings.push(ProfileFinding {
                    severity: ProfileSeverity::Missing,
                    component: id.clone(),
                    message: "declared, not installed".into(),
                    fix_hint: Some(format!("setup install {}", id)),
                });
            }
        }
    }
}

fn collect_installed_not_declared(
    registry: &Registry,
    manifest: &crate::manifest::schema::Manifest,
    active: &BTreeSet<String>,
    findings: &mut Vec<ProfileFinding>,
) {
    for spec in &manifest.components {
        if active.contains(&spec.id) {
            continue;
        }
        if let Ok(component) = registry.get(&spec.id) {
            if component.is_installed().unwrap_or(false) {
                findings.push(ProfileFinding {
                    severity: ProfileSeverity::Extra,
                    component: spec.id.clone(),
                    message: "installed, not in active profile".into(),
                    fix_hint: None,
                });
            }
        }
    }
}

fn print_summary(report: &DriftReport) {
    println!("{}", style("Drift Summary").cyan().bold());

    if let Some(dotfiles) = &report.dotfiles {
        println!();
        println!("{}", style("Managed Dotfiles").bold());

        let drifting = dotfiles
            .iter()
            .filter(|entry| entry.status != DotfileStatus::Synced)
            .collect::<Vec<_>>();

        if drifting.is_empty() {
            println!("  {} all managed dotfiles are in sync", style("✓").green());
        } else {
            for entry in drifting {
                let (icon, text) = match entry.status {
                    DotfileStatus::Differs => (style("~").yellow(), "differs from repo"),
                    DotfileStatus::MissingHome => (style("○").dim(), "not installed in home"),
                    DotfileStatus::MissingRepo => (style("!").red(), "missing from repo"),
                    DotfileStatus::Synced => unreachable!(),
                };
                println!("  {} {} {}", icon, entry.name, text);
            }
        }
    }

    if let Some(profiles) = &report.profiles {
        println!();
        println!("{}", style("Profile Drift").bold());

        if profiles.skipped {
            println!(
                "  {} no active profiles - skipping profile drift",
                style("info:").yellow()
            );
        } else {
            println!("  active profiles: {}", profiles.active_profiles.join(", "));

            let noteworthy = profiles
                .findings
                .iter()
                .filter(|finding| finding.severity != ProfileSeverity::Ok)
                .collect::<Vec<_>>();

            if noteworthy.is_empty() {
                println!(
                    "  {} active profiles match installed state",
                    style("✓").green()
                );
            } else {
                for finding in noteworthy {
                    match finding.severity {
                        ProfileSeverity::Missing => {
                            println!(
                                "  {} {} {}",
                                style("✗").red(),
                                finding.component,
                                finding.message
                            );
                        }
                        ProfileSeverity::Extra => {
                            println!(
                                "  {} {} {}",
                                style("?").yellow(),
                                finding.component,
                                finding.message
                            );
                        }
                        ProfileSeverity::Ok => {}
                    }
                }
            }
        }
    }
}

fn show_diff(name: Option<&str>, json: bool) -> Result<()> {
    let entries = collect_dotfile_diff_entries(name)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&entries)?);
        return Ok(());
    }

    let changed = entries
        .iter()
        .filter(|entry| entry.diff.is_some())
        .collect::<Vec<_>>();

    if changed.is_empty() {
        println!("{}", style("All managed dotfiles are in sync!").green());
        return Ok(());
    }

    println!("{}", style("Managed dotfile diffs").cyan().bold());
    for entry in changed {
        println!("\n{} {}:", style("─").dim(), style(&entry.name).bold());
        if let Some(diff) = &entry.diff {
            println!("{}", diff);
        }
    }

    Ok(())
}

fn adopt_home_change(name: &str) -> Result<()> {
    let (managed_name, source, target) = dotfiles_config::get_managed_dotfile(name)
        .ok_or_else(|| anyhow!("Unknown managed dotfile: {}", name))?;

    if !target.exists() {
        anyhow::bail!(
            "Cannot adopt {} because the installed file is missing: {}",
            managed_name,
            target.display()
        );
    }

    copy_target_to_source(&source, &target)?;

    println!(
        "{} adopted {} from {} into {}",
        style("✓").green().bold(),
        managed_name,
        target.display(),
        source.display()
    );
    println!("  review with: git diff -- {}", source.display());
    Ok(())
}

fn copy_target_to_source(source: &Path, target: &Path) -> Result<()> {
    dotfiles_config::copy_dotfile(&target.to_path_buf(), &source.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn resolve_scope_defaults_to_both_sections() {
        let scope = resolve_scope(false, false);
        assert!(scope.dotfiles);
        assert!(scope.profiles);
    }

    #[test]
    fn resolve_scope_honors_explicit_section_flags() {
        let dotfiles_only = resolve_scope(true, false);
        assert!(dotfiles_only.dotfiles);
        assert!(!dotfiles_only.profiles);

        let profiles_only = resolve_scope(false, true);
        assert!(!profiles_only.dotfiles);
        assert!(profiles_only.profiles);
    }

    #[test]
    fn copy_target_to_source_adopts_home_contents_into_repo_file() {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let base = std::env::temp_dir().join(format!("setup-drift-{}", suffix));
        let repo_file = base.join("repo").join("ghostty").join("config");
        let home_file = base
            .join("home")
            .join(".config")
            .join("ghostty")
            .join("config");

        fs::create_dir_all(repo_file.parent().expect("repo parent")).expect("repo parent");
        fs::create_dir_all(home_file.parent().expect("home parent")).expect("home parent");
        fs::write(&repo_file, "repo-value\n").expect("write repo");
        fs::write(&home_file, "home-value\n").expect("write home");

        copy_target_to_source(&repo_file, &home_file).expect("adopt");

        assert_eq!(
            fs::read_to_string(&repo_file).expect("read repo"),
            "home-value\n"
        );

        let _ = fs::remove_dir_all(base);
    }
}
