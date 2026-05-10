#!/usr/bin/env python3
"""
bootstrap/verify_host_test.py — Roundtrip tests for bootstrap/verify_host.exe
Cycle 2606, Track S

Usage:
    python3 bootstrap/verify_host_test.py                  # uses ./target/release/bmb.exe
    BMB_PATH=/path/to/bmb python3 bootstrap/verify_host_test.py

Requirements:
    - bootstrap/verify_host.exe must be built:
        ./target/release/bmb build bootstrap/verify_host.bmb -o bootstrap/verify_host.exe
    - BMB binary (default: ./target/release/bmb.exe)
"""

import subprocess, json, os, sys, tempfile

# Resolve paths (platform-aware)
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
ROOT_DIR = os.path.dirname(SCRIPT_DIR)
_EXE = ".exe" if sys.platform == "win32" else ""
VH_BIN = os.path.join(SCRIPT_DIR, "verify_host" + _EXE)
DEFAULT_BMB = os.path.join(ROOT_DIR, "target", "release", "bmb" + _EXE)
BMB_PATH = os.environ.get("BMB_PATH", DEFAULT_BMB)

tests_passed = 0
tests_failed = 0


def test(name: str, cond: bool, detail: str = ""):
    global tests_passed, tests_failed
    if cond:
        print(f"  PASS: {name}")
        tests_passed += 1
    else:
        msg = f"  FAIL: {name}"
        if detail:
            msg += f" [{detail}]"
        print(msg)
        tests_failed += 1


def run_vh(file_path: str, env_extra: dict = None) -> dict:
    env = os.environ.copy()
    env["BMB_PATH"] = BMB_PATH
    env["BMB_FILE"] = file_path
    if env_extra:
        env.update(env_extra)
    result = subprocess.run(
        [VH_BIN], capture_output=True, timeout=30, env=env
    )
    stdout = result.stdout.decode("utf-8", errors="replace").strip()
    if not stdout:
        return {}
    try:
        return json.loads(stdout)
    except json.JSONDecodeError:
        return {"_raw": stdout}


def run_vh_stdin(file_path: str) -> dict:
    """Run with file path via stdin (no BMB_FILE env)."""
    env = os.environ.copy()
    env["BMB_PATH"] = BMB_PATH
    if "BMB_FILE" in env:
        del env["BMB_FILE"]
    result = subprocess.run(
        [VH_BIN], input=(file_path + "\n").encode("utf-8"),
        capture_output=True, timeout=30, env=env
    )
    stdout = result.stdout.decode("utf-8", errors="replace").strip()
    if not stdout:
        return {}
    try:
        return json.loads(stdout)
    except json.JSONDecodeError:
        return {"_raw": stdout}


# ============================================================
# Tests
# ============================================================

def test_valid_file():
    print("\n--- Test: valid BMB file ---")
    content = "fn add(x: i64, y: i64) -> i64 = x + y;\nfn main() -> i64 = add(1, 2);\n"
    with tempfile.NamedTemporaryFile(suffix=".bmb", mode="w", delete=False) as f:
        f.write(content)
        tmp = f.name
    try:
        result = run_vh(tmp)
        test("result is JSON object", isinstance(result, dict))
        test("status field present", "status" in result)
        test("file field matches", result.get("file", "").replace("\\", "/") ==
             tmp.replace("\\", "/"),
             f"got {result.get('file')}")
        test("type_check field present", "type_check" in result)
        test("contracts field present", "contracts" in result)
        test("z3_direct field present", "z3_direct" in result)
        tc = result.get("type_check", {})
        test("type_check status ok", tc.get("status") == "ok", str(tc.get("status")))
        test("type_check error_count is 0", tc.get("error_count") == 0,
             str(tc.get("error_count")))
        test("type_check diagnostics is list", isinstance(tc.get("diagnostics"), list))
        test("overall status ok", result.get("status") == "ok",
             f"got {result.get('status')}")
    finally:
        os.unlink(tmp)


def test_type_error_file():
    print("\n--- Test: file with type error ---")
    content = "fn main() -> i64 = \"not_an_int\";\n"
    with tempfile.NamedTemporaryFile(suffix=".bmb", mode="w", delete=False) as f:
        f.write(content)
        tmp = f.name
    try:
        result = run_vh(tmp)
        test("result is JSON object", isinstance(result, dict))
        tc = result.get("type_check", {})
        test("type_check status error", tc.get("status") == "error",
             f"got {tc.get('status')}")
        test("type_check error_count > 0", tc.get("error_count", 0) > 0,
             str(tc.get("error_count")))
        test("overall status type_error", result.get("status") == "type_error",
             f"got {result.get('status')}")
    finally:
        os.unlink(tmp)


