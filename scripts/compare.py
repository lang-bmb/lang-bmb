#!/usr/bin/env python3
"""
BMB Benchmark Comparison Tool
Part of the Bootstrap + Benchmark Cycle System

Compares benchmark results between baseline and current runs,
detecting performance regressions based on configurable thresholds.

Usage:
    python3 scripts/compare.py baseline.json current.json [options]

Options:
    --threshold N       Default threshold percentage (default: 5)
    --tier1-threshold N Tier 1 threshold percentage (default: 2)
    --tier1-strict      Fail on any Tier 1 regression
    --output FILE       Write detailed report to file
    --json              Output results in JSON format
    --ci                CI mode: output GitHub Actions annotations
"""

import json
import sys
import argparse
from dataclasses import dataclass
from typing import Optional
from pathlib import Path


@dataclass
class BenchmarkResult:
    tier: int
    name: str
    bmb_ms: Optional[float]
    c_ms: Optional[float]
    rust_ms: Optional[float]
    ratio_c: Optional[float]
    ratio_rust: Optional[float]


@dataclass
class Comparison:
    name: str
    tier: int
    baseline_ms: Optional[float]
    current_ms: Optional[float]
    delta_ms: Optional[float]
    delta_percent: Optional[float]
    threshold: float
    is_regression: bool
    is_improvement: bool


def load_results(path: str) -> list[BenchmarkResult]:
    """Load benchmark results from JSON file."""
    with open(path, 'r') as f:
        data = json.load(f)

    results = []
    for item in data.get('results', []):
        results.append(BenchmarkResult(
            tier=item.get('tier', 0),
            name=item.get('name', ''),
            bmb_ms=item.get('bmb_ms') if item.get('bmb_ms') != 'null' else None,
            c_ms=item.get('c_ms') if item.get('c_ms') != 'null' else None,
            rust_ms=item.get('rust_ms') if item.get('rust_ms') != 'null' else None,
            ratio_c=item.get('ratio_c') if item.get('ratio_c') != 'null' else None,
            ratio_rust=item.get('ratio_rust') if item.get('ratio_rust') != 'null' else None,
        ))

    return results


def compare_results(
    baseline: list[BenchmarkResult],
    current: list[BenchmarkResult],
    default_threshold: float,
    tier1_threshold: float
) -> list[Comparison]:
    """Compare baseline and current benchmark results."""

    # Build lookup by (tier, name)
    baseline_map = {(r.tier, r.name): r for r in baseline}
    current_map = {(r.tier, r.name): r for r in current}

    comparisons = []

    # Process all benchmarks from both sets
    all_keys = set(baseline_map.keys()) | set(current_map.keys())

    for key in sorted(all_keys):
        tier, name = key
        base = baseline_map.get(key)
        curr = current_map.get(key)

        baseline_ms = base.bmb_ms if base and base.bmb_ms else None
        current_ms = curr.bmb_ms if curr and curr.bmb_ms else None

        # Calculate delta
        delta_ms = None
        delta_percent = None
        is_regression = False
        is_improvement = False

        threshold = tier1_threshold if tier == 1 else default_threshold

        if baseline_ms is not None and current_ms is not None and baseline_ms > 0:
            delta_ms = current_ms - baseline_ms
            delta_percent = (delta_ms / baseline_ms) * 100

            if delta_percent > threshold:
                is_regression = True
            elif delta_percent < -threshold:
                is_improvement = True

        comparisons.append(Comparison(
            name=name,
            tier=tier,
            baseline_ms=baseline_ms,
            current_ms=current_ms,
            delta_ms=delta_ms,
            delta_percent=delta_percent,
            threshold=threshold,
            is_regression=is_regression,
            is_improvement=is_improvement,
        ))

    return comparisons


