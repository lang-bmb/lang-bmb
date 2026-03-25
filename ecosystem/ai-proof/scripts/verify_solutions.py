#!/usr/bin/env python3
"""Verify all BMB solutions build and pass their test cases."""
from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

_BASE = Path(__file__).resolve().parent.parent
_BMB = Path("D:/data/lang-bmb/target/release/bmb.exe")
_PROBLEMS = _BASE / "problems"


def run_test(exe: Path, stdin: str, expected: str, timeout: float = 10.0) -> tuple[bool, str]:
    """Run exe with stdin, compare stdout to expected."""
    try:
        result = subprocess.run(
            [str(exe)],
            input=stdin,
            capture_output=True,
            text=True,
            timeout=timeout,
        )
        actual = result.stdout
        if actual == expected:
            return True, ""
        return False, f"expected={expected!r} got={actual!r}"
    except subprocess.TimeoutExpired:
        return False, "TIMEOUT"
    except Exception as e:
        return False, str(e)


def verify_problem(problem_dir: Path, tmpdir: Path) -> tuple[int, int, list[str]]:
    """Build and test a single problem. Returns (pass, fail, errors)."""
    name = problem_dir.name
    solution = problem_dir / "solution.bmb"
    tests_file = problem_dir / "tests.json"

    if not solution.exists() or not tests_file.exists():
        return 0, 0, [f"{name}: missing solution.bmb or tests.json"]

    # Build
    exe = tmpdir / f"{name}.exe"
    build_result = subprocess.run(
        [str(_BMB), "build", str(solution), "-o", str(exe)],
        capture_output=True,
        text=True,
        timeout=60,
    )

    if build_result.returncode != 0 or not exe.exists():
        stderr = build_result.stdout + build_result.stderr
        return 0, 1, [f"{name}: BUILD FAILED - {stderr[:200]}"]

    # Run tests
    tests = json.loads(tests_file.read_text(encoding="utf-8"))
    passed = 0
    failed = 0
    errors = []

    for i, test in enumerate(tests):
        stdin = test["stdin"]
        expected = test["expected_stdout"]
        ok, msg = run_test(exe, stdin, expected)
        if ok:
            passed += 1
        else:
            failed += 1
            errors.append(f"{name} test#{i+1}: {msg} (input={stdin!r})")

    return passed, failed, errors


def main() -> int:
    if not _BMB.exists():
        print(f"ERROR: BMB compiler not found at {_BMB}")
        return 1

    problem_dirs = sorted(
        d for d in _PROBLEMS.iterdir()
        if d.is_dir() and d.name[0].isdigit()
    )

    if len(sys.argv) > 1:
        # Filter to specific problem numbers
        nums = set(sys.argv[1:])
        problem_dirs = [d for d in problem_dirs if d.name.split("_")[0] in nums]

    total_pass = 0
    total_fail = 0
    all_errors = []

    with tempfile.TemporaryDirectory() as tmpdir:
        tmppath = Path(tmpdir)
        for pdir in problem_dirs:
            p, f, errs = verify_problem(pdir, tmppath)
            total_pass += p
            total_fail += f
            all_errors.extend(errs)

            status = "PASS" if f == 0 else "FAIL"
            print(f"  {pdir.name:30s} {status} ({p}/{p+f} tests)")
            for e in errs:
                print(f"    ERROR: {e}")

    print(f"\nTotal: {total_pass} passed, {total_fail} failed")
    if all_errors:
        print(f"\n{len(all_errors)} error(s):")
        for e in all_errors:
            print(f"  - {e}")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
