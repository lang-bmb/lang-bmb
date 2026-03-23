"""Performance measurement utilities with IQR outlier removal."""

from __future__ import annotations

import statistics
import subprocess
import sys
import time
from pathlib import Path


def measure_binary(
    binary_path: Path,
    args: list[str] | None = None,
    stdin: str = "",
    iterations: int = 10,
) -> dict:
    """Run *binary_path* repeatedly and return timing statistics.

    Returns ``{median_ns, times_ns, n}`` after removing IQR outliers.
    """
    if args is None:
        args = []

    cmd: list[str] = [str(binary_path)] + args
    times_ns: list[int] = []

    for _ in range(iterations):
        start = time.perf_counter_ns()
        subprocess.run(
            cmd,
            input=stdin,
            capture_output=True,
            text=True,
            timeout=30,
        )
        elapsed = time.perf_counter_ns() - start
        times_ns.append(elapsed)

    # IQR outlier removal (need at least 4 data points)
    if len(times_ns) >= 4:
        sorted_t = sorted(times_ns)
        q1 = sorted_t[len(sorted_t) // 4]
        q3 = sorted_t[3 * len(sorted_t) // 4]
        iqr = q3 - q1
        lower = q1 - 1.5 * iqr
        upper = q3 + 1.5 * iqr
        filtered = [t for t in times_ns if lower <= t <= upper]
        if not filtered:
            filtered = times_ns  # fallback: keep all
    else:
        filtered = times_ns

    median_ns = int(statistics.median(filtered))

    return {
        "median_ns": median_ns,
        "times_ns": filtered,
        "n": len(filtered),
    }
