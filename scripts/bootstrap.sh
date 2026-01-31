#!/bin/bash
# BMB 3-Stage Bootstrap Verification Script
# Part of the Bootstrap + Benchmark Cycle System
#
# This script implements the standard 3-stage compiler bootstrap process:
# - Stage 0→1: Rust compiler builds BMB bootstrap compiler (BMB₁)
# - Stage 1→2: BMB₁ compiles BMB bootstrap compiler (BMB₂)
# - Stage 2→3: BMB₂ compiles BMB bootstrap compiler (BMB₃)
#
# Success: Stage 2 and Stage 3 output must be identical (fixed point)
#
# Usage:
#   ./scripts/bootstrap.sh [options]
#
# Options:
#   --json          Output results in JSON format
#   --stage1-only   Only run Stage 0→1 (fast check)
#   --verbose       Show detailed output
#   --clean         Clean intermediate files before starting
#   --output DIR    Output directory for artifacts
#
# Reference: Ken Thompson's "Reflections on Trusting Trust" (1984)

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
# Try MinGW build first (required on Windows for native compilation), then fall back to regular
if [ -f "${PROJECT_ROOT}/target/x86_64-pc-windows-gnu/release/bmb.exe" ]; then
    RUST_BMB="${PROJECT_ROOT}/target/x86_64-pc-windows-gnu/release/bmb.exe"
elif [ -f "${PROJECT_ROOT}/target/x86_64-pc-windows-gnu/release/bmb" ]; then
    RUST_BMB="${PROJECT_ROOT}/target/x86_64-pc-windows-gnu/release/bmb"
elif [ -f "${PROJECT_ROOT}/target/release/bmb.exe" ]; then
    RUST_BMB="${PROJECT_ROOT}/target/release/bmb.exe"
else
    RUST_BMB="${PROJECT_ROOT}/target/release/bmb"
fi
BOOTSTRAP_SRC="${PROJECT_ROOT}/bootstrap/compiler.bmb"
OUTPUT_DIR="${PROJECT_ROOT}/target/bootstrap"

# Parse arguments
JSON_OUTPUT=false
STAGE1_ONLY=false
VERBOSE=false
CLEAN=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --json)
            JSON_OUTPUT=true
            shift
            ;;
        --stage1-only)
            STAGE1_ONLY=true
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --clean)
            CLEAN=true
            shift
            ;;
        --output)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Colors for output (disabled for JSON mode)
if [ "$JSON_OUTPUT" = true ]; then
    RED=''
    GREEN=''
    YELLOW=''
    NC=''
