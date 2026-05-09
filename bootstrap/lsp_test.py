#!/usr/bin/env python3
"""
bootstrap/lsp_test.py — LSP roundtrip tests for bootstrap/lsp.exe
Cycle 2594, Track S

Usage:
    python3 bootstrap/lsp_test.py                   # uses ./target/release/bmb.exe
    BMB_PATH=/path/to/bmb python3 bootstrap/lsp_test.py

Requirements:
    - bootstrap/lsp.exe must be built:
        ./target/release/bmb build bootstrap/lsp.bmb -o bootstrap/lsp.exe
    - BMB binary for diagnostics (default: ./target/release/bmb.exe)
"""

import subprocess, json, os, sys, tempfile

# Resolve paths
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
ROOT_DIR = os.path.dirname(SCRIPT_DIR)
LSP_BIN = os.path.join(SCRIPT_DIR, "lsp.exe")
DEFAULT_BMB = os.path.join(ROOT_DIR, "target", "release", "bmb.exe")
BMB_PATH = os.environ.get("BMB_PATH", DEFAULT_BMB)


def make_msg(obj: dict) -> bytes:
    body = json.dumps(obj, separators=(",", ":")).encode("utf-8")
    return ("Content-Length: " + str(len(body)) + "\r\n\r\n").encode("utf-8") + body


def parse_responses(stdout_bytes: bytes) -> list:
    responses = []
    pos = 0
    cl_tag = b"Content-Length: "
    while pos < len(stdout_bytes):
        cl_start = stdout_bytes.find(cl_tag, pos)
        if cl_start < 0:
            break
        sep = stdout_bytes.find(b"\r\n\r\n", cl_start)
        if sep < 0:
            break
        cl = int(stdout_bytes[cl_start + len(cl_tag) : sep])
        body_start = sep + 4
        body_bytes = stdout_bytes[body_start : body_start + cl]
        responses.append(json.loads(body_bytes.decode("utf-8")))
        pos = body_start + cl
    return responses


def run_lsp(msgs: bytes) -> list:
    env = os.environ.copy()
    env["BMB_PATH"] = BMB_PATH
    proc = subprocess.run(
        [LSP_BIN], input=msgs, capture_output=True, timeout=30, env=env
    )
    if proc.returncode not in (0, 1):
        print("STDERR:", proc.stderr.decode("utf-8", errors="replace")[:500])
    return parse_responses(proc.stdout)


# ============================================================
# Test cases
# ============================================================

tests_passed = 0
tests_failed = 0


def test(name: str, condition: bool, detail: str = ""):
    global tests_passed, tests_failed
    if condition:
        print("  PASS:", name)
        tests_passed += 1
    else:
        print("  FAIL:", name, "|", detail)
        tests_failed += 1


def test_initialize():
    print("\n--- Test: initialize ---")
    msgs = (
        make_msg({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"capabilities": {}}})
        + make_msg({"jsonrpc": "2.0", "method": "exit"})
    )
    responses = run_lsp(msgs)
    init_resp = next((r for r in responses if r.get("id") == 1), None)
    test("initialize response received", init_resp is not None)
    if init_resp:
        caps = init_resp.get("result", {}).get("capabilities", {})
        test("textDocumentSync capability", "textDocumentSync" in caps)
        test("hoverProvider capability", caps.get("hoverProvider") is True)
        test("completionProvider capability", "completionProvider" in caps)
        info = init_resp.get("result", {}).get("serverInfo", {})
        test("serverInfo.name = bmb-lsp", info.get("name") == "bmb-lsp")


