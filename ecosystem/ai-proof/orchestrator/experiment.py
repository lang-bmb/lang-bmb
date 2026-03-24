"""Experiment orchestrator — drives the generate-build-test loop."""

from __future__ import annotations

import json
import subprocess
from dataclasses import asdict, dataclass, field
from pathlib import Path

from orchestrator.error_normalizer import normalize_error
from orchestrator.llm_client import LlmClient
from orchestrator.loop_classifier import classify_loop
from protocol.prompt_templates import build_error_feedback_prompt, build_initial_prompt
from runners.base import RunnerBase, RunResult
from runners.perf import measure_binary
from scripts.strip_contracts import strip_contracts

MAX_LOOPS = 10

# Map condition names to the language tag used for code extraction / prompts.
_CONDITION_LANG = {
    "bmb_contract": "bmb",
    "bmb_nocontract": "bmb",
    "rust": "rust",
    "python": "python",
}

# Map condition names to the runner dict key.
_CONDITION_RUNNER = {
    "bmb_contract": "bmb",
    "bmb_nocontract": "bmb",
    "rust": "rust",
    "python": "python",
}


@dataclass
class AttemptRecord:
    attempt: int
    code: str
    compiled: bool
    test_passed: bool
    error: dict | None
    loop_type: str | None


@dataclass
class RunRecord:
    run_id: int
    condition: str  # "bmb_contract", "bmb_nocontract", "rust", "python"
    loop_count: int
    loop_types: dict  # {"A": 0, "B": 0, "C": 0, "D": 0}
    final_correct: bool
    perf_ratio: float | None
    attempts: list[AttemptRecord] = field(default_factory=list)


class ExperimentRunner:
    """Drives the LLM generate → build → test feedback loop."""

    def __init__(
        self,
        llm,
        runners: dict[str, RunnerBase],
        results_dir: Path | None,
        reference: str | None,
    ):
        self.llm = llm
        self.runners = runners
        self.results_dir = results_dir
        self.reference = reference

    # ------------------------------------------------------------------
    # Public API
    # ------------------------------------------------------------------

    def run_single(
        self,
        problem,
        condition: str,
        run_id: int,
        work_dir,
    ) -> RunRecord:
        """Run one problem under one condition, returning a RunRecord."""
        lang = _CONDITION_LANG.get(condition, condition)
        runner_key = _CONDITION_RUNNER.get(condition, condition)
        runner = self.runners[runner_key]

        loop_types: dict[str, int] = {"A": 0, "B": 0, "C": 0, "D": 0}
        attempts: list[AttemptRecord] = []
        messages: list[dict] = []

        # Build test preview string for prompt (limit to 5 examples)
        preview_tests = problem.tests[:5]
        test_preview = "\n".join(
            f"stdin: {t.get('stdin', '')}  → stdout: {t.get('expected_stdout', t.get('expected', ''))}"
            for t in preview_tests
        )

        system_prompt = build_initial_prompt(
            problem.description, lang, test_preview, self.reference
        )
        messages.append({"role": "user", "content": system_prompt})

        # System instruction: code only, minimal explanation
        sys_instruction = (
            f"You are a {lang} programmer. Output ONLY the complete source code "
            f"inside a ```{lang} code block. No explanation, no comments outside code."
        )

        for attempt_num in range(1, MAX_LOOPS + 1):
            # --- Generate code ---
            response = self.llm.generate(sys_instruction, messages)
            code = LlmClient.extract_code(response, lang)

            # --- Strip contracts for nocontract condition ---
            build_code = code
            if condition == "bmb_nocontract":
                build_code = strip_contracts(code)

            # --- Build ---
            build_result: RunResult = runner.build(build_code, work_dir)

            if not build_result.compiled:
                error = normalize_error(build_result.error_msg, lang)
                lt = classify_loop(error, lang)
                loop_types[lt.value] += 1

                attempts.append(AttemptRecord(
                    attempt=attempt_num,
                    code=code,
                    compiled=False,
                    test_passed=False,
                    error=error,
                    loop_type=lt.value,
                ))

                # Feed error back to LLM
                feedback = build_error_feedback_prompt(
                    error["type"],
                    error["normalized"],
                    error["location"],
                    error["raw"],
                )
                messages.append({"role": "assistant", "content": response})
                messages.append({"role": "user", "content": feedback})
                continue

            # --- Test ---
            test_result: RunResult = runner.test(work_dir, problem.tests)

            if not test_result.test_passed:
                error = normalize_error(
                    test_result.error_msg, lang, is_test_failure=True
                )
                lt = classify_loop(error, lang)
                loop_types[lt.value] += 1

                attempts.append(AttemptRecord(
                    attempt=attempt_num,
                    code=code,
                    compiled=True,
                    test_passed=False,
                    error=error,
                    loop_type=lt.value,
                ))

                feedback = build_error_feedback_prompt(
                    error["type"],
                    error["normalized"],
                    error["location"],
                    error["raw"],
                )
                messages.append({"role": "assistant", "content": response})
                messages.append({"role": "user", "content": feedback})
                continue

            # --- Success ---
            perf_ratio = self._compute_perf_ratio(
                problem, runner, work_dir
            )

            attempts.append(AttemptRecord(
                attempt=attempt_num,
                code=code,
                compiled=True,
                test_passed=True,
                error=None,
                loop_type=None,
            ))

            return RunRecord(
                run_id=run_id,
                condition=condition,
                loop_count=attempt_num,
                loop_types=loop_types,
                final_correct=True,
                perf_ratio=perf_ratio,
                attempts=attempts,
            )

        # --- Exhausted all loops ---
        return RunRecord(
            run_id=run_id,
            condition=condition,
            loop_count=MAX_LOOPS + 1,
            loop_types=loop_types,
            final_correct=False,
            perf_ratio=None,
            attempts=attempts,
        )

    def save_run(self, record: RunRecord, path: Path) -> None:
        """Persist a RunRecord to disk as JSON."""
        data = asdict(record)
        path.write_text(json.dumps(data, indent=2, ensure_ascii=False), encoding="utf-8")

    # ------------------------------------------------------------------
    # Internal helpers
    # ------------------------------------------------------------------

    def _compute_perf_ratio(self, problem, runner, work_dir) -> float | None:
        """Compute BMB_time / C_baseline_time, or None."""
        if not problem.baseline_c:
            return None

        # Use first test case's stdin for performance measurement
        perf_stdin = ""
        if problem.tests:
            perf_stdin = problem.tests[0].get("stdin", "")

        c_ns = self._measure_c_baseline(problem, work_dir, perf_stdin)
        if c_ns is None:
            return None

        bmb_ns = runner.measure_perf(work_dir, stdin=perf_stdin)
        if bmb_ns is None:
            return None

        return bmb_ns / c_ns

    def _measure_c_baseline(self, problem, work_dir, stdin: str = "") -> int | None:
        """Compile baseline.c with clang -O2 and measure. Returns median_ns or None."""
        if not problem.baseline_c:
            return None

        work = Path(work_dir) if not isinstance(work_dir, Path) else work_dir
        c_src = work / "baseline.c"
        c_bin = work / "baseline"

        try:
            c_src.write_text(problem.baseline_c, encoding="utf-8")
            result = subprocess.run(
                ["clang", "-O2", str(c_src), "-o", str(c_bin)],
                capture_output=True,
                text=True,
                timeout=30,
            )
            if result.returncode != 0:
                return None
            stats = measure_binary(c_bin, stdin=stdin)
            return stats["median_ns"]
        except (FileNotFoundError, subprocess.TimeoutExpired, OSError):
            return None
