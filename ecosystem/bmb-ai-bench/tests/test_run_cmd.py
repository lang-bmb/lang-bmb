"""Tests for the run subcommand (no LLM API calls)."""

from __future__ import annotations

import json
import sys
from pathlib import Path
from unittest.mock import MagicMock, patch

import pytest

from bmb_ai_bench.cli import main
from bmb_ai_bench.run_cmd import _select_problems, run_run

PROBLEMS_DIR = Path(__file__).resolve().parent.parent / "problems"


# ── _select_problems ───────────────────────────────────────────────

@pytest.mark.skipif(not PROBLEMS_DIR.exists(), reason="problems dir not found")
def test_select_all():
    dirs = _select_problems(PROBLEMS_DIR)
    assert len(dirs) >= 50


@pytest.mark.skipif(not PROBLEMS_DIR.exists(), reason="problems dir not found")
def test_select_pilot():
    dirs = _select_problems(PROBLEMS_DIR, pilot=True)
    nums = {int(d.name.split("_")[0]) for d in dirs}
    assert nums == {1, 21, 50}


@pytest.mark.skipif(not PROBLEMS_DIR.exists(), reason="problems dir not found")
def test_select_by_numbers():
    dirs = _select_problems(PROBLEMS_DIR, problem_nums="1,2,3")
    nums = {int(d.name.split("_")[0]) for d in dirs}
    assert nums == {1, 2, 3}


@pytest.mark.skipif(not PROBLEMS_DIR.exists(), reason="problems dir not found")
def test_select_category():
    dirs = _select_problems(PROBLEMS_DIR, category="algorithm")
    for d in dirs:
        meta_file = d / "metadata.json"
        if meta_file.exists():
            meta = json.loads(meta_file.read_text())
            assert meta.get("category") == "algorithm"


# ── dry-run (no LLM) ──────────────────────────────────────────────

@pytest.mark.skipif(not PROBLEMS_DIR.exists(), reason="problems dir not found")
def test_run_dry_run_human(capsys):
    rc = main(["run", "--model", "test-model", "--pilot", "--dry-run"])
    assert rc == 0
    out = capsys.readouterr().out
    assert "DRY RUN" in out
    assert "test-model" in out


@pytest.mark.skipif(not PROBLEMS_DIR.exists(), reason="problems dir not found")
def test_run_dry_run_json(capsys):
    rc = main(["run", "--model", "test-model", "--pilot", "--dry-run", "--json"])
    assert rc == 0
    data = json.loads(capsys.readouterr().out)
    assert data["dry_run"] is True
    assert data["model"] == "test-model"
    assert data["problems"] == 3


@pytest.mark.skipif(not PROBLEMS_DIR.exists(), reason="problems dir not found")
def test_run_dry_run_category_json(capsys):
    rc = main(["run", "--model", "m", "--category", "algorithm", "--dry-run", "--json"])
    assert rc == 0
    data = json.loads(capsys.readouterr().out)
    assert data["problems"] > 0


# ── missing model error ───────────────────────────────────────────

def test_run_no_model_returns_error(capsys):
    rc = main(["run", "--dry-run"])
    assert rc == 1


def test_run_no_model_json_returns_error(capsys):
    rc = main(["run", "--dry-run", "--json"])
    assert rc == 1
    data = json.loads(capsys.readouterr().out)
    assert "error" in data


# ── run_run direct API ────────────────────────────────────────────

@pytest.mark.skipif(not PROBLEMS_DIR.exists(), reason="problems dir not found")
def test_run_run_dry_run_returns_zero():
    rc = run_run(
        model="test-model",
        api_base="http://localhost:9999/v1",
        api_key="test-key",
        pilot=True,
        dry_run=True,
        json_output=True,
    )
    assert rc == 0


def test_run_run_empty_selection_returns_error():
    rc = run_run(
        model="test-model",
        api_base="http://localhost:9999/v1",
        api_key="test-key",
        category="nonexistent-category-xyz",
        dry_run=True,
        json_output=True,
    )
    assert rc == 1
