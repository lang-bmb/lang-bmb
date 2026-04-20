#!/bin/bash
# Cycle 2359: E2E smoke test for stdlib/net TCP echo round-trip.
#
# Builds tests/bench/net_echo_server.bmb with the Stage 1 bootstrap binary,
# runs it in the background (single-shot server), then uses a Python client
# to connect to 127.0.0.1:18765, send a payload, receive the echo, and
# verify equality byte-for-byte.
#
# Exit 0 if echo round-trip succeeds. Non-zero on any failure.
#
# Usage:
#   ./scripts/test-net-echo.sh
#
# Environment:
#   BMB  - Override BMB compiler binary (default: auto-detect bootstrap/release)
#   PORT - Override server port (default: 18765; must match net_echo_server.bmb)

set -u

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

if [ -n "${BMB:-}" ]; then
    :
elif [ -f "${PROJECT_ROOT}/target/bootstrap/bmb-stage1.exe" ]; then
    BMB="${PROJECT_ROOT}/target/bootstrap/bmb-stage1.exe"
elif [ -f "${PROJECT_ROOT}/target/bootstrap/bmb-stage1" ]; then
    BMB="${PROJECT_ROOT}/target/bootstrap/bmb-stage1"
elif [ -f "${PROJECT_ROOT}/target/release/bmb.exe" ]; then
    BMB="${PROJECT_ROOT}/target/release/bmb.exe"
elif [ -f "${PROJECT_ROOT}/target/release/bmb" ]; then
    BMB="${PROJECT_ROOT}/target/release/bmb"
else
    echo "FAIL: bmb binary not found (bootstrap or release)"
    exit 1
fi

PORT="${PORT:-18765}"
SRC="${PROJECT_ROOT}/tests/bench/net_echo_server.bmb"
TMPDIR="$(mktemp -d)"
SERVER_BIN="${TMPDIR}/net_echo_server.exe"
SERVER_LOG="${TMPDIR}/server.log"
SERVER_PID=""

cleanup() {
    if [ -n "$SERVER_PID" ] && kill -0 "$SERVER_PID" 2>/dev/null; then
        kill "$SERVER_PID" 2>/dev/null || true
        wait "$SERVER_PID" 2>/dev/null || true
    fi
    rm -rf "$TMPDIR"
}
trap cleanup EXIT

python_cmd=""
if command -v python3 >/dev/null 2>&1; then
    python_cmd="python3"
elif command -v python >/dev/null 2>&1; then
    python_cmd="python"
else
    echo "FAIL: python not found"
    exit 1
fi

echo "[1/3] Building echo server..."
if ! "$BMB" build "$SRC" -o "$SERVER_BIN" >"$TMPDIR/build.log" 2>&1; then
    echo "FAIL: build failed"
    cat "$TMPDIR/build.log"
    exit 1
fi

echo "[2/3] Starting server on 127.0.0.1:${PORT}..."
"$SERVER_BIN" >"$SERVER_LOG" 2>&1 &
SERVER_PID=$!

# The server is single-shot (accept exactly one client).
# Run a client with retry: server may need up to 2s to bind in cold start.
# Payload is 2000 bytes of repeating alphabet (no NUL bytes — runtime uses
# strlen on write, so embedded NULs would truncate). Exercises the recv
# fragmentation loop on both ends.
echo "[3/3] Running echo client..."
"$python_cmd" - "$PORT" <<'PYEOF'
import socket, sys, time, string
port = int(sys.argv[1])
alphabet = (string.ascii_letters + string.digits).encode("ascii")
payload = (alphabet * 40)[:2000]  # 2000 bytes, fits runtime 4KB recv buffer
assert b"\x00" not in payload, "payload must not contain NUL"
deadline = time.time() + 5.0
last_err = None
while time.time() < deadline:
    try:
        s = socket.create_connection(("127.0.0.1", port), timeout=2.0)
        s.settimeout(2.0)
        s.sendall(payload)
        # Receive up to the echoed length. Server writes content handle which is
        # null-terminated at recv_len in the runtime; should match exactly.
        buf = b""
        while len(buf) < len(payload):
            chunk = s.recv(4096)
            if not chunk:
                break
            buf += chunk
        s.close()
        if buf == payload:
            print(f"PASS echo round-trip: {len(buf)} bytes matched")
            sys.exit(0)
        else:
            print(f"FAIL echo mismatch: sent={payload!r} recv={buf!r}")
            sys.exit(2)
    except (ConnectionRefusedError, socket.timeout, OSError) as e:
        last_err = e
        time.sleep(0.15)

print(f"FAIL: could not reach server within 5s (last error: {last_err})")
sys.exit(1)
PYEOF

CLIENT_EXIT=$?

# Give server a moment to exit cleanly after write+close
wait "$SERVER_PID" 2>/dev/null
SERVER_EXIT=$?
SERVER_PID=""

echo "--- server log ---"
if [ -s "$SERVER_LOG" ]; then
    cat "$SERVER_LOG"
else
    echo "(empty)"
fi
echo "--- server exit: $SERVER_EXIT ---"
echo "--- client exit: $CLIENT_EXIT ---"

if [ "$CLIENT_EXIT" -eq 0 ] && [ "$SERVER_EXIT" -eq 0 ]; then
    echo ""
    echo "PASS: net echo round-trip verified"
    exit 0
else
    echo ""
    echo "FAIL: client=$CLIENT_EXIT server=$SERVER_EXIT"
    exit 1
fi
