#!/bin/bash
# BMB stdlib E2E Test Runner
# Runs type-check verification on all stdlib test files
#
# Usage:
#   ./scripts/test-stdlib.sh           # Run all stdlib tests
#   ./scripts/test-stdlib.sh --verbose # Show details
#   ./scripts/test-stdlib.sh --json    # JSON output for CI

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

TESTS_DIR="${PROJECT_ROOT}/tests/stdlib"
STDLIB_DIR="${PROJECT_ROOT}/stdlib"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

# Options
VERBOSE=false
JSON_OUTPUT=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --verbose|-v) VERBOSE=true; shift ;;
        --json) JSON_OUTPUT=true; shift ;;
        *) echo "Unknown option: $1"; exit 1 ;;
    esac
done

# Find BMB compiler
if [[ -f "${PROJECT_ROOT}/target/release/bmb.exe" ]]; then
    BMB="${PROJECT_ROOT}/target/release/bmb.exe"
elif [[ -f "${PROJECT_ROOT}/target/release/bmb" ]]; then
    BMB="${PROJECT_ROOT}/target/release/bmb"
else
    echo -e "${RED}Error: BMB compiler not found. Run 'cargo build --release' first.${NC}"
    exit 1
fi

# Known failures (empty — all resolved with nested module support in v0.98)
KNOWN_FAIL=""

echo "=== BMB stdlib E2E Test Runner ==="
echo "Compiler: $BMB"
echo "Tests:    $TESTS_DIR"
echo "stdlib:   $STDLIB_DIR"
echo ""

PASS=0
FAIL=0
SKIP=0
TOTAL=0
FAILURES=""

for test_file in "${TESTS_DIR}"/test_*.bmb; do
    basename=$(basename "$test_file")
    TOTAL=$((TOTAL + 1))

    # Check if known failure
    if echo "$KNOWN_FAIL" | grep -q "$basename"; then
        if $VERBOSE; then
            echo -e "  ${YELLOW}SKIP${NC} $basename (known: core:: module resolution)"
        fi
        SKIP=$((SKIP + 1))
        continue
    fi

    # Run type-check with include path
    output=$("$BMB" check "$test_file" --include "$STDLIB_DIR" 2>&1)

    if echo "$output" | grep -q '"type":"error"'; then
        FAIL=$((FAIL + 1))
        FAILURES="${FAILURES}\n  ${basename}"
        if $VERBOSE; then
            echo -e "  ${RED}FAIL${NC} $basename"
            echo "$output" | grep '"type":"error"' | head -3 | while read -r line; do
                msg=$(echo "$line" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('message',''))" 2>/dev/null || echo "$line")
                echo "       $msg"
            done
        else
            echo -e "  ${RED}FAIL${NC} $basename"
        fi
    else
        PASS=$((PASS + 1))
        if $VERBOSE; then
            # Count warnings
            warn_count=$(echo "$output" | grep -c '"type":"warning"' || true)
            echo -e "  ${GREEN}PASS${NC} $basename ($warn_count warnings)"
        fi
    fi
done

echo ""
echo "=== Results ==="
echo -e "  ${GREEN}PASS${NC}: $PASS"
if [[ $FAIL -gt 0 ]]; then
    echo -e "  ${RED}FAIL${NC}: $FAIL"
fi
if [[ $SKIP -gt 0 ]]; then
    echo -e "  ${YELLOW}SKIP${NC}: $SKIP (known issues)"
fi
echo "  Total: $TOTAL"

if [[ -n "$FAILURES" ]]; then
    echo -e "\nFailed tests:${FAILURES}"
fi

if $JSON_OUTPUT; then
    echo ""
    echo "{\"pass\":$PASS,\"fail\":$FAIL,\"skip\":$SKIP,\"total\":$TOTAL}"
fi

# Exit with failure if any unexpected failures
if [[ $FAIL -gt 0 ]]; then
    exit 1
fi

echo ""
echo -e "${GREEN}All stdlib tests passed!${NC}"
