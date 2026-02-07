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

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Detect platform and set golden binary path
detect_platform() {
    case "$(uname -s)" in
        MINGW*|MSYS*|CYGWIN*)
            PLATFORM="windows-x64"
            EXE_EXT=".exe"
            ;;
        Linux)
            if [ "$(uname -m)" = "aarch64" ]; then
                PLATFORM="linux-aarch64"
            else
                PLATFORM="linux-x86_64"
            fi
            EXE_EXT=""
            ;;
        Darwin)
            PLATFORM="darwin-universal"
            EXE_EXT=""
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

    if ! command -v clang &> /dev/null; then
        log "${RED}Error: clang not found. Please install LLVM/Clang.${NC}"
        exit 1
    fi

    log "${GREEN}Prerequisites OK${NC}"
}

# Stage 1: Golden BMB → LLVM IR
stage1() {
    log "${YELLOW}[1/3] Stage 1: Golden BMB → LLVM IR${NC}"

    mkdir -p "$OUTPUT_DIR"

    local start=$(date +%s%3N 2>/dev/null || python3 -c 'import time; print(int(time.time() * 1000))')

    log_verbose "Command: $GOLDEN_BMB $BOOTSTRAP_SRC $OUTPUT_DIR/stage1.ll"
    "$GOLDEN_BMB" "$BOOTSTRAP_SRC" "$OUTPUT_DIR/stage1.ll"

    local end=$(date +%s%3N 2>/dev/null || python3 -c 'import time; print(int(time.time() * 1000))')
    local elapsed=$((end - start))

    local lines=$(wc -l < "$OUTPUT_DIR/stage1.ll")
    log "${GREEN}Stage 1 OK (${elapsed}ms, ${lines} lines)${NC}"
}

# Stage 2: Compile to native binary
stage2() {
    log "${YELLOW}[2/3] Stage 2: LLVM IR → Native Binary${NC}"

    local start=$(date +%s%3N 2>/dev/null || python3 -c 'import time; print(int(time.time() * 1000))')

    # Optimize
    log_verbose "Optimizing with opt -O3..."
    opt -O3 "$OUTPUT_DIR/stage1.ll" -S -o "$OUTPUT_DIR/stage1_opt.ll"

    # Compile and link with platform-specific libraries
    log_verbose "Compiling and linking with clang..."
    # v0.88: Add platform-specific libraries (ws2_32 for Windows sockets)
    if [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]] || [[ "$OSTYPE" == "win32" ]]; then
        clang -O3 "$OUTPUT_DIR/stage1_opt.ll" \
            "$RUNTIME_DIR/bmb_runtime.c" \
            -o "$OUTPUT_DIR/bmb-stage1${EXE_EXT}" \
            -lm -lws2_32
    else
        clang -O3 "$OUTPUT_DIR/stage1_opt.ll" \
            "$RUNTIME_DIR/bmb_runtime.c" \
            -o "$OUTPUT_DIR/bmb-stage1${EXE_EXT}" \
            -lm -lpthread
    fi

    local end=$(date +%s%3N 2>/dev/null || python3 -c 'import time; print(int(time.time() * 1000))')
    local elapsed=$((end - start))

    log "${GREEN}Stage 2 OK (${elapsed}ms)${NC}"
}

# Stage 3: Verify (optional)
stage3_verify() {
    log "${YELLOW}[3/3] Stage 3: Verification${NC}"

    local start=$(date +%s%3N 2>/dev/null || python3 -c 'import time; print(int(time.time() * 1000))')

    # Generate Stage 2 IR using Stage 1 binary
    "$OUTPUT_DIR/bmb-stage1${EXE_EXT}" "$BOOTSTRAP_SRC" "$OUTPUT_DIR/stage2.ll"

    # Generate Stage 3 IR using Stage 2 binary (compiled from Stage 1)
    # First compile Stage 2 binary
    opt -O3 "$OUTPUT_DIR/stage2.ll" -S -o "$OUTPUT_DIR/stage2_opt.ll"
    # v0.88: Add platform-specific libraries (ws2_32 for Windows sockets)
    if [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]] || [[ "$OSTYPE" == "win32" ]]; then
        clang -O3 "$OUTPUT_DIR/stage2_opt.ll" \
            "$RUNTIME_DIR/bmb_runtime.c" \
            -o "$OUTPUT_DIR/bmb-stage2${EXE_EXT}" \
            -lm -lws2_32
    else
        clang -O3 "$OUTPUT_DIR/stage2_opt.ll" \
            "$RUNTIME_DIR/bmb_runtime.c" \
            -o "$OUTPUT_DIR/bmb-stage2${EXE_EXT}" \
            -lm -lpthread
    fi

    # Generate Stage 3 IR
    "$OUTPUT_DIR/bmb-stage2${EXE_EXT}" "$BOOTSTRAP_SRC" "$OUTPUT_DIR/stage3.ll"

    local end=$(date +%s%3N 2>/dev/null || python3 -c 'import time; print(int(time.time() * 1000))')
    local elapsed=$((end - start))

    # Compare
    if diff -q "$OUTPUT_DIR/stage2.ll" "$OUTPUT_DIR/stage3.ll" > /dev/null 2>&1; then
        log "${GREEN}✓ Fixed Point VERIFIED: Stage 2 == Stage 3 (${elapsed}ms)${NC}"
    else
        log "${RED}✗ Fixed Point FAILED: Stage 2 != Stage 3${NC}"
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
    stage1
    stage2

    if [ "$VERIFY" = true ]; then
        stage3_verify
    fi

    log ""
    log "======================================"
    log "${GREEN}Bootstrap Complete${NC}"
    log "======================================"
    log ""
    log "Output: $OUTPUT_DIR/bmb-stage1${EXE_EXT}"
    log ""
    log "To use the bootstrapped compiler:"
    log "  export PATH=\"$OUTPUT_DIR:\$PATH\""
    log "  bmb-stage1 --help"
}

main
