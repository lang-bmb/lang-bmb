"""Problem registry — loads problem definitions from disk."""

from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path


@dataclass
class Problem:
    name: str
    number: int
    category: str  # "algorithm", "system", "contract"
    description: str
    tests: list[dict]
    baseline_c: str


def load_problem(problem_dir: Path) -> Problem:
    """Load a single problem from its directory."""
    desc = (problem_dir / "problem.md").read_text(encoding="utf-8")
    tests = json.loads((problem_dir / "tests.json").read_text(encoding="utf-8"))
    baseline = (
        (problem_dir / "baseline.c").read_text(encoding="utf-8")
        if (problem_dir / "baseline.c").exists()
        else ""
    )
    parts = problem_dir.name.split("_", 1)
    number = int(parts[0])
    name = parts[1] if len(parts) > 1 else problem_dir.name
    # Category from number range
    if number <= 10:
        category = "algorithm"
    elif number <= 20:
        category = "system"
    else:
        category = "contract"
    return Problem(
        name=name,
        number=number,
        category=category,
        description=desc,
        tests=tests,
        baseline_c=baseline,
    )


def load_all_problems(problems_dir: Path) -> list[Problem]:
    """Load all numbered problem directories, sorted by number."""
    dirs = sorted(
        d for d in problems_dir.iterdir() if d.is_dir() and d.name[0].isdigit()
    )
    return [load_problem(d) for d in dirs]
