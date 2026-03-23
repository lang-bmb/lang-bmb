"""Abstract runner interface for language runners."""

from __future__ import annotations

from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from pathlib import Path


@dataclass
class RunResult:
    """Result of a build, test, or performance measurement."""

    compiled: bool
    test_passed: bool
    error_msg: str
    perf_ns: int | None = None
    raw_output: str = ""


class RunnerBase(ABC):
    """Abstract base class for language runners."""

    @abstractmethod
    def build(self, source_code: str, work_dir: Path) -> RunResult:
        """Compile/prepare source code in *work_dir*."""
        ...

    @abstractmethod
    def test(self, work_dir: Path, tests: list[dict]) -> RunResult:
        """Run test cases against the built artefact."""
        ...

    @abstractmethod
    def measure_perf(self, work_dir: Path, iterations: int = 10) -> int | None:
        """Return median execution time in nanoseconds, or None."""
        ...
