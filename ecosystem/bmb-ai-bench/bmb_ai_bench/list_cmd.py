"""List command — enumerate all problems in the pool."""

from __future__ import annotations

import json
from pathlib import Path

from bmb_ai_bench.registry import load_all_problems


def _default_problems_dir() -> Path:
    return Path(__file__).resolve().parent.parent / "problems"


def run_list(category: str = "all", json_output: bool = False) -> int:
    pdir = _default_problems_dir()
    if not pdir.exists():
        msg = f"ERROR: problems directory not found: {pdir}"
        if json_output:
            print(json.dumps({"ok": False, "error": msg}))
        else:
            print(msg)
        return 1

    problems = load_all_problems(pdir, category=category)

    if json_output:
        print(json.dumps(
            [
                {
                    "number": p.number,
                    "name": p.name,
                    "category": p.category,
                    "difficulty": p.difficulty,
                    "tags": p.tags,
                    "bmb_features_tested": p.bmb_features_tested,
                }
                for p in problems
            ],
            indent=2,
        ))
        return 0

    if not problems:
        print(f"No problems found (category={category!r})")
        return 0

    print(f"{'#':>4}  {'Name':<30}  {'Category':<12}  {'Difficulty':<10}")
    print("-" * 62)
    for p in problems:
        print(f"{p.number:>4}  {p.name:<30}  {p.category:<12}  {p.difficulty:<10}")
    print(f"\nTotal: {len(problems)} problem(s)")
    return 0
