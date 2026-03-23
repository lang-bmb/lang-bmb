"""Tests for experiment orchestrator and problem registry."""

from __future__ import annotations

import json
import tempfile
from pathlib import Path
from unittest.mock import MagicMock, patch

import pytest

from problems.registry import Problem, load_problem, load_all_problems
from orchestrator.experiment import ExperimentRunner, MAX_LOOPS, AttemptRecord, RunRecord
from runners.base import RunResult


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _make_problem(**overrides):
    defaults = dict(
        name="test",
        number=0,
        category="algorithm",
        description="Add two numbers",
        tests=[{"args": ["2", "3"], "expected": "5"}],
        baseline_c="",
    )
    defaults.update(overrides)
    return Problem(**defaults)


# ---------------------------------------------------------------------------
# Registry tests
# ---------------------------------------------------------------------------

class TestRegistry:
    def test_load_problem(self, tmp_path):
        d = tmp_path / "001_two_sum"
        d.mkdir()
        (d / "problem.md").write_text("Find two numbers.", encoding="utf-8")
        (d / "tests.json").write_text(
            json.dumps([{"args": ["1", "2"], "expected": "3"}]), encoding="utf-8"
        )
        (d / "baseline.c").write_text("int main(){return 0;}", encoding="utf-8")

        p = load_problem(d)
        assert p.name == "two_sum"
        assert p.number == 1
        assert p.category == "algorithm"
        assert "two numbers" in p.description.lower()
        assert len(p.tests) == 1
        assert p.baseline_c.startswith("int main")

    def test_load_problem_no_baseline(self, tmp_path):
        d = tmp_path / "005_sort"
        d.mkdir()
        (d / "problem.md").write_text("Sort an array.", encoding="utf-8")
        (d / "tests.json").write_text("[]", encoding="utf-8")

        p = load_problem(d)
        assert p.baseline_c == ""
        assert p.number == 5

    def test_load_problem_system_category(self, tmp_path):
        d = tmp_path / "015_tcp_echo"
        d.mkdir()
        (d / "problem.md").write_text("TCP echo server.", encoding="utf-8")
        (d / "tests.json").write_text("[]", encoding="utf-8")

        p = load_problem(d)
        assert p.category == "system"

    def test_load_problem_contract_category(self, tmp_path):
        d = tmp_path / "025_verified_add"
        d.mkdir()
        (d / "problem.md").write_text("Verified addition.", encoding="utf-8")
        (d / "tests.json").write_text("[]", encoding="utf-8")

        p = load_problem(d)
        assert p.category == "contract"

    def test_load_all_problems(self, tmp_path):
        for name in ["003_fizzbuzz", "001_hello", "010_sort"]:
            d = tmp_path / name
            d.mkdir()
            (d / "problem.md").write_text("desc", encoding="utf-8")
            (d / "tests.json").write_text("[]", encoding="utf-8")
        # non-digit dir should be skipped
        (tmp_path / "readme").mkdir()

        problems = load_all_problems(tmp_path)
        assert len(problems) == 3
        assert [p.number for p in problems] == [1, 3, 10]


# ---------------------------------------------------------------------------
# Experiment orchestrator tests
# ---------------------------------------------------------------------------

