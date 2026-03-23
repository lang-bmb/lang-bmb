#!/bin/bash
# Build all BMB binding libraries
# Usage: ./ecosystem/build_all.sh [--test] [--debug] [library_name]
#
# Prerequisites:
#   - BMB compiler built: cargo build --release --features llvm
#   - LLVM tools in PATH (opt, llc)
#   - GCC/Clang for linking
#   - Python 3.8+ (for --test)

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

# Find BMB compiler
BMB="$ROOT_DIR/target/release/bmb"
if [ ! -f "$BMB" ]; then
    echo "ERROR: BMB compiler not found at $BMB"
    echo "Run: cargo build --release --features llvm"
    exit 1
fi

# Platform-specific library naming
case "$(uname -s)" in
    Linux*)     LIB_PREFIX="lib"; LIB_EXT=".so";;
    Darwin*)    LIB_PREFIX="lib"; LIB_EXT=".dylib";;
    MINGW*|MSYS*|CYGWIN*) LIB_PREFIX=""; LIB_EXT=".dll";;
    *)          LIB_PREFIX="lib"; LIB_EXT=".so";;
esac

# Parse args
RUN_TESTS=0
RELEASE="--release"
TARGET_LIB=""

for arg in "$@"; do
    case "$arg" in
        --test) RUN_TESTS=1;;
        --debug) RELEASE="";;
        bmb-*) TARGET_LIB="$arg";;
    esac
done

LIBRARIES="bmb-algo bmb-compute bmb-crypto bmb-text bmb-json"
if [ -n "$TARGET_LIB" ]; then
    LIBRARIES="$TARGET_LIB"
fi

echo "=== Building BMB Binding Libraries ==="
echo "Platform: $(uname -s), Extension: $LIB_EXT"
echo

PASS=0
FAIL=0

for lib in $LIBRARIES; do
    module=$(echo "$lib" | tr '-' '_')
    src="$SCRIPT_DIR/$lib/src/lib.bmb"
    out_name="${LIB_PREFIX}${module}${LIB_EXT}"
    out_path="$SCRIPT_DIR/$lib/$out_name"

    echo -n "[$lib] "

    if "$BMB" build "$src" --shared $RELEASE -o "$out_path" 2>/dev/null; then
        size=$(du -h "$out_path" | cut -f1)
        echo "OK ($size)"

        # Copy to bindings/python/
        binding_dir="$SCRIPT_DIR/$lib/bindings/python"
        if [ -d "$binding_dir" ]; then
            cp "$out_path" "$binding_dir/$out_name"
        fi

        PASS=$((PASS + 1))

        if [ "$RUN_TESTS" = "1" ]; then
            echo -n "  Tests: "
            (cd "$SCRIPT_DIR/$lib" && python3 -m pytest tests/ -q --tb=short 2>&1 | tail -1)
        fi
    else
        echo "FAIL"
        FAIL=$((FAIL + 1))
    fi
done

echo
echo "Built: $PASS/$((PASS + FAIL)) libraries (128 @export functions)"
echo "  bmb-algo:    49 algorithms"
echo "  bmb-compute: 33 functions"
echo "  bmb-crypto:  11 functions"
echo "  bmb-text:    23 functions"
echo "  bmb-json:    12 functions"

if [ "$FAIL" -gt 0 ]; then
    exit 1
fi
echo "All libraries built successfully!"
