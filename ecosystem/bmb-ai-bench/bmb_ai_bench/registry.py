"""Problem registry — loads problem definitions with metadata from disk."""

from __future__ import annotations

import json
from dataclasses import dataclass, field
from pathlib import Path


@dataclass
class Problem:
    id: str
    number: int
    name: str
    category: str
    difficulty: str
    tags: list[str]
    description: str
    tests: list[dict]
    baseline_c: str
    solution_bmb: str
    solution_rs: str
    bmb_features_tested: list[str] = field(default_factory=list)
    perf_target_ratio: float = 1.10
    c_baseline_flags: str = "-O2 -march=native"


def load_problem(problem_dir: Path) -> Problem:
    """Load a single problem with metadata."""
    desc = (problem_dir / "problem.md").read_text(encoding="utf-8") if (problem_dir / "problem.md").exists() else ""
    tests = json.loads((problem_dir / "tests.json").read_text(encoding="utf-8"))
    baseline = (problem_dir / "baseline.c").read_text(encoding="utf-8") if (problem_dir / "baseline.c").exists() else ""
    sol_bmb = (problem_dir / "solution.bmb").read_text(encoding="utf-8") if (problem_dir / "solution.bmb").exists() else ""
    sol_rs = (problem_dir / "solution.rs").read_text(encoding="utf-8") if (problem_dir / "solution.rs").exists() else ""

    parts = problem_dir.name.split("_", 1)
    number = int(parts[0])
    name = parts[1] if len(parts) > 1 else problem_dir.name

    # Load metadata.json if it exists, otherwise infer
    meta_file = problem_dir / "metadata.json"
    if meta_file.exists():
        meta = json.loads(meta_file.read_text(encoding="utf-8"))
    else:
        meta = {}

    pid = meta.get("id", problem_dir.name)
    category = meta.get("category", _infer_category(number))
    difficulty = meta.get("difficulty", "medium")
    tags = meta.get("tags", [])
    features = meta.get("bmb_features_tested", [])
    perf_ratio = meta.get("perf_target_ratio", 1.10)
    c_flags = meta.get("c_baseline_flags", "-O2 -march=native")

    return Problem(
        id=pid,
        number=number,
        name=name,
        category=category,
        difficulty=difficulty,
        tags=tags,
        description=desc,
        tests=tests,
        baseline_c=baseline,
        solution_bmb=sol_bmb,
        solution_rs=sol_rs,
        bmb_features_tested=features,
        perf_target_ratio=perf_ratio,
        c_baseline_flags=c_flags,
    )


def _infer_category(number: int) -> str:
    if number <= 10:
        return "algorithm"
    if number <= 20:
        return "system"
    if number <= 30:
        return "contract"
    if number <= 45:
        return "performance"
    if number <= 60:
        return "practical"
    if number <= 75:
        return "edge"
    return "integration"


def load_all_problems(problems_dir: Path, category: str = "all") -> list[Problem]:
    """Load all problems, optionally filtered by category."""
    dirs = sorted(
        d for d in problems_dir.iterdir()
        if d.is_dir() and d.name[0].isdigit()
    )
    problems = [load_problem(d) for d in dirs]
    if category != "all":
        problems = [p for p in problems if p.category == category]
    return problems
