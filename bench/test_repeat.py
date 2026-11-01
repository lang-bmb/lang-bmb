#!/usr/bin/env python3
"""Tests for repeat.py (ADR-0077, backlog T0-12). No model, no network."""
import json
import os
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import repeat  # noqa: E402


def run_file(d, k, successes, kind="write", task_set="m1b", agent="openai:qwen3.8-27b", commit="c"):
    path = os.path.join(d, f"r{k}-{task_set}-{commit}-{agent.replace(':', '_')}.jsonl")
    with open(path, "w") as f:
        f.write(json.dumps({"summary": True, "task_set": task_set, "agent": agent, "commit": commit,
                            "repeat": k}) + "\n")
        for i, ok in enumerate(successes):
            f.write(json.dumps({"id": f"t{i}", "kind": kind, "success": ok}) + "\n")
    return path


class Interval(unittest.TestCase):
    def test_t_interval_over_runs(self):
        # Five runs of ten tasks: 5, 6, 7, 6, 6 successes -> mean 0.6, s = 0.0707.
        lo, hi = repeat.interval([0.5, 0.6, 0.7, 0.6, 0.6], 30, 50)
        h = 2.776 * 0.070711 / 5 ** 0.5
        wl, wh = repeat.wilson(30, 50)
        self.assertAlmostEqual(lo, min(0.6 - h, wl), places=4)
        self.assertAlmostEqual(hi, max(0.6 + h, wh), places=4)

    def test_agreeing_runs_keep_the_sampling_interval(self):
        # Identical runs: s = 0, so the Wilson interval of the pooled trials stays.
        lo, hi = repeat.interval([0.8] * 5, 40, 50)
        self.assertEqual((lo, hi), repeat.wilson(40, 50))
        self.assertLess(lo, 0.8)
        self.assertGreater(hi, 0.8)

    def test_verdict(self):
        self.assertEqual(repeat.verdict(0.61, 0.8, 0.6), "충족")
        self.assertEqual(repeat.verdict(0.6, 0.8, 0.6), "충족")
        self.assertEqual(repeat.verdict(0.3, 0.59, 0.6), "미충족")
        self.assertEqual(repeat.verdict(0.5, 0.7, 0.6), "미정")


class Summary(unittest.TestCase):
    def test_table_stability_and_minimum_difference(self):
        with tempfile.TemporaryDirectory() as d:
            runs = [[True, True, False, False], [True, False, False, False], [True, True, True, False]]
            files = [run_file(d, k + 1, s) for k, s in enumerate(runs)]
            text = repeat.summary(files, 0.6)
        self.assertIn("## m1b: 반복 3회", text)
        self.assertIn("| 작성 | 2/4, 1/4, 3/4 | 50.0% |", text)
        # t0 always, t3 never, t1 and t2 unstable.
        self.assertIn("| 1 / 1 / 2 |", text)
        self.assertIn("흔들린 과제: t1, t2", text)
        self.assertIn("최소 차이(ADR-0077 §4): 50%p", text)

    def test_mixed_runs_are_refused(self):
        with tempfile.TemporaryDirectory() as d:
            a = run_file(d, 1, [True])
            for bad in (run_file(d, 2, [True], commit="other"), run_file(d, 1, [False], agent="openai:x")):
                with self.assertRaises(SystemExit):
                    repeat.summary([a, bad], 0.6)
            with self.assertRaises(SystemExit):  # the same repeat twice
                repeat.summary([a, a], 0.6)
            plain = os.path.join(d, "plain.jsonl")
            with open(plain, "w") as f:
                f.write(json.dumps({"summary": True, "task_set": "m1b"}) + "\n")
            with self.assertRaises(SystemExit):
                repeat.summary([plain], 0.6)


if __name__ == "__main__":
    unittest.main()
