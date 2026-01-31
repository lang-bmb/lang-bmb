#!/bin/bash
# BMB Full Cycle Verification Script
# Part of the Bootstrap + Benchmark Cycle System
#
# Complete pre-PR verification:
# 1. cargo test --release
# 2. Full 3-stage bootstrap (Stage 0→1→2→3)
# 3. All Tier 0-3 benchmarks
# 4. Regression comparison (if baseline exists)
#
# Usage:
#   ./scripts/full-cycle.sh [options]
#
# Options:
#   --baseline FILE     Baseline JSON file for comparison
#   --output DIR        Output directory for results
#   --skip-tests        Skip cargo tests
#   --tier1-strict      Fail on any Tier 1 regression (default: warning only)
#   --verbose           Show detailed output
#   --json              Output results in JSON format

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
OUTPUT_DIR="${PROJECT_ROOT}/target/full-cycle"
BASELINE_FILE=""

# Parse arguments
SKIP_TESTS=false
TIER1_STRICT=false
VERBOSE=false
JSON_OUTPUT=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --baseline)
            BASELINE_FILE="$2"
            shift 2
            ;;
        --output)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        --skip-tests)
            SKIP_TESTS=true
            shift
            ;;
        --tier1-strict)
            TIER1_STRICT=true
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

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Results
declare -A RESULTS
RESULTS[tests_passed]=false
RESULTS[tests_time_ms]=0
RESULTS[bootstrap_passed]=false
RESULTS[bootstrap_fixed_point]=false
RESULTS[bootstrap_time_ms]=0
RESULTS[benchmark_time_ms]=0
RESULTS[comparison_passed]=true
RESULTS[total_time_ms]=0

TOTAL_START=$(get_time_ms)

log "======================================"
log "${BLUE}BMB Full Cycle Verification${NC}"
log "======================================"
log "Output: $OUTPUT_DIR"
log ""

# Step 1: Cargo Test
if [ "$SKIP_TESTS" = false ]; then
    log "${YELLOW}[1/4] Running cargo test --release...${NC}"
    TEST_START=$(get_time_ms)

    cd "$PROJECT_ROOT"
    if cargo test --release 2>&1 | tee "$OUTPUT_DIR/test_output.txt" | tail -20; then
        RESULTS[tests_passed]=true
        log "${GREEN}Tests passed${NC}"
    else
        log "${RED}Tests failed${NC}"
        log "Full output: $OUTPUT_DIR/test_output.txt"
        RESULTS[total_time_ms]=$(($(get_time_ms) - TOTAL_START))
        exit 1
    fi

    TEST_END=$(get_time_ms)
    RESULTS[tests_time_ms]=$((TEST_END - TEST_START))
    log ""
else
    log "${YELLOW}[1/4] Skipping tests (--skip-tests)${NC}"
    RESULTS[tests_passed]=true
    log ""
fi

# Step 2: Full 3-Stage Bootstrap
log "${YELLOW}[2/4] Running full 3-stage bootstrap...${NC}"
BOOTSTRAP_START=$(get_time_ms)

cd "$PROJECT_ROOT"

BOOTSTRAP_ARGS="--output $OUTPUT_DIR"
[ "$VERBOSE" = true ] && BOOTSTRAP_ARGS="$BOOTSTRAP_ARGS --verbose"

if "$SCRIPT_DIR/bootstrap.sh" $BOOTSTRAP_ARGS --json > "$OUTPUT_DIR/bootstrap.json" 2>&1; then
    RESULTS[bootstrap_passed]=true

    # Check fixed point
    FIXED_POINT=$(python3 -c "import json; print(json.load(open('$OUTPUT_DIR/bootstrap.json'))['bootstrap']['fixed_point'])")
    if [ "$FIXED_POINT" = "True" ] || [ "$FIXED_POINT" = "true" ]; then
        RESULTS[bootstrap_fixed_point]=true
        log "${GREEN}Bootstrap passed (fixed point reached)${NC}"
    else
        log "${YELLOW}Bootstrap passed (fixed point not reached)${NC}"
    fi
else
    log "${RED}Bootstrap failed${NC}"
    cat "$OUTPUT_DIR/bootstrap.json"
    RESULTS[total_time_ms]=$(($(get_time_ms) - TOTAL_START))
    exit 1
fi

BOOTSTRAP_END=$(get_time_ms)
RESULTS[bootstrap_time_ms]=$((BOOTSTRAP_END - BOOTSTRAP_START))
log ""

# Step 3: All Benchmarks
log "${YELLOW}[3/4] Running all benchmarks (Tier 0-3)...${NC}"
BENCHMARK_START=$(get_time_ms)

cd "$PROJECT_ROOT"

BENCHMARK_ARGS="--tier all --runs 5 --output $OUTPUT_DIR/benchmarks.json"
[ "$VERBOSE" = true ] && BENCHMARK_ARGS="$BENCHMARK_ARGS --verbose"

"$SCRIPT_DIR/benchmark.sh" $BENCHMARK_ARGS 2>&1 | tee "$OUTPUT_DIR/benchmark_output.txt"

BENCHMARK_END=$(get_time_ms)
RESULTS[benchmark_time_ms]=$((BENCHMARK_END - BENCHMARK_START))
log ""

# Step 4: Regression Comparison
log "${YELLOW}[4/4] Checking for regressions...${NC}"