class TestExperiment:
    def test_first_attempt_success(self):
        """Loop count = 1 when first attempt passes."""
        llm = MagicMock()
        llm.generate.return_value = "```bmb\nfn main() = 42;\n```"

        runner = MagicMock()
        runner.build.return_value = RunResult(
            compiled=True, test_passed=False, error_msg=""
        )
        runner.test.return_value = RunResult(
            compiled=True, test_passed=True, error_msg=""
        )

        exp = ExperimentRunner(llm, {"bmb": runner}, None, None)
        record = exp.run_single(_make_problem(), "bmb_contract", 1, MagicMock())
        assert record.loop_count == 1
        assert record.final_correct is True

    def test_max_loops_exceeded(self):
        """Loop count = MAX_LOOPS+1 when all attempts fail build."""
        llm = MagicMock()
        llm.generate.return_value = "```bmb\nbad\n```"

        runner = MagicMock()
        runner.build.return_value = RunResult(
            compiled=False, test_passed=False, error_msg="syntax error"
        )

        exp = ExperimentRunner(llm, {"bmb": runner}, None, None)
        record = exp.run_single(_make_problem(), "bmb_contract", 1, MagicMock())
        assert record.loop_count == MAX_LOOPS + 1
        assert record.final_correct is False
        assert len(record.attempts) == MAX_LOOPS

    def test_build_fail_then_success(self):
        """Two loops: first fails build, second succeeds."""
        llm = MagicMock()
        llm.generate.side_effect = [
            "```bmb\nbad\n```",
            "```bmb\ngood\n```",
        ]

        runner = MagicMock()
        runner.build.side_effect = [
            RunResult(compiled=False, test_passed=False, error_msg="error"),
            RunResult(compiled=True, test_passed=False, error_msg=""),
        ]
        runner.test.return_value = RunResult(
            compiled=True, test_passed=True, error_msg=""
        )

        exp = ExperimentRunner(llm, {"bmb": runner}, None, None)
        record = exp.run_single(_make_problem(), "bmb_contract", 1, MagicMock())
        assert record.loop_count == 2
        assert record.final_correct is True

    def test_loop_types_counted(self):
        """Verify loop type classification is recorded."""
        llm = MagicMock()
        llm.generate.side_effect = [
            "```bmb\nbad\n```",
            "```bmb\ngood\n```",
        ]

        runner = MagicMock()
        runner.build.side_effect = [
            RunResult(
                compiled=False, test_passed=False, error_msg="unexpected token }"
            ),
            RunResult(compiled=True, test_passed=False, error_msg=""),
        ]
        runner.test.return_value = RunResult(
            compiled=True, test_passed=True, error_msg=""
        )

        exp = ExperimentRunner(llm, {"bmb": runner}, None, None)
        record = exp.run_single(_make_problem(), "bmb_contract", 1, MagicMock())
        assert record.loop_types["B"] >= 1  # syntax error

    def test_nocontract_strips_contracts(self):
        """bmb_nocontract condition strips pre/post from generated code."""
        llm = MagicMock()
        llm.generate.return_value = (
            "```bmb\nfn foo(x: i64) -> i64\n    pre x > 0\n= x;\n```"
        )

        runner = MagicMock()
        runner.build.return_value = RunResult(
            compiled=True, test_passed=False, error_msg=""
        )
        runner.test.return_value = RunResult(
            compiled=True, test_passed=True, error_msg=""
        )

        exp = ExperimentRunner(llm, {"bmb": runner}, None, None)
        record = exp.run_single(_make_problem(), "bmb_nocontract", 1, MagicMock())
        # Check that the code passed to build had contracts stripped
        build_call_args = runner.build.call_args[0][0]  # first positional arg
        assert "pre" not in build_call_args

    def test_test_fail_then_success(self):
        """Test failure triggers feedback loop, second attempt passes."""
        llm = MagicMock()
        llm.generate.side_effect = [
            "```bmb\nfn main() = 41;\n```",
            "```bmb\nfn main() = 42;\n```",
        ]

        runner = MagicMock()
        runner.build.return_value = RunResult(
            compiled=True, test_passed=False, error_msg=""
        )
        runner.test.side_effect = [
            RunResult(
                compiled=True,
                test_passed=False,
                error_msg="expected 42, got 41",
            ),
            RunResult(compiled=True, test_passed=True, error_msg=""),
        ]

        exp = ExperimentRunner(llm, {"bmb": runner}, None, None)
        record = exp.run_single(_make_problem(), "bmb_contract", 1, MagicMock())
        assert record.loop_count == 2
        assert record.final_correct is True
        assert record.loop_types["D"] >= 1  # test failure loop

    def test_perf_ratio_computed(self):
        """When C baseline exists, perf_ratio is computed."""
        llm = MagicMock()
        llm.generate.return_value = "```bmb\nfn main() = 42;\n```"

        runner = MagicMock()
        runner.build.return_value = RunResult(
            compiled=True, test_passed=False, error_msg=""
        )
        runner.test.return_value = RunResult(
            compiled=True, test_passed=True, error_msg=""
        )
        runner.measure_perf.return_value = 500_000  # 500us

        exp = ExperimentRunner(llm, {"bmb": runner}, None, None)
        # Patch _measure_c_baseline to return a known value
        exp._measure_c_baseline = MagicMock(return_value=250_000)

        record = exp.run_single(
            _make_problem(baseline_c="int main(){}"), "bmb_contract", 1, MagicMock()
        )
        assert record.final_correct is True
        assert record.perf_ratio == pytest.approx(2.0)

    def test_perf_ratio_none_without_baseline(self):
        """When no C baseline, perf_ratio is None."""
        llm = MagicMock()
        llm.generate.return_value = "```bmb\nfn main() = 42;\n```"

        runner = MagicMock()
        runner.build.return_value = RunResult(
            compiled=True, test_passed=False, error_msg=""
        )
        runner.test.return_value = RunResult(
            compiled=True, test_passed=True, error_msg=""
        )

        exp = ExperimentRunner(llm, {"bmb": runner}, None, None)
        record = exp.run_single(_make_problem(), "bmb_contract", 1, MagicMock())
        assert record.perf_ratio is None

    def test_rust_condition_uses_rust_runner(self):
        """Rust condition uses the rust runner key."""
        llm = MagicMock()
        llm.generate.return_value = "```rust\nfn main() {}\n```"

        rust_runner = MagicMock()
        rust_runner.build.return_value = RunResult(
            compiled=True, test_passed=False, error_msg=""
        )
        rust_runner.test.return_value = RunResult(
            compiled=True, test_passed=True, error_msg=""
        )

        exp = ExperimentRunner(llm, {"rust": rust_runner}, None, None)
        record = exp.run_single(_make_problem(), "rust", 1, MagicMock())
        assert record.final_correct is True
        rust_runner.build.assert_called_once()

    def test_save_run(self, tmp_path):
        """save_run persists RunRecord as JSON."""
        llm = MagicMock()
        exp = ExperimentRunner(llm, {}, None, None)

        record = RunRecord(
            run_id=1,
            condition="bmb_contract",
            loop_count=2,
            loop_types={"A": 0, "B": 1, "C": 0, "D": 0},
            final_correct=True,
            perf_ratio=1.05,
            attempts=[
                AttemptRecord(
                    attempt=1,
                    code="bad",
                    compiled=False,
                    test_passed=False,
                    error={"type": "compile_error"},
                    loop_type="B",
                ),
                AttemptRecord(
                    attempt=2,
                    code="good",
                    compiled=True,
                    test_passed=True,
                    error=None,
                    loop_type=None,
                ),
            ],
        )

        out = tmp_path / "result.json"
        exp.save_run(record, out)
        data = json.loads(out.read_text(encoding="utf-8"))
        assert data["run_id"] == 1
        assert data["final_correct"] is True
        assert len(data["attempts"]) == 2
