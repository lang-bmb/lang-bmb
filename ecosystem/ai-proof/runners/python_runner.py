"""Python language runner — build (write), test, and measure performance."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

from runners.base import RunnerBase, RunResult

_TEST_TIMEOUT = 10


class PythonRunner(RunnerBase):
    """Runner for Python source code.

    ``build()`` simply writes the file (Python is interpreted).
    ``measure_perf()`` always returns ``None`` — Python serves as a
    correctness-only comparison baseline.
    """

    def __init__(self, python: str | None = None):
        self._python = python or sys.executable

    # ------------------------------------------------------------------
    # Build
    # ------------------------------------------------------------------

    def build(self, source_code: str, work_dir: Path) -> RunResult:
        src = work_dir / "solution.py"
        src.write_text(source_code, encoding="utf-8")
        return RunResult(compiled=True, test_passed=False, error_msg="")

    # ------------------------------------------------------------------
    # Test
    # ------------------------------------------------------------------

    def test(self, work_dir: Path, tests: list[dict]) -> RunResult:
        src = work_dir / "solution.py"
        if not src.exists():
            return RunResult(
                compiled=False,
                test_passed=False,
                error_msg="solution.py not found; build first",
            )

        all_output: list[str] = []
        for i, tc in enumerate(tests):
            args: list[str] = tc.get("args", [])
            stdin_data: str = tc.get("stdin", "")
            expected: str = tc.get("expected_stdout", "")

            try:
                proc = subprocess.run(
                    [self._python, str(src)] + [str(a) for a in args],
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

            if proc.returncode != 0:
                err = (proc.stderr + proc.stdout).strip()
                return RunResult(
                    compiled=True,
                    test_passed=False,
                    error_msg=f"Test {i} runtime error: {err}",
                    raw_output=err,
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

    def measure_perf(self, work_dir: Path, iterations: int = 10) -> None:
        """Python is correctness-only; no performance measurement."""
        return None