if [ -n "$BASELINE_FILE" ] && [ -f "$BASELINE_FILE" ]; then
    log "Comparing against baseline: $BASELINE_FILE"

    COMPARE_ARGS="--output $OUTPUT_DIR/comparison_report.txt"
    [ "$TIER1_STRICT" = true ] && COMPARE_ARGS="$COMPARE_ARGS --tier1-strict"

    if python3 "$SCRIPT_DIR/compare.py" "$BASELINE_FILE" "$OUTPUT_DIR/benchmarks.json" $COMPARE_ARGS; then
        RESULTS[comparison_passed]=true
        log "${GREEN}No critical regressions detected${NC}"
    else
        RESULTS[comparison_passed]=false
        log "${RED}Tier 1 regression detected!${NC}"
        cat "$OUTPUT_DIR/comparison_report.txt"
    fi
elif [ -f "$PROJECT_ROOT/.baseline.json" ]; then
    log "Comparing against default baseline: .baseline.json"

    COMPARE_ARGS="--output $OUTPUT_DIR/comparison_report.txt"
    [ "$TIER1_STRICT" = true ] && COMPARE_ARGS="$COMPARE_ARGS --tier1-strict"

    if python3 "$SCRIPT_DIR/compare.py" "$PROJECT_ROOT/.baseline.json" "$OUTPUT_DIR/benchmarks.json" $COMPARE_ARGS; then
        RESULTS[comparison_passed]=true
        log "${GREEN}No critical regressions detected${NC}"
    else
        RESULTS[comparison_passed]=false
        log "${RED}Tier 1 regression detected!${NC}"
        cat "$OUTPUT_DIR/comparison_report.txt"
    fi
else
    log "${YELLOW}No baseline file found, skipping regression check${NC}"
    log "To create a baseline, run:"
    log "  cp $OUTPUT_DIR/benchmarks.json .baseline.json"
fi
log ""

RESULTS[total_time_ms]=$(($(get_time_ms) - TOTAL_START))

# Summary
log "======================================"
log "${BLUE}Full Cycle Summary${NC}"
log "======================================"
log ""
log "| Step                      | Status                    | Time      |"
log "|---------------------------|---------------------------|-----------|"
log "| Tests                     | ${RESULTS[tests_passed]}$(printf '%*s' $((25-5-${#RESULTS[tests_passed]})) '')| ${RESULTS[tests_time_ms]}ms$(printf '%*s' $((10-${#RESULTS[tests_time_ms]}-2)) '')|"
log "| Bootstrap                 | ${RESULTS[bootstrap_passed]}$(printf '%*s' $((25-5-${#RESULTS[bootstrap_passed]})) '')| ${RESULTS[bootstrap_time_ms]}ms$(printf '%*s' $((10-${#RESULTS[bootstrap_time_ms]}-2)) '')|"
log "| Fixed Point               | ${RESULTS[bootstrap_fixed_point]}$(printf '%*s' $((25-5-${#RESULTS[bootstrap_fixed_point]})) '')| -         |"
log "| Benchmarks                | complete$(printf '%*s' $((25-5-8)) '')| ${RESULTS[benchmark_time_ms]}ms$(printf '%*s' $((10-${#RESULTS[benchmark_time_ms]}-2)) '')|"
log "| Regression Check          | ${RESULTS[comparison_passed]}$(printf '%*s' $((25-5-${#RESULTS[comparison_passed]})) '')| -         |"
log "|---------------------------|---------------------------|-----------|"
log "| Total                     |                           | ${RESULTS[total_time_ms]}ms$(printf '%*s' $((10-${#RESULTS[total_time_ms]}-2)) '')|"
log ""

# Generated files
log "Generated files:"
log "  $OUTPUT_DIR/test_output.txt        - Test output"
log "  $OUTPUT_DIR/bootstrap.json         - Bootstrap results"
log "  $OUTPUT_DIR/benchmarks.json        - Benchmark results"
log "  $OUTPUT_DIR/benchmark_output.txt   - Benchmark output"
[ -f "$OUTPUT_DIR/comparison_report.txt" ] && log "  $OUTPUT_DIR/comparison_report.txt  - Comparison report"
log ""

# JSON output
if [ "$JSON_OUTPUT" = true ]; then
    cat <<EOF
{
  "full_cycle": {
    "tests": {
      "passed": ${RESULTS[tests_passed]},
      "time_ms": ${RESULTS[tests_time_ms]}
    },
    "bootstrap": {
      "passed": ${RESULTS[bootstrap_passed]},
      "fixed_point": ${RESULTS[bootstrap_fixed_point]},
      "time_ms": ${RESULTS[bootstrap_time_ms]}
    },
    "benchmarks": {
      "time_ms": ${RESULTS[benchmark_time_ms]}
    },
    "comparison": {
      "passed": ${RESULTS[comparison_passed]}
    },
    "total_time_ms": ${RESULTS[total_time_ms]}
  }
}
EOF
fi

# Final status
if [ "${RESULTS[tests_passed]}" = true ] && \
   [ "${RESULTS[bootstrap_passed]}" = true ] && \
   [ "${RESULTS[comparison_passed]}" = true ]; then
    log "${GREEN}Full cycle verification PASSED${NC}"
    exit 0
else
    log "${RED}Full cycle verification FAILED${NC}"
    exit 1
fi