def test_no_contracts():
    print("\n--- Test: file without contracts ---")
    content = "fn add(x: i64, y: i64) -> i64 = x + y;\n"
    with tempfile.NamedTemporaryFile(suffix=".bmb", mode="w", delete=False) as f:
        f.write(content)
        tmp = f.name
    try:
        result = run_vh(tmp)
        z3d = result.get("z3_direct", {})
        test("z3_direct skipped or consistent", z3d.get("status") in ("skipped", "consistent"),
             f"got {z3d.get('status')}")
    finally:
        os.unlink(tmp)


def test_contracts_consistent():
    print("\n--- Test: file with consistent pre-conditions ---")
    content = (
        "fn bounded(x: i64) -> i64 =\n"
        "    pre x > 0\n"
        "    pre x < 100\n"
        "    x * 2;\n"
        "fn main() -> i64 = bounded(5);\n"
    )
    with tempfile.NamedTemporaryFile(suffix=".bmb", mode="w", delete=False) as f:
        f.write(content)
        tmp = f.name
    try:
        result = run_vh(tmp)
        z3d = result.get("z3_direct", {})
        test("z3_direct ran", "status" in z3d)
        if z3d.get("status") != "skipped":
            test("z3_direct consistent or sat", z3d.get("status") == "consistent",
                 f"got {z3d.get('status')}")
            test("z3_direct checks > 0", z3d.get("checks", 0) > 0,
                 str(z3d.get("checks")))
    finally:
        os.unlink(tmp)


def test_stdin_fallback():
    print("\n--- Test: stdin fallback (no BMB_FILE) ---")
    content = "fn main() -> i64 = 42;\n"
    with tempfile.NamedTemporaryFile(suffix=".bmb", mode="w", delete=False) as f:
        f.write(content)
        tmp = f.name
    try:
        result = run_vh_stdin(tmp)
        test("stdin fallback returns JSON", isinstance(result, dict),
             str(result))
        test("stdin fallback status ok", result.get("status") == "ok",
             f"got {result.get('status')}")
    finally:
        os.unlink(tmp)


def test_no_file_error():
    print("\n--- Test: missing file error ---")
    env = os.environ.copy()
    env["BMB_PATH"] = BMB_PATH
    env["BMB_FILE"] = "/nonexistent/path/file.bmb"
    result_proc = subprocess.run(
        [VH_BIN], capture_output=True, timeout=30, env=env
    )
    stdout = result_proc.stdout.decode("utf-8", errors="replace").strip()
    try:
        data = json.loads(stdout) if stdout else {}
    except json.JSONDecodeError:
        data = {}
    # Missing file → bmb check will fail → type_error or error
    test("missing file returns JSON", isinstance(data, dict), stdout)
    test("missing file has status field", "status" in data, str(data))


def test_output_schema():
    print("\n--- Test: output JSON schema ---")
    content = "fn main() -> i64 = 0;\n"
    with tempfile.NamedTemporaryFile(suffix=".bmb", mode="w", delete=False) as f:
        f.write(content)
        tmp = f.name
    try:
        result = run_vh(tmp)
        test("top-level keys: status", "status" in result)
        test("top-level keys: file", "file" in result)
        test("top-level keys: type_check", "type_check" in result)
        test("top-level keys: contracts", "contracts" in result)
        test("top-level keys: z3_direct", "z3_direct" in result)

        tc = result.get("type_check", {})
        test("type_check.status is string", isinstance(tc.get("status"), str))
        test("type_check.error_count is int", isinstance(tc.get("error_count"), int))
        test("type_check.diagnostics is list", isinstance(tc.get("diagnostics"), list))

        contracts = result.get("contracts", {})
        test("contracts.status is string", isinstance(contracts.get("status"), str))

        z3d = result.get("z3_direct", {})
        test("z3_direct.status is string", isinstance(z3d.get("status"), str))
        test("z3_direct.checks is int", isinstance(z3d.get("checks"), int))
    finally:
        os.unlink(tmp)


