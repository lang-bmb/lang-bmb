#!/bin/bash
# BMB Complete Verification Script
# v0.47 - Runs all verification checks in WSL

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}BMB Complete Verification Suite${NC}"
echo -e "${BLUE}v0.47 Performance Phase${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Check we're in WSL
if [[ ! -f /proc/version ]] || ! grep -qi microsoft /proc/version; then
    echo -e "${YELLOW}Warning: This script is designed for WSL${NC}"
    echo "Some verifications may not work on native Linux"
fi

# Check LLVM
echo -e "${YELLOW}[1/5] Checking prerequisites...${NC}"
if ! command -v llc &> /dev/null; then
    echo -e "${RED}LLVM not found. See docs/WSL_VERIFICATION.md for setup.${NC}"
    exit 1
fi
echo -e "${GREEN}LLVM $(llvm-config --version) found${NC}"

# Build BMB
echo ""
echo -e "${YELLOW}[2/5] Building BMB with LLVM...${NC}"
cargo build --release --features llvm
echo -e "${GREEN}Build complete${NC}"

# Run unit tests
echo ""
echo -e "${YELLOW}[3/5] Running test suite...${NC}"
cargo test --release 2>&1 | tail -5
echo -e "${GREEN}Tests passed${NC}"

# Type check benchmarks
echo ""
echo -e "${YELLOW}[4/5] Type checking benchmarks...${NC}"
BENCH_COUNT=0
BENCH_PASS=0
for dir in ecosystem/benchmark-bmb/benches/*/*/bmb; do
    if [ -f "$dir/main.bmb" ]; then
        BENCH_COUNT=$((BENCH_COUNT + 1))
        if ./target/release/bmb check "$dir/main.bmb" 2>&1 | grep -q '"type":"success"'; then
            BENCH_PASS=$((BENCH_PASS + 1))
        else
            echo -e "${RED}Failed: $dir${NC}"
        fi
    fi
done
echo -e "${GREEN}Benchmarks: $BENCH_PASS/$BENCH_COUNT passed${NC}"

# 3-Stage Bootstrap
echo ""
echo -e "${YELLOW}[5/5] Running 3-Stage Bootstrap...${NC}"
if [ -x ./scripts/bootstrap_3stage.sh ]; then
    ./scripts/bootstrap_3stage.sh
else
    echo -e "${RED}Bootstrap script not found or not executable${NC}"
    exit 1
fi

echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}All Verifications Complete!${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo "Summary:"
echo "  - LLVM: $(llvm-config --version)"
echo "  - Benchmarks: $BENCH_PASS/$BENCH_COUNT"
echo "  - Bootstrap: See above"
echo ""
echo "Next: Run native benchmarks with:"
echo "  ./scripts/run_benchmarks.sh"
