//! `monitoring` component - system monitoring tooling.
//!
//! Install preserves the legacy behavior: install packages and configure
//! services, scripts, and cron jobs.
//!
//! Uninstall: unsupported for now because cleanup spans packages, service
//! state, system config, and user-owned artifacts.

use anyhow::{Context, Result};
use std::fs;

use super::util::{apt_install, run_command, run_sudo};
use super::Component;

pub struct Monitoring;

impl Component for Monitoring {
    fn id(&self) -> &str {
        "monitoring"
    }

    fn is_installed(&self) -> Result<bool> {
        Ok(which::which("htop").is_ok() && which::which("netdata").is_ok())
    }

    fn install(&self) -> Result<()> {
        install_monitoring()
    }
}

fn install_monitoring() -> Result<()> {
    let packages = [
        "htop", "iotop", "nethogs", "sysstat", "netdata", "logwatch", "fail2ban",
    ];

    run_sudo("apt", &["update"])?;
    apt_install(&packages)?;

    configure_logwatch()?;
    configure_fail2ban()?;
    configure_netdata()?;
    create_health_check_script()?;
    add_monitoring_cron()?;

    Ok(())
}

fn configure_logwatch() -> Result<()> {
    let source = "/usr/share/logwatch/default.conf/logwatch.conf";
    let dest = "/etc/logwatch/conf/logwatch.conf";

    if std::path::Path::new(source).exists() {
        let _ = run_sudo("mkdir", &["-p", "/etc/logwatch/conf"]);
        let _ = run_sudo("cp", &[source, dest]);

        let user = std::env::var("USER").unwrap_or_else(|_| "root".to_string());
        let _ = run_sudo(
            "sed",
            &["-i", &format!("s/MailTo = root/MailTo = {}/g", user), dest],
        );
        let _ = run_sudo("sed", &["-i", "s/Detail = Low/Detail = High/g", dest]);
    }
    Ok(())
}

fn configure_fail2ban() -> Result<()> {
    let source = "/etc/fail2ban/jail.conf";
    let dest = "/etc/fail2ban/jail.local";

    if std::path::Path::new(source).exists() && !std::path::Path::new(dest).exists() {
        run_sudo("cp", &[source, dest])?;
    }

    if let Err(e) = run_sudo("systemctl", &["enable", "fail2ban"]) {
        eprintln!("Warning: Could not enable fail2ban service: {}", e);
    }
    if let Err(e) = run_sudo("systemctl", &["start", "fail2ban"]) {
        eprintln!("Warning: Could not start fail2ban service: {}", e);
    }
    Ok(())
}

fn configure_netdata() -> Result<()> {
    if let Err(e) = run_sudo("systemctl", &["enable", "netdata"]) {
        eprintln!("Warning: Could not enable netdata service: {}", e);
    }
    if let Err(e) = run_sudo("systemctl", &["start", "netdata"]) {
        eprintln!("Warning: Could not start netdata service: {}", e);
    }
    Ok(())
}

fn create_health_check_script() -> Result<()> {
    let home = dirs::home_dir().context("Could not find home directory")?;
    let monitoring_dir = home.join(".monitoring");
    fs::create_dir_all(&monitoring_dir)?;

    let script = r#"#!/bin/bash
log_file="$HOME/.monitoring/health_report.log"

echo "=== System Health Report $(date) ===" > "$log_file"

echo -e "\nDisk Usage:" >> "$log_file"
df -h | grep -v "tmpfs" >> "$log_file"

echo -e "\nMemory Usage:" >> "$log_file"
free -h >> "$log_file"

echo -e "\nCPU Load:" >> "$log_file"
uptime >> "$log_file"

echo -e "\nCritical Services Status:" >> "$log_file"
systemctl status fail2ban netdata 2>/dev/null | grep "Active:" >> "$log_file"

echo -e "\nRecent System Errors:" >> "$log_file"
journalctl -p err -n 20 --no-pager >> "$log_file"
"#;

    let temp_path = "/tmp/check_monitoring.sh";
    fs::write(temp_path, script)?;
    run_sudo("mv", &[temp_path, "/usr/local/bin/check_monitoring.sh"])?;
    run_sudo("chmod", &["+x", "/usr/local/bin/check_monitoring.sh"])?;
    Ok(())
}

fn add_monitoring_cron() -> Result<()> {
    let _ = run_command(
        "sh",
        &[
            "-c",
            r#"(crontab -l 2>/dev/null | grep -v check_monitoring; echo "0 0 * * * /usr/local/bin/check_monitoring.sh") | crontab -"#,
        ],
    );
    Ok(())
}
