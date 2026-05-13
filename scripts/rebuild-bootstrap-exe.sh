#!/bin/bash
# Rebuild bootstrap/compiler.exe when compiler.bmb has changed.
#
# Root cause prevented: bootstrap/compiler.exe with 2MB stack (pre-Cycle-2780 build)
# caused STATUS_STACK_OVERFLOW on deeply nested sources (e.g. hash_table benchmark).
# The Rust bmb linker applies -Wl,--stack,67108864 (64MB) automatically; rebuilding
# with `bmb build --fast-compile` restores this.
#
# Usage:
#   ./scripts/rebuild-bootstrap-exe.sh [--force] [--json] [--check-only]
#
# Options:
#   --force       Rebuild even if timestamp is current
#   --json        Machine-readable output
#   --check-only  Exit 1 if stale (no rebuild); use in CI to detect drift
#
# Exit codes:
#   0  exe is current (or was successfully rebuilt)
#   1  --check-only and exe is stale, OR rebuild failed

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BOOTSTRAP_SRC="${PROJECT_ROOT}/bootstrap/compiler.bmb"
BOOTSTRAP_EXE="${PROJECT_ROOT}/bootstrap/compiler.exe"
# On Linux/macOS the exe has no extension
[ ! -f "$BOOTSTRAP_EXE" ] && BOOTSTRAP_EXE="${PROJECT_ROOT}/bootstrap/compiler"

# Locate the Rust bmb binary
if [ -f "${PROJECT_ROOT}/target/release/bmb.exe" ]; then
    RUST_BMB="${PROJECT_ROOT}/target/release/bmb.exe"
elif [ -f "${PROJECT_ROOT}/target/release/bmb" ]; then
    RUST_BMB="${PROJECT_ROOT}/target/release/bmb"
else
    echo '{"status":"error","message":"Rust bmb binary not found — run cargo build --release first"}' >&2
    exit 1
fi

FORCE=false
JSON_OUTPUT=false
CHECK_ONLY=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --force)      FORCE=true;      shift ;;
        --json)       JSON_OUTPUT=true; shift ;;
        --check-only) CHECK_ONLY=true;  shift ;;
        *) echo "Unknown option: $1" >&2; exit 1 ;;
    esac
done

log() { [ "$JSON_OUTPUT" = false ] && echo -e "$1" || true; }

# ── staleness check ─────────────────────────────────────────────────────────
is_stale() {
    # exe missing → definitely stale
    [ ! -f "$BOOTSTRAP_EXE" ] && return 0
    # force flag → treat as stale
    [ "$FORCE" = true ] && return 0
    # exe older than source → stale
    [ "$BOOTSTRAP_SRC" -nt "$BOOTSTRAP_EXE" ] && return 0
    return 1
}

# ── stack-size check (Windows PE32+) ────────────────────────────────────────
check_stack_mb() {
    local exe="$1"
    if command -v python3 &>/dev/null; then
        python3 - "$exe" <<'PYEOF'
import sys, struct
data = open(sys.argv[1], 'rb').read()
pe_off = struct.unpack_from('<I', data, 0x3C)[0]
magic  = struct.unpack_from('<H', data, pe_off + 24)[0]
if magic == 0x20B:  # PE32+
    reserve = struct.unpack_from('<Q', data, pe_off + 24 + 72)[0]
    print(reserve // (1024*1024))
else:
    print(0)
PYEOF
    else
        echo "0"
    fi
}

# ── main ────────────────────────────────────────────────────────────────────
if ! is_stale; then
    STACK_MB=$(check_stack_mb "$BOOTSTRAP_EXE" 2>/dev/null || echo "0")
    if [ "$JSON_OUTPUT" = true ]; then
        echo "{\"status\":\"current\",\"stack_mb\":${STACK_MB},\"rebuilt\":false}"
    else
        log "bootstrap/compiler.exe is current (stack: ${STACK_MB} MB)"
    fi
    exit 0
fi

if [ "$CHECK_ONLY" = true ]; then
    if [ "$JSON_OUTPUT" = true ]; then
        echo '{"status":"stale","rebuilt":false}'
    else
        log "ERROR: bootstrap/compiler.exe is stale vs compiler.bmb (run rebuild-bootstrap-exe.sh)" >&2
    fi
    exit 1
fi

log "bootstrap/compiler.exe is stale — rebuilding with --fast-compile ..."
BUILD_START=$(date +%s%3N 2>/dev/null || python3 -c 'import time; print(int(time.time()*1000))')

if "$RUST_BMB" build "$BOOTSTRAP_SRC" -o "$BOOTSTRAP_EXE" --fast-compile >&2 2>&1; then
    BUILD_END=$(date +%s%3N 2>/dev/null || python3 -c 'import time; print(int(time.time()*1000))')
    BUILD_MS=$((BUILD_END - BUILD_START))
    STACK_MB=$(check_stack_mb "$BOOTSTRAP_EXE" 2>/dev/null || echo "0")

    if [ "$STACK_MB" -gt 0 ] && [ "$STACK_MB" -lt 32 ]; then
        if [ "$JSON_OUTPUT" = true ]; then
            echo "{\"status\":\"warn\",\"stack_mb\":${STACK_MB},\"rebuilt\":true,\"build_ms\":${BUILD_MS},\"message\":\"stack < 32 MB — check linker flags\"}"
        else
            log "WARNING: rebuilt exe has only ${STACK_MB} MB stack (expected ≥ 64 MB)"
        fi
    else
        if [ "$JSON_OUTPUT" = true ]; then
            echo "{\"status\":\"ok\",\"stack_mb\":${STACK_MB},\"rebuilt\":true,\"build_ms\":${BUILD_MS}}"
        else
            log "OK: bootstrap/compiler.exe rebuilt in ${BUILD_MS} ms (stack: ${STACK_MB} MB)"
        fi
    fi
else
    if [ "$JSON_OUTPUT" = true ]; then
        echo '{"status":"error","message":"bmb build failed","rebuilt":false}'
    else
        log "ERROR: bmb build bootstrap/compiler.bmb failed" >&2
    fi
    exit 1
fi
