#!/bin/bash
# BMB Golden Binary Bootstrap Script
# Bootstraps the BMB compiler without requiring Rust
#
# Usage:
#   ./scripts/golden-bootstrap.sh [options]
#
# Options:
#   --verify    Run 3-stage verification after bootstrap
#   --verbose   Show detailed output
#   --output    Output directory (default: target/golden-bootstrap)
#   --test      Run golden tests after bootstrap

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Detect platform and set golden binary path
detect_platform() {
    case "$(uname -s)" in
        MINGW*|MSYS*|CYGWIN*)
            PLATFORM="windows-x64"
            EXE_EXT=".exe"
            LINK_LIBS="-lm -lws2_32"
            ;;
        Linux)
            if [ "$(uname -m)" = "aarch64" ]; then
                PLATFORM="linux-aarch64"
            else
                PLATFORM="linux-x86_64"
            fi
            EXE_EXT=""
            LINK_LIBS="-lm -lpthread"
            ;;
        Darwin)
            PLATFORM="darwin-universal"
            EXE_EXT=""
            LINK_LIBS="-lm -lpthread"
            ;;
        *)
            echo "Unsupported platform: $(uname -s)"
            exit 1
            ;;
    esac
}

detect_platform

GOLDEN_BMB="${PROJECT_ROOT}/golden/${PLATFORM}/bmb${EXE_EXT}"
BOOTSTRAP_SRC="${PROJECT_ROOT}/bootstrap/compiler.bmb"
RUNTIME_DIR="${PROJECT_ROOT}/bmb/runtime"
OUTPUT_DIR="${PROJECT_ROOT}/target/golden-bootstrap"

# Parse arguments
VERIFY=false
VERBOSE=false
RUN_TESTS=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --verify)
            VERIFY=true
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --output)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        --test)
            RUN_TESTS=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log() {
    echo -e "$1"
}

log_verbose() {
    if [ "$VERBOSE" = true ]; then
        echo -e "$1"
    fi
}

get_time_ms() {
    date +%s%3N 2>/dev/null || python3 -c 'import time; print(int(time.time() * 1000))'
}

# Check prerequisites
check_prerequisites() {
    log "${YELLOW}Checking prerequisites...${NC}"

    if [ ! -f "$GOLDEN_BMB" ]; then
        log "${RED}Error: Golden binary not found at $GOLDEN_BMB${NC}"
        log "Please ensure golden binaries are available for your platform."
        exit 1
    fi

    if ! command -v opt &> /dev/null; then
        log "${RED}Error: LLVM opt not found. Please install LLVM.${NC}"
        exit 1
    fi

    # Check for linker (gcc or clang)
    if command -v gcc &> /dev/null; then
        LINKER="gcc"
    elif command -v clang &> /dev/null; then
        LINKER="clang"
    else
        log "${RED}Error: No C compiler found. Please install gcc or clang.${NC}"
        exit 1
    fi

    log "${GREEN}Prerequisites OK (linker: $LINKER)${NC}"
}

# Build runtime library if needed
build_runtime() {
    BMB_RUNTIME="$RUNTIME_DIR/libbmb_runtime.a"
    if [ ! -f "$BMB_RUNTIME" ]; then
        log_verbose "Building BMB runtime library..."
        local CC="${LINKER}"
        (cd "$RUNTIME_DIR" && \
            $CC -c -O2 bmb_runtime.c -o bmb_runtime.o && \
            $CC -c -O2 bmb_event_loop.c -o bmb_event_loop.o && \
            ar rcs libbmb_runtime.a bmb_runtime.o bmb_event_loop.o)
        log_verbose "Runtime library built"
    fi
}

# Compile LLVM IR to native binary
# Usage: compile_ir_to_binary <input.ll> <output_binary>
compile_ir_to_binary() {
    local IR_FILE="$1"
    local OUTPUT_BIN="$2"
    local BASE="${OUTPUT_BIN%.exe}"
    BASE="${BASE%}"

    log_verbose "  opt -O3,scalarizer ${IR_FILE}..."
    opt -passes='default<O3>,scalarizer' "$IR_FILE" -o "${BASE}.bc"

    log_verbose "  llc -O3 ${BASE}.bc..."
    llc -filetype=obj -O3 "${BASE}.bc" -o "${BASE}.o"

    log_verbose "  $LINKER ${BASE}.o + runtime..."
    $LINKER "${BASE}.o" "$RUNTIME_DIR/libbmb_runtime.a" -o "$OUTPUT_BIN" $LINK_LIBS -no-pie

    # Cleanup intermediate files
    rm -f "${BASE}.bc" "${BASE}.o"
}

