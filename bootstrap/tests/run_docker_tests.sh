#!/bin/bash

# shellcheck disable=SC2034
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to check if Docker is installed
check_docker() {
  if ! command -v docker &>/dev/null; then
    echo -e "${RED}Error: Docker is not installed${NC}"
    exit 1
  fi
}

# Function to build Docker image
build_image() {
  echo -e "${YELLOW}Building Docker image...${NC}"
  if ! docker build -t bootstrap-tests -f bootstrap/tests/Dockerfile .; then
    echo "Docker build failed"
    exit 1
  fi
}

# Function to run tests in Docker
run_tests() {
  echo -e "${YELLOW}Running tests in Docker container...${NC}"
  if ! docker run --rm bootstrap-tests; then
    echo "Docker test run failed"
    exit 1
  fi
}

# Main execution
echo -e "${YELLOW}Starting Docker test environment${NC}"
echo "============================================"

# Check Docker installation
check_docker

# Build Docker image
build_image

# Run tests
run_tests

echo -e "${GREEN}All tests completed successfully${NC}"
