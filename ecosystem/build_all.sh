#!/bin/bash
# Build all BMB binding libraries
# Usage: ./ecosystem/build_all.sh

set -e

BMB="./target/release/bmb"

echo "=== Building BMB Binding Libraries ==="
echo

# Check compiler exists
if [ ! -f "$BMB" ]; then
    echo "ERROR: BMB compiler not found at $BMB"
    echo "Run: cargo build --release"
    exit 1
fi

# bmb-algo (27 algorithms)
echo "[1/4] Building bmb-algo..."
$BMB build ecosystem/bmb-algo/src/lib.bmb --shared -o ecosystem/bmb-algo/bmb_algo.dll 2>&1 | grep -E "build_success|error"
cp ecosystem/bmb-algo/bmb_algo.dll ecosystem/bmb-algo/bindings/python/ 2>/dev/null || true

# bmb-crypto (11 functions)
echo "[2/4] Building bmb-crypto..."
$BMB build ecosystem/bmb-crypto/src/lib.bmb --shared -o ecosystem/bmb-crypto/bmb_crypto.dll 2>&1 | grep -E "build_success|error"
cp ecosystem/bmb-crypto/bmb_crypto.dll ecosystem/bmb-crypto/bindings/python/ 2>/dev/null || true

# bmb-text (16 functions)
echo "[3/4] Building bmb-text..."
$BMB build ecosystem/bmb-text/src/lib.bmb --shared -o ecosystem/bmb-text/bmb_text.dll 2>&1 | grep -E "build_success|error"
cp ecosystem/bmb-text/bmb_text.dll ecosystem/bmb-text/bindings/python/ 2>/dev/null || true

# bmb-json (8 functions)
echo "[4/4] Building bmb-json..."
$BMB build ecosystem/bmb-json/src/lib.bmb --shared -o ecosystem/bmb-json/bmb_json.dll 2>&1 | grep -E "build_success|error"
cp ecosystem/bmb-json/bmb_json.dll ecosystem/bmb-json/bindings/python/ 2>/dev/null || true

echo
echo "=== All 4 libraries built ==="
echo "  bmb-algo:   27 algorithms"
echo "  bmb-crypto: 11 functions"
echo "  bmb-text:   16 functions"
echo "  bmb-json:   8 functions"
echo "  Total:      62 @export functions"
