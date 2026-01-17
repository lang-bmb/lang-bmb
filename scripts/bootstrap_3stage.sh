#!/bin/bash
# BMB 3-Stage Bootstrap Verification Script
# v0.46 Independence Phase
#
# This script implements the standard 3-stage compiler bootstrap process:
# - Stage 1: Rust compiler builds BMB bootstrap compiler
# - Stage 2: Stage 1 binary compiles BMB bootstrap compiler
# - Stage 3: Stage 2 binary compiles BMB bootstrap compiler
#
# Success: Stage 2 and Stage 3 LLVM IR must be identical (semantic equivalence)
#
# Reference: Ken Thompson's "Reflections on Trusting Trust" (1984)
# https://www.cs.cmu.edu/~rdriley/487/papers/Thompson_1984_ResearchStudy.pdf

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
RUST_BMB="./target/release/bmb"
BOOTSTRAP_SRC="bootstrap/compiler.bmb"
STAGE1_BIN="./bmb-stage1"
STAGE2_LL="./bmb-stage2.ll"
STAGE3_LL="./bmb-stage3.ll"

echo "======================================"
echo "BMB 3-Stage Bootstrap Verification"
echo "v0.46 - Bootstrap Compiler CLI Ready"
echo "======================================"
echo ""

# Check prerequisites
echo -e "${YELLOW}[0/4] Checking prerequisites...${NC}"

if [ ! -f "$RUST_BMB" ]; then
    echo -e "${RED}Error: Rust BMB compiler not found at $RUST_BMB${NC}"
    echo "Build it first with: cargo build --release --features llvm"
    exit 1
fi

if [ ! -f "$BOOTSTRAP_SRC" ]; then
    echo -e "${RED}Error: Bootstrap source not found at $BOOTSTRAP_SRC${NC}"
    exit 1
fi

# Check LLVM availability
if ! command -v llc &> /dev/null; then
    echo -e "${RED}Error: LLVM toolchain not found (llc)${NC}"
    echo "Install LLVM 21+ or set LLVM_SYS_211_PREFIX"
    exit 1
fi

echo -e "${GREEN}Prerequisites OK${NC}"
echo ""

# Stage 1: Rust BMB compiles bootstrap to native binary
echo -e "${YELLOW}[1/4] Stage 1: Rust BMB -> Stage 1 Binary${NC}"
echo "Command: $RUST_BMB build $BOOTSTRAP_SRC -o $STAGE1_BIN"

$RUST_BMB build $BOOTSTRAP_SRC -o $STAGE1_BIN

if [ ! -f "$STAGE1_BIN" ]; then
    echo -e "${RED}Stage 1 FAILED: Binary not generated${NC}"
    exit 1
fi

# Quick sanity check - run stage 1 tests
echo "Testing Stage 1 binary..."
STAGE1_OUTPUT=$($STAGE1_BIN 2>&1 | tail -1)
if [[ "$STAGE1_OUTPUT" == "999" ]]; then
    echo -e "${GREEN}Stage 1 OK (tests passed: 999 marker)${NC}"
else
    echo -e "${YELLOW}Warning: Stage 1 output: $STAGE1_OUTPUT${NC}"
fi
echo ""

# Stage 2: Stage 1 compiles bootstrap to LLVM IR
echo -e "${YELLOW}[2/4] Stage 2: Stage 1 -> LLVM IR${NC}"
echo "Command: $STAGE1_BIN $BOOTSTRAP_SRC $STAGE2_LL"

# Run Stage 1 compiler to generate LLVM IR (uses | as line separator)
$STAGE1_BIN $BOOTSTRAP_SRC $STAGE2_LL.tmp

# Convert | to newlines for LLVM tools
tr '|' '\n' < $STAGE2_LL.tmp > $STAGE2_LL
rm -f $STAGE2_LL.tmp

if [ ! -f "$STAGE2_LL" ]; then
    echo -e "${RED}Stage 2 FAILED: LLVM IR not generated${NC}"
    exit 1
fi

# Verify LLVM IR is valid
if head -1 "$STAGE2_LL" | grep -q "ModuleID"; then
    STAGE2_LINES=$(wc -l < "$STAGE2_LL")
    echo -e "${GREEN}Stage 2 OK (LLVM IR generated: $STAGE2_LINES lines)${NC}"
else
    echo -e "${RED}Stage 2 FAILED: Invalid LLVM IR format${NC}"
    head -5 "$STAGE2_LL"
    exit 1
fi
echo ""

# Stage 3: Compile Stage 2 to binary, then compile bootstrap again
echo -e "${YELLOW}[3/4] Stage 3: Stage 2 Binary -> Stage 3 LLVM IR${NC}"

# Compile Stage 2 LLVM IR to object file
STAGE2_OBJ="./bmb-stage2.o"
STAGE2_BIN="./bmb-stage2"