else
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[1;33m'
    NC='\033[0m'
fi

# Timing functions
get_time_ms() {
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        python3 -c 'import time; print(int(time.time() * 1000))'
    else
        # Linux/Windows (MSYS2)
        date +%s%3N 2>/dev/null || python3 -c 'import time; print(int(time.time() * 1000))'
    fi
}

# Logging
log() {
    if [ "$JSON_OUTPUT" = false ]; then
        echo -e "$1"
    fi
}

log_verbose() {
    if [ "$VERBOSE" = true ] && [ "$JSON_OUTPUT" = false ]; then
        echo -e "$1"
    fi
}

# JSON output accumulator
declare -A RESULTS
RESULTS[stage1_success]=false
RESULTS[stage1_time_ms]=0
RESULTS[stage2_success]=false
RESULTS[stage2_time_ms]=0
RESULTS[stage3_success]=false
RESULTS[stage3_time_ms]=0
RESULTS[fixed_point]=false
RESULTS[total_time_ms]=0

output_json() {
    cat <<EOF
{
  "bootstrap": {
    "stage1": {
      "success": ${RESULTS[stage1_success]},
      "time_ms": ${RESULTS[stage1_time_ms]}
    },
    "stage2": {
      "success": ${RESULTS[stage2_success]},
      "time_ms": ${RESULTS[stage2_time_ms]}
    },
    "stage3": {
      "success": ${RESULTS[stage3_success]},
      "time_ms": ${RESULTS[stage3_time_ms]}
    },
    "fixed_point": ${RESULTS[fixed_point]},
    "total_time_ms": ${RESULTS[total_time_ms]}
  }
}
EOF
}

# Cleanup function
cleanup() {
    if [ "$CLEAN" = true ]; then
        rm -rf "$OUTPUT_DIR"
    fi
}

trap cleanup EXIT

# Create output directory
mkdir -p "$OUTPUT_DIR"

TOTAL_START=$(get_time_ms)

log "======================================"
log "BMB 3-Stage Bootstrap Verification"
log "======================================"
log ""

# Check prerequisites
log "${YELLOW}[0/4] Checking prerequisites...${NC}"

if [ ! -f "$RUST_BMB" ]; then
    log "${RED}Error: Rust BMB compiler not found at $RUST_BMB${NC}"
    log "Build it first with: cargo build --release --features llvm"
    if [ "$JSON_OUTPUT" = true ]; then
        output_json
    fi
    exit 1
fi

if [ ! -f "$BOOTSTRAP_SRC" ]; then
    log "${RED}Error: Bootstrap source not found at $BOOTSTRAP_SRC${NC}"
    if [ "$JSON_OUTPUT" = true ]; then
        output_json
    fi
    exit 1
fi

# Check LLVM availability (required for native compilation)
if ! command -v llc &> /dev/null; then
    log "${YELLOW}Warning: LLVM toolchain not found (llc)${NC}"
    log "Native compilation will use fallback mode"
fi

log "${GREEN}Prerequisites OK${NC}"
log ""

# Stage 1: Rust BMB compiles bootstrap to native binary
STAGE1_BIN="${OUTPUT_DIR}/bmb-stage1"
STAGE1_START=$(get_time_ms)

log "${YELLOW}[1/4] Stage 1: Rust BMB → Stage 1 Binary${NC}"
log_verbose "Command: $RUST_BMB build $BOOTSTRAP_SRC -o $STAGE1_BIN"

if $RUST_BMB build "$BOOTSTRAP_SRC" -o "$STAGE1_BIN" 2>&1; then
    STAGE1_END=$(get_time_ms)
    RESULTS[stage1_time_ms]=$((STAGE1_END - STAGE1_START))

    if [ -f "$STAGE1_BIN" ] || [ -f "${STAGE1_BIN}.exe" ]; then
        RESULTS[stage1_success]=true
        log "${GREEN}Stage 1 OK (${RESULTS[stage1_time_ms]}ms)${NC}"
    else
        log "${RED}Stage 1 FAILED: Binary not generated${NC}"
        if [ "$JSON_OUTPUT" = true ]; then
            RESULTS[total_time_ms]=$(($(get_time_ms) - TOTAL_START))
            output_json
        fi
        exit 1
    fi
else
    log "${RED}Stage 1 FAILED: Compilation error${NC}"
    if [ "$JSON_OUTPUT" = true ]; then
        RESULTS[total_time_ms]=$(($(get_time_ms) - TOTAL_START))
        output_json
    fi
    exit 1
fi

# Quick sanity check - run stage 1 binary
STAGE1_BIN_ACTUAL="$STAGE1_BIN"
[ -f "${STAGE1_BIN}.exe" ] && STAGE1_BIN_ACTUAL="${STAGE1_BIN}.exe"

log_verbose "Testing Stage 1 binary..."
STAGE1_OUTPUT=$("$STAGE1_BIN_ACTUAL" 2>&1 | tail -1 || echo "")
if [[ "$STAGE1_OUTPUT" == "999" ]]; then
    log_verbose "${GREEN}Stage 1 tests passed (999 marker)${NC}"
fi
log ""

# Early exit for stage1-only mode
if [ "$STAGE1_ONLY" = true ]; then
    RESULTS[total_time_ms]=$(($(get_time_ms) - TOTAL_START))
    log "${GREEN}Stage 1 only mode: Complete${NC}"
    if [ "$JSON_OUTPUT" = true ]; then
        output_json
    fi
    exit 0
fi

# Stage 2: Stage 1 compiles bootstrap to LLVM IR
STAGE2_LL="${OUTPUT_DIR}/bmb-stage2.ll"
STAGE2_START=$(get_time_ms)

log "${YELLOW}[2/4] Stage 2: Stage 1 → LLVM IR${NC}"
log_verbose "Command: $STAGE1_BIN_ACTUAL $BOOTSTRAP_SRC $STAGE2_LL.tmp"

if "$STAGE1_BIN_ACTUAL" "$BOOTSTRAP_SRC" "$STAGE2_LL.tmp" 2>&1; then
    # Convert | to newlines for LLVM tools
    tr '|' '\n' < "$STAGE2_LL.tmp" > "$STAGE2_LL"
    rm -f "$STAGE2_LL.tmp"

    STAGE2_END=$(get_time_ms)
    RESULTS[stage2_time_ms]=$((STAGE2_END - STAGE2_START))

    if [ -f "$STAGE2_LL" ] && head -1 "$STAGE2_LL" | grep -q "ModuleID"; then
        RESULTS[stage2_success]=true
        STAGE2_LINES=$(wc -l < "$STAGE2_LL")
        log "${GREEN}Stage 2 OK (${RESULTS[stage2_time_ms]}ms, $STAGE2_LINES lines)${NC}"
    else
        log "${RED}Stage 2 FAILED: Invalid LLVM IR format${NC}"
        if [ "$JSON_OUTPUT" = true ]; then
            RESULTS[total_time_ms]=$(($(get_time_ms) - TOTAL_START))
            output_json
        fi
        exit 1
    fi
else
    log "${RED}Stage 2 FAILED: Compilation error${NC}"
    if [ "$JSON_OUTPUT" = true ]; then
        RESULTS[total_time_ms]=$(($(get_time_ms) - TOTAL_START))
        output_json
    fi
    exit 1
fi
log ""

# Stage 3: Compile Stage 2 to binary, then compile bootstrap again
STAGE2_OBJ="${OUTPUT_DIR}/bmb-stage2.o"
STAGE2_BIN="${OUTPUT_DIR}/bmb-stage2"
STAGE3_LL="${OUTPUT_DIR}/bmb-stage3.ll"
STAGE3_START=$(get_time_ms)

log "${YELLOW}[3/4] Stage 3: Stage 2 Binary → Stage 3 LLVM IR${NC}"

# Compile Stage 2 LLVM IR to object file
log_verbose "Compiling Stage 2 LLVM IR to native binary..."

if command -v llc &> /dev/null; then
    llc -filetype=obj -O2 "$STAGE2_LL" -o "$STAGE2_OBJ"

    if [ ! -f "$STAGE2_OBJ" ]; then
        log "${RED}Stage 3 FAILED: Could not compile LLVM IR to object file${NC}"
        if [ "$JSON_OUTPUT" = true ]; then
            RESULTS[total_time_ms]=$(($(get_time_ms) - TOTAL_START))
            output_json
        fi
        exit 1
    fi

    # Find BMB runtime library
    BMB_RUNTIME="${BMB_RUNTIME_PATH:-${PROJECT_ROOT}/bmb/runtime/libbmb_runtime.a}"
    if [ ! -f "$BMB_RUNTIME" ]; then
        log_verbose "Building BMB runtime library..."
        (cd "${PROJECT_ROOT}/bmb/runtime" && clang -c bmb_runtime.c -o bmb_runtime.o -O2 && ar rcs libbmb_runtime.a bmb_runtime.o)
        BMB_RUNTIME="${PROJECT_ROOT}/bmb/runtime/libbmb_runtime.a"
    fi

    if [ -f "$BMB_RUNTIME" ]; then
        # Link Stage 2 binary with runtime
        log_verbose "Linking Stage 2 binary with runtime..."
        clang "$STAGE2_OBJ" "$BMB_RUNTIME" -o "$STAGE2_BIN" -lm -no-pie

        if [ -f "$STAGE2_BIN" ]; then
            log_verbose "${GREEN}Stage 2 binary created${NC}"

            # Run Stage 2 binary to generate Stage 3 LLVM IR
            "$STAGE2_BIN" "$BOOTSTRAP_SRC" "$STAGE3_LL.tmp"
        else
            log "${YELLOW}Falling back to interpreter for Stage 3${NC}"
            $RUST_BMB run "$BOOTSTRAP_SRC" "$BOOTSTRAP_SRC" "$STAGE3_LL.tmp"
        fi
    else
        log "${YELLOW}BMB runtime not found, falling back to interpreter${NC}"
        $RUST_BMB run "$BOOTSTRAP_SRC" "$BOOTSTRAP_SRC" "$STAGE3_LL.tmp"
    fi
else
    log "${YELLOW}LLVM not available, falling back to interpreter${NC}"
    $RUST_BMB run "$BOOTSTRAP_SRC" "$BOOTSTRAP_SRC" "$STAGE3_LL.tmp"
fi

# Convert | to newlines
tr '|' '\n' < "$STAGE3_LL.tmp" > "$STAGE3_LL"
rm -f "$STAGE3_LL.tmp"

STAGE3_END=$(get_time_ms)
RESULTS[stage3_time_ms]=$((STAGE3_END - STAGE3_START))

if [ -f "$STAGE3_LL" ]; then
    RESULTS[stage3_success]=true
    STAGE3_LINES=$(wc -l < "$STAGE3_LL")
    log "${GREEN}Stage 3 OK (${RESULTS[stage3_time_ms]}ms, $STAGE3_LINES lines)${NC}"
else
    log "${RED}Stage 3 FAILED: LLVM IR not generated${NC}"
    if [ "$JSON_OUTPUT" = true ]; then
        RESULTS[total_time_ms]=$(($(get_time_ms) - TOTAL_START))
        output_json
    fi
    exit 1
fi
log ""

# Verification: Compare Stage 2 and Stage 3 LLVM IR
log "${YELLOW}[4/4] Verification: Comparing Stage 2 and Stage 3${NC}"

if diff -q "$STAGE2_LL" "$STAGE3_LL" > /dev/null 2>&1; then
    RESULTS[fixed_point]=true
    log "${GREEN}✓ 3-Stage Bootstrap PASSED: Stage 2 == Stage 3${NC}"
    log "The bootstrap compiler generates identical output when compiled by:"
    log "  - Rust compiler (Stage 1 → Stage 2)"
    log "  - Itself (Stage 2 → Stage 3)"
else
    log "${YELLOW}Stage 2 and Stage 3 differ${NC}"
    if [ "$VERBOSE" = true ]; then
        log "Differences:"
        diff "$STAGE2_LL" "$STAGE3_LL" | head -20
    fi
fi
log ""

RESULTS[total_time_ms]=$(($(get_time_ms) - TOTAL_START))

# Summary
log "======================================"
log "Bootstrap Status Summary"
log "======================================"
log "Stage 1 (Rust BMB → BMB₁):     ${RESULTS[stage1_success]} (${RESULTS[stage1_time_ms]}ms)"
log "Stage 2 (BMB₁ → LLVM IR):      ${RESULTS[stage2_success]} (${RESULTS[stage2_time_ms]}ms)"
log "Stage 3 (BMB₂ → LLVM IR):      ${RESULTS[stage3_success]} (${RESULTS[stage3_time_ms]}ms)"
log "Fixed Point (S2 == S3):        ${RESULTS[fixed_point]}"
log "Total Time:                    ${RESULTS[total_time_ms]}ms"
log ""
log "Generated files:"
log "  $STAGE1_BIN - Stage 1 native binary"
log "  $STAGE2_LL - LLVM IR generated by Stage 1"
[ -f "$STAGE2_BIN" ] && log "  $STAGE2_BIN - Stage 2 native binary"
log "  $STAGE3_LL - LLVM IR generated by Stage 2"
log ""

# JSON output
if [ "$JSON_OUTPUT" = true ]; then
    output_json
fi

# Exit with error if fixed point not reached (during CI)
if [ "${RESULTS[fixed_point]}" = false ]; then
    exit 1
fi

# Cleanup intermediate files
rm -f "$STAGE2_OBJ" "${STAGE2_LL}.tmp" "${STAGE3_LL}.tmp"

log "${GREEN}Bootstrap verification complete${NC}"
