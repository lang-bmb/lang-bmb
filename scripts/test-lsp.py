"""Test BMB LSP server with JSON-RPC messages."""
import subprocess
import sys
import os
import json

PROJECT_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

# Find BMB compiler
bmb = os.path.join(PROJECT_ROOT, "target", "release", "bmb.exe")
if not os.path.exists(bmb):
    bmb = os.path.join(PROJECT_ROOT, "target", "release", "bmb")

stdlib = os.path.join(PROJECT_ROOT, "stdlib")
lsp = os.path.join(PROJECT_ROOT, "bootstrap", "lsp.bmb")

def make_message(obj):
    body = json.dumps(obj)
    header = f"Content-Length: {len(body)}\r\n\r\n"
    return (header + body).encode("utf-8")

def run_lsp(messages):
    """Send messages to LSP server and return raw output."""
    stdin_data = b""
    for msg in messages:
        stdin_data += make_message(msg)

    proc = subprocess.run(
        [bmb, "run", lsp, "--include", stdlib],
        input=stdin_data,
        capture_output=True,
        timeout=30,
    )
    return proc.stdout, proc.stderr

def parse_responses(raw):
    """Parse Content-Length framed responses."""
    responses = []
    pos = 0
    data = raw.decode("utf-8", errors="replace")
    while pos < len(data):
        # Find Content-Length header
        cl_idx = data.find("Content-Length: ", pos)
        if cl_idx < 0:
            break
        end_of_num = data.find("\r\n", cl_idx)
        length = int(data[cl_idx + 16:end_of_num])
        # Skip \r\n\r\n
        body_start = data.find("\r\n\r\n", cl_idx) + 4
        body = data[body_start:body_start + length]
        try:
            responses.append(json.loads(body))
        except json.JSONDecodeError:
            responses.append({"raw": body})
        pos = body_start + length
    return responses

# Test sequence
messages = [
    {"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"capabilities": {}}},
    {"jsonrpc": "2.0", "method": "initialized", "params": {}},
    {"jsonrpc": "2.0", "id": 2, "method": "shutdown"},
    {"jsonrpc": "2.0", "method": "exit"},
]

print(f"Testing BMB LSP: {lsp}")
print(f"Compiler: {bmb}")
print(f"Sending {len(messages)} messages...")
print()

try:
    stdout, stderr = run_lsp(messages)
except subprocess.TimeoutExpired:
    print("TIMEOUT: LSP server did not exit")
    sys.exit(1)

if stderr:
    stderr_text = stderr.decode("utf-8", errors="replace")
    if "error" in stderr_text.lower():
        print(f"STDERR: {stderr_text}")
        sys.exit(1)

responses = parse_responses(stdout)

if not responses:
    print("ERROR: No responses received")
    if stdout:
        print(f"Raw stdout: {stdout[:500]}")
    sys.exit(1)

passed = 0
failed = 0

# Check initialize response
if len(responses) >= 1:
    r = responses[0]
    if r.get("id") == 1 and "result" in r:
        caps = r["result"].get("capabilities", {})
        if caps.get("textDocumentSync") == 1:
            print("  PASS initialize — capabilities received")
            passed += 1
        else:
            print(f"  FAIL initialize — unexpected caps: {caps}")
            failed += 1
    else:
        print(f"  FAIL initialize — unexpected response: {r}")
        failed += 1
else:
    print("  FAIL initialize — no response")
    failed += 1

# Check shutdown response
if len(responses) >= 2:
    r = responses[1]
    if r.get("id") == 2 and r.get("result") is None:
        print("  PASS shutdown — null result")
        passed += 1
    else:
        print(f"  FAIL shutdown — unexpected: {r}")
        failed += 1
else:
    print("  FAIL shutdown — no response")
    failed += 1

print()
print(f"Results: {passed} passed, {failed} failed")
print(f"Responses: {len(responses)}")

if failed > 0:
    print("\nAll responses:")
    for i, r in enumerate(responses):
        print(f"  [{i}] {json.dumps(r, indent=2)[:200]}")
    sys.exit(1)
else:
    print("\nBMB LSP server working!")
