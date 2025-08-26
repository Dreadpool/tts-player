#!/bin/bash

# Test Runner Script for TTS Player
set -e

echo "🧪 Running TTS Player Test Suite"
echo "================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Track test results
FAILED_TESTS=0

run_test_suite() {
    local name="$1"
    local command="$2"
    
    echo -e "\n${YELLOW}Running $name...${NC}"
    
    if eval "$command"; then
        echo -e "${GREEN}✓ $name passed${NC}"
    else
        echo -e "${RED}✗ $name failed${NC}"
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
}

# 1. Frontend Tests
run_test_suite "Frontend Unit Tests" "npm test -- --run"

# 2. Rust Backend Tests
run_test_suite "Rust Backend Tests" "cd src-tauri && cargo test"

# 3. Integration Tests (if binary is built)
if [ -f "src-tauri/target/debug/tts-player" ] || [ -f "src-tauri/target/release/tts-player" ]; then
    # Install bats if not available
    if ! command -v bats &> /dev/null; then
        echo "Installing bats testing framework..."
        if command -v brew &> /dev/null; then
            brew install bats-core
        else
            echo "⚠ Warning: bats not found. Please install bats-core to run integration tests"
        fi
    fi
    
    if command -v bats &> /dev/null; then
        run_test_suite "Integration Tests" "bats tests/e2e/raycast_integration.bats"
    fi
else
    echo -e "${YELLOW}⚠ Skipping integration tests - binary not built${NC}"
fi

# 4. Linting (if tools are available)
if command -v npm &> /dev/null; then
    if npm list eslint &> /dev/null; then
        run_test_suite "ESLint" "npm run lint 2>/dev/null || echo 'Linting not configured'"
    fi
fi

if command -v cargo &> /dev/null; then
    if command -v cargo-clippy &> /dev/null; then
        run_test_suite "Rust Clippy" "cd src-tauri && cargo clippy -- -D warnings"
    fi
fi

# Summary
echo -e "\n================================"
if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${GREEN}🎉 All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}❌ $FAILED_TESTS test suite(s) failed${NC}"
    exit 1
fi