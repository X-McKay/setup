//! `setup doctor` - read-only health and drift report.

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
pub(crate) struct Report {
    /// PATH, symlinks, dotfiles, verify - independent of profile intent.
    machine_findings: Vec<Finding>,
    /// declared-missing, installed-not-declared - depend on active set.
    drift_findings: Vec<Finding>,
    /// True when no active set was resolvable.
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
    Ok,
    Missing,
    Broken,
    Drift,
    Info,
}

pub fn run(args: DoctorArgs) -> Result<()> {
    let report = build_report(&args)?;
    print_report(&report);
    let code = compute_exit(&report, args.warn_only);
    std::process::exit(code);
}

pub(crate) fn build_report(args: &DoctorArgs) -> Result<Report> {
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

    Ok(report)
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
            "{} intent file references unknown profile {:?} - ignoring",
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
        if p.is_symlink() && !p.exists() {
            report.machine_findings.push(Finding {
                severity: Severity::Broken,
                subject: p.display().to_string(),
                message: "dangling symlink".into(),
                fix_hint: Some(format!("rm {}", p.display())),
            });
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
            "{} no active profiles - skipping profile-drift checks. Run 'setup install --profile <name>' or 'setup profile activate <name>' to declare intent.",
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
                .map(|h| format!("       -> {}", h))
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
