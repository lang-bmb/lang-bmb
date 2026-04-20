#!/bin/bash
# Cycle 2371-2372: Full E2E UDP echo round-trip for stdlib/net.
#
# The BMB server (tests/bench/net_udp_echo_server.bmb) binds 127.0.0.1:18766,
# receives one datagram, echoes the payload back to 127.0.0.1:18767, exits 0.
# The Python client binds 18767, sends to 18766, recvs the echo, verifies
# byte-exact match.

set -u

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

if [ -n "${BMB:-}" ]; then
    :
elif [ -f "${PROJECT_ROOT}/target/bootstrap/bmb-stage1.exe" ]; then
    BMB="${PROJECT_ROOT}/target/bootstrap/bmb-stage1.exe"
elif [ -f "${PROJECT_ROOT}/target/bootstrap/bmb-stage1" ]; then
    BMB="${PROJECT_ROOT}/target/bootstrap/bmb-stage1"
else
    echo "FAIL: bmb stage1 binary not found"
    exit 1
fi

SERVER_PORT="${SERVER_PORT:-18766}"
CLIENT_PORT="${CLIENT_PORT:-18767}"
SRC="${PROJECT_ROOT}/tests/bench/net_udp_echo_server.bmb"
TMPDIR="$(mktemp -d)"
SERVER_BIN="${TMPDIR}/net_udp_echo.exe"
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
if command -v python3 >/dev/null 2>&1; then python_cmd="python3"
elif command -v python >/dev/null 2>&1; then python_cmd="python"
else echo "FAIL: python not found"; exit 1
fi

echo "[1/3] Building UDP echo server..."
if ! "$BMB" build "$SRC" -o "$SERVER_BIN" >"$TMPDIR/build.log" 2>&1; then
    echo "FAIL: build"
    cat "$TMPDIR/build.log"
    exit 1
fi

echo "[2/3] Starting server on 127.0.0.1:${SERVER_PORT}..."
"$SERVER_BIN" >"$SERVER_LOG" 2>&1 &
SERVER_PID=$!

echo "[3/3] Running UDP echo client..."
"$python_cmd" - "$SERVER_PORT" "$CLIENT_PORT" <<'PYEOF'
import socket, sys, time
server_port = int(sys.argv[1])
client_port = int(sys.argv[2])
payload = b"bmb udp echo 2371 full round-trip"

c = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
c.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
c.bind(("127.0.0.1", client_port))

# Give server a moment to bind
time.sleep(0.5)
c.settimeout(3.0)

for attempt in range(10):
    try:
        c.sendto(payload, ("127.0.0.1", server_port))
    except (ConnectionResetError, OSError):
        time.sleep(0.1)
        continue
    try:
        data, addr = c.recvfrom(4096)
    except socket.timeout:
        continue
    except ConnectionResetError:
        time.sleep(0.1)
        continue
    c.close()
    if data == payload:
        print(f"PASS udp echo round-trip: {len(data)} bytes matched (from {addr}, attempt {attempt+1})")
        sys.exit(0)
    else:
        print(f"FAIL udp echo mismatch: sent={payload!r} recv={data!r}")
        sys.exit(3)

c.close()
print("FAIL: no echo received after 10 attempts")
sys.exit(2)
PYEOF

CLIENT_EXIT=$?
wait "$SERVER_PID" 2>/dev/null
SERVER_EXIT=$?
SERVER_PID=""

echo "--- server log ---"
[ -s "$SERVER_LOG" ] && cat "$SERVER_LOG" || echo "(empty)"
echo "--- server exit: $SERVER_EXIT, client exit: $CLIENT_EXIT ---"

if [ "$CLIENT_EXIT" -eq 0 ] && [ "$SERVER_EXIT" -eq 0 ]; then
    echo ""
    echo "PASS: UDP full echo round-trip verified"
    exit 0
else
    echo ""
    echo "FAIL: client=$CLIENT_EXIT server=$SERVER_EXIT"
    exit 1
fi
