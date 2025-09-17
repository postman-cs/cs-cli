#!/bin/bash

# CS-CLI Comprehensive Regression Test Suite Runner
#
# This script runs the full regression test suite with different configurations
# to ensure the tool works correctly in all scenarios.

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get the directory containing this script (tests directory)
TESTS_DIR="$(dirname "$0")"

# Change to project root directory (parent of tests/)
cd "$TESTS_DIR/.."

# Test configuration - Load from .env if available (in tests directory)
if [ -f "$TESTS_DIR/.env" ]; then
    set -a  # Automatically export all variables
    source "$TESTS_DIR/.env"
    set +a  # Turn off auto-export
fi

# Default test configuration (can be overridden by .env or environment)
export RUST_LOG=${RUST_LOG:-cs_cli=debug}
export RUST_BACKTRACE=${RUST_BACKTRACE:-1}
export TEST_CUSTOMER_NAME=${TEST_CUSTOMER_NAME:-Fiserv}
export TEST_DAYS_BACK=${TEST_DAYS_BACK:-30}
export RUSTFLAGS=${RUSTFLAGS:-'--cfg reqwest_unstable'}

# Display header and configuration after loading environment
echo "==========================================="
echo "CS-CLI Regression Test Suite"
echo "==========================================="
echo "Test Configuration:"
echo "  TEST_CUSTOMER_NAME: $TEST_CUSTOMER_NAME"
echo "  TEST_DAYS_BACK: $TEST_DAYS_BACK"
echo "  RUST_LOG: $RUST_LOG"
echo "  RUSTFLAGS: $RUSTFLAGS"
echo "==========================================="
echo ""

# Function to run tests with specific configuration
run_test_suite() {
    local suite_name=$1
    local test_pattern=$2

    echo -e "${YELLOW}Running: $suite_name${NC}"
    echo "----------------------------------------"

    cargo test $test_pattern -- --nocapture 2>&1 | tee test_output.log
    local test_result=${PIPESTATUS[0]}
    
    if [ $test_result -eq 0 ]; then
        echo -e "${GREEN}✓ $suite_name passed${NC}"
        return 0
    else
        echo -e "${RED}✗ $suite_name failed${NC}"
        return 1
    fi
    echo ""
}

# Function to run real API integration tests (always run - no prompts)
run_integration_tests() {
    echo -e "${YELLOW}Running Integration Tests (Real API)${NC}"
    echo "----------------------------------------"
    echo "Prerequisites verified:"
    echo "  ✓ Active browser session logged into Gong (required)"
    echo "  ✓ Valid Gong account with access to customer: $TEST_CUSTOMER_NAME"
    echo "  ✓ Network connectivity to Gong APIs"
    echo "  ✓ Testing with $TEST_DAYS_BACK days of history"
    echo ""

    # Always run integration tests - no prompts
    echo "Running API Integration Tests..."
    if cargo test --test api_integration -- --nocapture; then
        echo -e "${GREEN}✓ API Integration Tests passed${NC}"
        integration_result=0
    else
        echo -e "${RED}✗ API Integration Tests failed${NC}"
        integration_result=1
    fi
    
    echo ""
    echo "Running E2E Regression Tests..."
    if cargo test --test e2e_regression -- --nocapture; then
        echo -e "${GREEN}✓ E2E Regression Tests passed${NC}"
        e2e_result=0
    else
        echo -e "${RED}✗ E2E Regression Tests failed${NC}"
        e2e_result=1
    fi
    
    # Return combined result
    return $((integration_result + e2e_result))
}

# Track test results
FAILED_TESTS=()

echo "1. Running Unit Tests"
echo "===================="
if ! run_test_suite "HTML Processing Tests" "--lib"; then
    FAILED_TESTS+=("HTML Processing")
fi

if ! run_test_suite "Performance Tests" "--test performance_test"; then
    FAILED_TESTS+=("Performance")
fi

echo ""
echo "2. Running Mock Integration Tests"
echo "================================="

if ! run_test_suite "Authentication Mock Tests" "--test auth_integration test_authentication_without_cookies"; then
    FAILED_TESTS+=("Auth Mocks")
fi

echo ""
echo "3. Running Real API Integration & E2E Tests"
echo "==========================================="
if ! run_integration_tests; then
    FAILED_TESTS+=("Integration & E2E")
fi

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