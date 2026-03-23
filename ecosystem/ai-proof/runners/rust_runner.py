"""Rust language runner — build, test, and measure performance."""

from __future__ import annotations

import shutil
import subprocess
import sys
from pathlib import Path

from runners.base import RunnerBase, RunResult
from runners.perf import measure_binary

_BUILD_TIMEOUT = 60
_TEST_TIMEOUT = 10


class RustRunner(RunnerBase):
    """Runner for Rust source code."""

    def __init__(self, rustc: str | None = None):
        self._rustc = rustc or "rustc"

    # ------------------------------------------------------------------
    # Build
    # ------------------------------------------------------------------

    def build(self, source_code: str, work_dir: Path) -> RunResult:
        src = work_dir / "solution.rs"
        src.write_text(source_code, encoding="utf-8")

        out_name = "solution.exe" if sys.platform == "win32" else "solution"
        out_path = work_dir / out_name

        try:
            proc = subprocess.run(
                [self._rustc, "--edition", "2021", "-O", str(src), "-o", str(out_path)],
                capture_output=True,
                text=True,
                timeout=_BUILD_TIMEOUT,
                cwd=str(work_dir),
            )
        except subprocess.TimeoutExpired:
            return RunResult(
                compiled=False,
                test_passed=False,
                error_msg="Rust build timed out",
            )
        except FileNotFoundError:
            return RunResult(
                compiled=False,
                test_passed=False,
                error_msg="rustc not found on PATH",
            )

        if proc.returncode != 0:
            err = (proc.stderr + proc.stdout).strip()
            return RunResult(
                compiled=False,
                test_passed=False,
                error_msg=err,
                raw_output=err,
            )

        return RunResult(compiled=True, test_passed=False, error_msg="")

    # ------------------------------------------------------------------
    # Test
    # ------------------------------------------------------------------

    def test(self, work_dir: Path, tests: list[dict]) -> RunResult:
        bin_name = "solution.exe" if sys.platform == "win32" else "solution"
        binary = work_dir / bin_name
        if not binary.exists():
            return RunResult(
                compiled=False,
                test_passed=False,
                error_msg="Binary not found; build first",
            )

        all_output: list[str] = []
        for i, tc in enumerate(tests):
            args: list[str] = tc.get("args", [])
            stdin_data: str = tc.get("stdin", "")
            expected: str = tc.get("expected_stdout", "")

            try:
                proc = subprocess.run(
                    [str(binary)] + [str(a) for a in args],
                    input=stdin_data,
                    capture_output=True,
                    text=True,
                    timeout=_TEST_TIMEOUT,
                    cwd=str(work_dir),
                )
            except subprocess.TimeoutExpired:
                return RunResult(
                    compiled=True,
                    test_passed=False,
                    error_msg=f"Test case {i} timed out",
                )

            actual = proc.stdout
            all_output.append(actual)

            if actual != expected:
                return RunResult(
                    compiled=True,
                    test_passed=False,
                    error_msg=f"Test {i}: expected {expected!r}, got {actual!r}",
                    raw_output=actual,
                )

        return RunResult(
            compiled=True,
            test_passed=True,
            error_msg="",
            raw_output="\n".join(all_output),
        )

    # ------------------------------------------------------------------
    # Perf
    # ------------------------------------------------------------------

    def measure_perf(self, work_dir: Path, iterations: int = 10) -> int | None:
        bin_name = "solution.exe" if sys.platform == "win32" else "solution"
        binary = work_dir / bin_name
        if not binary.exists():
            return None
        result = measure_binary(binary, iterations=iterations)
        return result["median_ns"]
