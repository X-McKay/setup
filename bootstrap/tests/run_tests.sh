#!/bin/bash

# Source the test framework
source "$(dirname "$0")/test_framework.sh"

# Set test mode
export TEST_MODE=1

# shellcheck disable=SC2317
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

# Run all tests in the current directory
echo "Starting test suite"
echo "============================================"

for test_file in *_test.sh; do
  if [ -f "$test_file" ]; then
    run_tests "$test_file"
  fi
done

echo -e "\nTest Suite Complete"

# Exit with error if any tests failed
if [ $TESTS_FAILED -gt 0 ]; then
  exit 1
else
  exit 0
fi

# shellcheck disable=SC2317
# This is a mock function for testing
mock_function() {
  echo "Mock function called"
}
