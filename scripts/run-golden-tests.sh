#!/bin/bash
# BMB Golden Binary Test Runner
# Compiles and runs golden tests using the bootstrap Stage 1 compiler
#
# Usage:
#   ./scripts/run-golden-tests.sh [options]
#
# Options:
#   --stage1 <path>   Path to Stage 1 binary (default: auto-detect)
#   --verbose         Show detailed output
#   --json            JSON output for CI
#   --build-stage1    Build Stage 1 from Rust compiler first

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

TESTS_DIR="${PROJECT_ROOT}/tests/bootstrap"
MANIFEST="${TESTS_DIR}/golden_tests.txt"
RUNTIME_DIR="${PROJECT_ROOT}/bmb/runtime"
OUTPUT_DIR="${PROJECT_ROOT}/target/golden-tests"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Options
STAGE1=""
VERBOSE=false
JSON_OUTPUT=false
BUILD_STAGE1=false

# Timeouts (seconds)
COMPILE_TIMEOUT=60
OPT_TIMEOUT=60
LINK_TIMEOUT=60
RUN_TIMEOUT=30

# Detect platform
case "$(uname -s)" in
    MINGW*|MSYS*|CYGWIN*)
        EXE_EXT=".exe"
        LINK_LIBS="-lm -lws2_32"
        ;;
    *)
        EXE_EXT=""
        LINK_LIBS="-lm -lpthread"
        ;;
esac

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --stage1)
            STAGE1="$2"
            shift 2
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --json)
            JSON_OUTPUT=true
            shift
            ;;
        --build-stage1)
            BUILD_STAGE1=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

log() {
    if [ "$JSON_OUTPUT" = false ]; then
        echo -e "$1"
    fi
}

log_verbose() {
    if [ "$VERBOSE" = true ] && [ "$JSON_OUTPUT" = false ]; then
        echo -e "  $1"
    fi
}

# Find or build Stage 1 binary
find_stage1() {
    if [ -n "$STAGE1" ]; then
        if [ ! -f "$STAGE1" ]; then
            log "${RED}Error: Stage 1 binary not found at $STAGE1${NC}"
            exit 1
        fi
        return
    fi

    # Auto-detect Stage 1 binary locations (most recent first)
    local candidates=(
        "${PROJECT_ROOT}/bootstrap_stage1${EXE_EXT}"
        "${PROJECT_ROOT}/target/bootstrap/bmb-stage1${EXE_EXT}"
        "${PROJECT_ROOT}/target/golden-bootstrap/bmb-stage1${EXE_EXT}"
        "${PROJECT_ROOT}/target/stage1_verify${EXE_EXT}"
    )

    for candidate in "${candidates[@]}"; do
        if [ -f "$candidate" ]; then
            STAGE1="$candidate"
            return
        fi
    done

    if [ "$BUILD_STAGE1" = true ]; then
        log "${YELLOW}Building Stage 1 from Rust compiler...${NC}"
        BMB_RUNTIME_PATH="${RUNTIME_DIR}" "${PROJECT_ROOT}/target/x86_64-pc-windows-gnu/release/bmb${EXE_EXT}" \
            build "${PROJECT_ROOT}/bootstrap/compiler.bmb" -o "${PROJECT_ROOT}/target/golden-tests/bmb-stage1${EXE_EXT}" \
            --runtime "${RUNTIME_DIR}" 2>&1
        STAGE1="${PROJECT_ROOT}/target/golden-tests/bmb-stage1${EXE_EXT}"
    else
        log "${RED}Error: No Stage 1 binary found. Use --stage1 <path> or --build-stage1${NC}"
        log "Looked in:"
        for candidate in "${candidates[@]}"; do
            log "  $candidate"
        done
        exit 1
    fi
}