# Stage 1: Golden BMB → Stage 1 Binary
# The golden binary is a bootstrap compiler: it emits LLVM IR, not native code.
# We need to: golden → IR → opt → llc → link → Stage 1 binary
stage1() {
    log "${YELLOW}[1/3] Stage 1: Golden BMB → Stage 1 Binary${NC}"

    mkdir -p "$OUTPUT_DIR"

    local start=$(get_time_ms)

    # Golden binary generates LLVM IR (pipe-delimited)
    log_verbose "  Golden binary compiling compiler.bmb to LLVM IR..."
    BMB_ARENA_MAX_SIZE=8G "$GOLDEN_BMB" "$BOOTSTRAP_SRC" "$OUTPUT_DIR/stage1.ll.tmp"

    # Convert pipe-delimited to newlines
    tr '|' '\n' < "$OUTPUT_DIR/stage1.ll.tmp" > "$OUTPUT_DIR/stage1.ll"
    rm -f "$OUTPUT_DIR/stage1.ll.tmp"

    local ir_lines=$(wc -l < "$OUTPUT_DIR/stage1.ll")
    log_verbose "  Stage 1 IR: $ir_lines lines"

    # Compile IR to native binary
    compile_ir_to_binary "$OUTPUT_DIR/stage1.ll" "$OUTPUT_DIR/bmb-stage1${EXE_EXT}"

    local end=$(get_time_ms)
    local elapsed=$((end - start))

    if [ -f "$OUTPUT_DIR/bmb-stage1${EXE_EXT}" ]; then
        local size=$(stat -c '%s' "$OUTPUT_DIR/bmb-stage1${EXE_EXT}" 2>/dev/null || stat -f '%z' "$OUTPUT_DIR/bmb-stage1${EXE_EXT}" 2>/dev/null || echo "?")
        log "${GREEN}Stage 1 OK (${elapsed}ms, $ir_lines lines IR, ${size} bytes)${NC}"
    else
        log "${RED}Stage 1 FAILED: Binary not created${NC}"
        exit 1
    fi
}

# Stage 2+3 Verification: Stage 1 self-compiles, then Stage 2 self-compiles
stage_verify() {
    log "${YELLOW}[2/3] Stage 2: Stage 1 → Stage 2 IR${NC}"

    local start=$(get_time_ms)

    # Stage 1 generates Stage 2 IR
    BMB_ARENA_MAX_SIZE=8G "$OUTPUT_DIR/bmb-stage1${EXE_EXT}" "$BOOTSTRAP_SRC" "$OUTPUT_DIR/stage2.ll.tmp"
    tr '|' '\n' < "$OUTPUT_DIR/stage2.ll.tmp" > "$OUTPUT_DIR/stage2.ll"
    rm -f "$OUTPUT_DIR/stage2.ll.tmp"

    local stage2_lines=$(wc -l < "$OUTPUT_DIR/stage2.ll")

    local end=$(get_time_ms)
    log "${GREEN}Stage 2 OK ($((end - start))ms, $stage2_lines lines)${NC}"

    log "${YELLOW}[3/3] Stage 3: Stage 2 → Stage 3 IR (Fixed Point Check)${NC}"

    start=$(get_time_ms)

    # Compile Stage 2 IR to binary
    compile_ir_to_binary "$OUTPUT_DIR/stage2.ll" "$OUTPUT_DIR/bmb-stage2${EXE_EXT}"

    # Stage 2 generates Stage 3 IR
    BMB_ARENA_MAX_SIZE=8G "$OUTPUT_DIR/bmb-stage2${EXE_EXT}" "$BOOTSTRAP_SRC" "$OUTPUT_DIR/stage3.ll.tmp"
    tr '|' '\n' < "$OUTPUT_DIR/stage3.ll.tmp" > "$OUTPUT_DIR/stage3.ll"
    rm -f "$OUTPUT_DIR/stage3.ll.tmp"

    local stage3_lines=$(wc -l < "$OUTPUT_DIR/stage3.ll")

    end=$(get_time_ms)
    log "Stage 3 generated ($((end - start))ms, $stage3_lines lines)"

    # Compare Stage 2 and Stage 3
    if diff -q "$OUTPUT_DIR/stage2.ll" "$OUTPUT_DIR/stage3.ll" > /dev/null 2>&1; then
        log "${GREEN}✓ Fixed Point VERIFIED: Stage 2 == Stage 3${NC}"
    else
        log "${RED}✗ Fixed Point FAILED: Stage 2 != Stage 3${NC}"
        if [ "$VERBOSE" = true ]; then
            diff "$OUTPUT_DIR/stage2.ll" "$OUTPUT_DIR/stage3.ll" | head -20
        fi
        exit 1
    fi
}

# Run golden tests using Stage 1 binary
run_golden_tests() {
    log ""
    log "${YELLOW}Running Golden Tests...${NC}"

    if [ -f "$SCRIPT_DIR/run-golden-tests.sh" ]; then
        bash "$SCRIPT_DIR/run-golden-tests.sh" --stage1 "$OUTPUT_DIR/bmb-stage1${EXE_EXT}"
    else
        log "${RED}Golden test runner not found at $SCRIPT_DIR/run-golden-tests.sh${NC}"
        exit 1
    fi
}

# Main
main() {
    log "======================================"
    log "BMB Golden Binary Bootstrap"
    log "======================================"
    log ""
    log "Golden Binary: $GOLDEN_BMB"
    log "Version: $(cat ${PROJECT_ROOT}/golden/VERSION 2>/dev/null | head -1 || echo 'unknown')"
    log ""

    check_prerequisites
    build_runtime
    stage1

    if [ "$VERIFY" = true ]; then
        stage_verify
    fi

    if [ "$RUN_TESTS" = true ]; then
        run_golden_tests
    fi

    log ""
    log "======================================"
    log "${GREEN}Bootstrap Complete${NC}"
    log "======================================"
    log ""
    log "Stage 1 binary: $OUTPUT_DIR/bmb-stage1${EXE_EXT}"
    if [ "$VERIFY" = true ]; then
        log "Stage 2 binary: $OUTPUT_DIR/bmb-stage2${EXE_EXT}"
    fi
    log ""
    log "Usage:"
    log "  # Compile a BMB program to LLVM IR"
    log "  $OUTPUT_DIR/bmb-stage1${EXE_EXT} <input.bmb> <output.ll>"
    log ""
    log "  # Type check only"
    log "  $OUTPUT_DIR/bmb-stage1${EXE_EXT} check <input.bmb>"
}

main
