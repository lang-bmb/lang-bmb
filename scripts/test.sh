#!/bin/bash
# BMB Test Runner Script
# Phase v0.62: Dogfooding I - Test Framework
#
# Runs bmb-test in interpreter mode to execute all tests
#
# Usage:
#   ./scripts/test.sh           # Run all tests
#   ./scripts/test.sh --verbose # Run with verbose output

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "=== BMB Test Runner ==="
echo ""

# Check if BMB compiler exists
if [[ -f "./target/release/bmb.exe" ]]; then
    BMB="./target/release/bmb.exe"
elif [[ -f "./target/release/bmb" ]]; then
    BMB="./target/release/bmb"
else
    echo -e "${RED}Error: BMB compiler not found. Run 'cargo build --release' first.${NC}"
    exit 1
fi

echo "Using compiler: $BMB"
echo "Running tests via: $BMB run tools/bmb-test/main.bmb"
echo ""

# Run the test runner
if $BMB run tools/bmb-test/main.bmb; then
    echo ""
    echo -e "${GREEN}All tests passed!${NC}"
    exit 0
else
    echo ""
    echo -e "${RED}Some tests failed.${NC}"
    exit 1
fi
