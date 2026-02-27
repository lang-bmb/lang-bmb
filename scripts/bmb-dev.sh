#!/bin/bash
# BMB Bootstrap-First Development Script
# Modify compiler.bmb → build → test → verify — all without Rust
#
# Usage:
#   ./scripts/bmb-dev.sh build          Build Stage 1 from golden binary
#   ./scripts/bmb-dev.sh test           Run golden tests
#   ./scripts/bmb-dev.sh verify         Run 3-stage fixed point verification
#   ./scripts/bmb-dev.sh full           Build + test + verify
#   ./scripts/bmb-dev.sh compile <file> Compile a BMB file to native binary
#
# Workflow:
#   1. Edit bootstrap/compiler.bmb
#   2. Run: ./scripts/bmb-dev.sh full
#   3. If all tests pass and fixed point verified, commit

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

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

GOLDEN_BMB="${PROJECT_ROOT}/golden/windows-x64/bmb${EXE_EXT}"
BOOTSTRAP_SRC="${PROJECT_ROOT}/bootstrap/compiler.bmb"
RUNTIME_DIR="${PROJECT_ROOT}/bmb/runtime"
DEV_DIR="${PROJECT_ROOT}/target/bmb-dev"
STAGE1="${DEV_DIR}/bmb-stage1${EXE_EXT}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

get_time_ms() {
    date +%s%3N 2>/dev/null || python3 -c 'import time; print(int(time.time() * 1000))'
}

# Detect linker
if command -v gcc &> /dev/null; then
    LINKER="gcc"
elif command -v clang &> /dev/null; then
    LINKER="clang"
else
    echo -e "${RED}Error: No C compiler found (gcc or clang)${NC}"
    exit 1
fi

# Ensure runtime library exists
ensure_runtime() {
    if [ ! -f "$RUNTIME_DIR/libbmb_runtime.a" ]; then
        echo -e "${YELLOW}Building runtime library...${NC}"
        (cd "$RUNTIME_DIR" && \
            $LINKER -c -O2 bmb_runtime.c -o bmb_runtime.o && \
            $LINKER -c -O2 bmb_event_loop.c -o bmb_event_loop.o && \
            ar rcs libbmb_runtime.a bmb_runtime.o bmb_event_loop.o)
    fi
}

# Compile LLVM IR to native binary
compile_ir() {
    local IR="$1"
    local OUT="$2"
    local BASE="${OUT%.*}"
    opt -passes='default<O3>,scalarizer' "$IR" -o "${BASE}.bc" 2>&1
    llc -filetype=obj -O3 "${BASE}.bc" -o "${BASE}.o" 2>&1
    $LINKER "${BASE}.o" "$RUNTIME_DIR/libbmb_runtime.a" -o "$OUT" $LINK_LIBS -no-pie 2>&1
    rm -f "${BASE}.bc" "${BASE}.o"
}

# Build Stage 1 from golden binary
cmd_build() {
    mkdir -p "$DEV_DIR"
    ensure_runtime

    echo -e "${CYAN}=== Building Stage 1 from Golden Binary ===${NC}"
    local start=$(get_time_ms)

    # Golden binary → LLVM IR
    BMB_ARENA_MAX_SIZE=${BMB_ARENA_MAX_SIZE:-16G} "$GOLDEN_BMB" "$BOOTSTRAP_SRC" "$DEV_DIR/stage1.ll.tmp" 2>&1
    tr '|' '\n' < "$DEV_DIR/stage1.ll.tmp" > "$DEV_DIR/stage1.ll"
    rm -f "$DEV_DIR/stage1.ll.tmp"

    local ir_lines=$(wc -l < "$DEV_DIR/stage1.ll")

    # IR → native binary
    compile_ir "$DEV_DIR/stage1.ll" "$STAGE1"

    local end=$(get_time_ms)
    local size=$(stat -c '%s' "$STAGE1" 2>/dev/null || stat -f '%z' "$STAGE1" 2>/dev/null || echo "?")
    echo -e "${GREEN}Stage 1 built: $ir_lines lines IR, ${size} bytes ($((end - start))ms)${NC}"
}

# Run golden tests
cmd_test() {
    if [ ! -f "$STAGE1" ]; then
        echo -e "${YELLOW}Stage 1 not found, building first...${NC}"
        cmd_build
    fi

    echo -e "${CYAN}=== Running Golden Tests ===${NC}"
    bash "$SCRIPT_DIR/run-golden-tests.sh" --stage1 "$STAGE1"
}

