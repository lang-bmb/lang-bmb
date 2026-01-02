#!/bin/bash
# BMB Bootstrap LLVM IR Validation Script
# Validates all test programs can be compiled to valid object files

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "=== BMB Bootstrap LLVM IR Validation ==="
echo ""

# Find clang and llvm-nm
if command -v clang &> /dev/null; then
    CLANG="clang"
    LLVM_NM="llvm-nm"
elif [ -f "/c/Program Files/LLVM/bin/clang.exe" ]; then
    CLANG="/c/Program Files/LLVM/bin/clang.exe"
    LLVM_NM="/c/Program Files/LLVM/bin/llvm-nm.exe"
else
    echo "ERROR: clang not found"
    exit 1
fi

TESTS=("fibonacci" "factorial")
PASSED=0
FAILED=0

for test in "${TESTS[@]}"; do
    echo "--- Testing: $test ---"

    # Check if .ll file exists
    if [ ! -f "$test.ll" ]; then
        echo "  SKIP: $test.ll not found"
        continue
    fi

    # Compile to object file
    echo "  Compiling $test.ll..."
    if "$CLANG" -c "$test.ll" -o "$test.o" 2>&1 | grep -v "warning:"; then
        echo "  ✓ Compiled successfully"
    else
        if [ -f "$test.o" ]; then
            echo "  ✓ Compiled with warnings"
        else
            echo "  ✗ Compilation failed"
            FAILED=$((FAILED + 1))
            continue
        fi
    fi

    # Verify symbols
    echo "  Verifying symbols..."
    SYMBOLS=$("$LLVM_NM" "$test.o" 2>/dev/null || echo "")

    # Check for main symbol
    if echo "$SYMBOLS" | grep -q " T main"; then
        echo "  ✓ 'main' symbol found"
    else
        echo "  ✗ 'main' symbol not found"
        FAILED=$((FAILED + 1))
        continue
    fi

    # Check for println reference
    if echo "$SYMBOLS" | grep -q " U println"; then
        echo "  ✓ 'println' external reference found"
    else
        echo "  ✗ 'println' external reference not found"
        FAILED=$((FAILED + 1))
        continue
    fi

    # Show file size
    SIZE=$(stat -c%s "$test.o" 2>/dev/null || stat -f%z "$test.o" 2>/dev/null || echo "unknown")
    echo "  Object file size: $SIZE bytes"

    PASSED=$((PASSED + 1))
    echo "  ✓ $test PASSED"
    echo ""
done

# Cleanup
rm -f *.o

echo "=== Summary ==="
echo "Passed: $PASSED"
echo "Failed: $FAILED"
echo ""

if [ $FAILED -eq 0 ]; then
    echo "All tests passed!"
    exit 0
else
    echo "Some tests failed!"
    exit 1
fi
