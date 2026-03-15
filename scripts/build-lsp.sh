#!/bin/bash
# Build the self-hosted BMB LSP server
#
# Usage:
#   ./scripts/build-lsp.sh [--debug]
#
# Produces: lsp/bmb-lsp.exe (or lsp/bmb-lsp on Linux/Mac)
# Requires: bmb compiler (text backend), opt, clang, runtime objects

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BMB="${PROJECT_ROOT}/target/release/bmb"
RUNTIME_OBJ="${PROJECT_ROOT}/bmb/runtime/bmb_runtime.o"
RUNTIME_EVT="${PROJECT_ROOT}/bmb/runtime/bmb_event_loop.o"
LSP_SRC="${PROJECT_ROOT}/lsp/server.bmb"
LSP_DIR="${PROJECT_ROOT}/lsp"
BUILD_DIR="/tmp/bmb-lsp-build"

OPT_LEVEL="-O3"
SUFFIX=""

if [[ "$1" == "--debug" ]]; then
    OPT_LEVEL="-O0"
    SUFFIX="-debug"
fi

# Check prerequisites
BMB_ACTUAL="$BMB"
[ -f "${BMB}.exe" ] && BMB_ACTUAL="${BMB}.exe"
[ ! -f "$BMB_ACTUAL" ] && { echo "Error: BMB compiler not found at $BMB"; exit 1; }
[ ! -f "$RUNTIME_OBJ" ] && { echo "Error: Runtime not found at $RUNTIME_OBJ"; exit 1; }
command -v opt &>/dev/null || { echo "Error: opt not found"; exit 1; }
command -v clang &>/dev/null || { echo "Error: clang not found"; exit 1; }

mkdir -p "$BUILD_DIR"

EXE_EXT=""
LINK_FLAGS="-lm"
[[ "$OSTYPE" == "msys"* || "$OSTYPE" == "mingw"* || "$OSTYPE" == "cygwin"* ]] && {
    EXE_EXT=".exe"
    LINK_FLAGS="$LINK_FLAGS -lws2_32"
}

OUTPUT="${LSP_DIR}/bmb-lsp${SUFFIX}${EXE_EXT}"

echo "Building BMB LSP server..."
echo "  Source: $LSP_SRC"
echo "  Output: $OUTPUT"
echo "  Opt: $OPT_LEVEL"

# Step 1: Generate LLVM IR
echo "  [1/3] Generating LLVM IR..."
"$BMB_ACTUAL" build "$LSP_SRC" --emit-ir -o "$BUILD_DIR/server.ll"

# Step 2: Optimize
echo "  [2/3] Optimizing ($OPT_LEVEL)..."
opt $OPT_LEVEL --mcpu=native "$BUILD_DIR/server.ll" -S -o "$BUILD_DIR/server_opt.ll"

# Copy optimized IR for reference
cp "$BUILD_DIR/server_opt.ll" "$LSP_DIR/server.ll"

# Step 3: Compile and link
echo "  [3/3] Compiling and linking..."
LLD_FLAG=""
command -v ld.lld &>/dev/null && LLD_FLAG="-fuse-ld=lld -Wl,--gc-sections"
clang -w -O3 -fno-unroll-loops -march=native -ffunction-sections \
    $LLD_FLAG "$BUILD_DIR/server_opt.ll" "$RUNTIME_OBJ" "$RUNTIME_EVT" \
    -o "$OUTPUT" $LINK_FLAGS

echo ""
echo "Done: $OUTPUT ($(wc -c < "$OUTPUT" | tr -d ' ') bytes)"
echo ""
echo "Usage in VS Code:"
echo "  Set bmb.lspServerPath to: $(cd "$LSP_DIR" && pwd)/bmb-lsp${SUFFIX}${EXE_EXT}"
echo "  Set bmb.serverPath to: $(cd "$(dirname "$BMB_ACTUAL")" && pwd)/$(basename "$BMB_ACTUAL")"
