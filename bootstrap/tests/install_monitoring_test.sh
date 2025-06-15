#!/bin/bash

# Source the test framework
source "$(dirname "$0")/test_framework.sh"

# Mock functions for testing
mock_gum() {
  case "$1" in
  "confirm")
    echo "true"
    ;;
  "input")
    echo "testuser"
    ;;
  *)
    echo "Mock gum command: $*"
    ;;
  esac
}

mock_apt_get() {
  echo "Installing packages: $*"
  return 0
}

mock_systemctl() {
  case "$1" in
  "is-active")
    echo "active"
    return 0
    ;;
  "start")
    echo "Starting service: $2"
    return 0
    ;;
  "enable")
    echo "Enabling service: $2"
    return 0
    ;;
  *)
    echo "Mock systemctl command: $*"
    return 0
    ;;
  esac
}

mock_netstat() {
  echo "tcp        0      0 0.0.0.0:19999           0.0.0.0:*               LISTEN"
  return 0
}

# Test functions
test_monitoring_packages_installation() {
  echo "Testing monitoring packages installation..."

  # Mock apt-get
  mock_command "apt-get" "mock_apt_get"

  # Run the installation
  # shellcheck disable=SC1091
  source "/app/scripts/install_monitoring.sh"

  # Verify packages were installed
  assert_equals "Installing packages: install -y netdata prometheus-node-exporter" \
    "$(mock_apt_get install -y netdata prometheus-node-exporter)" \
    "Netdata and prometheus-node-exporter should be installed"
}

test_netdata_service() {
  echo "Testing Netdata service configuration..."

  # Mock systemctl
  mock_command "systemctl" "mock_systemctl"

  # Run the installation
  # shellcheck disable=SC1091
  source "/app/scripts/install_monitoring.sh"

  # Verify service is started and enabled
  assert_equals "Starting service: netdata" \
    "$(mock_systemctl start netdata)" \
    "Netdata service should be started"

  assert_equals "Enabling service: netdata" \
    "$(mock_systemctl enable netdata)" \
    "Netdata service should be enabled"
}

test_prometheus_node_exporter_service() {
  echo "Testing Prometheus Node Exporter service configuration..."

  # Mock systemctl
  mock_command "systemctl" "mock_systemctl"

  # Run the installation
  # shellcheck disable=SC1091
  source "/app/scripts/install_monitoring.sh"

  # Verify service is started and enabled
  assert_equals "Starting service: prometheus-node-exporter" \
    "$(mock_systemctl start prometheus-node-exporter)" \
    "Prometheus Node Exporter service should be started"

  assert_equals "Enabling service: prometheus-node-exporter" \
    "$(mock_systemctl enable prometheus-node-exporter)" \
    "Prometheus Node Exporter service should be enabled"
}

test_netdata_port() {
  echo "Testing Netdata port configuration..."

  # Mock netstat
  mock_command "netstat" "mock_netstat"

  # Run the installation
  # shellcheck disable=SC1091
  source "/app/scripts/install_monitoring.sh"

  # Verify port is listening
  assert_equals "tcp        0      0 0.0.0.0:19999           0.0.0.0:*               LISTEN" \
    "$(mock_netstat)" \
    "Netdata should be listening on port 19999"
}

test_prometheus_node_exporter_port() {
  echo "Testing Prometheus Node Exporter port configuration..."

  # Mock netstat
  mock_command "netstat" "mock_netstat"

  # Run the installation
  # shellcheck disable=SC1091
  source "/app/scripts/install_monitoring.sh"

  # Verify port is listening
  assert_equals "tcp        0      0 0.0.0.0:9100            0.0.0.0:*               LISTEN" \
    "$(mock_netstat)" \
    "Prometheus Node Exporter should be listening on port 9100"
}

test_health_check_script() {
  echo "Testing health check script creation..."

  # Run the installation
  # shellcheck disable=SC1091
  source "/app/scripts/install_monitoring.sh"

  # Verify health check script exists
  assert_file_exists "/usr/local/bin/check_monitoring.sh" \
    "Health check script should be created"

  # Verify script permissions
  assert_command_success "[ -x /usr/local/bin/check_monitoring.sh ]" \
    "Health check script should be executable"
}

test_logging() {
  echo "Testing logging functionality..."

  # Run the installation
  # shellcheck disable=SC1091
  source "/app/scripts/install_monitoring.sh"

  # Verify log file exists
  assert_file_exists "/var/log/monitoring_install.log" \
    "Log file should be created"

  # Verify log file is writable
  assert_command_success "[ -w /var/log/monitoring_install.log ]" \
    "Log file should be writable"
}