def test_completion():
    print("\n--- Test: textDocument/completion ---")
    msgs = (
        make_msg({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"capabilities": {}}})
        + make_msg({"jsonrpc": "2.0", "method": "initialized", "params": {}})
        + make_msg({"jsonrpc": "2.0", "id": 2, "method": "textDocument/completion",
                    "params": {"textDocument": {"uri": "file:///test.bmb"}, "position": {"line": 0, "character": 0}}})
        + make_msg({"jsonrpc": "2.0", "id": 3, "method": "shutdown", "params": {}})
        + make_msg({"jsonrpc": "2.0", "method": "exit"})
    )
    responses = run_lsp(msgs)
    comp_resp = next((r for r in responses if r.get("id") == 2), None)
    test("completion response received", comp_resp is not None)
    if comp_resp:
        result = comp_resp.get("result", {})
        items = result.get("items", [])
        test("isIncomplete = false", result.get("isIncomplete") is False)
        test("items count >= 10", len(items) >= 10, f"got {len(items)}")
        labels = [i["label"] for i in items]
        test("fn keyword in completions", "fn" in labels)
        test("let keyword in completions", "let" in labels)
        test("println builtin in completions", "println" in labels)
        test("pre keyword in completions", "pre" in labels)


def test_hover():
    print("\n--- Test: textDocument/hover ---")
    with tempfile.NamedTemporaryFile(suffix=".bmb", mode="w", delete=False) as f:
        f.write("fn main() -> i64 = 0;\n")
        tmp_path = f.name
    try:
        uri = "file:///" + tmp_path.replace("\\", "/")
        msgs = (
            make_msg({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"capabilities": {}}})
            + make_msg({"jsonrpc": "2.0", "method": "initialized", "params": {}})
            + make_msg({"jsonrpc": "2.0", "id": 2, "method": "textDocument/hover",
                        "params": {"textDocument": {"uri": uri}, "position": {"line": 0, "character": 0}}})
            + make_msg({"jsonrpc": "2.0", "id": 3, "method": "textDocument/hover",
                        "params": {"textDocument": {"uri": uri}, "position": {"line": 0, "character": 100}}})
            + make_msg({"jsonrpc": "2.0", "id": 4, "method": "shutdown", "params": {}})
            + make_msg({"jsonrpc": "2.0", "method": "exit"})
        )
        responses = run_lsp(msgs)
        hover_fn = next((r for r in responses if r.get("id") == 2), None)
        hover_eof = next((r for r in responses if r.get("id") == 3), None)
        test("hover response received", hover_fn is not None)
        if hover_fn:
            result = hover_fn.get("result")
            test("hover on 'fn' returns content", result is not None and "contents" in result,
                 f"result={result}")
            if result:
                val = result["contents"].get("value", "")
                test("hover content mentions 'fn'", "fn" in val)
        if hover_eof:
            test("hover past EOF returns null", hover_eof.get("result") is None,
                 str(hover_eof.get("result")))
    finally:
        os.unlink(tmp_path)


def test_diagnostics():
    print("\n--- Test: textDocument/publishDiagnostics ---")
    if not os.path.exists(BMB_PATH):
        print("  SKIP: BMB_PATH not found:", BMB_PATH)
        return
    with tempfile.NamedTemporaryFile(suffix=".bmb", mode="w", delete=False) as f:
        # Intentional type error
        f.write('fn main() -> i64 = { let x: i64 = "wrong"; 0 };\n')
        err_path = f.name
    try:
        uri = "file:///" + err_path.replace("\\", "/")
        valid_content = "fn main() -> i64 = 0;\n"
        msgs = (
            make_msg({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"capabilities": {}}})
            + make_msg({"jsonrpc": "2.0", "method": "initialized", "params": {}})
            + make_msg({"jsonrpc": "2.0", "method": "textDocument/didOpen",
                        "params": {"textDocument": {"uri": uri, "languageId": "bmb", "version": 1,
                                                     "text": open(err_path).read()}}})
            + make_msg({"jsonrpc": "2.0", "method": "textDocument/didChange",
                        "params": {"textDocument": {"uri": uri, "version": 2},
                                   "contentChanges": [{"text": valid_content}]}})
            + make_msg({"jsonrpc": "2.0", "id": 2, "method": "shutdown", "params": {}})
            + make_msg({"jsonrpc": "2.0", "method": "exit"})
        )
        responses = run_lsp(msgs)
        diag_msgs = [r for r in responses if r.get("method") == "textDocument/publishDiagnostics"]
        test("two diagnostic notifications received", len(diag_msgs) == 2, f"got {len(diag_msgs)}")
        if len(diag_msgs) >= 1:
            first_diags = diag_msgs[0].get("params", {}).get("diagnostics", [])
            test("first notification has error", len(first_diags) > 0, f"got {len(first_diags)}")
            if first_diags:
                test("error severity = 1", first_diags[0].get("severity") == 1)
                test("error mentions type", "i64" in first_diags[0].get("message", "") or
                     "String" in first_diags[0].get("message", ""),
                     first_diags[0].get("message", ""))
        if len(diag_msgs) >= 2:
            second_diags = diag_msgs[1].get("params", {}).get("diagnostics", [])
            test("second notification clears errors", len(second_diags) == 0, f"got {len(second_diags)}")
    finally:
        os.unlink(err_path)


