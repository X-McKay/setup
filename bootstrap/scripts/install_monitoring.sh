#!/bin/bash

# Get the directory where the script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"

# Source logging function if available
if [ -f "$SCRIPT_DIR/../main.sh" ]; then
  source "$SCRIPT_DIR/../main.sh"
else
  # Fallback logging function
  log() {
    local level=$1
    local message=$2
    local timestamp
    timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo "[$timestamp] [$level] $message" | tee -a "$HOME/.setup.log"
  }
fi

# Function to verify package installation
verify_package() {
  local package=$1
  if dpkg -l | grep -q "^ii  $package "; then
    log "INFO" "Package $package verified successfully"
    return 0
  else
    log "ERROR" "Package $package installation verification failed"
    return 1
  fi
}

# Install monitoring tools
log "INFO" "Installing system monitoring tools..."

# Ensure package list is up to date and universe repo is enabled
sudo apt-get update
sudo apt-get install -y software-properties-common
sudo add-apt-repository universe
sudo apt-get update

MONITORING_PACKAGES=(
  htop                     # Interactive process viewer
  iotop                    # I/O monitoring
  nethogs                  # Network traffic monitoring
  sysstat                  # System performance tools (sar, iostat)
  prometheus-node-exporter # System metrics exporter
  netdata                  # Real-time system monitoring
  logwatch                 # Log analysis and reporting
  fail2ban                 # Intrusion prevention
)

# Install packages
for package in "${MONITORING_PACKAGES[@]}"; do
  log "INFO" "Installing $package..."
  if sudo apt install -y "$package"; then
    if ! verify_package "$package"; then
      log "ERROR" "Failed to verify installation of $package"
      exit 1
    fi
  else
    log "ERROR" "Failed to install $package"
    exit 1
  fi
done

# Configure logwatch
log "INFO" "Configuring logwatch..."
sudo cp /usr/share/logwatch/default.conf/logwatch.conf /etc/logwatch/conf/
sudo sed -i 's/MailTo = root/MailTo = '"$USER"'/g' /etc/logwatch/conf/logwatch.conf
sudo sed -i 's/Detail = Low/Detail = High/g' /etc/logwatch/conf/logwatch.conf

# Configure fail2ban
log "INFO" "Configuring fail2ban..."
sudo cp /etc/fail2ban/jail.conf /etc/fail2ban/jail.local
sudo systemctl enable fail2ban
sudo systemctl start fail2ban

# Configure netdata
log "INFO" "Configuring netdata..."
sudo systemctl enable netdata
sudo systemctl start netdata

# Create monitoring directory
mkdir -p ~/.monitoring

# Create a basic system health check script
cat >/usr/local/bin/check_monitoring.sh <<'EOF'
#!/bin/bash

# System health check script
log_file="$HOME/.monitoring/health_report.log"

echo "=== System Health Report $(date) ===" > "$log_file"

# Check disk usage
echo -e "\nDisk Usage:" >> "$log_file"
df -h | grep -v "tmpfs" >> "$log_file"

# Check memory usage
echo -e "\nMemory Usage:" >> "$log_file"
free -h >> "$log_file"

# Check CPU load
echo -e "\nCPU Load:" >> "$log_file"
uptime >> "$log_file"

# Check system services
echo -e "\nCritical Services Status:" >> "$log_file"
systemctl status fail2ban netdata prometheus-node-exporter 2>/dev/null | grep "Active:" >> "$log_file"

# Check system logs for errors
echo -e "\nRecent System Errors:" >> "$log_file"
journalctl -p err -n 20 --no-pager >> "$log_file"
EOF

chmod +x /usr/local/bin/check_monitoring.sh

# Create log file
touch /var/log/monitoring_install.log
chmod 666 /var/log/monitoring_install.log

# Add cron job for daily health checks
(
  crontab -l 2>/dev/null
  echo "0 0 * * * /usr/local/bin/check_monitoring.sh"
) | crontab -

log "INFO" "System monitoring tools installation completed"