def format_report(comparisons: list[Comparison], tier1_strict: bool) -> tuple[str, bool]:
    """Format comparison report and determine if there are failures."""

    lines = []
    has_failure = False

    # Group by tier
    tiers = {}
    for c in comparisons:
        if c.tier not in tiers:
            tiers[c.tier] = []
        tiers[c.tier].append(c)

    tier_names = {
        0: "Tier 0: Bootstrap",
        1: "Tier 1: Core Compute",
        2: "Tier 2: Contract Features",
        3: "Tier 3: Real World",
    }

    lines.append("=" * 70)
    lines.append("BMB Benchmark Comparison Report")
    lines.append("=" * 70)
    lines.append("")

    summary = {"regressions": 0, "improvements": 0, "unchanged": 0, "missing": 0}

    for tier in sorted(tiers.keys()):
        tier_comparisons = tiers[tier]
        tier_name = tier_names.get(tier, f"Tier {tier}")

        lines.append(f"=== {tier_name} ===")
        lines.append("")
        lines.append(f"{'Benchmark':<20} {'Baseline':>10} {'Current':>10} {'Delta':>10} {'Status':>12}")
        lines.append("-" * 65)

        for c in sorted(tier_comparisons, key=lambda x: x.name):
            baseline_str = f"{c.baseline_ms:.0f}ms" if c.baseline_ms else "N/A"
            current_str = f"{c.current_ms:.0f}ms" if c.current_ms else "N/A"

            if c.delta_percent is not None:
                delta_str = f"{c.delta_percent:+.1f}%"
                if c.is_regression:
                    status = f"REGRESSION"
                    summary["regressions"] += 1
                    if tier == 1 and tier1_strict:
                        has_failure = True
                elif c.is_improvement:
                    status = f"IMPROVED"
                    summary["improvements"] += 1
                else:
                    status = "OK"
                    summary["unchanged"] += 1
            else:
                delta_str = "N/A"
                status = "MISSING"
                summary["missing"] += 1

            lines.append(f"{c.name:<20} {baseline_str:>10} {current_str:>10} {delta_str:>10} {status:>12}")

        lines.append("")

    lines.append("=" * 70)
    lines.append("Summary")
    lines.append("=" * 70)
    lines.append(f"  Regressions:  {summary['regressions']}")
    lines.append(f"  Improvements: {summary['improvements']}")
    lines.append(f"  Unchanged:    {summary['unchanged']}")
    lines.append(f"  Missing:      {summary['missing']}")
    lines.append("")

    if has_failure:
        lines.append("RESULT: FAILED (Tier 1 regression detected)")
    elif summary["regressions"] > 0:
        lines.append("RESULT: WARNING (Non-critical regressions detected)")
    else:
        lines.append("RESULT: PASSED")

    return "\n".join(lines), has_failure


def format_json(comparisons: list[Comparison]) -> str:
    """Format comparison results as JSON."""
    results = []

    for c in comparisons:
        results.append({
            "name": c.name,
            "tier": c.tier,
            "baseline_ms": c.baseline_ms,
            "current_ms": c.current_ms,
            "delta_ms": c.delta_ms,
            "delta_percent": c.delta_percent,
            "threshold": c.threshold,
            "is_regression": c.is_regression,
            "is_improvement": c.is_improvement,
        })

    regressions = [c for c in comparisons if c.is_regression]
    improvements = [c for c in comparisons if c.is_improvement]

    output = {
        "summary": {
            "total": len(comparisons),
            "regressions": len(regressions),
            "improvements": len(improvements),
            "unchanged": len(comparisons) - len(regressions) - len(improvements),
        },
        "comparisons": results,
    }

    return json.dumps(output, indent=2)


def format_ci_annotations(comparisons: list[Comparison], tier1_strict: bool) -> list[str]:
    """Format GitHub Actions annotations for CI mode."""
    annotations = []

    for c in comparisons:
        if c.is_regression:
            if c.tier == 1 and tier1_strict:
                level = "error"
            else:
                level = "warning"

            msg = f"Performance regression in {c.name}: {c.delta_percent:+.1f}% (threshold: {c.threshold}%)"
            annotations.append(f"::{level}::{msg}")
        elif c.is_improvement:
            msg = f"Performance improvement in {c.name}: {c.delta_percent:+.1f}%"
            annotations.append(f"::notice::{msg}")

    return annotations


def main():
    parser = argparse.ArgumentParser(
        description="Compare BMB benchmark results between baseline and current runs"
    )
    parser.add_argument("baseline", help="Path to baseline results JSON")
    parser.add_argument("current", help="Path to current results JSON")
    parser.add_argument("--threshold", type=float, default=5.0,
                        help="Default threshold percentage (default: 5)")
    parser.add_argument("--tier1-threshold", type=float, default=2.0,
                        help="Tier 1 threshold percentage (default: 2)")
    parser.add_argument("--tier1-strict", action="store_true",
                        help="Fail on any Tier 1 regression")
    parser.add_argument("--output", help="Write detailed report to file")
    parser.add_argument("--json", action="store_true",
                        help="Output results in JSON format")
    parser.add_argument("--ci", action="store_true",
                        help="CI mode: output GitHub Actions annotations")

    args = parser.parse_args()

    # Load results
    try:
        baseline = load_results(args.baseline)
        current = load_results(args.current)
    except FileNotFoundError as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)
    except json.JSONDecodeError as e:
        print(f"Error parsing JSON: {e}", file=sys.stderr)
        sys.exit(1)

    # Compare results
    comparisons = compare_results(
        baseline,
        current,
        args.threshold,
        args.tier1_threshold,
    )

    # Output format
    if args.json:
        print(format_json(comparisons))
    else:
        report, has_failure = format_report(comparisons, args.tier1_strict)
        print(report)

        if args.ci:
            for annotation in format_ci_annotations(comparisons, args.tier1_strict):
                print(annotation)

        if args.output:
            with open(args.output, 'w') as f:
                f.write(report)
            print(f"\nReport written to: {args.output}")

        # Exit with error if Tier 1 regression in strict mode
        if has_failure:
            sys.exit(1)


if __name__ == "__main__":
    main()