def test_shutdown():
    print("\n--- Test: shutdown + exit ---")
    msgs = (
        make_msg({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"capabilities": {}}})
        + make_msg({"jsonrpc": "2.0", "id": 2, "method": "shutdown", "params": {}})
        + make_msg({"jsonrpc": "2.0", "method": "exit"})
    )
    responses = run_lsp(msgs)
    shutdown_resp = next((r for r in responses if r.get("id") == 2), None)
    test("shutdown response received", shutdown_resp is not None)
    if shutdown_resp:
        test("shutdown result = null", shutdown_resp.get("result") is None)


def test_unknown_method():
    print("\n--- Test: unknown method ---")
    msgs = (
        make_msg({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"capabilities": {}}})
        + make_msg({"jsonrpc": "2.0", "id": 2, "method": "workspace/nonExistent", "params": {}})
        + make_msg({"jsonrpc": "2.0", "id": 3, "method": "shutdown", "params": {}})
        + make_msg({"jsonrpc": "2.0", "method": "exit"})
    )
    responses = run_lsp(msgs)
    # Unknown method with id should get null response
    unk_resp = next((r for r in responses if r.get("id") == 2), None)
    test("unknown method gets null response", unk_resp is not None and unk_resp.get("result") is None,
         str(unk_resp))


def test_document_symbols():
    print("\n--- Test: textDocument/documentSymbol ---")
    content = (
        "fn main() -> i64 = 0;\n"
        "fn helper(x: i64) -> i64 = x + 1;\n"
        "struct Point { x: i64, y: i64 }\n"
        "enum Color { Red, Green, Blue }\n"
        "pub fn pub_fn() -> i64 = 42;\n"
    )
    with tempfile.NamedTemporaryFile(suffix=".bmb", mode="w", delete=False) as f:
        f.write(content)
        tmp_path = f.name
    try:
        uri = "file:///" + tmp_path.replace("\\", "/")
        msgs = (
            make_msg({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"capabilities": {}}})
            + make_msg({"jsonrpc": "2.0", "method": "initialized", "params": {}})
            + make_msg({"jsonrpc": "2.0", "id": 2, "method": "textDocument/documentSymbol",
                        "params": {"textDocument": {"uri": uri}}})
            + make_msg({"jsonrpc": "2.0", "id": 3, "method": "shutdown", "params": {}})
            + make_msg({"jsonrpc": "2.0", "method": "exit"})
        )
        responses = run_lsp(msgs)
        sym_resp = next((r for r in responses if r.get("id") == 2), None)
        test("documentSymbol response received", sym_resp is not None)
        if sym_resp:
            syms = sym_resp.get("result", [])
            test("5 symbols found", len(syms) == 5, f"got {len(syms)}")
            names = [s["name"] for s in syms]
            kinds = {s["name"]: s["kind"] for s in syms}
            test("fn main found", "main" in names)
            test("fn helper found", "helper" in names)
            test("struct Point found", "Point" in names)
            test("enum Color found", "Color" in names)
            test("pub fn pub_fn found", "pub_fn" in names)
            if "main" in kinds:
                test("main kind = Function (12)", kinds["main"] == 12)
            if "Point" in kinds:
                test("Point kind = Struct (23)", kinds["Point"] == 23)
            if "Color" in kinds:
                test("Color kind = Enum (10)", kinds["Color"] == 10)
    finally:
        os.unlink(tmp_path)


