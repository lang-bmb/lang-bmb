#!/usr/bin/env python3
"""Run a mock experiment to validate the full pipeline end-to-end.

Uses golden solutions instead of a real LLM to test:
- Experiment orchestrator
- BMB runner (check + build + test)
- Error normalizer + loop classifier
- Result serialization
- Report generation

Usage:
    python scripts/run_mock_experiment.py [--with-errors] [--problem NUM]
"""
from __future__ import annotations

import json
import sys
import tempfile
from pathlib import Path

# Add parent to path
sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from orchestrator.experiment import ExperimentRunner, RunRecord
from orchestrator.llm_client import LlmClient
from problems.registry import load_all_problems
from runners.bmb_runner import BmbRunner


class MockLlmClient:
    """Fake LLM that returns pre-canned solutions.

    If with_errors=True, first attempt returns a deliberately broken solution
    (common Rust-ism), and second attempt returns the golden solution.
    This tests the error feedback loop.
    """

    def __init__(self, golden_solutions: dict[str, str], with_errors: bool = False):
        self.golden = golden_solutions
        self.with_errors = with_errors
        self._call_count: dict[str, int] = {}
        self._current_key: str = "unknown"
        self._broken_code = (
            'fn main() -> i64 = {\n'
            '    let v: Vec<i64> = Vec::new();\n'
            '    v.push(42);\n'
            '    println!("{}", v.len());\n'
            '    0\n'
            '};\n'
        )

    def set_current_problem(self, key: str) -> None:
        """Set the problem key before calling generate."""
        self._current_key = key

    def generate(self, system: str, messages: list[dict],
                 retries: int = 2) -> str:
        key = self._current_key
        call_num = self._call_count.get(key, 0) + 1
        self._call_count[key] = call_num

        if self.with_errors and call_num == 1:
            return f"```bmb\n{self._broken_code}```"

        code = self.golden.get(key, "fn main() -> i64 = 0;")
        return f"```bmb\n{code}```"


def main() -> int:
    import argparse
    parser = argparse.ArgumentParser(description="Mock experiment pipeline validation")
    parser.add_argument("--with-errors", action="store_true",
                        help="First attempt returns broken code to test error feedback")
    parser.add_argument("--problem", type=int, help="Run only this problem number")
    args = parser.parse_args()

    problems_dir = Path(__file__).resolve().parent.parent / "problems"
    problems = load_all_problems(problems_dir)

    if args.problem:
        problems = [p for p in problems if p.number == args.problem]
        if not problems:
            print(f"ERROR: Problem {args.problem} not found")
            return 1

    # Load golden solutions
    golden = {}
    for p in problems:
        sol_path = problems_dir / f"{p.number:02d}_{p.name}" / "solution.bmb"
        if sol_path.exists():
            golden[f"{p.number:02d}_{p.name}"] = sol_path.read_text(encoding="utf-8")

    # Create mock LLM and runner
    mock_llm = MockLlmClient(golden, with_errors=args.with_errors)
    bmb_runner = BmbRunner()
    runners = {"bmb": bmb_runner}

    reference = None
    ref_path = Path(__file__).resolve().parent.parent / "protocol" / "bmb_reference.md"
    if ref_path.exists():
        reference = ref_path.read_text(encoding="utf-8")

    experiment = ExperimentRunner(
        llm=mock_llm,
        runners=runners,
        results_dir=None,
        reference=reference,
    )

    # Run experiment
    total_pass = 0
    total_fail = 0
    results: list[dict] = []

    for p in problems:
        with tempfile.TemporaryDirectory() as tmpdir:
            try:
                mock_llm.set_current_problem(f"{p.number:02d}_{p.name}")
                record = experiment.run_single(
                    problem=p,
                    condition="bmb_contract",
                    run_id=1,
                    work_dir=Path(tmpdir),
                )

                status = "PASS" if record.final_correct else "FAIL"
                loops = record.loop_count
                types = record.loop_types

                if record.final_correct:
                    total_pass += 1
                else:
                    total_fail += 1

                results.append({
                    "problem": f"{p.number:02d}_{p.name}",
                    "status": status,
                    "loops": loops,
                    "types": types,
                })

                type_str = " ".join(f"{k}={v}" for k, v in types.items() if v > 0)
                print(f"  {p.number:02d}_{p.name:25s} {status}  loops={loops}  {type_str}")

            except Exception as e:
                total_fail += 1
                results.append({
                    "problem": f"{p.number:02d}_{p.name}",
                    "status": "ERROR",
                    "error": str(e)[:200],
                })
                print(f"  {p.number:02d}_{p.name:25s} ERROR: {str(e)[:100]}")

    print(f"\nTotal: {total_pass} pass, {total_fail} fail")

    if args.with_errors:
        error_problems = [r for r in results if r.get("loops", 0) > 1]
        print(f"Error feedback tested: {len(error_problems)} problems had multi-loop")

    return 0 if total_fail == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
