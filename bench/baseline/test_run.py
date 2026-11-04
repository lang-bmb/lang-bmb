#!/usr/bin/env python3
"""bench/baseline/run.py: a ratio above the limit fails only when a second
measurement confirms it (backlog INF-25)."""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import run  # noqa: E402


def test_noise_in_one_measurement_is_not_a_failure():
    first = {"gather": 1.053, "sieve": 0.97}
    assert run.confirmed(first, {"gather": 0.99}, 1.05) == []


def test_a_regression_in_both_measurements_fails():
    first = {"gather": 1.2, "sieve": 0.97}
    assert run.confirmed(first, {"gather": 1.18}, 1.05) == [("gather", 1.2, 1.18)]


def test_ratio_is_the_median_ratio():
    t = {("b", "bmb"): [3.0, 1.0, 2.0], ("b", "safe"): [1.0, 1.0, 4.0]}
    assert run.ratio(t, "b") == 2.0


if __name__ == "__main__":
    for name, fn in list(globals().items()):
        if name.startswith("test_"):
            fn()
    print("baseline run: ok")
