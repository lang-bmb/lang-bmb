#!/bin/bash
# BMB Quick Check Script
# Part of the Bootstrap + Benchmark Cycle System
#
# Fast local verification for development:
# 1. cargo test
# 2. Stage 0→1 bootstrap only
# 3. Tier 0 benchmarks (bootstrap compile times)
#
# Usage:
#   ./scripts/quick-check.sh [options]
#
# Options:
#   --skip-tests    Skip cargo test
#   --verbose       Show detailed output
#   --json          Output results in JSON format

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Parse arguments
SKIP_TESTS=false
VERBOSE=false
JSON_OUTPUT=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --skip-tests)
            SKIP_TESTS=true
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --json)
            JSON_OUTPUT=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Colors
if [ "$JSON_OUTPUT" = true ]; then
    RED=''
    GREEN=''
    YELLOW=''
    NC=''
else
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[1;33m'
    BLUE='\033[0;34m'
    NC='\033[0m'
fi

log() {
    if [ "$JSON_OUTPUT" = false ]; then
        echo -e "$1"
    fi
}

# Timing
get_time_ms() {
    if [[ "$OSTYPE" == "darwin"* ]]; then
        python3 -c 'import time; print(int(time.time() * 1000))'
    else
        date +%s%3N 2>/dev/null || python3 -c 'import time; print(int(time.time() * 1000))'
    fi
}

# Results accumulator
declare -A RESULTS
RESULTS[tests_passed]=false
RESULTS[tests_time_ms]=0
RESULTS[bootstrap_passed]=false
RESULTS[bootstrap_time_ms]=0
RESULTS[tier0_time_ms]=0
RESULTS[total_time_ms]=0

TOTAL_START=$(get_time_ms)

log "======================================"
log "${BLUE}BMB Quick Check${NC}"
log "======================================"
log ""

# Step 1: Cargo Test
if [ "$SKIP_TESTS" = false ]; then
    log "${YELLOW}[1/3] Running cargo test...${NC}"
    TEST_START=$(get_time_ms)

    cd "$PROJECT_ROOT"
    if cargo test --release 2>&1 | tail -20; then
        RESULTS[tests_passed]=true
        log "${GREEN}Tests passed${NC}"
    else
        log "${RED}Tests failed${NC}"
        RESULTS[total_time_ms]=$(($(get_time_ms) - TOTAL_START))
        exit 1
    fi

    TEST_END=$(get_time_ms)
    RESULTS[tests_time_ms]=$((TEST_END - TEST_START))
    log ""
else
    log "${YELLOW}[1/3] Skipping tests (--skip-tests)${NC}"
    RESULTS[tests_passed]=true
    log ""
fi

# Step 2: Stage 0→1 Bootstrap
log "${YELLOW}[2/3] Running Stage 0→1 bootstrap...${NC}"
BOOTSTRAP_START=$(get_time_ms)

cd "$PROJECT_ROOT"

# Run bootstrap and capture output (preserve exit code with PIPESTATUS)
BOOTSTRAP_OUTPUT=$("$SCRIPT_DIR/bootstrap.sh" --stage1-only ${VERBOSE:+--verbose} 2>&1)
BOOTSTRAP_EXIT=$?

echo "$BOOTSTRAP_OUTPUT" | tail -10

if [ $BOOTSTRAP_EXIT -eq 0 ]; then
    RESULTS[bootstrap_passed]=true
    log "${GREEN}Bootstrap Stage 1 passed${NC}"
else
    log "${RED}Bootstrap Stage 1 failed${NC}"
    RESULTS[total_time_ms]=$(($(get_time_ms) - TOTAL_START))
    exit 1
fi

BOOTSTRAP_END=$(get_time_ms)
RESULTS[bootstrap_time_ms]=$((BOOTSTRAP_END - BOOTSTRAP_START))
log ""

# Step 3: Tier 0 Benchmarks
log "${YELLOW}[3/3] Running Tier 0 benchmarks...${NC}"
TIER0_START=$(get_time_ms)

cd "$PROJECT_ROOT"
"$SCRIPT_DIR/benchmark.sh" --tier 0 --runs 3 ${VERBOSE:+--verbose}

TIER0_END=$(get_time_ms)
RESULTS[tier0_time_ms]=$((TIER0_END - TIER0_START))
log ""

RESULTS[total_time_ms]=$(($(get_time_ms) - TOTAL_START))

# Summary
log "======================================"
log "${GREEN}Quick Check Summary${NC}"
log "======================================"
log "Tests:     ${RESULTS[tests_passed]} (${RESULTS[tests_time_ms]}ms)"
log "Bootstrap: ${RESULTS[bootstrap_passed]} (${RESULTS[bootstrap_time_ms]}ms)"
log "Tier 0:    (${RESULTS[tier0_time_ms]}ms)"
log "Total:     ${RESULTS[total_time_ms]}ms"
log ""

# JSON output
if [ "$JSON_OUTPUT" = true ]; then
    cat <<EOF
{
  "quick_check": {
    "tests": {
      "passed": ${RESULTS[tests_passed]},
      "time_ms": ${RESULTS[tests_time_ms]}
    },
    "bootstrap": {
      "passed": ${RESULTS[bootstrap_passed]},
      "time_ms": ${RESULTS[bootstrap_time_ms]}
    },
    "tier0_time_ms": ${RESULTS[tier0_time_ms]},
    "total_time_ms": ${RESULTS[total_time_ms]}
  }
}
EOF
fi

log "${GREEN}Quick check passed!${NC}"
