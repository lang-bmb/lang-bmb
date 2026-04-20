#!/bin/bash
# Smoke test for `bmb bench --compare`.
#
# Validates exit codes and output status strings across the 5 classification
# categories (OK / REGRESSION / IMPROVEMENT / MISSING / NEW) plus error paths
# (invalid JSON, missing file, missing arg).
#
# Usage:
#   ./scripts/test-bench-compare.sh
#
# Exit 0 if all assertions pass, 1 otherwise.

set -u

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

if [ -f "${PROJECT_ROOT}/target/release/bmb.exe" ]; then
    BMB="${PROJECT_ROOT}/target/release/bmb.exe"
elif [ -f "${PROJECT_ROOT}/target/release/bmb" ]; then
    BMB="${PROJECT_ROOT}/target/release/bmb"
else
    echo "FAIL: bmb binary not found in target/release/"
    exit 1
fi

TMPDIR="$(mktemp -d)"
trap 'rm -rf "$TMPDIR"' EXIT

PASS=0
FAIL=0

check() {
    local name="$1"; shift
    local expected_exit="$1"; shift
    local expected_substr="$1"; shift
    local output
    output="$("$@" 2>&1)"
    local actual_exit=$?
    if [ "$actual_exit" -ne "$expected_exit" ]; then
        echo "FAIL [$name]: expected exit $expected_exit, got $actual_exit"
        echo "  output: $output"
        FAIL=$((FAIL + 1))
        return
    fi
    if [ -n "$expected_substr" ] && ! echo "$output" | grep -q "$expected_substr"; then
        echo "FAIL [$name]: output did not contain '$expected_substr'"
        echo "  output: $output"
        FAIL=$((FAIL + 1))
        return
    fi
    echo "PASS [$name]"
    PASS=$((PASS + 1))
}

BASELINE="$TMPDIR/baseline.json"
CURRENT_SAME="$TMPDIR/current_same.json"
CURRENT_REG="$TMPDIR/current_reg.json"
CURRENT_PARTIAL="$TMPDIR/current_partial.json"
BAD="$TMPDIR/bad.json"

cat > "$BASELINE" <<'EOF'
{"type":"bench","name":"a","file":"t","samples":10,"warmup":2,"min_ns":1000,"median_ns":1000,"p99_ns":1100,"mean_ns":1000.00,"stddev_ns":10}
{"type":"bench","name":"b","file":"t","samples":10,"warmup":2,"min_ns":2000,"median_ns":2000,"p99_ns":2100,"mean_ns":2000.00,"stddev_ns":10}
{"type":"bench_result","benches":2}
EOF

cp "$BASELINE" "$CURRENT_SAME"

cat > "$CURRENT_REG" <<'EOF'
{"type":"bench","name":"a","file":"t","samples":10,"warmup":2,"min_ns":1500,"median_ns":1500,"p99_ns":1600,"mean_ns":1500.00,"stddev_ns":10}
{"type":"bench","name":"b","file":"t","samples":10,"warmup":2,"min_ns":1500,"median_ns":1500,"p99_ns":1600,"mean_ns":1500.00,"stddev_ns":10}
{"type":"bench_result","benches":2}
EOF

cat > "$CURRENT_PARTIAL" <<'EOF'
{"type":"bench","name":"a","file":"t","samples":10,"warmup":2,"min_ns":1000,"median_ns":1000,"p99_ns":1100,"mean_ns":1000.00,"stddev_ns":10}
{"type":"bench","name":"c","file":"t","samples":10,"warmup":2,"min_ns":500,"median_ns":500,"p99_ns":600,"mean_ns":500.00,"stddev_ns":10}
{"type":"bench_result","benches":2}
EOF

echo "not json" > "$BAD"

# 1. Same inputs → exit 0, OK
check "same-inputs"       0 '"status":"OK"'           "$BMB" bench --compare "$BASELINE" "$CURRENT_SAME"

# 2. Regression + improvement → exit 1
check "regression-found"  1 '"status":"REGRESSION"'   "$BMB" bench --compare "$BASELINE" "$CURRENT_REG"
check "improvement-found" 1 '"status":"IMPROVEMENT"'  "$BMB" bench --compare "$BASELINE" "$CURRENT_REG"

# 3. MISSING + NEW (no regression)
check "missing-present"   0 '"status":"MISSING"'      "$BMB" bench --compare "$BASELINE" "$CURRENT_PARTIAL"
check "new-present"       0 '"status":"NEW"'          "$BMB" bench --compare "$BASELINE" "$CURRENT_PARTIAL"

# 4. Custom threshold overrides default
check "threshold-strict"  1 '"status":"REGRESSION"'   "$BMB" bench --compare "$BASELINE" "$CURRENT_REG" --threshold 0.1
check "threshold-loose"   0 '"status":"OK"'           "$BMB" bench --compare "$BASELINE" "$CURRENT_REG" --threshold 100

# 5. Error paths
check "missing-file"      1 'Failed to read'          "$BMB" bench --compare "$TMPDIR/does-not-exist.json" "$BASELINE"
check "invalid-json"      1 'invalid JSON'            "$BMB" bench --compare "$BAD" "$BASELINE"
check "missing-args"      1 'bench requires'          "$BMB" bench

echo ""
echo "Summary: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ]