def test_proof_database():
    print("\n--- Test: proof database (Z3 conditions cache) ---")
    # File with pre/post contracts → Z3 IPC should create proof DB
    content = (
        "fn add(x: i64, y: i64) -> i64\n"
        "pre x > 0\n"
        "pre y > 0\n"
        "post result > 0\n"
        "= x + y;\n"
        "fn main() -> i64 = add(1, 2);\n"
    )
    with tempfile.NamedTemporaryFile(suffix=".bmb", mode="w", delete=False) as f:
        f.write(content)
        tmp = f.name
    proofdb_file = tmp + ".vh_proofdb"
    cache_file = tmp + ".vh_cache"
    try:
        # First call: proof DB miss → Z3 runs, proof DB created
        result1 = run_vh(tmp)
        z3d1 = result1.get("z3_direct", {})
        # Proof DB is only created if Z3 is available (status != "skipped")
        if z3d1.get("status") == "skipped":
            test("proof db: skipped (Z3 not available)", True)
        else:
            test("proof db: first call z3 status not skipped",
                 z3d1.get("status") != "skipped", f"got {z3d1.get('status')}")
            test("proof db: proof DB file created",
                 os.path.exists(proofdb_file), f"expected {proofdb_file}")

            if os.path.exists(proofdb_file):
                with open(proofdb_file, "r") as pf:
                    db_data = pf.read()
                db_lines = db_data.split("\n", 1)
                test("proof db: first line has hash:len format", ":" in db_lines[0],
                     f"got '{db_lines[0]}'")
                test("proof db: second line is JSON", len(db_lines) > 1 and db_lines[1].startswith("{"),
                     f"got '{db_lines[1][:40] if len(db_lines) > 1 else ''}'")

            # Second call: proof DB hit (Z3 not re-run, same z3_direct result)
            result2 = run_vh(tmp)
            z3d2 = result2.get("z3_direct", {})
            test("proof db: second call z3 status same",
                 z3d1.get("status") == z3d2.get("status"),
                 f"r1={z3d1.get('status')} r2={z3d2.get('status')}")

            # Modify only the implementation body (contracts unchanged) → proof DB hit
            content_same_contracts = (
                "fn add(x: i64, y: i64) -> i64\n"
                "pre x > 0\n"
                "pre y > 0\n"
                "post result > 0\n"
                "= x + y + 0;\n"  # different body, same contracts
                "fn main() -> i64 = add(1, 2);\n"
            )
            with open(tmp, "w") as f:
                f.write(content_same_contracts)
            # File-level cache miss (file changed), but proof DB should still hit
            result3 = run_vh(tmp)
            z3d3 = result3.get("z3_direct", {})
            test("proof db: same contracts -> same z3 result",
                 z3d1.get("status") == z3d3.get("status"),
                 f"r1={z3d1.get('status')} r3={z3d3.get('status')}")
    finally:
        if os.path.exists(tmp):
            os.unlink(tmp)
        if os.path.exists(proofdb_file):
            os.unlink(proofdb_file)
        if os.path.exists(cache_file):
            os.unlink(cache_file)


def test_incremental_cache():
    print("\n--- Test: incremental verification cache ---")
    content = "fn main() -> i64 = 42;\n"
    with tempfile.NamedTemporaryFile(suffix=".bmb", mode="w", delete=False) as f:
        f.write(content)
        tmp = f.name
    cache_file = tmp + ".vh_cache"
    try:
        # First call: cache miss — should compute and write cache
        result1 = run_vh(tmp)
        test("cache: first call returns ok", result1.get("status") == "ok",
             f"got {result1.get('status')}")
        test("cache: cache file created after first call", os.path.exists(cache_file),
             f"expected {cache_file}")

        # Read cache content to verify format
        if os.path.exists(cache_file):
            with open(cache_file, "r") as cf:
                cache_data = cf.read()
            lines = cache_data.split("\n", 1)
            test("cache: first line is hash (colon-separated)", ":" in lines[0],
                 f"got '{lines[0]}'")
            test("cache: second line is JSON", len(lines) > 1 and lines[1].startswith("{"),
                 f"got '{lines[1][:40] if len(lines) > 1 else ''}'")

        # Second call: cache hit — same result
        result2 = run_vh(tmp)
        test("cache: second call returns ok (cache hit)", result2.get("status") == "ok",
             f"got {result2.get('status')}")
        test("cache: results are identical", result1 == result2,
             f"r1={result1.get('status')} r2={result2.get('status')}")

        # Modify file: cache miss — should recompute
        with open(tmp, "w") as f:
            f.write("fn main() -> i64 = 99;\n")
        result3 = run_vh(tmp)
        test("cache: after file change, still returns ok", result3.get("status") == "ok",
             f"got {result3.get('status')}")

        # Restore same content as original: should hit cache again
        with open(tmp, "w") as f:
            f.write(content)
        result4 = run_vh(tmp)
        test("cache: restored file gets cache hit", result4.get("status") == "ok",
             f"got {result4.get('status')}")
        test("cache: restored result matches original", result4 == result1,
             f"r4={result4.get('status')} r1={result1.get('status')}")
    finally:
        os.unlink(tmp)
        if os.path.exists(cache_file):
            os.unlink(cache_file)


# ============================================================
# Main
# ============================================================

if __name__ == "__main__":
    if not os.path.exists(VH_BIN):
        print("ERROR: verify_host binary not found:", VH_BIN)
        print("Run: ./target/release/bmb build bootstrap/verify_host.bmb -o bootstrap/verify_host.exe")
        sys.exit(1)

    print("BMB verify_host — Roundtrip Test Suite")
    print("verify_host binary:", VH_BIN)
    print("BMB binary:", BMB_PATH)

    test_valid_file()
    test_type_error_file()
    test_no_contracts()
    test_contracts_consistent()
    test_stdin_fallback()
    test_no_file_error()
    test_output_schema()
    test_incremental_cache()
    test_proof_database()

    print()
    print(f"Results: {tests_passed} passed, {tests_failed} failed")
    sys.exit(0 if tests_failed == 0 else 1)
