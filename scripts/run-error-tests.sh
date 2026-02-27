#!/bin/bash
# BMB Error Detection Test Runner
# Verifies the bootstrap compiler correctly rejects malformed inputs
#
# Usage:
#   ./scripts/run-error-tests.sh --stage1 <path>

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Parse arguments
STAGE1=""
while [[ $# -gt 0 ]]; do
    case $1 in
        --stage1)
            STAGE1="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

if [ -z "$STAGE1" ]; then
    STAGE1="${PROJECT_ROOT}/target/golden-tests/bmb-stage1.exe"
fi

if [ ! -f "$STAGE1" ]; then
    echo "Error: Stage 1 binary not found at $STAGE1"
    exit 1
fi

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

PASSED=0
FAILED=0
TOTAL=0

# Test a file that should produce a parse/type error
# Usage: test_error "description" "bmb_code" "expected_error_fragment"
test_error() {
    local desc="$1"
    local code="$2"
    local expected="$3"
    TOTAL=$((TOTAL + 1))

    local tmpfile=$(mktemp /tmp/bmb_err_test_XXXXXX.bmb)
    echo "$code" > "$tmpfile"

    local output
    output=$("$STAGE1" check "$tmpfile" 2>&1) || true
    rm -f "$tmpfile"

    if echo "$output" | grep -q "$expected"; then
        echo -e "  ${GREEN}PASS${NC} $desc"
        PASSED=$((PASSED + 1))
    else
        echo -e "  ${RED}FAIL${NC} $desc"
        echo "    Expected: $expected"
        echo "    Got: $output"
        FAILED=$((FAILED + 1))
    fi
}

# Test a file that should compile successfully
test_ok() {
    local desc="$1"
    local code="$2"
    TOTAL=$((TOTAL + 1))

    local tmpfile=$(mktemp /tmp/bmb_ok_test_XXXXXX.bmb)
    echo "$code" > "$tmpfile"

    local output
    if output=$("$STAGE1" check "$tmpfile" 2>&1); then
        echo -e "  ${GREEN}PASS${NC} $desc"
        PASSED=$((PASSED + 1))
    else
        echo -e "  ${RED}FAIL${NC} $desc (expected OK, got error)"
        echo "    Output: $output"
        FAILED=$((FAILED + 1))
    fi
    rm -f "$tmpfile"
}

echo "========================================"
echo "BMB Error Detection Tests"
echo "========================================"
echo ""

# === Parse Error Tests ===
echo "Parse Errors:"

test_error "Missing closing brace" \
    'fn main() -> i64 = {
    let x = 42;
    x' \
    "PARSE:ERR"

test_error "Missing parens after fn name" \
    'fn main -> i64 = 0;' \
    "PARSE:ERR"

test_error "Missing parameter name" \
    'fn foo( = 42;' \
    "PARSE:ERR"

test_error "Missing let variable name" \
    'fn main() -> i64 = { let = 5; 0 };' \
    "PARSE:ERR"

test_error "Unclosed string literal" \
    'fn main() -> i64 = { let s = "hello; 0 };' \
    "PARSE:ERR"

test_error "Missing match arm body" \
    'fn main() -> i64 = match 1 { 1 => };' \
    "PARSE:ERR"

echo ""

# === Valid Program Tests ===
echo "Valid Programs (should not error):"

test_ok "Simple function" \
    'fn main() -> i64 = 42;'

test_ok "If-else expression" \
    'fn main() -> i64 = if true { 1 } else { 0 };'

test_ok "Match expression" \
    'fn main() -> i64 = match 1 { 1 => 10, _ => 0, };'

test_ok "While loop" \
    'fn main() -> i64 = { let mut i = 0; while i < 10 { i = i + 1; }; i };'

test_ok "String operations" \
    'fn main() -> i64 = { let s = "hello"; s.len() };'

test_ok "Array operations" \
    'fn main() -> i64 = { let a = [1, 2, 3]; a[0] };'

test_ok "Nested if-else" \
    'fn main() -> i64 = if true { if false { 1 } else { 2 } } else { 3 };'

test_ok "Multiple functions" \
    'fn add(a: i64, b: i64) -> i64 = a + b;
fn main() -> i64 = add(1, 2);'

echo ""

# === Summary ===
echo "========================================"
echo -e "Results: $PASSED/$TOTAL passed, $FAILED failed"
echo "========================================"

if [ $FAILED -gt 0 ]; then
    exit 1
fi
