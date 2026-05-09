"""Dashboard — problem pool stats by category (no LLM run required)."""

from __future__ import annotations

import json
from collections import Counter
from pathlib import Path

from bmb_ai_bench.registry import load_all_problems


def _default_problems_dir() -> Path:
    return Path(__file__).resolve().parent.parent.parent / "problems"


def run_dashboard(json_output: bool = False) -> int:
    pdir = _default_problems_dir()
    if not pdir.exists():
        msg = f"ERROR: problems directory not found: {pdir}"
        if json_output:
            print(json.dumps({"ok": False, "error": msg}))
        else:
            print(msg)
        return 1

    problems = load_all_problems(pdir)
    total = len(problems)

    by_category: dict[str, list] = {}
    for p in problems:
        by_category.setdefault(p.category, []).append(p)

    by_difficulty = Counter(p.difficulty for p in problems)

    features: Counter = Counter()
    for p in problems:
        for f in p.bmb_features_tested:
            features[f] += 1

    if json_output:
        print(json.dumps({
            "total": total,
            "categories": {
                cat: {
                    "count": len(ps),
                    "difficulties": dict(Counter(p.difficulty for p in ps)),
                }
                for cat, ps in sorted(by_category.items())
            },
            "difficulty_breakdown": dict(by_difficulty),
            "top_features": dict(features.most_common(10)),
        }, indent=2))
        return 0

    print("bmb-ai-bench dashboard")
    print("=" * 60)
    print(f"  Total problems: {total}")
    print()
    print(f"  {'Category':<14}  {'Count':>5}  {'Easy':>4}  {'Med':>4}  {'Hard':>4}")
    print("  " + "-" * 40)
    for cat, ps in sorted(by_category.items()):
        diff = Counter(p.difficulty for p in ps)
        print(
            f"  {cat:<14}  {len(ps):>5}  "
            f"{diff.get('easy', 0):>4}  "
            f"{diff.get('medium', 0):>4}  "
            f"{diff.get('hard', 0):>4}"
        )
    print()
    print(f"  Difficulty breakdown: {dict(by_difficulty)}")
    if features:
        print(f"  Top BMB features tested: {', '.join(k for k, _ in features.most_common(5))}")
    print("=" * 60)
    print("  Performance policy: tracking only — no hard gate (≤1.05×=15pts, ≤1.10×=10pts, ≤1.20×=5pts)")
    return 0