# Run 3-stage fixed point verification
cmd_verify() {
    if [ ! -f "$STAGE1" ]; then
        echo -e "${YELLOW}Stage 1 not found, building first...${NC}"
        cmd_build
    fi

    echo -e "${CYAN}=== 3-Stage Fixed Point Verification ===${NC}"

    # Stage 2: Stage 1 compiles bootstrap
    echo -e "${YELLOW}Stage 2: Stage 1 → Stage 2 IR${NC}"
    local start=$(get_time_ms)
    BMB_ARENA_MAX_SIZE=${BMB_ARENA_MAX_SIZE:-16G} "$STAGE1" "$BOOTSTRAP_SRC" "$DEV_DIR/stage2.ll.tmp" 2>&1
    tr '|' '\n' < "$DEV_DIR/stage2.ll.tmp" > "$DEV_DIR/stage2.ll"
    rm -f "$DEV_DIR/stage2.ll.tmp"
    local s2_lines=$(wc -l < "$DEV_DIR/stage2.ll")
    echo -e "${GREEN}Stage 2: $s2_lines lines ($(($(get_time_ms) - start))ms)${NC}"

    # Stage 3: Stage 2 binary compiles bootstrap
    echo -e "${YELLOW}Stage 3: Stage 2 → Stage 3 IR${NC}"
    start=$(get_time_ms)
    compile_ir "$DEV_DIR/stage2.ll" "$DEV_DIR/bmb-stage2${EXE_EXT}"
    BMB_ARENA_MAX_SIZE=${BMB_ARENA_MAX_SIZE:-16G} "$DEV_DIR/bmb-stage2${EXE_EXT}" "$BOOTSTRAP_SRC" "$DEV_DIR/stage3.ll.tmp" 2>&1
    tr '|' '\n' < "$DEV_DIR/stage3.ll.tmp" > "$DEV_DIR/stage3.ll"
    rm -f "$DEV_DIR/stage3.ll.tmp"
    local s3_lines=$(wc -l < "$DEV_DIR/stage3.ll")
    echo -e "Stage 3: $s3_lines lines ($(($(get_time_ms) - start))ms)"

    # Compare
    if diff -q "$DEV_DIR/stage2.ll" "$DEV_DIR/stage3.ll" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Fixed Point VERIFIED: Stage 2 == Stage 3${NC}"
    else
        echo -e "${RED}✗ Fixed Point FAILED: Stage 2 != Stage 3${NC}"
        diff "$DEV_DIR/stage2.ll" "$DEV_DIR/stage3.ll" | head -20
        exit 1
    fi
}

# Compile a BMB file to native binary
cmd_compile() {
    local INPUT="$1"
    local OUTPUT="${2:-${INPUT%.bmb}${EXE_EXT}}"

    if [ -z "$INPUT" ]; then
        echo "Usage: bmb-dev.sh compile <input.bmb> [output]"
        exit 1
    fi

    if [ ! -f "$STAGE1" ]; then
        echo -e "${YELLOW}Stage 1 not found, building first...${NC}"
        cmd_build
    fi

    ensure_runtime

    echo -e "${CYAN}=== Compiling $INPUT ===${NC}"
    local IR="${DEV_DIR}/$(basename "${INPUT%.bmb}").ll"

    "$STAGE1" "$INPUT" "${IR}.tmp" 2>&1
    tr '|' '\n' < "${IR}.tmp" > "$IR"
    rm -f "${IR}.tmp"

    compile_ir "$IR" "$OUTPUT"
    echo -e "${GREEN}Compiled: $OUTPUT${NC}"
}

# Full cycle: build + test + verify
cmd_full() {
    local total_start=$(get_time_ms)
    cmd_build
    echo ""
    cmd_test
    echo ""
    cmd_verify
    echo ""
    local total_end=$(get_time_ms)
    echo -e "${GREEN}=== Full Cycle Complete ($((total_end - total_start))ms) ===${NC}"
}

# Main
case "${1:-}" in
    build)
        cmd_build
        ;;
    test)
        cmd_test
        ;;
    verify)
        cmd_verify
        ;;
    full)
        cmd_full
        ;;
    compile)
        cmd_compile "$2" "$3"
        ;;
    *)
        echo "BMB Bootstrap-First Development Tool"
        echo ""
        echo "Usage:"
        echo "  $0 build            Build Stage 1 from golden binary"
        echo "  $0 test             Run golden tests"
        echo "  $0 verify           Run 3-stage fixed point verification"
        echo "  $0 full             Build + test + verify"
        echo "  $0 compile <file>   Compile a BMB file to native binary"
        echo ""
        echo "Development Workflow:"
        echo "  1. Edit bootstrap/compiler.bmb"
        echo "  2. Run: $0 full"
        echo "  3. If all tests pass, commit"
        ;;
esac
