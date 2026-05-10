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
    content = (
        "fn helper(x: i64, y: i64) -> i64 = x + y;\n"
        "fn main() -> i64 = helper(1, 2);\n"
    )
    with tempfile.NamedTemporaryFile(suffix=".bmb", mode="w", delete=False) as f:
        f.write(content)
        tmp_path = f.name
    try:
        uri = "file:///" + tmp_path.replace("\\", "/")
        msgs = (
            make_msg({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"capabilities": {}}})
            + make_msg({"jsonrpc": "2.0", "method": "initialized", "params": {}})
            # hover on 'fn' keyword (line 0, char 0)
            + make_msg({"jsonrpc": "2.0", "id": 2, "method": "textDocument/hover",
                        "params": {"textDocument": {"uri": uri}, "position": {"line": 0, "character": 0}}})
            # hover past EOF
            + make_msg({"jsonrpc": "2.0", "id": 3, "method": "textDocument/hover",
                        "params": {"textDocument": {"uri": uri}, "position": {"line": 0, "character": 100}}})
            # hover on 'helper' call site (line 1, char 21 = 'h' in helper(...))
            + make_msg({"jsonrpc": "2.0", "id": 4, "method": "textDocument/hover",
                        "params": {"textDocument": {"uri": uri}, "position": {"line": 1, "character": 21}}})
            + make_msg({"jsonrpc": "2.0", "id": 5, "method": "shutdown", "params": {}})
            + make_msg({"jsonrpc": "2.0", "method": "exit"})
        )
        responses = run_lsp(msgs)
        hover_fn = next((r for r in responses if r.get("id") == 2), None)
        hover_eof = next((r for r in responses if r.get("id") == 3), None)
        hover_user = next((r for r in responses if r.get("id") == 4), None)
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
        test("hover on user-defined fn response received", hover_user is not None)
        if hover_user:
            result = hover_user.get("result")
            test("hover on 'helper' returns content", result is not None,
                 str(result))
            if result:
                val = result.get("contents", {}).get("value", "")
                test("hover shows 'helper' signature", "helper" in val, val[:80])
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


def test_references():
    print("\n--- Test: textDocument/references ---")
    content = (
        "fn main() -> i64 = helper();\n"
        "fn helper() -> i64 = 42;\n"
        "fn other() -> i64 = helper() + helper();\n"
    )
    with tempfile.NamedTemporaryFile(suffix=".bmb", mode="w", delete=False) as f:
        f.write(content)
        tmp_path = f.name
    try:
        uri = "file:///" + tmp_path.replace("\\", "/")
        msgs = (
            make_msg({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"capabilities": {}}})
            + make_msg({"jsonrpc": "2.0", "method": "initialized", "params": {}})
            # Find all references to 'helper' (appears 4 times: 1 decl + 3 usages)
            + make_msg({"jsonrpc": "2.0", "id": 2, "method": "textDocument/references",
                        "params": {"textDocument": {"uri": uri},
                                   "position": {"line": 1, "character": 3},
                                   "context": {"includeDeclaration": True}}})
            # Find all references to 'main' (appears 1 time: decl only)
            + make_msg({"jsonrpc": "2.0", "id": 3, "method": "textDocument/references",
                        "params": {"textDocument": {"uri": uri},
                                   "position": {"line": 0, "character": 3},
                                   "context": {"includeDeclaration": True}}})
            + make_msg({"jsonrpc": "2.0", "id": 4, "method": "shutdown", "params": {}})
            + make_msg({"jsonrpc": "2.0", "method": "exit"})
        )
        responses = run_lsp(msgs)
        refs_helper = next((r for r in responses if r.get("id") == 2), None)
        refs_main = next((r for r in responses if r.get("id") == 3), None)
        test("references response received", refs_helper is not None)
        if refs_helper:
            result = refs_helper.get("result", [])
            test("helper references count >= 4", len(result) >= 4,
                 f"got {len(result)}")
            if result:
                test("all references in same uri", all(r.get("uri") == uri for r in result),
                     str([r.get("uri") for r in result[:3]]))
        if refs_main:
            result = refs_main.get("result", [])
            test("main references count = 1", len(result) == 1, f"got {len(result)}")
    finally:
        os.unlink(tmp_path)


