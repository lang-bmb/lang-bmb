#!/bin/bash
# BMB Bootstrap End-to-End Test Script

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUNTIME_DIR="$SCRIPT_DIR/../../runtime"
PROJECT_DIR="$SCRIPT_DIR/../.."

cd "$SCRIPT_DIR"

echo "=== BMB Bootstrap End-to-End Test ==="
echo ""

# Find clang
if command -v clang &> /dev/null; then
    CLANG="clang"
elif [ -f "/c/Program Files/LLVM/bin/clang.exe" ]; then
    CLANG="/c/Program Files/LLVM/bin/clang.exe"
else
    echo "ERROR: clang not found"
    exit 1
fi

echo "Using clang: $CLANG"
echo ""

# Step 1: Run with interpreter (baseline)
echo "[1/5] Running with BMB interpreter..."
cd "$PROJECT_DIR"
INTERPRETER_RESULT=$(cargo run --release --bin bmb -- run examples/bootstrap_test/fibonacci.bmb 2>&1 | grep -E '^[0-9]+$' | head -1)
cd "$SCRIPT_DIR"
echo "  Interpreter result: $INTERPRETER_RESULT"

# Step 2: Compile LLVM IR to object file
echo "[2/5] Compiling LLVM IR..."
"$CLANG" -c fibonacci.ll -o fibonacci.o 2>&1 | grep -v "warning:" || true
if [ ! -f fibonacci.o ]; then
    echo "  ERROR: Failed to compile LLVM IR"
    exit 1
fi
echo "  Created fibonacci.o"

# Step 3: Compile runtime
echo "[3/5] Compiling runtime..."
"$CLANG" -c "$RUNTIME_DIR/runtime.c" -o runtime.o 2>&1 | grep -v "warning:" || true
if [ ! -f runtime.o ]; then
    echo "  ERROR: Failed to compile runtime"
    exit 1
fi
echo "  Created runtime.o"

# Step 4: Link
echo "[4/5] Linking..."
"$CLANG" fibonacci.o runtime.o -o fibonacci 2>&1 | grep -v "warning:" || true
if [ ! -f fibonacci ]; then
    echo "  ERROR: Failed to link"
    exit 1
fi
SIZE=$(stat -c%s fibonacci 2>/dev/null || stat -f%z fibonacci 2>/dev/null || echo "unknown")
echo "  Created fibonacci ($SIZE bytes)"

# Step 5: Run and compare
echo "[5/5] Running native executable..."
NATIVE_RESULT=$(./fibonacci 2>&1 | grep -E '^[0-9]+$' | head -1)
echo "  Native result: $NATIVE_RESULT"

echo ""
echo "=== Results ==="
echo "Interpreter: $INTERPRETER_RESULT"
echo "Native:      $NATIVE_RESULT"

if [ "$INTERPRETER_RESULT" = "$NATIVE_RESULT" ]; then
    echo ""
    echo "SUCCESS: Results match!"
    exit 0
else
    echo ""
    echo "FAILURE: Results do not match!"
    exit 1
fi
