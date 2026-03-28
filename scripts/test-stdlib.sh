#!/bin/bash
# BMB stdlib E2E Test Runner
# Runs type-check and/or interpreter verification on all stdlib test files
#
# Usage:
#   ./scripts/test-stdlib.sh               # Type-check all stdlib tests
#   ./scripts/test-stdlib.sh --run         # Also run in interpreter mode
#   ./scripts/test-stdlib.sh --verbose     # Show details
#   ./scripts/test-stdlib.sh --json        # JSON output for CI

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

TESTS_DIR="${PROJECT_ROOT}/tests/stdlib"
STDLIB_DIR="${PROJECT_ROOT}/stdlib"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Options
VERBOSE=false
JSON_OUTPUT=false
RUN_MODE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --verbose|-v) VERBOSE=true; shift ;;
        --json) JSON_OUTPUT=true; shift ;;
        --run|-r) RUN_MODE=true; shift ;;
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

MODE="check"
$RUN_MODE && MODE="check+run"

echo "=== BMB stdlib E2E Test Runner ==="
echo "Compiler: $BMB"
echo "Mode:     $MODE"
echo ""

CHECK_PASS=0
CHECK_FAIL=0
RUN_PASS=0
RUN_FAIL=0
TOTAL=0
FAILURES=""

for test_file in "${TESTS_DIR}"/test_*.bmb; do
    basename=$(basename "$test_file")
    TOTAL=$((TOTAL + 1))

    # Phase 1: Type-check
    output=$("$BMB" check "$test_file" --include "$STDLIB_DIR" 2>&1)

    if echo "$output" | grep -q '"type":"error"'; then
        CHECK_FAIL=$((CHECK_FAIL + 1))
        FAILURES="${FAILURES}\n  ${basename} (check)"
        if $VERBOSE; then
            echo -e "  ${RED}FAIL${NC} $basename [check]"
        else
            echo -e "  ${RED}FAIL${NC} $basename"
        fi
        continue
    fi

    CHECK_PASS=$((CHECK_PASS + 1))

    # Phase 2: Interpreter run (if --run)
    if $RUN_MODE; then
        run_output=$("$BMB" run "$test_file" --include "$STDLIB_DIR" 2>&1)
        if echo "$run_output" | grep -qi "error"; then
            RUN_FAIL=$((RUN_FAIL + 1))
            FAILURES="${FAILURES}\n  ${basename} (run)"
            if $VERBOSE; then
                echo -e "  ${GREEN}check${NC} ${RED}FAIL${NC} $basename [run]"
                echo "$run_output" | head -2 | while read -r line; do echo "       $line"; done
            else
                echo -e "  ${RED}FAIL${NC} $basename [run]"
            fi
        else
            RUN_PASS=$((RUN_PASS + 1))
            $VERBOSE && echo -e "  ${GREEN}PASS${NC} $basename [check+run]"
        fi
    else
        $VERBOSE && echo -e "  ${GREEN}PASS${NC} $basename [check]"
    fi
done

echo ""
echo "=== Results ==="
echo -e "  check: ${GREEN}${CHECK_PASS}${NC} pass, ${CHECK_FAIL} fail"
if $RUN_MODE; then
    echo -e "  run:   ${GREEN}${RUN_PASS}${NC} pass, ${RUN_FAIL} fail"
fi
echo "  total: $TOTAL"

if [[ -n "$FAILURES" ]]; then
    echo -e "\nFailed:${FAILURES}"
fi

if $JSON_OUTPUT; then
    echo ""
    echo "{\"check_pass\":$CHECK_PASS,\"check_fail\":$CHECK_FAIL,\"run_pass\":$RUN_PASS,\"run_fail\":$RUN_FAIL,\"total\":$TOTAL}"
fi

TOTAL_FAIL=$((CHECK_FAIL + RUN_FAIL))
if [[ $TOTAL_FAIL -gt 0 ]]; then
    exit 1
fi

echo ""
echo -e "${GREEN}All stdlib tests passed!${NC}"
