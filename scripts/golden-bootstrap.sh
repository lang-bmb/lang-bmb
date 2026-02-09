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

# Stage 1: Golden BMB → Native Binary (using build command)
stage1() {
    log "${YELLOW}[1/3] Stage 1: Golden BMB → Stage 1 Binary${NC}"

    mkdir -p "$OUTPUT_DIR"

    # Build runtime library if needed
    BMB_RUNTIME="$RUNTIME_DIR/libbmb_runtime.a"
    if [ ! -f "$BMB_RUNTIME" ]; then
        log_verbose "Building BMB runtime library..."
        (cd "$RUNTIME_DIR" && \
            clang -c -O3 bmb_runtime.c -o bmb_runtime.o && \
            clang -c -O3 bmb_event_loop.c -o bmb_event_loop.o && \
            ar rcs libbmb_runtime.a bmb_runtime.o bmb_event_loop.o)
    fi

    local start=$(date +%s%3N 2>/dev/null || python3 -c 'import time; print(int(time.time() * 1000))')

    # v0.90: Use build command directly (no manual opt/clang steps)
    log_verbose "Command: $GOLDEN_BMB build $BOOTSTRAP_SRC -o $OUTPUT_DIR/bmb-stage1 --runtime $RUNTIME_DIR"
    "$GOLDEN_BMB" build "$BOOTSTRAP_SRC" -o "$OUTPUT_DIR/bmb-stage1${EXE_EXT}" --runtime "$RUNTIME_DIR" 2>&1

    local end=$(date +%s%3N 2>/dev/null || python3 -c 'import time; print(int(time.time() * 1000))')
    local elapsed=$((end - start))

    if [ -f "$OUTPUT_DIR/bmb-stage1${EXE_EXT}" ]; then
        log "${GREEN}Stage 1 OK (${elapsed}ms)${NC}"
    else
        log "${RED}Stage 1 FAILED: Binary not created${NC}"
        exit 1
    fi
}

# Stage 2 is no longer needed as a separate step (build command handles everything)
stage2() {
    log "${GREEN}Stage 2 merged into Stage 1 (build command handles opt+link)${NC}"
}

# Stage 3: Verify (optional)
stage3_verify() {
    log "${YELLOW}[3/3] Stage 3: Verification${NC}"

    local start=$(date +%s%3N 2>/dev/null || python3 -c 'import time; print(int(time.time() * 1000))')

    # v0.90: Use build command for self-hosting chain
    # Stage 1 binary builds itself (Stage 2)
    "$OUTPUT_DIR/bmb-stage1${EXE_EXT}" "$BOOTSTRAP_SRC" "$OUTPUT_DIR/stage2.ll"

    # Build Stage 2 binary using Stage 1's build command
    "$OUTPUT_DIR/bmb-stage1${EXE_EXT}" build "$BOOTSTRAP_SRC" -o "$OUTPUT_DIR/bmb-stage2${EXE_EXT}" --runtime "$RUNTIME_DIR" 2>&1

    # Stage 2 binary generates Stage 3 IR
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
