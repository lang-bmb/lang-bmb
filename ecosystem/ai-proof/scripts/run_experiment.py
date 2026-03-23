#!/usr/bin/env python3
"""BMB AI-Native Proof -- Experiment Runner

Usage:
    python scripts/run_experiment.py --pilot --runs 1   # 3 pilot problems
    python scripts/run_experiment.py --phase 1 --runs 3  # Full Phase 1
    python scripts/run_experiment.py --h1-only           # H1 only
    python scripts/run_experiment.py --h2-only           # H2 only
    python scripts/run_experiment.py --dry-run --pilot   # Validate setup
"""
from __future__ import annotations

import argparse
import datetime
import json
import sys
import tempfile
from pathlib import Path

# Ensure the ai-proof package root is on sys.path so relative imports work
# when running the script directly.
_BASE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(_BASE))

_BMB_COMPILER = Path("D:/data/lang-bmb/target/release/bmb.exe")


def main() -> int:
    parser = argparse.ArgumentParser(
        description="BMB AI-Native Proof Experiment Runner"
    )
    parser.add_argument("--phase", type=int, default=1,
                        help="Experiment phase (default: 1)")
    parser.add_argument("--runs", type=int, default=3,
                        help="Number of runs per condition (default: 3)")
    parser.add_argument("--pilot", action="store_true",
                        help="Pilot mode: only problems 1, 2, 21")
    parser.add_argument("--h1-only", action="store_true",
                        help="Run H1 conditions only (bmb_contract vs bmb_nocontract)")
    parser.add_argument("--h2-only", action="store_true",
                        help="Run H2 conditions only (bmb_contract vs rust vs python)")
    parser.add_argument("--dry-run", action="store_true",
                        help="Validate setup without calling LLM")
    parser.add_argument("--model", type=str, default="claude-opus-4-6",
                        help="LLM model name (default: claude-opus-4-6)")
    parser.add_argument("--temperature", type=float, default=0.0,
                        help="LLM temperature (default: 0.0)")
    args = parser.parse_args()

    problems_dir = _BASE / "problems"
    results_dir = _BASE / "results"
    results_dir.mkdir(parents=True, exist_ok=True)

    # Load BMB reference material
    ref_path = _BASE / "protocol" / "bmb_reference.md"
    bmb_ref = ref_path.read_text(encoding="utf-8") if ref_path.exists() else ""

    # Import project modules (after sys.path setup)
    from problems.registry import load_all_problems
    from runners.bmb_runner import BmbRunner
    from runners.python_runner import PythonRunner
    from runners.rust_runner import RustRunner

    # Load problems
    problems = load_all_problems(problems_dir)

    if args.pilot:
        problems = [p for p in problems if p.number in (1, 2, 21)]
        print(f"Pilot mode: {len(problems)} problems")

    if not problems:
        print("ERROR: No problems found.")
        return 1

    # --- Dry-run mode: validate setup and exit ---
    if args.dry_run:
        print("DRY RUN -- validating setup only\n")
        print(f"Loaded {len(problems)} problems:")
        for p in problems:
            print(f"  [{p.number:02d}] {p.name} ({p.category}) -- {len(p.tests)} tests")

        print(f"\nBMB compiler: {_BMB_COMPILER}")
        print(f"  exists: {_BMB_COMPILER.exists()}")

        print(f"\nBMB reference: {ref_path}")
        print(f"  exists: {ref_path.exists()}")
        if bmb_ref:
            print(f"  length: {len(bmb_ref)} chars")

        print(f"\nResults dir: {results_dir}")
        print(f"Phase: {args.phase}  Runs: {args.runs}")

        # Determine conditions
        h1 = ["bmb_contract", "bmb_nocontract"]
        h2 = ["bmb_contract", "rust", "python"]
        conditions = _select_conditions(args.h1_only, args.h2_only, h1, h2)
        print(f"Conditions: {conditions}")

        print("\nSetup OK. Ready to run experiment.")
        return 0

    # --- Full experiment ---
    from orchestrator.experiment import ExperimentRunner
    from orchestrator.llm_client import LlmClient

    runners = {
        "bmb": BmbRunner(bmb_exe=_BMB_COMPILER),
        "rust": RustRunner(),
        "python": PythonRunner(),
    }

    llm = LlmClient(model=args.model, temperature=args.temperature)
    exp = ExperimentRunner(
        llm=llm,
        runners=runners,
        results_dir=results_dir,
        reference=bmb_ref,
    )

    h1_conditions = ["bmb_contract", "bmb_nocontract"]
    h2_conditions = ["bmb_contract", "rust", "python"]
    conditions = _select_conditions(args.h1_only, args.h2_only,
                                    h1_conditions, h2_conditions)

    raw_dir = results_dir / "raw"
    raw_dir.mkdir(parents=True, exist_ok=True)

    for problem in problems:
        prob_key = f"{problem.number:02d}_{problem.name}"
        for condition in conditions:
            for run_id in range(1, args.runs + 1):
                result_dir = raw_dir / prob_key / f"run{run_id}"
                result_file = result_dir / f"{condition}_result.json"

                if result_file.exists():
                    print(f"  SKIP {prob_key}/{condition}/run{run_id}")
                    continue

                print(f"  Running {prob_key} / {condition} / run {run_id}...")

                with tempfile.TemporaryDirectory() as tmp:
                    record = exp.run_single(
                        problem, condition, run_id, Path(tmp)
                    )
                    result_dir.mkdir(parents=True, exist_ok=True)
                    exp.save_run(record, result_file)

                    status = (
                        "PASS"
                        if record.final_correct
                        else f"FAIL (loops={record.loop_count})"
                    )
                    print(f"    -> {status}")

    # Write summary
    summary = {
        "experiment": f"ai-native-proof-phase{args.phase}",
        "date": datetime.date.today().isoformat(),
        "llm": {"model": args.model, "temperature": args.temperature},
        "runs_per_condition": args.runs,
        "total_problems": len(problems),
        "conditions": conditions,
    }
    summary_path = results_dir / "summary.json"
    summary_path.write_text(
        json.dumps(summary, indent=2, ensure_ascii=False), encoding="utf-8"
    )
    print(f"\nDone. Results: {summary_path}")
    return 0


def _select_conditions(
    h1_only: bool,
    h2_only: bool,
    h1: list[str],
    h2: list[str],
) -> list[str]:
    """Determine which experimental conditions to run."""
    if h1_only:
        return list(h1)
    if h2_only:
        return list(h2)
    # Both: merge without duplicates, preserving order
    seen: set[str] = set()
    result: list[str] = []
    for c in h1 + h2:
        if c not in seen:
            seen.add(c)
            result.append(c)
    return result


if __name__ == "__main__":
    sys.exit(main())
