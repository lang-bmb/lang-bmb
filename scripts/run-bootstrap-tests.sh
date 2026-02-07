#!/bin/bash
# BMB Bootstrap Test Runner
# Runs all tests defined in bootstrap/tests/tests.txt

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

TESTS_DIR="${PROJECT_ROOT}/bootstrap/tests"
MANIFEST="${TESTS_DIR}/tests.txt"
RUNTIME_DIR="${PROJECT_ROOT}/bmb/runtime"
OUTPUT_DIR="${PROJECT_ROOT}/target/test-runner"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Detect platform
detect_platform() {
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
}

detect_platform
mkdir -p "$OUTPUT_DIR"

# Use golden bootstrap to get a working Stage 1 binary
if [ -f "${PROJECT_ROOT}/target/bootstrap/bmb-stage1${EXE_EXT}" ]; then
    BMB="${PROJECT_ROOT}/target/bootstrap/bmb-stage1${EXE_EXT}"
    echo -e "${YELLOW}Using bootstrap Stage 1 binary${NC}"
elif [ -f "${PROJECT_ROOT}/target/golden-bootstrap/bmb-stage1${EXE_EXT}" ]; then
    BMB="${PROJECT_ROOT}/target/golden-bootstrap/bmb-stage1${EXE_EXT}"
    echo -e "${YELLOW}Using golden bootstrap Stage 1 binary${NC}"
else
    echo -e "${YELLOW}Running bootstrap to get Stage 1...${NC}"
    bash "${PROJECT_ROOT}/scripts/bootstrap.sh" --stage1-only
    BMB="${PROJECT_ROOT}/target/bootstrap/bmb-stage1${EXE_EXT}"
fi

PASSED=0
FAILED=0

echo ""
echo "========================================"
echo "BMB Bootstrap Test Runner"
echo "========================================"
echo ""

# Read manifest and run each test
while IFS= read -r line || [ -n "$line" ]; do
    # Skip comments and empty lines
    [[ "$line" =~ ^#.*$ ]] && continue
    [[ -z "$line" ]] && continue
    [[ ! "$line" =~ \.bmb\| ]] && continue

    # Parse: filename|expected_count
    FILENAME=$(echo "$line" | cut -d'|' -f1)
    EXPECTED=$(echo "$line" | cut -d'|' -f2)

    TEST_FILE="${TESTS_DIR}/${FILENAME}"
    TEST_NAME=$(basename "$FILENAME" .bmb)

    if [ ! -f "$TEST_FILE" ]; then
        echo -e "${RED}✗ ${TEST_NAME}: File not found${NC}"
        ((FAILED++))
        continue
    fi

    echo -n "Running ${TEST_NAME}... "

    # Compile test
    IR_FILE="${OUTPUT_DIR}/${TEST_NAME}.ll"
    OPT_FILE="${OUTPUT_DIR}/${TEST_NAME}_opt.ll"
    EXE_FILE="${OUTPUT_DIR}/${TEST_NAME}${EXE_EXT}"

    # Step 1: Generate LLVM IR
    if ! "$BMB" "$TEST_FILE" "$IR_FILE" 2>/dev/null; then
        echo -e "${RED}✗ Compile failed${NC}"
        ((FAILED++))
        continue
    fi

    # Step 2: Optimize
    if ! opt -O2 -S "$IR_FILE" -o "$OPT_FILE" 2>/dev/null; then
        echo -e "${RED}✗ opt failed${NC}"
        ((FAILED++))
        continue
    fi

    # Step 3: Link
    if ! clang -O2 "$OPT_FILE" "${RUNTIME_DIR}/bmb_runtime.c" -o "$EXE_FILE" $LINK_LIBS 2>/dev/null; then
        echo -e "${RED}✗ Link failed${NC}"
        ((FAILED++))
        continue
    fi

    # Step 4: Run and capture output
    OUTPUT=$("$EXE_FILE" 2>&1)

    # Parse output: look for 777 (start), count, 999 (end)
    START_MARKER=$(echo "$OUTPUT" | grep -c "^777$" || true)
    END_MARKER=$(echo "$OUTPUT" | grep -c "^999$" || true)

    if [ "$START_MARKER" -ne 1 ] || [ "$END_MARKER" -ne 1 ]; then
        echo -e "${RED}✗ Invalid output format${NC}"
        ((FAILED++))
        continue
    fi

    # Extract test count (line before 999)
    ACTUAL=$(echo "$OUTPUT" | grep -B1 "^999$" | head -1)

    if [ "$ACTUAL" == "$EXPECTED" ]; then
        echo -e "${GREEN}✓ ${ACTUAL}/${EXPECTED} tests passed${NC}"
        ((PASSED++))
    else
        echo -e "${RED}✗ ${ACTUAL}/${EXPECTED} tests passed${NC}"
        ((FAILED++))
    fi

done < "$MANIFEST"

echo ""
echo "========================================"
echo "Results: ${PASSED} passed, ${FAILED} failed"
echo "========================================"

if [ "$FAILED" -gt 0 ]; then
    exit 1
fi
