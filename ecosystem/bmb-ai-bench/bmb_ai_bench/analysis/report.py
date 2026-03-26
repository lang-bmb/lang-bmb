"""Report generation from experiment results."""

from __future__ import annotations

import json
import statistics
from pathlib import Path


def generate_report(results_dir: Path, output_format: str = "markdown") -> str:
    """Generate a summary report from experiment results."""
    results_file = results_dir / "results.json"
    if not results_file.exists():
        return "ERROR: results.json not found"

    data = json.loads(results_file.read_text(encoding="utf-8"))

    problems = data.get("problems", {})
    if not problems:
        return "ERROR: no problem results found"

    # Aggregate metrics
    loop_counts = []
    successes = 0
    total = 0
    category_scores: dict[str, list[int]] = {}

    for pid, pdata in problems.items():
        runs = pdata.get("runs", [])
        for run in runs:
            total += 1
            lc = run.get("loop_count", 999)
            loop_counts.append(lc)
            if run.get("final_correct", False):
                successes += 1

            cat = pdata.get("category", "unknown")
            score = _score_run(run)
            category_scores.setdefault(cat, []).append(score)

    median_loops = statistics.median(loop_counts) if loop_counts else 0
    success_rate = (successes / total * 100) if total else 0
    overall_score = statistics.mean(
        s for scores in category_scores.values() for s in scores
    ) if category_scores else 0

    lines = [
        f"# AI-Friendly Benchmark Report",
        f"",
        f"## Summary",
        f"- AI-F Score: {overall_score:.1f}",
        f"- Median Loops: {median_loops:.1f}",
        f"- Success Rate: {success_rate:.0f}%",
        f"- Total Runs: {total}",
        f"",
        f"## Category Scores",
        f"| Category | Score | Median Loops |",
        f"|----------|-------|-------------|",
    ]
    for cat, scores in sorted(category_scores.items()):
        cat_loops = [
            r["loop_count"]
            for pid, pd in problems.items()
            if pd.get("category") == cat
            for r in pd.get("runs", [])
        ]
        lines.append(
            f"| {cat} | {statistics.mean(scores):.1f} | "
            f"{statistics.median(cat_loops):.1f} |"
        )

    return "\n".join(lines)


def _score_run(run: dict) -> int:
    """Score a single run (0-100)."""
    score = 0
    lc = run.get("loop_count", 999)

    # Loop count (30 pts)
    if lc == 1:
        score += 30
    elif lc == 2:
        score += 25
    elif lc == 3:
        score += 20
    elif lc <= 5:
        score += 10
    elif lc <= 10:
        score += 5

    # Correctness (20 pts)
    if run.get("final_correct", False):
        score += 20

    # Build success (20 pts)
    if run.get("compiled", False):
        score += 20

    # Perf ratio (15 pts)
    perf = run.get("perf_ratio")
    if perf is not None:
        if perf <= 1.05:
            score += 15
        elif perf <= 1.10:
            score += 10
        elif perf <= 1.20:
            score += 5

    # Code quality (15 pts) — placeholder, needs manual/heuristic scoring
    score += 15  # default full for now

    return score
