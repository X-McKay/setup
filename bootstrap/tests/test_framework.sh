#!/bin/bash

# Test framework for unit testing
# This provides common testing utilities and assertions

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test statistics
declare -i TESTS_RUN=0
declare -i TESTS_PASSED=0
declare -i TESTS_FAILED=0

# Prevent recursive test execution
if [ -n "$TEST_FRAMEWORK_LOADED" ]; then
  return 0
fi
export TEST_FRAMEWORK_LOADED=1

# Mock gum command for non-interactive testing
gum() {
  if [ "$TEST_MODE" = "1" ]; then
    mock_gum "$@"
  else
    command gum "$@"
  fi
}

# Setup test environment
setup_test_env() {
  # Create temporary directory for test files
  TEST_DIR=$(mktemp -d)
  export TEST_DIR

  # Create test log file
  TEST_LOG="$TEST_DIR/test.log"
  touch "$TEST_LOG"

  # Setup test environment variables
  export TEST_MODE=1
  export TEST_USER=${TEST_USER:-"testuser"}
  export TEST_HOME=${TEST_HOME:-"/home/$TEST_USER"}
  mkdir -p "$TEST_HOME"

  # Mock system commands
  mock_command "sudo" "echo 'sudo command executed'" 0
  mock_command "systemctl" "echo 'Service status: active'" 0
  mock_command "netstat" "echo 'tcp        0      0 0.0.0.0:19999           0.0.0.0:*               LISTEN'" 0

  # Create necessary directories in test environment
  mkdir -p "$TEST_HOME/.monitoring"
  mkdir -p "$TEST_HOME/.backup"
  mkdir -p "$TEST_HOME/.bootstrap"
}

# Cleanup test environment
cleanup_test_env() {
  if [ -d "$TEST_DIR" ]; then
    rm -rf "$TEST_DIR"
  fi
}

# Assertion functions
assert_equals() {
  local expected=$1
  local actual=$2
  local message=${3:-"Values should be equal"}

  ((TESTS_RUN++))

  if [ "$expected" = "$actual" ]; then
    echo -e "${GREEN}✓ PASS${NC}: $message"
    echo "Expected: $expected"
    echo "Actual: $actual"
    ((TESTS_PASSED++))
    return 0
  else
    echo -e "${RED}✗ FAIL${NC}: $message"
    echo "Expected: $expected"
    echo "Actual: $actual"
    ((TESTS_FAILED++))
    return 1
  fi
}

assert_not_equals() {
  local expected=$1
  local actual=$2
  local message=${3:-"Values should not be equal"}

  ((TESTS_RUN++))

  if [ "$expected" != "$actual" ]; then
    echo -e "${GREEN}✓ PASS${NC}: $message"
    ((TESTS_PASSED++))
    return 0
  else
    echo -e "${RED}✗ FAIL${NC}: $message"
    echo "Expected: $expected"
    echo "Actual: $actual"
    ((TESTS_FAILED++))
    return 1
  fi
}

assert_file_exists() {
  local file=$1
  local message=${2:-"File should exist"}

  ((TESTS_RUN++))

  if [ -f "$file" ]; then
    echo -e "${GREEN}✓ PASS${NC}: $message"
    ((TESTS_PASSED++))
    return 0
  else
    echo -e "${RED}✗ FAIL${NC}: $message"
    echo "File not found: $file"
    ((TESTS_FAILED++))
    return 1
  fi
}

assert_directory_exists() {
  local dir=$1
  local message=${2:-"Directory should exist"}

  ((TESTS_RUN++))

  if [ -d "$dir" ]; then
    echo -e "${GREEN}✓ PASS${NC}: $message"
    ((TESTS_PASSED++))
    return 0
  else
    echo -e "${RED}✗ FAIL${NC}: $message"
    echo "Directory not found: $dir"
    ((TESTS_FAILED++))
    return 1
  fi
}

assert_command_success() {
  local command=$1
  local message=${2:-"Command should succeed"}

  ((TESTS_RUN++))

  if eval "$command" >/dev/null 2>&1; then
    echo -e "${GREEN}✓ PASS${NC}: $message"
    ((TESTS_PASSED++))
    return 0
  else
    echo -e "${RED}✗ FAIL${NC}: $message"
    echo "Command failed: $command"
    ((TESTS_FAILED++))
    return 1
  fi
}

assert_command_fails() {
  local command=$1
  local message=${2:-"Command should fail"}

  ((TESTS_RUN++))

  if ! eval "$command" >/dev/null 2>&1; then
    echo -e "${GREEN}✓ PASS${NC}: $message"
    ((TESTS_PASSED++))
    return 0
  else
    echo -e "${RED}✗ FAIL${NC}: $message"
    echo "Command succeeded when it should have failed: $command"
    ((TESTS_FAILED++))
    return 1
  fi
}

# Mock functions for testing
mock_command() {
  local command=$1
  local output=$2
  local exit_code=${3:-0}

  # Create mock function
  eval "$command() { echo \"$output\"; return $exit_code; }"
  export -f "${command?}"
}

# Test runner
run_tests() {
  local test_file=$1
  local test_name
  test_name=$(basename "$test_file" .sh)

  echo -e "\n${YELLOW}Running tests in $test_name${NC}"
  echo "============================================"

  # shellcheck source=/dev/null
  source "$test_file"

  # Run all test functions
  grep -E '^test_[a-zA-Z0-9_]+' "$test_file" | cut -d'(' -f1 | while read -r func; do
    echo -e "\n${YELLOW}Running $func${NC}"
    $func
  done

  echo -e "\n${YELLOW}Test Summary for $test_name${NC}"
  echo "============================================"
  echo "Tests Run: $TESTS_RUN"
  echo "Tests Passed: $TESTS_PASSED"
  echo "Tests Failed: $TESTS_FAILED"
  echo "============================================"

  # Reset test counters
  TESTS_RUN=0
  TESTS_PASSED=0
  TESTS_FAILED=0
}

# Run all tests in a directory
run_all_tests() {
  local test_dir=$1

  echo -e "${YELLOW}Starting test suite${NC}"
  echo "============================================"

  # Setup test environment
  setup_test_env

  # Run each test file
  for test_file in "$test_dir"/*_test.sh; do
    if [ -f "$test_file" ]; then
      run_tests "$test_file"
    fi
  done

  # Cleanup test environment
  cleanup_test_env

  echo -e "\n${YELLOW}Test Suite Summary${NC}"
  echo "============================================"
  echo "Total Tests Run: $TESTS_RUN"
  echo "Total Tests Passed: $TESTS_PASSED"
  echo "Total Tests Failed: $TESTS_FAILED"
  echo "============================================"

  # Exit with error if any tests failed
  if [ $TESTS_FAILED -gt 0 ]; then
    exit 1
  else
    exit 0
  fi
}
