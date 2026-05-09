"""CLI smoke tests — no BMB compiler required."""

from __future__ import annotations

import json
import pytest
from bmb_ai_bench.cli import main


def test_no_command_exits_zero():
    assert main([]) == 0


def test_list_all(capsys):
    rc = main(["list"])
    assert rc == 0
    out = capsys.readouterr().out
    assert "Total:" in out


def test_list_json(capsys):
    rc = main(["list", "--json"])
    assert rc == 0
    data = json.loads(capsys.readouterr().out)
    assert isinstance(data, list)
    assert len(data) > 0
    first = data[0]
    assert "number" in first
    assert "name" in first
    assert "category" in first
    assert "difficulty" in first


def test_list_category_filter(capsys):
    rc = main(["list", "--category", "algorithm"])
    assert rc == 0
    out = capsys.readouterr().out
    assert "Total:" in out


def test_list_json_category_filter(capsys):
    rc = main(["list", "--category", "algorithm", "--json"])
    assert rc == 0
    data = json.loads(capsys.readouterr().out)
    assert isinstance(data, list)
    for p in data:
        assert p["category"] == "algorithm"


def test_dashboard(capsys):
    rc = main(["dashboard"])
    assert rc == 0
    out = capsys.readouterr().out
    assert "Total problems:" in out
    assert "tracking only" in out


def test_dashboard_json(capsys):
    rc = main(["dashboard", "--json"])
    assert rc == 0
    data = json.loads(capsys.readouterr().out)
    assert "total" in data
    assert "categories" in data
    assert data["total"] > 0
