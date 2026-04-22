#!/bin/bash
# Build Python wheels for all BMB binding libraries.
#
# Usage:
#   ./scripts/build-wheel.sh                  # Build all 5 libs (skip compiler if bmb exists)
#   ./scripts/build-wheel.sh --lib bmb-algo   # Build only one lib
#   ./scripts/build-wheel.sh --dry-run        # Print plan, build nothing
#   ./scripts/build-wheel.sh --skip-compiler  # Assume compiler already built (CI)
#   ./scripts/build-wheel.sh --skip-libs      # Assume .dll/.so/.dylib already built
#   ./scripts/build-wheel.sh --verify         # After build, run twine check + install-import
#                                              # smoke test on every produced wheel
#
# Output: dist/wheels/<lib>-<version>-py3-none-<platform>.whl
#
# Expected wheel tag (set by each lib's setup.py shim):
#   py3-none-win_amd64     (Windows x86_64)
#   py3-none-manylinux_*   (Linux, via auditwheel if needed)
#   py3-none-macosx_*      (macOS, native)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

ALL_LIBS=(bmb-algo bmb-compute bmb-crypto bmb-text bmb-json)

LIB=""
DRY_RUN=0
SKIP_COMPILER=0
SKIP_LIBS=0
VERIFY=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --lib)
            LIB="$2"; shift 2 ;;
        --dry-run)
            DRY_RUN=1; shift ;;
        --skip-compiler)
            SKIP_COMPILER=1; shift ;;
        --skip-libs)
            SKIP_LIBS=1; shift ;;
        --verify)
            VERIFY=1; shift ;;
        -h|--help)
            sed -n '2,22p' "$0"; exit 0 ;;
        *)
            echo "Unknown option: $1" >&2; exit 2 ;;
    esac
done

# Select libs to build
if [[ -n "$LIB" ]]; then
    LIBS=("$LIB")
else
    LIBS=("${ALL_LIBS[@]}")
fi

# Validate each requested lib has a pyproject.toml
for lib in "${LIBS[@]}"; do
    if [[ ! -f "$PROJECT_ROOT/ecosystem/$lib/pyproject.toml" ]]; then
        echo "ERROR: ecosystem/$lib/pyproject.toml not found" >&2
        exit 3
    fi
done

# Locate or build the BMB compiler
find_bmb() {
    for candidate in \
        "$PROJECT_ROOT/target/x86_64-pc-windows-gnu/release/bmb.exe" \
        "$PROJECT_ROOT/target/x86_64-pc-windows-msvc/release/bmb.exe" \
        "$PROJECT_ROOT/target/release/bmb.exe" \
        "$PROJECT_ROOT/target/x86_64-unknown-linux-gnu/release/bmb" \
        "$PROJECT_ROOT/target/x86_64-apple-darwin/release/bmb" \
        "$PROJECT_ROOT/target/aarch64-apple-darwin/release/bmb" \
        "$PROJECT_ROOT/target/release/bmb"; do
        if [[ -x "$candidate" ]]; then
            echo "$candidate"
            return 0
        fi
    done
    return 1
}

echo "=== BMB wheel build ==="
echo "Project:   $PROJECT_ROOT"
echo "Libraries: ${LIBS[*]}"
echo "Dry-run:   $DRY_RUN"

# Step 1: ensure BMB compiler is available
if [[ "$SKIP_COMPILER" -eq 0 ]]; then
    if ! BMB_COMPILER="$(find_bmb)"; then
        if [[ "$DRY_RUN" -eq 1 ]]; then
            echo "[dry-run] Would run: cargo build --release --features llvm"
            BMB_COMPILER="$PROJECT_ROOT/target/release/bmb"
        else
            echo "--- BMB compiler not found, building (cargo build --release --features llvm) ---"
            cd "$PROJECT_ROOT"
            # Use GNU target on Windows to avoid MSVC LLVM header conflicts
            case "$(uname -s 2>/dev/null || echo Unknown)" in
                MINGW*|MSYS*|CYGWIN*|Windows*)
                    cargo build --release --features llvm --target x86_64-pc-windows-gnu
                    ;;
                *)
                    cargo build --release --features llvm
                    ;;
            esac
            BMB_COMPILER="$(find_bmb)"
        fi
    fi
    echo "BMB compiler: $BMB_COMPILER"
