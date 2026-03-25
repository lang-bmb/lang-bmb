"""BMB language runner — build, test, and measure performance."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

from runners.base import RunnerBase, RunResult
from runners.perf import measure_binary

# Resolve BMB compiler location relative to the repo root.
_REPO_ROOT = Path(__file__).resolve().parents[3]  # lang-bmb/
_BMB_EXE = _REPO_ROOT / "target" / "release" / ("bmb.exe" if sys.platform == "win32" else "bmb")

_BUILD_TIMEOUT = 60  # seconds
_TEST_TIMEOUT = 10   # seconds per test case


class BmbRunner(RunnerBase):
    """Runner for BMB source code."""

    def __init__(self, bmb_exe: Path | None = None):
        self._bmb = bmb_exe or _BMB_EXE

    # ------------------------------------------------------------------
    # Build
    # ------------------------------------------------------------------

    def build(self, source_code: str, work_dir: Path) -> RunResult:
        src = work_dir / "solution.bmb"
        src.write_text(source_code, encoding="utf-8")

        out_name = "solution.exe" if sys.platform == "win32" else "solution"
        out_path = work_dir / out_name

        # Step 1: Run 'check' first — produces enriched JSONL with suggestions
        try:
            check_proc = subprocess.run(
                [str(self._bmb), "check", str(src)],
                capture_output=True,
                text=True,
                timeout=_BUILD_TIMEOUT,
                cwd=str(work_dir),
            )
        except subprocess.TimeoutExpired:
            return RunResult(
                compiled=False, test_passed=False,
                error_msg="BMB check timed out",
            )

        if check_proc.returncode != 0:
            # Use check output (enriched JSONL with suggestions)
            err = (check_proc.stdout + check_proc.stderr).strip()
            return RunResult(
                compiled=False, test_passed=False,
                error_msg=err, raw_output=err,
            )

        # Step 2: Check passed — now build to get the binary
        try:
            build_proc = subprocess.run(
                [str(self._bmb), "build", str(src), "-o", str(out_path), "--release"],
                capture_output=True,
                text=True,
                timeout=_BUILD_TIMEOUT,
                cwd=str(work_dir),
            )
        except subprocess.TimeoutExpired:
            return RunResult(
                compiled=False, test_passed=False,
                error_msg="BMB build timed out",
            )

        if build_proc.returncode != 0:
            err = (build_proc.stderr + build_proc.stdout).strip()
            return RunResult(
                compiled=False, test_passed=False,
                error_msg=err, raw_output=err,
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

    def measure_perf(self, work_dir: Path, iterations: int = 10,
                     stdin: str = "") -> int | None:
        bin_name = "solution.exe" if sys.platform == "win32" else "solution"
        binary = work_dir / bin_name
        if not binary.exists():
            return None
        result = measure_binary(binary, stdin=stdin, iterations=iterations)
        return result["median_ns"]