def test_definition():
    print("\n--- Test: textDocument/definition ---")
    content = (
        "fn main() -> i64 = 0;\n"
        "fn helper(x: i64) -> i64 = x + 1;\n"
        "struct Point { x: i64, y: i64 }\n"
    )
    with tempfile.NamedTemporaryFile(suffix=".bmb", mode="w", delete=False) as f:
        f.write(content)
        tmp_path = f.name
    try:
        uri = "file:///" + tmp_path.replace("\\", "/")
        msgs = (
            make_msg({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"capabilities": {}}})
            + make_msg({"jsonrpc": "2.0", "method": "initialized", "params": {}})
            # cursor on 'main' (line 0, char 3 = inside 'main')
            + make_msg({"jsonrpc": "2.0", "id": 2, "method": "textDocument/definition",
                        "params": {"textDocument": {"uri": uri}, "position": {"line": 0, "character": 3}}})
            # cursor on 'helper' (line 1, char 3)
            + make_msg({"jsonrpc": "2.0", "id": 3, "method": "textDocument/definition",
                        "params": {"textDocument": {"uri": uri}, "position": {"line": 1, "character": 3}}})
            # cursor past EOF
            + make_msg({"jsonrpc": "2.0", "id": 4, "method": "textDocument/definition",
                        "params": {"textDocument": {"uri": uri}, "position": {"line": 100, "character": 0}}})
            + make_msg({"jsonrpc": "2.0", "id": 5, "method": "shutdown", "params": {}})
            + make_msg({"jsonrpc": "2.0", "method": "exit"})
        )
        responses = run_lsp(msgs)
        def_main = next((r for r in responses if r.get("id") == 2), None)
        def_helper = next((r for r in responses if r.get("id") == 3), None)
        def_eof = next((r for r in responses if r.get("id") == 4), None)
        test("definition response received", def_main is not None)
        if def_main:
            result = def_main.get("result")
            test("definition for 'main' returns location", result is not None,
                 f"result={result}")
            if result:
                start = result.get("range", {}).get("start", {})
                test("definition 'main' at line 0", start.get("line") == 0, str(start))
                test("definition 'main' at col 3", start.get("character") == 3, str(start))
        if def_helper:
            result = def_helper.get("result")
            test("definition for 'helper' returns location", result is not None,
                 f"result={result}")
            if result:
                start = result.get("range", {}).get("start", {})
                test("definition 'helper' at line 1", start.get("line") == 1, str(start))
        if def_eof:
            test("definition past EOF returns null", def_eof.get("result") is None,
                 str(def_eof.get("result")))
    finally:
        os.unlink(tmp_path)


# ============================================================
# Main
# ============================================================

if __name__ == "__main__":
    if not os.path.exists(LSP_BIN):
        print("ERROR: LSP binary not found:", LSP_BIN)
        print("Run: ./target/release/bmb build bootstrap/lsp.bmb -o bootstrap/lsp.exe")
        sys.exit(1)

    print("BMB LSP Bootstrap — Roundtrip Test Suite")
    print("LSP binary:", LSP_BIN)
    print("BMB binary:", BMB_PATH)

    test_initialize()
    test_completion()
    test_hover()
    test_diagnostics()
    test_shutdown()
    test_unknown_method()
    test_document_symbols()
    test_definition()

    print()
    print(f"Results: {tests_passed} passed, {tests_failed} failed")
    sys.exit(0 if tests_failed == 0 else 1)