# Main test runner
main() {
    mkdir -p "$OUTPUT_DIR"

    find_stage1

    log "========================================"
    log "BMB Golden Binary Test Runner"
    log "========================================"
    log ""
    log "Stage 1: $STAGE1"
    log "Tests:   $MANIFEST"
    log ""

    if [ ! -f "$MANIFEST" ]; then
        log "${RED}Error: Test manifest not found at $MANIFEST${NC}"
        exit 1
    fi

    local PASSED=0
    local FAILED=0
    local TOTAL=0
    local FAILURES=""
    local start_time=$(date +%s%3N 2>/dev/null || python3 -c 'import time; print(int(time.time() * 1000))')

    while IFS= read -r line || [ -n "$line" ]; do
        # Skip comments and empty lines
        [[ "$line" =~ ^#.*$ ]] && continue
        [[ -z "$line" ]] && continue
        [[ ! "$line" =~ \| ]] && continue

        FILENAME=$(echo "$line" | cut -d'|' -f1)
        EXPECTED=$(echo "$line" | cut -d'|' -f2)
        TEST_FILE="${TESTS_DIR}/${FILENAME}"
        TEST_NAME=$(basename "$FILENAME" .bmb)
        ((TOTAL++)) || true

        if [ ! -f "$TEST_FILE" ]; then
            log "  ${RED}FAIL${NC} ${TEST_NAME}: File not found"
            ((FAILED++)) || true
            FAILURES="${FAILURES}  ${TEST_NAME}: File not found\n"
            continue
        fi

        echo -n "  ${TEST_NAME}... "

        # Paths for intermediate files
        local IR_FILE="${OUTPUT_DIR}/${TEST_NAME}.ll"
        local OPT_FILE="${OUTPUT_DIR}/${TEST_NAME}_opt.ll"
        local OBJ_FILE="${OUTPUT_DIR}/${TEST_NAME}.o"
        local EXE_FILE="${OUTPUT_DIR}/${TEST_NAME}${EXE_EXT}"
        local OUT_FILE="${OUTPUT_DIR}/${TEST_NAME}_output.txt"

        # Step 1: Compile to LLVM IR via Stage 1
        log_verbose "Compiling: $STAGE1 $TEST_FILE $IR_FILE"
        if ! timeout "$COMPILE_TIMEOUT" "$STAGE1" "$TEST_FILE" "$IR_FILE" >/dev/null 2>&1; then
            echo -e "${RED}FAIL (compile)${NC}"
            ((FAILED++)) || true
            FAILURES="${FAILURES}  ${TEST_NAME}: Stage 1 compilation failed\n"
            continue
        fi

        # Step 2: Optimize with opt -O2 (--slp-max-vf=1 prevents SLP vectorization
        # of integer division which scalarizes poorly on x86-64, causing 2.4x regression)
        log_verbose "Optimizing: opt -O2 --slp-max-vf=1 -S $IR_FILE -o $OPT_FILE"
        if ! timeout "$OPT_TIMEOUT" opt -O2 --slp-max-vf=1 -S "$IR_FILE" -o "$OPT_FILE" 2>/dev/null; then
            echo -e "${RED}FAIL (opt)${NC}"
            ((FAILED++)) || true
            FAILURES="${FAILURES}  ${TEST_NAME}: opt -O2 failed\n"
            continue
        fi

        # Step 3: Compile to object file with llc
        log_verbose "Codegen: llc -O3 -filetype=obj $OPT_FILE -o $OBJ_FILE"
        if ! timeout "$OPT_TIMEOUT" llc -O3 -filetype=obj "$OPT_FILE" -o "$OBJ_FILE" 2>/dev/null; then
            echo -e "${RED}FAIL (llc)${NC}"
            ((FAILED++)) || true
            FAILURES="${FAILURES}  ${TEST_NAME}: llc failed\n"
            continue
        fi

        # Step 4: Link with gcc
        log_verbose "Linking: gcc -O2 -o $EXE_FILE $OBJ_FILE $RUNTIME_DIR/libbmb_runtime.a $LINK_LIBS"
        if ! timeout "$LINK_TIMEOUT" gcc -O2 -o "$EXE_FILE" "$OBJ_FILE" "${RUNTIME_DIR}/libbmb_runtime.a" $LINK_LIBS 2>/dev/null; then
            echo -e "${RED}FAIL (link)${NC}"
            ((FAILED++)) || true
            FAILURES="${FAILURES}  ${TEST_NAME}: linking failed\n"
            continue
        fi

        # Step 5: Run and capture stdout only (stderr goes to /dev/null for dbg/eprint tests)
        timeout "$RUN_TIMEOUT" "$EXE_FILE" > "$OUT_FILE" 2>/dev/null || true

        # Check output (first line should be expected value)
        ACTUAL=$(head -1 "$OUT_FILE" 2>/dev/null | tr -d '\r')

        if [ "$ACTUAL" == "$EXPECTED" ]; then
            echo -e "${GREEN}PASS${NC} ($ACTUAL)"
            ((PASSED++)) || true
        else
            echo -e "${RED}FAIL${NC} (expected=$EXPECTED, got=$ACTUAL)"
            ((FAILED++)) || true
            FAILURES="${FAILURES}  ${TEST_NAME}: expected=$EXPECTED, got=$ACTUAL\n"
        fi

    done < "$MANIFEST"

    local end_time=$(date +%s%3N 2>/dev/null || python3 -c 'import time; print(int(time.time() * 1000))')
    local elapsed=$((end_time - start_time))

    log ""
    log "========================================"
    log "Results: ${PASSED}/${TOTAL} passed, ${FAILED} failed (${elapsed}ms)"
    log "========================================"

    if [ "$FAILED" -gt 0 ]; then
        log ""
        log "${RED}Failures:${NC}"
        echo -e "$FAILURES"
    fi

    # JSON output for CI
    if [ "$JSON_OUTPUT" = true ]; then
        echo "{\"type\":\"golden_tests\",\"passed\":${PASSED},\"failed\":${FAILED},\"total\":${TOTAL},\"elapsed_ms\":${elapsed}}"
    fi

    if [ "$FAILED" -gt 0 ]; then
        exit 1
    fi
}

main
