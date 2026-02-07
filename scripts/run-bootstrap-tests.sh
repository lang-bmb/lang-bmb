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

# Timeout (seconds) for each step
COMPILE_TIMEOUT=60
OPT_TIMEOUT=60
LINK_TIMEOUT=60
RUN_TIMEOUT=30

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
TOTAL=0

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
    ((TOTAL++)) || true

    if [ ! -f "$TEST_FILE" ]; then
        echo -e "${RED}  FAIL ${TEST_NAME}: File not found${NC}"
        ((FAILED++)) || true
        continue
    fi

    echo -n "  ${TEST_NAME}... "

    # Compile test
    IR_FILE="${OUTPUT_DIR}/${TEST_NAME}.ll"
    OPT_FILE="${OUTPUT_DIR}/${TEST_NAME}_opt.ll"
    EXE_FILE="${OUTPUT_DIR}/${TEST_NAME}${EXE_EXT}"
    OUT_FILE="${OUTPUT_DIR}/${TEST_NAME}_output.txt"

    # Step 1: Generate LLVM IR
    if ! timeout "$COMPILE_TIMEOUT" "$BMB" "$TEST_FILE" "$IR_FILE" >/dev/null 2>&1; then
        echo -e "${RED}FAIL (compile)${NC}"
        ((FAILED++)) || true
        continue
    fi

    # Step 2: Optimize
    if ! timeout "$OPT_TIMEOUT" opt -O2 -S "$IR_FILE" -o "$OPT_FILE" 2>/dev/null; then
        echo -e "${RED}FAIL (opt)${NC}"
        ((FAILED++)) || true
        continue
    fi

    # Step 3: Link
    if ! timeout "$LINK_TIMEOUT" clang -O2 "$OPT_FILE" "${RUNTIME_DIR}/bmb_runtime.c" -o "$EXE_FILE" $LINK_LIBS 2>/dev/null; then
        echo -e "${RED}FAIL (link)${NC}"
        ((FAILED++)) || true
        continue
    fi

    # Step 4: Run and capture output to file (avoids MSYS2 $() hang)
    timeout "$RUN_TIMEOUT" "$EXE_FILE" > "$OUT_FILE" 2>&1 || true

    # Parse output: look for 777 (start), count, 999 (end)
    START_MARKER=$(grep -c "^777$" "$OUT_FILE" 2>/dev/null || echo "0")
    END_MARKER=$(grep -c "^999$" "$OUT_FILE" 2>/dev/null || echo "0")

    if [ "$START_MARKER" -ne 1 ] || [ "$END_MARKER" -ne 1 ]; then
        echo -e "${RED}FAIL (invalid output)${NC}"
        ((FAILED++)) || true
        continue
    fi

    # Extract test count (line before 999)
    ACTUAL=$(grep -B1 "^999$" "$OUT_FILE" | head -1)

    if [ "$ACTUAL" == "$EXPECTED" ]; then
        echo -e "${GREEN}PASS ${ACTUAL}/${EXPECTED}${NC}"
        ((PASSED++)) || true
    else
        echo -e "${RED}FAIL ${ACTUAL}/${EXPECTED}${NC}"
        ((FAILED++)) || true
    fi

done < "$MANIFEST"

echo ""
echo "========================================"
echo "Results: ${PASSED}/${TOTAL} passed, ${FAILED} failed"
echo "========================================"

if [ "$FAILED" -gt 0 ]; then
    exit 1
fi
