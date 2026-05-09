"""Registry unit tests."""

from __future__ import annotations

import json
import pytest
from pathlib import Path
from bmb_ai_bench.registry import load_problem, load_all_problems, _infer_category


PROBLEMS_DIR = Path(__file__).resolve().parent.parent / "problems"


def test_infer_category():
    assert _infer_category(1) == "algorithm"
    assert _infer_category(10) == "algorithm"
    assert _infer_category(11) == "system"
    assert _infer_category(20) == "system"
    assert _infer_category(21) == "contract"
    assert _infer_category(30) == "contract"
    assert _infer_category(31) == "performance"
    assert _infer_category(45) == "performance"
    assert _infer_category(46) == "practical"
    assert _infer_category(60) == "practical"
    assert _infer_category(61) == "edge"
    assert _infer_category(75) == "edge"
    assert _infer_category(76) == "integration"
    assert _infer_category(100) == "integration"


@pytest.mark.skipif(not PROBLEMS_DIR.exists(), reason="problems dir not found")
def test_load_all_problems_count():
    problems = load_all_problems(PROBLEMS_DIR)
    assert len(problems) >= 50


@pytest.mark.skipif(not PROBLEMS_DIR.exists(), reason="problems dir not found")
def test_load_all_problems_fields():
    problems = load_all_problems(PROBLEMS_DIR)
    for p in problems:
        assert isinstance(p.number, int)
        assert p.number >= 1
        assert p.name
        assert p.category in {
            "algorithm", "system", "contract", "performance",
            "practical", "edge", "integration", "unknown"
        }
        assert p.difficulty in {"easy", "medium", "hard", ""}
        assert isinstance(p.perf_target_ratio, float)
        assert p.perf_target_ratio > 0


@pytest.mark.skipif(not PROBLEMS_DIR.exists(), reason="problems dir not found")
def test_load_all_category_filter():
    algo = load_all_problems(PROBLEMS_DIR, category="algorithm")
    assert all(p.category == "algorithm" for p in algo)
    assert len(algo) > 0


@pytest.mark.skipif(not PROBLEMS_DIR.exists(), reason="problems dir not found")
def test_perf_ratio_is_informational():
    """perf_target_ratio must not cause hard gating — verify it's just metadata."""
    problems = load_all_problems(PROBLEMS_DIR)
    for p in problems:
        # 1.0 = perfect parity with C. All problems should allow ≥1.0.
        assert p.perf_target_ratio >= 1.0
