"""CLI entry point for bmb-ai-bench."""

from __future__ import annotations

import argparse
import sys

from bmb_ai_bench.doctor import run_doctor
from bmb_ai_bench.validate import run_validate


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        prog="bmb-ai-bench",
        description="BMB AI-Friendly Benchmark Suite",
    )
    sub = parser.add_subparsers(dest="command")

    # doctor
    doc_p = sub.add_parser("doctor", help="Check environment prerequisites")
    doc_p.add_argument("--json", action="store_true", help="Output JSON")

    # validate
    val_p = sub.add_parser("validate", help="Validate problem pool (solutions pass all tests)")
    val_p.add_argument("--category", default="all", help="Category filter (all, algorithm, system, contract, performance, practical, edge, integration)")
    val_p.add_argument("--problems-dir", default=None, help="Override problems directory")
    val_p.add_argument("--json", action="store_true", help="Output JSON")

    # run (placeholder)
    run_p = sub.add_parser("run", help="Run LLM experiment")
    run_p.add_argument("--model", required=True)
    run_p.add_argument("--api-base", default="https://api.anthropic.com/v1")
    run_p.add_argument("--api-key", default=None)
    run_p.add_argument("--category", default="all")
    run_p.add_argument("--runs", type=int, default=5)
    run_p.add_argument("--output", default="results/")

    # analyze (placeholder)
    sub.add_parser("analyze", help="Analyze results and generate report")

    args = parser.parse_args(argv)

    if args.command == "doctor":
        return run_doctor(json_output=args.json)
    elif args.command == "validate":
        return run_validate(
            category=args.category,
            problems_dir=args.problems_dir,
            json_output=args.json,
        )
    elif args.command == "run":
        print("ERROR: 'run' command requires LLM API endpoint (not yet implemented)")
        return 1
    elif args.command == "analyze":
        print("ERROR: 'analyze' command not yet implemented")
        return 1
    else:
        parser.print_help()
        return 0


if __name__ == "__main__":
    sys.exit(main())
