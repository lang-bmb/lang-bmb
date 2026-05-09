"""Tests for the analyze subcommand and report generation."""

from __future__ import annotations

import json
import tempfile
from pathlib import Path

import pytest

from bmb_ai_bench.cli import main
from bmb_ai_bench.analysis.report import generate_report


SAMPLE_RESULTS = {
    "model": "test-model",
    "date": "2026-05-09",
    "problems": {
        "01_binary_search": {
            "category": "algorithm",
            "difficulty": "easy",
            "runs": [
                {"run_id": 1, "loop_count": 1, "final_correct": True, "compiled": True, "perf_ratio": None},
                {"run_id": 2, "loop_count": 2, "final_correct": True, "compiled": True, "perf_ratio": 1.05},
            ],
        },
        "21_bounded_counter": {
            "category": "contract",
            "difficulty": "medium",
            "runs": [
                {"run_id": 1, "loop_count": 5, "final_correct": False, "compiled": True, "perf_ratio": None},
            ],
        },
    },
}


def test_generate_report_markdown():
    with tempfile.TemporaryDirectory() as tmpdir:
        d = Path(tmpdir)
        (d / "results.json").write_text(json.dumps(SAMPLE_RESULTS), encoding="utf-8")
        report = generate_report(d, "markdown")
    assert "AI-Friendly Benchmark Report" in report
    assert "Summary" in report
    assert "algorithm" in report or "contract" in report


def test_generate_report_missing_file():
    with tempfile.TemporaryDirectory() as tmpdir:
        d = Path(tmpdir)
        report = generate_report(d, "markdown")
    assert "ERROR" in report


def test_analyze_cli_with_results_json(capsys):
    with tempfile.TemporaryDirectory() as tmpdir:
        d = Path(tmpdir)
        (d / "results.json").write_text(json.dumps(SAMPLE_RESULTS), encoding="utf-8")
        rc = main(["analyze", "--results-dir", str(d)])
    assert rc == 0
    out = capsys.readouterr().out
    assert "AI-Friendly Benchmark Report" in out


def test_analyze_cli_missing_dir(capsys):
    rc = main(["analyze", "--results-dir", "/nonexistent/path/xyz"])
    assert rc == 0
    out = capsys.readouterr().out
    assert "ERROR" in out
