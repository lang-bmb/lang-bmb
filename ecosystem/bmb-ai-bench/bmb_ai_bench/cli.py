"""CLI entry point for bmb-ai-bench."""

from __future__ import annotations

import argparse
import sys

from bmb_ai_bench.doctor import run_doctor
from bmb_ai_bench.list_cmd import run_list
from bmb_ai_bench.validate import run_validate
from bmb_ai_bench.analysis.dashboard import run_dashboard


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        prog="bmb-ai-bench",
        description="BMB AI-Friendly Benchmark Suite",
    )
    sub = parser.add_subparsers(dest="command")

    # doctor
    doc_p = sub.add_parser("doctor", help="Check environment prerequisites")
    doc_p.add_argument("--json", action="store_true", help="Output JSON")

    # list
    list_p = sub.add_parser("list", help="List all problems in the pool")
    list_p.add_argument("--category", default="all", help="Category filter (all, algorithm, system, contract, performance, practical, edge, integration)")
    list_p.add_argument("--json", action="store_true", help="Output JSON")

    # validate
    val_p = sub.add_parser("validate", help="Validate problem pool (solutions pass all tests)")
    val_p.add_argument("--category", default="all", help="Category filter")
    val_p.add_argument("--problems-dir", default=None, help="Override problems directory")
    val_p.add_argument("--json", action="store_true", help="Output JSON")

    # dashboard
    dash_p = sub.add_parser("dashboard", help="Show problem pool dashboard (stats by category)")
    dash_p.add_argument("--json", action="store_true", help="Output JSON")

    # run (placeholder — requires LLM API key)
    run_p = sub.add_parser("run", help="Run LLM experiment (requires API key)")
    run_p.add_argument("--model", required=True)
    run_p.add_argument("--api-base", default="https://api.anthropic.com/v1")
    run_p.add_argument("--api-key", default=None)
    run_p.add_argument("--category", default="all")
    run_p.add_argument("--runs", type=int, default=5)
    run_p.add_argument("--output", default="results/")

    # analyze (placeholder — processes saved run results)
    analyze_p = sub.add_parser("analyze", help="Analyze saved run results")
    analyze_p.add_argument("--results-dir", default="results/", help="Directory with results.json")
    analyze_p.add_argument("--format", default="markdown", choices=["markdown", "json"], help="Output format")

    args = parser.parse_args(argv)

    if args.command == "doctor":
        return run_doctor(json_output=args.json)
    elif args.command == "list":
        return run_list(category=args.category, json_output=args.json)
    elif args.command == "validate":
        return run_validate(
            category=args.category,
            problems_dir=args.problems_dir,
            json_output=args.json,
        )
    elif args.command == "dashboard":
        return run_dashboard(json_output=args.json)
    elif args.command == "run":
        print("ERROR: 'run' command requires LLM API endpoint (not yet implemented in this release)")
        return 1
    elif args.command == "analyze":
        from bmb_ai_bench.analysis.report import generate_report
        from pathlib import Path
        report = generate_report(Path(args.results_dir), args.format)
        print(report)
        return 0
    else:
        parser.print_help()
        return 0


if __name__ == "__main__":
    sys.exit(main())
