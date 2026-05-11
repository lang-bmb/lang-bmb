#!/bin/bash
# Golden Manifest Audit: build & run each golden test, compare manifest expected
# vs actual stdout first line. Outputs candidate manifest fixes.
#
# Usage:
#   ./scripts/audit-golden-manifest.sh [--stage1 <path>] [--limit N]
#
# Output (to stdout):
#   {filename}|{manifest_expected}|{actual_stdout_line1}|{status}
# Status: OK | MISMATCH | BUILD_FAIL | RUN_FAIL

set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TESTS_DIR="${PROJECT_ROOT}/tests/bootstrap"
MANIFEST="${TESTS_DIR}/golden_tests.txt"
RUNTIME_DIR="${PROJECT_ROOT}/bmb/runtime"
OUTPUT_DIR="${PROJECT_ROOT}/target/golden-tests"

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

STAGE1=""
LIMIT=0

while [[ $# -gt 0 ]]; do
    case $1 in
        --stage1) STAGE1="$2"; shift 2 ;;
        --limit) LIMIT="$2"; shift 2 ;;
        *) echo "Unknown: $1" >&2; exit 1 ;;
    esac
done

if [ -z "$STAGE1" ]; then
    for c in \
        "${PROJECT_ROOT}/bootstrap_stage1${EXE_EXT}" \
        "${PROJECT_ROOT}/target/bootstrap/bmb-stage1${EXE_EXT}" \
        "${PROJECT_ROOT}/target/golden-tests/bmb-stage1${EXE_EXT}" \
    ; do
        if [ -f "$c" ]; then STAGE1="$c"; break; fi
    done
fi
[ -f "$STAGE1" ] || { echo "Stage 1 not found" >&2; exit 1; }

mkdir -p "$OUTPUT_DIR"

count=0
while IFS= read -r line || [ -n "$line" ]; do
    [[ "$line" =~ ^#.*$ ]] && continue
    [[ -z "$line" ]] && continue
    [[ ! "$line" =~ \| ]] && continue

    FILENAME=$(echo "$line" | cut -d'|' -f1)
    EXPECTED=$(echo "$line" | cut -d'|' -f2)
    TEST_FILE="${TESTS_DIR}/${FILENAME}"
    TEST_NAME=$(basename "$FILENAME" .bmb)

    [ -f "$TEST_FILE" ] || { echo "${FILENAME}|${EXPECTED}||MISSING_FILE"; continue; }

    IR_FILE="${OUTPUT_DIR}/${TEST_NAME}.ll"
    OPT_FILE="${OUTPUT_DIR}/${TEST_NAME}_opt.ll"
    OBJ_FILE="${OUTPUT_DIR}/${TEST_NAME}.o"
    EXE_FILE="${OUTPUT_DIR}/${TEST_NAME}${EXE_EXT}"
    OUT_FILE="${OUTPUT_DIR}/${TEST_NAME}_output.txt"

    if ! timeout 60 "$STAGE1" "$TEST_FILE" "$IR_FILE" >/dev/null 2>&1; then
        echo "${FILENAME}|${EXPECTED}||BUILD_FAIL_COMPILE"; continue
    fi
    if ! timeout 60 opt -O2 --slp-max-vf=1 -S "$IR_FILE" -o "$OPT_FILE" 2>/dev/null; then
        echo "${FILENAME}|${EXPECTED}||BUILD_FAIL_OPT"; continue
    fi
    if ! timeout 60 llc -O3 -filetype=obj "$OPT_FILE" -o "$OBJ_FILE" 2>/dev/null; then
        echo "${FILENAME}|${EXPECTED}||BUILD_FAIL_LLC"; continue
    fi
    if ! timeout 60 gcc -O2 -o "$EXE_FILE" "$OBJ_FILE" "${RUNTIME_DIR}/libbmb_runtime.a" $LINK_LIBS 2>/dev/null; then
        echo "${FILENAME}|${EXPECTED}||BUILD_FAIL_LINK"; continue
    fi

    timeout 30 "$EXE_FILE" > "$OUT_FILE" 2>/dev/null
    rc=$?
    ACTUAL=$(head -1 "$OUT_FILE" 2>/dev/null | tr -d '\r')

    if [ $rc -ne 0 ] && [ -z "$ACTUAL" ]; then
        echo "${FILENAME}|${EXPECTED}||RUN_FAIL_RC${rc}"; continue
    fi

    if [ "$EXPECTED" == "$ACTUAL" ]; then
        echo "${FILENAME}|${EXPECTED}|${ACTUAL}|OK"
    else
        echo "${FILENAME}|${EXPECTED}|${ACTUAL}|MISMATCH"
    fi

    count=$((count+1))
    if [ "$LIMIT" -gt 0 ] && [ "$count" -ge "$LIMIT" ]; then
        break
    fi
done < "$MANIFEST"
