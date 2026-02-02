#!/bin/bash
# Bootstrap Compiler Test Runner
# Run with: ./scripts/run_bootstrap_tests.sh

set -e

BMB="./target/x86_64-pc-windows-gnu/release/bmb.exe"
TEST_DIR="tests/bootstrap"

echo "====================================="
echo "Bootstrap Compiler Test Suite"
echo "====================================="
echo ""

TOTAL_PASS=0
TOTAL_FAIL=0

run_test() {
    local test_file=$1
    local expected=$2
    local test_name=$(basename "$test_file" .bmb)

    echo -n "Running $test_name... "

    # Run the test and capture output
    output=$($BMB run "$test_file" 2>&1)
    # Get second to last line (total before final marker 999)
    result=$(echo "$output" | tail -2 | head -1)

    if [ "$result" = "$expected" ]; then
        echo "PASS ($result)"
        TOTAL_PASS=$((TOTAL_PASS + 1))
    else
        echo "FAIL (expected $expected, got $result)"
        TOTAL_FAIL=$((TOTAL_FAIL + 1))
    fi
}

# Run tests
run_test "$TEST_DIR/phi_patterns_test.bmb" "39"
run_test "$TEST_DIR/compiler_fixes_test.bmb" "17"

# Also run the selfhost_test from bootstrap directory
if [ -f "bootstrap/selfhost_test.bmb" ]; then
    echo -n "Running selfhost_test... "
    output=$($BMB run "bootstrap/selfhost_test.bmb" 2>&1)
    # Check for expected markers
    if echo "$output" | grep -q "^999$"; then
        echo "PASS"
        TOTAL_PASS=$((TOTAL_PASS + 1))
    else
        echo "FAIL"
        TOTAL_FAIL=$((TOTAL_FAIL + 1))
    fi
fi

echo ""
echo "====================================="
echo "Results: $TOTAL_PASS passed, $TOTAL_FAIL failed"
echo "====================================="

if [ $TOTAL_FAIL -gt 0 ]; then
    exit 1
fi