else
    BMB_COMPILER="$(find_bmb)" || true
    echo "BMB compiler: ${BMB_COMPILER:-<skipped lookup>}"
fi

# Step 2: ensure BMB_RUNTIME_PATH is set for build_all.py
export BMB_RUNTIME_PATH="${BMB_RUNTIME_PATH:-$PROJECT_ROOT/bmb/runtime}"
echo "BMB_RUNTIME_PATH: $BMB_RUNTIME_PATH"

# Step 3: build all shared libraries (.dll/.so/.dylib) via build_all.py
if [[ "$SKIP_LIBS" -eq 0 ]]; then
    if [[ "$DRY_RUN" -eq 1 ]]; then
        echo "[dry-run] Would run: python ecosystem/build_all.py"
    else
        echo "--- Building native shared libraries ---"
        cd "$PROJECT_ROOT"
        if [[ -n "$LIB" ]]; then
            python ecosystem/build_all.py "$LIB"
        else
            python ecosystem/build_all.py
        fi
    fi
fi

# Step 4: build wheels into dist/wheels/
DIST_DIR="$PROJECT_ROOT/dist/wheels"
if [[ "$DRY_RUN" -eq 1 ]]; then
    echo "[dry-run] Would create: $DIST_DIR"
else
    mkdir -p "$DIST_DIR"
fi

for lib in "${LIBS[@]}"; do
    lib_dir="$PROJECT_ROOT/ecosystem/$lib"
    if [[ "$DRY_RUN" -eq 1 ]]; then
        echo "[dry-run] Would run: (cd $lib_dir && pip wheel . --no-deps -w $DIST_DIR)"
    else
        echo "--- Building wheel: $lib ---"
        (cd "$lib_dir" && pip wheel . --no-deps -w "$DIST_DIR" --quiet)
    fi
done

# Step 5: summary
if [[ "$DRY_RUN" -eq 0 ]]; then
    echo
    echo "=== Built wheels ==="
    ls -1 "$DIST_DIR"/*.whl 2>/dev/null || { echo "No wheels produced!" >&2; exit 4; }

    # Validate platform tagging (fail if any wheel is py3-none-any)
    if ls "$DIST_DIR"/*-py3-none-any.whl >/dev/null 2>&1; then
        echo
        echo "ERROR: detected pure-Python wheel(s) with tag py3-none-any:" >&2
        ls "$DIST_DIR"/*-py3-none-any.whl >&2
        echo "Expected platform-specific tag (py3-none-<platform>)." >&2
        echo "Verify setup.py shim in each ecosystem/<lib>/." >&2
        exit 5
    fi
fi

# Step 6 (optional): twine check + install-import smoke test
if [[ "$VERIFY" -eq 1 && "$DRY_RUN" -eq 0 ]]; then
    echo
    echo "=== Verifying wheels (twine check + install-import) ==="
    if ! command -v twine >/dev/null 2>&1; then
        echo "--- Installing twine (one-time) ---"
        pip install --quiet twine
    fi
    twine check "$DIST_DIR"/*.whl

    VENV_DIR="$(mktemp -d 2>/dev/null || echo /tmp/bmb-verify-venv)"
    VENV_DIR="$VENV_DIR/bmb-verify-venv"
    rm -rf "$VENV_DIR"
    python -m venv "$VENV_DIR"
    if [[ -f "$VENV_DIR/Scripts/python.exe" ]]; then
        VENV_PY="$VENV_DIR/Scripts/python.exe"
    else
        VENV_PY="$VENV_DIR/bin/python"
    fi

    fail=0
    for whl in "$DIST_DIR"/*.whl; do
        lib_mod="$(basename "$whl" | cut -d- -f1)"   # e.g. bmb_algo
        echo "--- $lib_mod ---"
        if ! "$VENV_PY" -m pip install --quiet --force-reinstall "$whl"; then
            echo "ERROR: pip install failed for $lib_mod" >&2; fail=1; continue
        fi
        if ! "$VENV_PY" -c "import $lib_mod; funcs=[x for x in dir($lib_mod) if not x.startswith('_')]; print(f'  {len(funcs)} public funcs')"; then
            echo "ERROR: import failed for $lib_mod" >&2; fail=1
        fi
    done
    if [[ $fail -ne 0 ]]; then
        echo "Verification FAILED." >&2
        exit 6
    fi
fi

echo
echo "Done."
