#!/bin/bash

# shellcheck disable=SC2317
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

# Test results tracking
PASSED=0
FAILED=0
TOTAL=0

# Function to run a test
run_test() {
  local test_name=$1
  local test_command=$2
  local expected_exit=$3

  echo "Running test: $test_name"
  eval "$test_command"
  local exit_code=$?

  if [ $exit_code -eq "${expected_exit:-0}" ]; then
    echo "✅ PASSED: $test_name"
    ((PASSED++))
  else
    echo "❌ FAILED: $test_name (Exit code: $exit_code)"
    ((FAILED++))
  fi
  ((TOTAL++))
  echo "----------------------------------------"
}

# Function to check if a service is running
check_service() {
  local service=$1
  systemctl is-active --quiet "$service"
  return $?
}

# Function to check if a port is listening
check_port() {
  local port=$1
  netstat -tuln | grep -q ":$port "
  return $?
}

echo "Starting monitoring and backup system tests..."
echo "============================================"

# Test 1: Verify monitoring tools installation
run_test "Verify monitoring tools installation" \
  "dpkg -l | grep -E 'htop|iotop|nethogs|sysstat|prometheus-node-exporter|netdata|logwatch|fail2ban'"

# Test 2: Check if monitoring services are running
echo "Checking monitoring services..."
services=("fail2ban" "netdata" "prometheus-node-exporter")
for service in "${services[@]}"; do
  run_test "Check $service service" "check_service $service"
done

# Test 3: Verify monitoring ports
echo "Checking monitoring ports..."
run_test "Check Netdata port (19999)" "check_port 19999"
run_test "Check Prometheus Node Exporter port (9100)" "check_port 9100"

# Test 4: Test health check script
run_test "Test health check script" \
  "bash ~/.monitoring/health_check.sh && [ -f ~/.monitoring/health_report.log ]"

# Test 5: Verify health report contents
run_test "Verify health report contents" \
  "grep -q 'System Health Report' ~/.monitoring/health_report.log && \
     grep -q 'Disk Usage' ~/.monitoring/health_report.log && \
     grep -q 'Memory Usage' ~/.monitoring/health_report.log && \
     grep -q 'CPU Load' ~/.monitoring/health_report.log"

# Test 6: Test monitoring tools functionality
run_test "Test htop" "htop --version"
run_test "Test iotop" "iotop --version"
run_test "Test nethogs" "nethogs --version"
run_test "Test sysstat tools" "sar --version && iostat --version"

# Test 7: Verify logwatch configuration
run_test "Verify logwatch configuration" \
  "[ -f /etc/logwatch/conf/logwatch.conf ] && \
     grep -q 'Detail = High' /etc/logwatch/conf/logwatch.conf"

# Test 8: Test fail2ban configuration
run_test "Verify fail2ban configuration" \
  "[ -f /etc/fail2ban/jail.local ] && \
     systemctl is-active --quiet fail2ban"

# Test 9: Check monitoring directory structure
run_test "Verify monitoring directory structure" \
  "[ -d ~/.monitoring ] && \
     [ -f ~/.monitoring/health_check.sh ] && \
     [ -x ~/.monitoring/health_check.sh ]"

# Test 10: Verify cron job for health checks
run_test "Verify health check cron job" \
  "crontab -l | grep -q 'health_check.sh'"

# Test 11: Test backup system
run_test "Verify backup directory" \
  "[ -d ~/.backup ]"

run_test "Test backup creation" \
  "backup-manager --backup"

run_test "List available backups" \
  "backup-manager --list"

# Print test summary
echo "============================================"
echo "Test Summary:"
echo "Total Tests: $TOTAL"
echo "Passed: $PASSED"
echo "Failed: $FAILED"
echo "============================================"

# Exit with error if any tests failed
if [ $FAILED -gt 0 ]; then
  log "ERROR" "Some tests failed. Please check the output above."
  exit 1
else
  log "INFO" "All tests passed successfully!"
  exit 0
fi
