"""Dashboard unit tests."""

from __future__ import annotations

import json
import pytest
from bmb_ai_bench.cli import main


def test_dashboard_total_nonzero(capsys):
    rc = main(["dashboard", "--json"])
    assert rc == 0
    data = json.loads(capsys.readouterr().out)
    assert data["total"] >= 50


def test_dashboard_categories_present(capsys):
    rc = main(["dashboard", "--json"])
    assert rc == 0
    data = json.loads(capsys.readouterr().out)
    cats = set(data["categories"].keys())
    expected = {"algorithm", "system", "contract", "performance", "practical", "edge", "integration"}
    assert cats & expected, f"No expected categories found, got: {cats}"


def test_dashboard_difficulty_breakdown(capsys):
    rc = main(["dashboard", "--json"])
    assert rc == 0
    data = json.loads(capsys.readouterr().out)
    assert "difficulty_breakdown" in data
    total_diff = sum(data["difficulty_breakdown"].values())
    assert total_diff == data["total"]
