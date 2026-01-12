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

# Install backup tools
log "INFO" "Installing backup and recovery tools..."

BACKUP_PACKAGES=(
  rsync        # File synchronization and transfer
  rdiff-backup # Incremental backup
  duplicity    # Encrypted backup
  timeshift    # System restore tool
  deja-dup     # Simple backup tool
)

# Install packages
for package in "${BACKUP_PACKAGES[@]}"; do
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

# Create backup directory structure
mkdir -p ~/.backup/{configs,data,system}

# Create backup configuration
cat >~/.backup/configs/backup_config.sh <<'EOF'
#!/bin/bash

# Backup configuration
BACKUP_ROOT="$HOME/.backup"
CONFIGS_DIR="$BACKUP_ROOT/configs"
DATA_DIR="$BACKUP_ROOT/data"
SYSTEM_DIR="$BACKUP_ROOT/system"

# Important directories to backup
IMPORTANT_DIRS=(
    "$HOME/.config"
    "$HOME/.local"
    "$HOME/Documents"
    "$HOME/Pictures"
    "$HOME/.ssh"
)

# System files to backup
SYSTEM_FILES=(
    "/etc/fstab"
    "/etc/hosts"
    "/etc/apt/sources.list"
    "/etc/apt/sources.list.d"
)

# Create backup function
create_backup() {
    local backup_type=$1
    local timestamp=$(date +%Y%m%d_%H%M%S)
    local backup_dir="$BACKUP_ROOT/$backup_type"

    case $backup_type in
        "configs")
            # Backup important directories
            for dir in "${IMPORTANT_DIRS[@]}"; do
                if [ -d "$dir" ]; then
                    rsync -av --delete "$dir" "$backup_dir/"
                fi
            done
            ;;
        "system")
            # Backup system files
            for file in "${SYSTEM_FILES[@]}"; do
                if [ -e "$file" ]; then
                    sudo rsync -av "$file" "$backup_dir/"
                fi
            done
            ;;
        "data")
            # Timeshift backups are now handled by Timeshift's built-in scheduler
            # See /etc/timeshift/timeshift.json for retention settings
            ;;
    esac
}

# Restore function
restore_backup() {
    local backup_type=$1
    local backup_date=$2

    case $backup_type in
        "configs")
            # Restore from configs backup
            rsync -av "$BACKUP_ROOT/configs/$backup_date/" "$HOME/"
            ;;
        "system")
            # Restore system files
            sudo rsync -av "$BACKUP_ROOT/system/$backup_date/" "/"
            ;;
        "data")
            # Restore using timeshift
            sudo timeshift --restore --snapshot "$backup_date"
            ;;
    esac
}
EOF

chmod +x ~/.backup/configs/backup_config.sh

# Create backup script
cat >~/.backup/backup.sh <<'EOF'
#!/bin/bash

source "$HOME/.backup/configs/backup_config.sh"

# Create timestamp
timestamp=$(date +%Y%m%d_%H%M%S)

# Create backup directories
mkdir -p "$CONFIGS_DIR/$timestamp"
mkdir -p "$SYSTEM_DIR/$timestamp"

# Run backups
create_backup "configs"
create_backup "system"
create_backup "data"

# Cleanup old backups (keep last 7 days)
find "$CONFIGS_DIR" -type d -mtime +7 -exec rm -rf {} \;
find "$SYSTEM_DIR" -type d -mtime +7 -exec rm -rf {} \;

# Log backup completion
echo "Backup completed at $(date)" >> "$BACKUP_ROOT/backup.log"
EOF

chmod +x ~/.backup/backup.sh

# Create restore script
cat >~/.backup/restore.sh <<'EOF'
#!/bin/bash

source "$HOME/.backup/configs/backup_config.sh"

# Check arguments
if [ $# -ne 2 ]; then
    echo "Usage: $0 <backup_type> <backup_date>"
    echo "Example: $0 configs 20240315_120000"
    exit 1
fi

backup_type=$1
backup_date=$2

# Verify backup exists
if [ ! -d "$BACKUP_ROOT/$backup_type/$backup_date" ]; then
    echo "Backup not found: $BACKUP_ROOT/$backup_type/$backup_date"
    exit 1
fi

# Confirm restore
read -p "Are you sure you want to restore from $backup_type backup dated $backup_date? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Restore cancelled"
    exit 1
fi

# Perform restore
restore_backup "$backup_type" "$backup_date"

echo "Restore completed at $(date)" >> "$BACKUP_ROOT/restore.log"
EOF

chmod +x ~/.backup/restore.sh

# Add cron job for daily backups
(
  crontab -l 2>/dev/null
  echo "0 2 * * * $HOME/.backup/backup.sh"
) | crontab -

# Create initial backup
log "INFO" "Creating initial backup..."
"$HOME/.backup/backup.sh"

log "INFO" "Backup and recovery setup completed"
