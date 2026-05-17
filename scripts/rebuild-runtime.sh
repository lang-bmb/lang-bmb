#!/bin/bash
# Rebuild bmb/runtime/libbmb_runtime.a when bmb_runtime.c or bmb_event_loop.c changes.
#
# Root cause prevented: stale libbmb_runtime.a causes new built-in functions to
# be missing at link time, producing "undefined reference to bmb_xxx" errors.
# Running cargo build or bmb build manually doesn't rebuild the C runtime — this
# script fills that gap.
#
# Usage:
#   ./scripts/rebuild-runtime.sh [--force] [--json] [--check-only | --ci]
#
# Options:
#   --force       Rebuild even if timestamp is current
#   --json        Machine-readable output
#   --check-only  Exit 1 if stale (no rebuild); also accepts --ci
#
# Exit codes:
#   0  library is current (or was successfully rebuilt)
#   1  --check-only/--ci and library is stale, OR rebuild failed

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

RUNTIME_DIR="${PROJECT_ROOT}/bmb/runtime"
RUNTIME_C="${RUNTIME_DIR}/bmb_runtime.c"
EVENTLOOP_C="${RUNTIME_DIR}/bmb_event_loop.c"
LIB_PRIMARY="${RUNTIME_DIR}/libbmb_runtime.a"
# Second copy used by some scripts / gotgan package manager
LIB_SECONDARY="${PROJECT_ROOT}/runtime/libbmb_runtime.a"

FORCE=false
JSON_OUTPUT=false
CHECK_ONLY=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --force)                 FORCE=true;       shift ;;
        --json)                  JSON_OUTPUT=true;  shift ;;
        --check-only | --ci)     CHECK_ONLY=true;   shift ;;
        *) echo "Unknown option: $1" >&2; exit 1 ;;
    esac
done

log() { [ "$JSON_OUTPUT" = false ] && echo -e "$1" || true; }

# ── compiler selection ───────────────────────────────────────────────────────
pick_cc() {
    if command -v clang &>/dev/null; then echo "clang"
    elif command -v gcc &>/dev/null; then echo "gcc"
    else echo ""; fi
}

# ── staleness check ──────────────────────────────────────────────────────────
is_stale() {
    [ ! -f "$LIB_PRIMARY" ] && return 0
    [ "$FORCE" = true ]      && return 0
    [ "$RUNTIME_C"  -nt "$LIB_PRIMARY" ] && return 0
    [ "$EVENTLOOP_C" -nt "$LIB_PRIMARY" ] && return 0
    return 1
}

# ── main ─────────────────────────────────────────────────────────────────────
if ! is_stale; then
    SIZE=$(wc -c < "$LIB_PRIMARY" 2>/dev/null || echo 0)
    if [ "$JSON_OUTPUT" = true ]; then
        echo "{\"status\":\"current\",\"rebuilt\":false,\"lib\":\"${LIB_PRIMARY}\",\"bytes\":${SIZE}}"
    else
        log "bmb/runtime/libbmb_runtime.a is current (${SIZE} bytes)"
    fi
    exit 0
fi

if [ "$CHECK_ONLY" = true ]; then
    if [ "$JSON_OUTPUT" = true ]; then
        echo '{"status":"stale","rebuilt":false}'
    else
        log "ERROR: libbmb_runtime.a is stale vs bmb_runtime.c / bmb_event_loop.c" >&2
        log "  Run: ./scripts/rebuild-runtime.sh" >&2
    fi
    exit 1
fi

CC=$(pick_cc)
if [ -z "$CC" ]; then
    if [ "$JSON_OUTPUT" = true ]; then
        echo '{"status":"error","message":"No C compiler found (clang or gcc required)","rebuilt":false}'
    else
        log "ERROR: No C compiler found. Install clang or gcc." >&2
    fi
    exit 1
fi

log "libbmb_runtime.a is stale — rebuilding with ${CC} ..."
BUILD_START=$(date +%s%3N 2>/dev/null || python3 -c 'import time; print(int(time.time()*1000))')

CFLAGS="-O2 -ffunction-sections -fdata-sections -I${RUNTIME_DIR}"

OBJ_RUNTIME="${RUNTIME_DIR}/bmb_runtime.o"
OBJ_EVENTLOOP="${RUNTIME_DIR}/bmb_event_loop.o"

if "$CC" -c "$RUNTIME_C"  -o "$OBJ_RUNTIME"   $CFLAGS 2>&1 \
&& "$CC" -c "$EVENTLOOP_C" -o "$OBJ_EVENTLOOP" $CFLAGS 2>&1 \
&& ar rcs "$LIB_PRIMARY" "$OBJ_RUNTIME" "$OBJ_EVENTLOOP" 2>&1; then

    # Sync secondary copy if directory exists
    if [ -d "$(dirname "$LIB_SECONDARY")" ]; then
        cp "$LIB_PRIMARY" "$LIB_SECONDARY"
        log "  → synced to runtime/libbmb_runtime.a"
    fi

    # Clean up object files
    rm -f "$OBJ_RUNTIME" "$OBJ_EVENTLOOP"

    BUILD_END=$(date +%s%3N 2>/dev/null || python3 -c 'import time; print(int(time.time()*1000))')
    BUILD_MS=$((BUILD_END - BUILD_START))
    SIZE=$(wc -c < "$LIB_PRIMARY" 2>/dev/null || echo 0)

    if [ "$JSON_OUTPUT" = true ]; then
        echo "{\"status\":\"ok\",\"rebuilt\":true,\"compiler\":\"${CC}\",\"build_ms\":${BUILD_MS},\"bytes\":${SIZE}}"
    else
        log "OK: libbmb_runtime.a rebuilt in ${BUILD_MS} ms (${SIZE} bytes, compiler: ${CC})"
    fi
else
    rm -f "$OBJ_RUNTIME" "$OBJ_EVENTLOOP"
    if [ "$JSON_OUTPUT" = true ]; then
        echo "{\"status\":\"error\",\"message\":\"compile failed\",\"rebuilt\":false}"
    else
        log "ERROR: Failed to rebuild libbmb_runtime.a" >&2
    fi
    exit 1
fi
