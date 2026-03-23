#!/usr/bin/env python3
"""Validate all problem solutions compile and produce correct output.

Iterates every numbered problem directory under problems/, attempts to
compile each solution (BMB, Rust, C), runs the first test case, and
reports pass/fail per solution.
"""
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

_BASE = Path(__file__).resolve().parent.parent
_PROBLEMS_DIR = _BASE / "problems"
_BMB_EXE = Path("D:/data/lang-bmb/target/release/bmb.exe")
_WIN = sys.platform == "win32"

_BUILD_TIMEOUT = 60
_RUN_TIMEOUT = 10


def _run_binary(binary: Path, stdin_data: str) -> tuple[bool, str]:
    """Run a compiled binary with stdin, return (ok, stdout)."""
    try:
        proc = subprocess.run(
            [str(binary)],
            input=stdin_data,
            capture_output=True,
            text=True,
            timeout=_RUN_TIMEOUT,
        )
        if proc.returncode != 0:
            return False, f"runtime error (rc={proc.returncode}): {proc.stderr[:200]}"
        return True, proc.stdout
    except subprocess.TimeoutExpired:
        return False, "timeout"
    except OSError as exc:
        return False, f"exec error: {exc}"


def _check_output(actual: str, expected: str) -> str:
    """Compare actual vs expected stdout. Returns status string."""
    if actual.strip() == expected.strip():
        return "PASS"
    return f"FAIL (expected={expected.strip()!r}, got={actual.strip()!r})"


def validate_bmb(problem_dir: Path, first_test: dict | None) -> str:
    """Build and test solution.bmb. Returns status string."""
    src = problem_dir / "solution.bmb"
    if not src.exists():
        return "SKIP (no solution.bmb)"

    out_name = "solution_bmb.exe" if _WIN else "solution_bmb"
    out_path = problem_dir / out_name

    env = dict(__import__("os").environ)
    env["BMB_RUNTIME_PATH"] = "d:/data/lang-bmb/bmb/runtime"

    try:
        proc = subprocess.run(
            [str(_BMB_EXE), "build", str(src), "-o", str(out_path), "--release"],
            capture_output=True,
            text=True,
            timeout=_BUILD_TIMEOUT,
            env=env,
        )
    except subprocess.TimeoutExpired:
        return "BUILD TIMEOUT"
    except OSError as exc:
        return f"BUILD ERROR: {exc}"

    if proc.returncode != 0:
        return f"BUILD FAIL: {(proc.stderr + proc.stdout).strip()[:200]}"

    if first_test is None:
        return "COMPILED (no tests)"

    ok, output = _run_binary(out_path, first_test.get("stdin", ""))
    if not ok:
        return f"RUN FAIL: {output}"
    return _check_output(output, first_test.get("expected_stdout", ""))


def validate_rust(problem_dir: Path, first_test: dict | None) -> str:
    """Build and test solution.rs. Returns status string."""
    src = problem_dir / "solution.rs"
    if not src.exists():
        return "SKIP (no solution.rs)"

    out_name = "solution_rs.exe" if _WIN else "solution_rs"
    out_path = problem_dir / out_name

    try:
        proc = subprocess.run(
            ["rustc", "--edition", "2021", "-O", str(src), "-o", str(out_path)],
            capture_output=True,
            text=True,
            timeout=_BUILD_TIMEOUT,
        )
    except subprocess.TimeoutExpired:
        return "BUILD TIMEOUT"
    except FileNotFoundError:
        return "SKIP (rustc not found)"
    except OSError as exc:
        return f"BUILD ERROR: {exc}"

    if proc.returncode != 0:
        return f"BUILD FAIL: {(proc.stderr + proc.stdout).strip()[:200]}"

    if first_test is None:
        return "COMPILED (no tests)"

    ok, output = _run_binary(out_path, first_test.get("stdin", ""))
    if not ok:
        return f"RUN FAIL: {output}"
    return _check_output(output, first_test.get("expected_stdout", ""))


def validate_c(problem_dir: Path, first_test: dict | None) -> str:
    """Build and test baseline.c. Returns status string."""
    src = problem_dir / "baseline.c"
    if not src.exists():
        return "SKIP (no baseline.c)"

    out_name = "baseline_c.exe" if _WIN else "baseline_c"
    out_path = problem_dir / out_name

    # Try gcc first, then clang
    for compiler in ["gcc", "clang"]:
        try:
            proc = subprocess.run(
                [compiler, "-O2", str(src), "-o", str(out_path), "-lm"],
                capture_output=True,
                text=True,
                timeout=_BUILD_TIMEOUT,
            )
            if proc.returncode == 0:
                break
        except FileNotFoundError:
            continue
        except subprocess.TimeoutExpired:
            return "BUILD TIMEOUT"
        except OSError:
            continue
    else:
        return "BUILD FAIL (no C compiler found)"

    if proc.returncode != 0:
        return f"BUILD FAIL: {(proc.stderr + proc.stdout).strip()[:200]}"

    if first_test is None:
        return "COMPILED (no tests)"

    ok, output = _run_binary(out_path, first_test.get("stdin", ""))
    if not ok:
        return f"RUN FAIL: {output}"
    return _check_output(output, first_test.get("expected_stdout", ""))


def cleanup(problem_dir: Path) -> None:
    """Remove compiled binaries from the problem directory."""
    patterns = [
        "solution_bmb", "solution_bmb.exe",
        "solution_rs", "solution_rs.exe",
        "baseline_c", "baseline_c.exe",
        # Also clean up .pdb files on Windows
        "solution_bmb.pdb", "solution_rs.pdb", "baseline_c.pdb",
        # LLVM intermediates
        "solution_bmb.ll", "solution_bmb.bc", "solution_bmb.o",
    ]
    for name in patterns:
        p = problem_dir / name
        if p.exists():
            p.unlink()


def main() -> int:
    total = 0
    passed = 0
    failed = 0

    dirs = sorted(
        d for d in _PROBLEMS_DIR.iterdir()
        if d.is_dir() and d.name[0:1].isdigit()
    )

    if not dirs:
        print("No problem directories found.")
        return 1

    for d in dirs:
        print(f"\n=== {d.name} ===")

        tests_path = d / "tests.json"
        if not tests_path.exists():
            print("  (no tests.json)")
            continue

        tests = json.loads(tests_path.read_text(encoding="utf-8"))
        first_test = tests[0] if tests else None

        for label, validator in [
            ("BMB ", validate_bmb),
            ("Rust", validate_rust),
            ("C   ", validate_c),
        ]:
            total += 1
            status = validator(d, first_test)
            is_pass = status == "PASS"
            if is_pass:
                passed += 1
            elif not status.startswith("SKIP"):
                failed += 1
            print(f"  {label}: {status}")

        cleanup(d)

    print(f"\n{'='*40}")
    print(f"Total: {total}  Pass: {passed}  Fail: {failed}  Skip: {total - passed - failed}")
    return 1 if failed > 0 else 0


if __name__ == "__main__":
    sys.exit(main())
