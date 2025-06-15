# Setup Scripts and Configurations

A comprehensive collection of scripts and configurations for setting up and maintaining a development environment on Ubuntu. This repository includes system setup scripts, monitoring tools, and Git workflow enhancements.

## System Requirements

- Ubuntu 22.04 LTS or Ubuntu 24.04 LTS
- Git
- Bash shell

## ⚠️ WARNING: USE AT YOUR OWN RISK

This setup modifies system configurations and installs various packages. Please review the scripts before running them.

## Getting Started

1. Clone the repository:
```bash
git clone https://github.com/X-McKay/setup.git
```

2. Run the bootstrap setup:
```bash
cd bootstrap
chmod +x main.sh scripts/*.sh
./main.sh
```

## Features

### System Setup
- Automated system configuration
- Package management
- Development tools installation
- System monitoring setup

### Monitoring Tools
The setup includes a comprehensive monitoring system with the following components:

#### System Monitoring
- **htop**: Interactive process viewer
  - Real-time process monitoring
  - CPU and memory usage visualization
  - Process tree view
  - Customizable display

- **iotop**: I/O monitoring
  - Real-time disk I/O monitoring
  - Per-process I/O usage
  - Read/write bandwidth tracking
  - I/O priority management

- **nethogs**: Network traffic monitoring
  - Per-process network usage
  - Real-time bandwidth monitoring
  - Protocol-based traffic analysis
  - Network connection tracking

- **sysstat**: System performance tools
  - `sar`: System activity reporter
  - `iostat`: I/O statistics
  - `mpstat`: CPU statistics
  - `pidstat`: Process statistics
  - Historical performance data collection

- **prometheus-node-exporter**: System metrics exporter
  - Hardware metrics collection
  - System resource monitoring
  - Prometheus-compatible metrics
  - Custom metric support

- **netdata**: Real-time system monitoring
  - Web-based monitoring dashboard
  - Real-time performance metrics
  - Custom alert configurations
  - Historical data visualization
  - Accessible at `http://localhost:19999`

- **logwatch**: Log analysis and reporting
  - Daily log summaries
  - Security event monitoring
  - System error tracking
  - Custom report configurations
  - Email notifications

- **fail2ban**: Intrusion prevention
  - SSH brute force protection
  - Custom jail configurations
  - IP-based blocking
  - Log monitoring
  - Automatic unbanning

#### Health Check System
A daily health check system is implemented with the following features:

1. **Automated Health Reports**
   - Daily reports at midnight
   - Stored in `~/.monitoring/health_report.log`
   - Comprehensive system status

2. **Report Contents**
   - Disk usage analysis
   - Memory utilization
   - CPU load statistics
   - Critical service status
   - Recent system errors
   - Network interface status

3. **Manual Health Checks**
   ```bash
   ~/.monitoring/health_check.sh
   ```
   - On-demand system health reports
   - Real-time status checking
   - Detailed error logging

4. **Monitoring Dashboard**
   - Access Netdata dashboard: `http://localhost:19999`
   - Real-time system metrics
   - Historical performance data
   - Custom alert configurations

#### Backup System
The setup includes a robust backup system with the following features:

1. **Automated Backups**
   - Daily incremental backups
   - Weekly full backups
   - Monthly archive backups
   - Configurable retention periods

2. **Backup Locations**
   - Local backup storage
   - Remote backup support
   - Cloud storage integration
   - Encrypted backup storage

3. **Backup Contents**
   - System configurations
   - User data
   - Application settings
   - Custom backup paths

4. **Backup Management**
   ```bash
   # List available backups
   backup-manager --list

   # Create manual backup
   backup-manager --backup

   # Restore from backup
   backup-manager --restore <backup-id>
   ```

5. **Backup Monitoring**
   - Backup success/failure notifications
   - Storage space monitoring
   - Backup integrity checks
   - Retention policy enforcement

### Git Workflow Enhancements

#### Pre-commit Configuration
The repository includes a comprehensive pre-commit configuration that enforces:
- YAML file validation
- JSON file validation
- Shell script linting
- Code formatting
- Security checks
- Large file prevention

#### Git Hooks
Custom Git hooks are included to maintain code quality:

1. **pre-commit hook**
   - Runs all pre-commit checks
   - Validates code style and quality
   - Prevents commits if checks fail

2. **commit-msg hook**
   - Enforces conventional commit messages
   - Format: `type(scope): description`
   - Supported types:
     - `feat`: New features
     - `fix`: Bug fixes
     - `docs`: Documentation changes
     - `style`: Code style changes
     - `refactor`: Code refactoring
     - `perf`: Performance improvements
     - `test`: Adding or modifying tests
     - `build`: Build system changes
     - `ci`: CI configuration changes
     - `chore`: Maintenance tasks
     - `revert`: Reverting changes

3. **pre-push hook**
   - Re-runs pre-commit checks
   - Verifies no uncommitted changes
   - Ensures branch is up to date with remote

### Testing
The setup includes a comprehensive test suite to verify all monitoring and backup functionality:

#### Running Tests
```bash
# Run all tests
./bootstrap/scripts/test_monitoring.sh
```

#### Test Coverage
The test suite verifies:

1. **Monitoring Tools**
   - Package installation verification
   - Service status checks
   - Port availability
   - Tool functionality tests

2. **Health Check System**
   - Script execution
   - Report generation
   - Report content verification
   - Cron job configuration

3. **Backup System**
   - Directory structure
   - Backup creation
   - Backup listing
   - Configuration verification

4. **Service Status**
   - fail2ban
   - netdata
   - prometheus-node-exporter
   - logwatch

#### Test Output
The test suite provides:
- Detailed test results
- Pass/fail status for each test
- Exit codes for failed tests
- Summary statistics
- Logging of test results

#### Troubleshooting
If tests fail:
1. Check the specific test output
2. Verify service status: `systemctl status <service-name>`
3. Check logs: `journalctl -u <service-name>`
4. Verify port availability: `netstat -tuln | grep <port>`
5. Check configuration files in `/etc/`

## Usage

### Making Commits
1. Write your code changes
2. Stage your changes: `git add .`
3. Commit with a conventional message:
   ```bash
   git commit -m "type(scope): description"
   ```
   Example: `git commit -m "feat(auth): add user authentication"`

### Pushing Changes
1. Ensure all changes are committed
2. Pull latest changes: `git pull`
3. Push your changes: `git push`

### Monitoring System Health
The setup includes a health check script that runs daily. You can also run it manually:
```bash
~/.monitoring/health_check.sh
```

## Directory Structure
```
.
├── bootstrap/
│   ├── main.sh
│   └── scripts/
│       ├── install_monitoring.sh
│       └── ...
├── .git/
│   └── hooks/
│       ├── pre-commit
│       ├── commit-msg
│       └── pre-push
├── .pre-commit-config.yaml
└── README.md
```

## Credits
- [Charmbracelet's Gum](https://github.com/charmbracelet/gum)
- [pre-commit](https://pre-commit.com/)
- [Conventional Commits](https://www.conventionalcommits.org/)
