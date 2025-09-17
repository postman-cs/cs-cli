#!/bin/bash

# CS-CLI Comprehensive Regression Test Suite Runner
#
# This script runs the full regression test suite with different configurations
# to ensure the tool works correctly in all scenarios.

set -e

echo "==========================================="
echo "CS-CLI Regression Test Suite"
echo "==========================================="
echo "Test Configuration:"
echo "  USE_REAL_API: $USE_REAL_API"
echo "  TEST_CUSTOMER_NAME: $TEST_CUSTOMER_NAME"
echo "  TEST_DAYS_BACK: $TEST_DAYS_BACK"
echo "  RUST_LOG: $RUST_LOG"
echo "  RUSTFLAGS: $RUSTFLAGS"
echo "==========================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test configuration - Load from .env if available
if [ -f .env ]; then
    export $(cat .env | grep -v ^# | xargs)
fi

# Default test configuration (can be overridden by .env or environment)
export RUST_LOG=${RUST_LOG:-cs_cli=debug}
export RUST_BACKTRACE=${RUST_BACKTRACE:-1}
export USE_REAL_API=${USE_REAL_API:-false}
export TEST_CUSTOMER_NAME=${TEST_CUSTOMER_NAME:-Fiserv}
export TEST_DAYS_BACK=${TEST_DAYS_BACK:-30}
export RUSTFLAGS=${RUSTFLAGS:-'--cfg reqwest_unstable'}

# Function to run tests with specific configuration
run_test_suite() {
    local suite_name=$1
    local test_pattern=$2
    local use_real_api=$3

    echo -e "${YELLOW}Running: $suite_name${NC}"
    echo "----------------------------------------"

    export USE_REAL_API=$use_real_api

    if cargo test $test_pattern -- --nocapture 2>&1 | tee test_output.log; then
        echo -e "${GREEN}✓ $suite_name passed${NC}"
        return 0
    else
        echo -e "${RED}✗ $suite_name failed${NC}"
        return 1
    fi
    echo ""
}

# Function to run ignored tests (that require real API)
run_integration_tests() {
    echo -e "${YELLOW}Running Integration Tests (Real API)${NC}"
    echo "----------------------------------------"
    echo "NOTE: These tests require:"
    echo "  - Active browser session logged into Gong"
    echo "  - Valid Gong account with access to customer: $TEST_CUSTOMER_NAME"
    echo "  - Network connectivity"
    echo "  - Will test with $TEST_DAYS_BACK days of history"
    echo ""

    read -p "Run integration tests? (y/n) " -n 1 -r
    echo ""

    if [[ $REPLY =~ ^[Yy]$ ]]; then
        export USE_REAL_API=true
        cargo test -- --ignored --nocapture
    else
        echo "Skipping integration tests"
    fi
}

# Track test results
FAILED_TESTS=()

echo "1. Running Unit Tests"
echo "===================="
if ! run_test_suite "HTML Processing Tests" "html_test" "false"; then
    FAILED_TESTS+=("HTML Processing")
fi

if ! run_test_suite "Performance Tests" "performance_test" "false"; then
    FAILED_TESTS+=("Performance")
fi

echo ""
echo "2. Running Mock Integration Tests"
echo "================================="
# These tests use mocked APIs
export USE_REAL_API=false

if ! run_test_suite "Authentication Mock Tests" "test_authentication_without_cookies" "false"; then
    FAILED_TESTS+=("Auth Mocks")
fi

if ! run_test_suite "E2E Workflow Tests" "e2e_regression::test_cli" "false"; then
    FAILED_TESTS+=("E2E Workflow")
fi

echo ""
echo "3. Optional: Real API Integration Tests"
echo "======================================="
run_integration_tests

echo ""
echo "4. Running Clippy Checks"
echo "========================"
if cargo clippy -- -D warnings; then
    echo -e "${GREEN}✓ Clippy checks passed${NC}"
else
    echo -e "${RED}✗ Clippy checks failed${NC}"
    FAILED_TESTS+=("Clippy")
fi

echo ""
echo "5. Running Format Check"
echo "======================="
if cargo fmt -- --check; then
    echo -e "${GREEN}✓ Format check passed${NC}"
else
    echo -e "${RED}✗ Format check failed${NC}"
    echo "Run 'cargo fmt' to fix formatting"
    FAILED_TESTS+=("Formatting")
fi

echo ""
echo "==========================================="
echo "Test Suite Summary"
echo "==========================================="

if [ ${#FAILED_TESTS[@]} -eq 0 ]; then
    echo -e "${GREEN}✓ All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}✗ Some tests failed:${NC}"
    for test in "${FAILED_TESTS[@]}"; do
        echo "  - $test"
    done
    exit 1
fi