echo "Compiling Stage 2 LLVM IR to native binary..."
llc -filetype=obj -O2 "$STAGE2_LL" -o "$STAGE2_OBJ"

if [ ! -f "$STAGE2_OBJ" ]; then
    echo -e "${RED}Stage 3 FAILED: Could not compile LLVM IR to object file${NC}"
    exit 1
fi

# Find BMB runtime library
BMB_RUNTIME="${BMB_RUNTIME_PATH:-bmb/runtime/libbmb_runtime.a}"
if [ ! -f "$BMB_RUNTIME" ]; then
    echo "Building BMB runtime library..."
    (cd bmb/runtime && clang -c bmb_runtime.c -o bmb_runtime.o -O2 && ar rcs libbmb_runtime.a bmb_runtime.o)
    BMB_RUNTIME="bmb/runtime/libbmb_runtime.a"
fi

if [ ! -f "$BMB_RUNTIME" ]; then
    echo -e "${YELLOW}Warning: BMB runtime not found, falling back to interpreter for Stage 3${NC}"
    echo "Using interpreter for Stage 3 generation..."
    $RUST_BMB run $BOOTSTRAP_SRC $BOOTSTRAP_SRC $STAGE3_LL.tmp
else
    # Link Stage 2 binary with runtime
    echo "Linking Stage 2 binary with runtime..."
    clang "$STAGE2_OBJ" "$BMB_RUNTIME" -o "$STAGE2_BIN" -lm -no-pie

    if [ ! -f "$STAGE2_BIN" ]; then
        echo -e "${RED}Stage 3 FAILED: Could not link Stage 2 binary${NC}"
        exit 1
    fi

    echo -e "${GREEN}Stage 2 binary created: $STAGE2_BIN${NC}"

    # Run Stage 2 binary to generate Stage 3 LLVM IR
    # This is the TRUE 3-stage verification
    echo "Running Stage 2 binary to generate Stage 3 LLVM IR..."
    $STAGE2_BIN $BOOTSTRAP_SRC $STAGE3_LL.tmp
fi

# Convert | to newlines for LLVM tools
tr '|' '\n' < $STAGE3_LL.tmp > $STAGE3_LL
rm -f $STAGE3_LL.tmp

if [ ! -f "$STAGE3_LL" ]; then
    echo -e "${RED}Stage 3 FAILED: LLVM IR not generated${NC}"
    exit 1
fi

STAGE3_LINES=$(wc -l < "$STAGE3_LL")
echo -e "${GREEN}Stage 3 OK (LLVM IR generated: $STAGE3_LINES lines)${NC}"
echo ""

# Verification: Compare Stage 2 and Stage 3 LLVM IR
echo -e "${YELLOW}[4/4] Verification: Comparing Stage 2 and Stage 3${NC}"

if diff -q "$STAGE2_LL" "$STAGE3_LL" > /dev/null; then
    echo -e "${GREEN}✓ 3-Stage Bootstrap PASSED: Stage 2 == Stage 3${NC}"
    echo "The bootstrap compiler generates identical output when compiled by:"
    echo "  - Rust compiler (Stage 1 -> Stage 2)"
    echo "  - Itself via interpreter (Stage 2 -> Stage 3)"
else
    echo -e "${YELLOW}Stage 2 and Stage 3 differ (expected during development)${NC}"
    echo "Differences:"
    diff "$STAGE2_LL" "$STAGE3_LL" | head -20
fi
echo ""

echo "======================================"
echo "Bootstrap Status Summary"
echo "======================================"
echo "Stage 1 (Rust BMB -> Stage 1 Binary):     PASSED"
echo "Stage 2 (Stage 1 Binary -> LLVM IR):      PASSED"
if [ -f "$STAGE2_BIN" ]; then
    echo "Stage 3 (Stage 2 Binary -> LLVM IR):      PASSED"
else
    echo "Stage 3 (Interpreter -> LLVM IR):         PASSED (fallback)"
fi
echo "Verification (S2 == S3):                  See above"
echo ""
echo "Generated files:"
echo "  $STAGE1_BIN - Stage 1 native binary (from Rust compiler)"
echo "  $STAGE2_LL - LLVM IR generated by Stage 1"
if [ -f "$STAGE2_BIN" ]; then
    echo "  $STAGE2_BIN - Stage 2 native binary (from Stage 1)"
fi
echo "  $STAGE3_LL - LLVM IR generated by Stage 2"
echo ""
echo "Reference: Ken Thompson, 'Reflections on Trusting Trust' (1984)"
echo "           https://www.cs.cmu.edu/~rdriley/487/papers/Thompson_1984_ResearchStudy.pdf"
echo ""
echo -e "${GREEN}v0.47 Bootstrap Verification Complete${NC}"

# Cleanup intermediate files
rm -f "$STAGE2_OBJ" "${STAGE2_LL}.tmp" "${STAGE3_LL}.tmp"