def test_workspace_symbol():
    print("\n--- Test: workspace/symbol ---")
    # Create a temp workspace directory with two .bmb files
    with tempfile.TemporaryDirectory() as ws_dir:
        # File 1: defines foo, bar
        file1 = os.path.join(ws_dir, "file1.bmb")
        with open(file1, "w") as f:
            f.write("fn foo() -> i64 = 1;\nfn bar() -> i64 = 2;\n")
        # File 2: defines baz, qux
        file2 = os.path.join(ws_dir, "file2.bmb")
        with open(file2, "w") as f:
            f.write("fn baz() -> i64 = 3;\nfn qux() -> i64 = 4;\n")

        env = os.environ.copy()
        env["BMB_PATH"] = BMB_PATH
        env["BMB_WORKSPACE"] = ws_dir

        # Test 1: empty query returns all symbols
        msgs = (
            make_msg({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"capabilities": {}}})
            + make_msg({"jsonrpc": "2.0", "method": "initialized", "params": {}})
            + make_msg({"jsonrpc": "2.0", "id": 2, "method": "workspace/symbol",
                        "params": {"query": ""}})
            + make_msg({"jsonrpc": "2.0", "id": 3, "method": "workspace/symbol",
                        "params": {"query": "ba"}})
            + make_msg({"jsonrpc": "2.0", "id": 4, "method": "shutdown", "params": {}})
            + make_msg({"jsonrpc": "2.0", "method": "exit"})
        )
        proc = subprocess.run(
            [LSP_BIN], input=msgs, capture_output=True, timeout=30, env=env
        )
        responses = parse_responses(proc.stdout)

        caps_resp = next((r for r in responses if r.get("id") == 1), None)
        all_syms = next((r for r in responses if r.get("id") == 2), None)
        ba_syms = next((r for r in responses if r.get("id") == 3), None)

        test("workspaceSymbolProvider in capabilities",
             caps_resp is not None and
             caps_resp.get("result", {}).get("capabilities", {}).get("workspaceSymbolProvider") is True)
        test("workspace/symbol empty query returns list", all_syms is not None)
        if all_syms:
            result = all_syms.get("result", [])
            test("workspace/symbol empty query >= 4 symbols", len(result) >= 4,
                 f"got {len(result)}")
            names = [s.get("name") for s in result]
            test("foo symbol present", "foo" in names, str(names))
            test("baz symbol present", "baz" in names, str(names))
            if result:
                loc = result[0].get("location", {})
                test("symbol has uri", "uri" in loc, str(list(loc.keys())))
        test("workspace/symbol query 'ba' returns results", ba_syms is not None)
        if ba_syms:
            result = ba_syms.get("result", [])
            names = [s.get("name") for s in result]
            test("query 'ba' matches bar", "bar" in names, str(names))
            test("query 'ba' matches baz", "baz" in names, str(names))
            test("query 'ba' excludes foo", "foo" not in names, str(names))


def test_signature_help():
    print("\n--- Test: textDocument/signatureHelp ---")
    # Line 0: fn add(x: i64, y: i64) -> i64 = x + y;
    # Line 1: fn main() -> i64 = add(1, 2);
    # In "add(1, 2)":
    #   char 22 = '(' (call open), char 23 = '1', char 24 = ',', char 26 = '2'
    content = (
        "fn add(x: i64, y: i64) -> i64 = x + y;\n"
        "fn main() -> i64 = add(1, 2);\n"
    )
    with tempfile.NamedTemporaryFile(suffix=".bmb", mode="w", delete=False) as f:
        f.write(content)
        tmp_path = f.name
    try:
        uri = "file:///" + tmp_path.replace("\\", "/")
        msgs = (
            make_msg({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"capabilities": {}}})
            + make_msg({"jsonrpc": "2.0", "method": "initialized", "params": {}})
            # cursor at first argument (char 23 = '1') -> activeParameter 0
            + make_msg({"jsonrpc": "2.0", "id": 2, "method": "textDocument/signatureHelp",
                        "params": {"textDocument": {"uri": uri}, "position": {"line": 1, "character": 23}}})
            # cursor at second argument (char 26 = '2') -> activeParameter 1
            + make_msg({"jsonrpc": "2.0", "id": 3, "method": "textDocument/signatureHelp",
                        "params": {"textDocument": {"uri": uri}, "position": {"line": 1, "character": 26}}})
            # cursor outside call (line 0, inside fn body) -> null
            + make_msg({"jsonrpc": "2.0", "id": 4, "method": "textDocument/signatureHelp",
                        "params": {"textDocument": {"uri": uri}, "position": {"line": 0, "character": 35}}})
            + make_msg({"jsonrpc": "2.0", "id": 5, "method": "shutdown", "params": {}})
            + make_msg({"jsonrpc": "2.0", "method": "exit"})
        )
        responses = run_lsp(msgs)
        caps_resp = next((r for r in responses if r.get("id") == 1), None)
        sh_first = next((r for r in responses if r.get("id") == 2), None)
        sh_second = next((r for r in responses if r.get("id") == 3), None)
        sh_outside = next((r for r in responses if r.get("id") == 4), None)

        test("signatureHelpProvider in capabilities",
             caps_resp is not None and
             "signatureHelpProvider" in caps_resp.get("result", {}).get("capabilities", {}))
        test("signatureHelp response for first param", sh_first is not None)
        if sh_first:
            result = sh_first.get("result")
            test("signatureHelp first param not null", result is not None, str(result))
            if result:
                sigs = result.get("signatures", [])
                test("signatureHelp has 1 signature", len(sigs) == 1, f"got {len(sigs)}")
                if sigs:
                    label = sigs[0].get("label", "")
                    test("signature label contains 'add'", "add" in label, label)
                    params = sigs[0].get("parameters", [])
                    test("signature has 2 parameters", len(params) == 2, f"got {len(params)}")
                test("activeSignature is 0", result.get("activeSignature") == 0)
                test("activeParameter is 0 for first arg", result.get("activeParameter") == 0,
                     f"got {result.get('activeParameter')}")
        if sh_second:
            result = sh_second.get("result")
            test("signatureHelp second param not null", result is not None, str(result))
            if result:
                test("activeParameter is 1 for second arg", result.get("activeParameter") == 1,
                     f"got {result.get('activeParameter')}")
        test("signatureHelp outside call returns null",
             sh_outside is not None and sh_outside.get("result") is None,
             str(sh_outside))
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
    test_references()
    test_workspace_symbol()
    test_signature_help()

    print()
    print(f"Results: {tests_passed} passed, {tests_failed} failed")
    sys.exit(0 if tests_failed == 0 else 1)
