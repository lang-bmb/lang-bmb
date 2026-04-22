#!/bin/bash
# BMB Windows DLL dependency regression check
#
# Ensures that `bmb build --shared` output on Windows does not depend on
# MinGW runtime DLLs (libgcc_s_seh-1.dll, libwinpthread-1.dll). These
# DLLs ship only with MSYS2 and are absent on stock Windows 10+ machines,
# so their presence would break `pip install bmb-*` for end users.
#
# This is a regression check for Cycle 2423's `-static -static-libgcc`
# fix in bmb/src/build/mod.rs. Run after `python ecosystem/build_all.py`.
#
# Usage:
#   ./scripts/verify-dll-deps.sh
#
# Exits non-zero if any disallowed dependency is found.

set -e

# Skip on non-Windows — MinGW DLLs are a Windows-specific concern.
case "$(uname -s)" in
    MINGW*|MSYS*|CYGWIN*) ;;
    *)
        echo "verify-dll-deps.sh: skipped (not Windows)"
        exit 0
        ;;
esac

if ! command -v objdump >/dev/null 2>&1; then
    echo "verify-dll-deps.sh: objdump not found — install binutils" >&2
    exit 2
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# DLLs that must not appear in `objdump -p <lib>.dll` output.
# If a .dll depends on any of these, a fresh-Windows `pip install` will
# fail with "DLL load failed: The specified module could not be found".
disallowed=(
    "libgcc_s_seh-1.dll"    # MinGW GCC SEH unwinder
    "libwinpthread-1.dll"   # MinGW POSIX threads
    "libstdc++-6.dll"       # MinGW C++ runtime (not used today but guard)
)

# Libraries to check. Each must have a built .dll at the expected path.
libs=(algo compute crypto text json)

fail=0
echo "Verifying Windows DLL dependencies (Cycle 2423 regression check)..."

for lib in "${libs[@]}"; do
    dll="$PROJECT_ROOT/ecosystem/bmb-$lib/bmb_$lib.dll"
    if [[ ! -f "$dll" ]]; then
        echo "  [WARN] $dll not found — skip"
        continue
    fi

    deps="$(objdump -p "$dll" 2>/dev/null | grep 'DLL Name:' | awk '{print $3}' | tr -d '\r')"

    bad=""
    for forbidden in "${disallowed[@]}"; do
        if echo "$deps" | grep -qi "^$forbidden$"; then
            bad="$bad $forbidden"
        fi
    done

    if [[ -n "$bad" ]]; then
        echo "  [FAIL] bmb_$lib.dll depends on:$bad"
        fail=1
    else
        echo "  [OK]   bmb_$lib.dll — no disallowed deps"
    fi
done

if [[ $fail -ne 0 ]]; then
    echo ""
    echo "::error::One or more binding DLLs depend on MinGW runtime DLLs."
    echo "  Expected: only Windows 10+ system DLLs (kernel32, ws2_32, api-ms-win-crt-*)."
    echo "  Fix: ensure '-static -static-libgcc' is present in bmb/src/build/mod.rs"
    echo "       Windows link paths (both inkwell link_native and text backend clang block)."
    echo "  See: claudedocs/cycle-logs/cycle-2423.md"
    exit 1
fi

echo ""
echo "All 5 binding DLLs verified — no MinGW runtime dependencies."
