"""Tests for language runners (BMB, Rust, Python) and perf measurement."""

import pytest
import sys
from pathlib import Path
from abc import ABC

from runners.base import RunResult, RunnerBase
from runners.bmb_runner import BmbRunner
from runners.rust_runner import RustRunner
from runners.python_runner import PythonRunner
from runners.perf import measure_binary


# ---------------------------------------------------------------------------
# RunResult dataclass tests
# ---------------------------------------------------------------------------

class TestRunResult:
    def test_fields_defaults(self):
        r = RunResult(compiled=True, test_passed=True, error_msg="")
        assert r.compiled is True
        assert r.test_passed is True
        assert r.error_msg == ""
        assert r.perf_ns is None
        assert r.raw_output == ""

    def test_fields_explicit(self):
        r = RunResult(
            compiled=False,
            test_passed=False,
            error_msg="boom",
            perf_ns=12345,
            raw_output="some output",
        )
        assert r.compiled is False
        assert r.error_msg == "boom"
        assert r.perf_ns == 12345
        assert r.raw_output == "some output"


# ---------------------------------------------------------------------------
# RunnerBase abstract tests
# ---------------------------------------------------------------------------

class TestRunnerBase:
    def test_is_abstract(self):
        assert issubclass(RunnerBase, ABC)
        with pytest.raises(TypeError):
            RunnerBase()  # type: ignore

    def test_subclasses_must_implement(self):
        """A subclass that doesn't implement abstract methods cannot be instantiated."""
        class Incomplete(RunnerBase):
            pass
        with pytest.raises(TypeError):
            Incomplete()  # type: ignore


# ---------------------------------------------------------------------------
# BmbRunner tests
# ---------------------------------------------------------------------------

class TestBmbRunner:
    @pytest.fixture
    def runner(self):
        return BmbRunner()

    @pytest.fixture
    def work(self, tmp_path):
        return tmp_path

    def test_build_valid_code(self, runner, work):
        source = "fn main() -> i64 = 42;"
        result = runner.build(source, work)
        assert result.compiled is True
        assert result.error_msg == ""
        # Binary should exist
        if sys.platform == "win32":
            assert (work / "solution.exe").exists()
        else:
            assert (work / "solution").exists()

    def test_build_invalid_code(self, runner, work):
        source = "this is not valid bmb code }{}{}{}"
        result = runner.build(source, work)
        assert result.compiled is False
        assert result.error_msg != ""

    def test_test_with_valid_build(self, runner, work):
        source = "fn main() -> i64 = { let _x = println(42); 0 };"
        runner.build(source, work)
        tests = [{"args": [], "expected_stdout": "42\n"}]
        result = runner.test(work, tests)
        assert result.test_passed is True

    def test_test_with_wrong_expected(self, runner, work):
        source = "fn main() -> i64 = { let _x = println(42); 0 };"
        runner.build(source, work)
        tests = [{"args": [], "expected_stdout": "999\n"}]
        result = runner.test(work, tests)
        assert result.test_passed is False

    def test_measure_perf_returns_int(self, runner, work):
        source = "fn main() -> i64 = 42;"
        runner.build(source, work)
        ns = runner.measure_perf(work, iterations=3)
        assert isinstance(ns, int)
        assert ns > 0


# ---------------------------------------------------------------------------
# RustRunner tests
# ---------------------------------------------------------------------------

class TestRustRunner:
    @pytest.fixture
    def runner(self):
        return RustRunner()

    @pytest.fixture
    def work(self, tmp_path):
        return tmp_path

    def test_build_valid_code(self, runner, work):
        source = 'fn main() { println!("42"); }'
        result = runner.build(source, work)
        assert result.compiled is True
        assert result.error_msg == ""
        if sys.platform == "win32":
            assert (work / "solution.exe").exists()
        else:
            assert (work / "solution").exists()

    def test_build_invalid_code(self, runner, work):
        source = "fn main( { not valid rust"
        result = runner.build(source, work)
        assert result.compiled is False
        assert result.error_msg != ""


# ---------------------------------------------------------------------------
# PythonRunner tests
# ---------------------------------------------------------------------------

class TestPythonRunner:
    @pytest.fixture
    def runner(self):
        return PythonRunner()

    @pytest.fixture
    def work(self, tmp_path):
        return tmp_path

    def test_build_always_succeeds(self, runner, work):
        source = "print('hello')"
        result = runner.build(source, work)
        assert result.compiled is True
        assert result.error_msg == ""
        assert (work / "solution.py").exists()

    def test_build_even_bad_syntax_succeeds(self, runner, work):
        """Build (write) always succeeds; errors surface at test time."""
        source = "this is not valid python {{{}}}"
        result = runner.build(source, work)
        assert result.compiled is True

    def test_test_with_valid_code(self, runner, work):
        source = "print('hello')"
        runner.build(source, work)
        tests = [{"args": [], "expected_stdout": "hello\n"}]
        result = runner.test(work, tests)
        assert result.test_passed is True

    def test_measure_perf_returns_none(self, runner, work):
        source = "print('hello')"
        runner.build(source, work)
        result = runner.measure_perf(work, iterations=3)
        assert result is None


# ---------------------------------------------------------------------------
# perf.measure_binary tests
# ---------------------------------------------------------------------------

class TestMeasureBinary:
    def test_measure_returns_dict(self, tmp_path):
        # Create a trivial script as a "binary"
        if sys.platform == "win32":
            script = tmp_path / "test.bat"
            script.write_text("@echo off\necho hello\n")
            binary = script
        else:
            script = tmp_path / "test.sh"
            script.write_text("#!/bin/sh\necho hello\n")
            script.chmod(0o755)
            binary = script
        result = measure_binary(binary, iterations=3)
        assert "median_ns" in result
        assert "times_ns" in result
        assert "n" in result
        assert isinstance(result["median_ns"], int)
        assert result["n"] <= 3
