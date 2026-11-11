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
        self.assertIn("집합별 최소 차이(ADR-0086 §2): m1b 50%p", text)

    def test_brb_rows_are_writes_of_their_arm(self):
        with tempfile.TemporaryDirectory() as d:
            files = []
            for k, succ in enumerate([[True, True], [True, False]]):
                path = os.path.join(d, f"b{k}.jsonl")
                with open(path, "w") as f:
                    f.write(json.dumps({"summary": True, "task_set": "brb-write", "arm": "bmb",
                                        "agent": "openai:qwen3.8-27b", "commit": "c", "repeat": k + 1}) + "\n")
                    for i, ok in enumerate(succ):
                        f.write(json.dumps({"id": f"t{i}", "arm": "bmb", "success": ok}) + "\n")
                files.append(path)
            files.append(run_file(d, 1, [True, False, False, False], task_set="bvb"))
            files.append(run_file(d, 2, [True, True, True, False], task_set="bvb"))
            text = repeat.summary(files, 0.6)
        self.assertIn("## brb-write-bmb: 반복 2회", text)
        self.assertIn("| 작성 | 2/2, 1/2 | 75.0% |", text)
        self.assertIn("집합별 최소 차이(ADR-0086 §2): brb-write-bmb 50%p, bvb 50%p", text)

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


class Diff(unittest.TestCase):
    """ADR-0112: the difference of two arms on the same tasks."""

    A, B = [20, 20, 21], [24, 23, 22]  # 2R-05 (qwen, 25 write tasks), before and after

    def test_welch_and_the_sampling_floor(self):
        ra, rb = [k / 25 for k in self.A], [k / 25 for k in self.B]
        d, lo, hi = repeat.welch(ra, rb)
        self.assertAlmostEqual(d, 8 / 75, places=6)
        h = 3.182 * (((1 / 3) / 3 + 1.0 / 3) ** 0.5) / 25  # df 3.2 rounded down to 3
        self.assertAlmostEqual(lo, d - h, places=6)
        self.assertGreater(lo, 0)  # Welch alone excludes 0
        nl, nh = repeat.newcombe(61, 75, 69, 75)
        self.assertLess(nl, 0)  # the pooled-trial floor does not
        self.assertGreater(nh, hi - 0.05)

    def test_agreeing_runs_keep_the_sampling_interval(self):
        d, lo, hi = repeat.welch([1.0, 1.0], [1.0, 1.0])
        self.assertEqual((d, lo, hi), (0.0, 0.0, 0.0))
        nl, nh = repeat.newcombe(50, 50, 50, 50)
        self.assertLess(nl, 0)
        self.assertGreater(nh, 0)

    def test_exact_permutation(self):
        # Three against three, fully separated: 2 of the 20 labelings are as extreme.
        self.assertAlmostEqual(repeat.permutation_p([0.80, 0.80, 0.84], [0.96, 0.92, 0.88]), 0.1)
        # Four against four: 2 of 70.
        self.assertAlmostEqual(repeat.permutation_p([0.1, 0.2, 0.3, 0.4], [0.5, 0.6, 0.7, 0.8]), 2 / 70)
        self.assertEqual(repeat.permutation_p([0.5, 0.5], [0.5, 0.5]), 1.0)

    def test_verdict(self):
        self.assertEqual(repeat.diff_verdict(-0.01, 0.2, 5, 5), "변동 안")
        self.assertEqual(repeat.diff_verdict(0.01, 0.2, 4, 4), "B가 높음")
        self.assertEqual(repeat.diff_verdict(-0.2, -0.01, 4, 5), "A가 높음")
        self.assertEqual(repeat.diff_verdict(0.01, 0.2, 3, 4), "B가 높음(잠정: 팔당 4회 미만)")

    def test_median_interval(self):
        med, iv = repeat.median_interval([1.0, 2.0, 4.0])
        self.assertEqual((med, iv), (2.0, None))  # three tasks cannot give 95%
        med, iv = repeat.median_interval([float(i) for i in range(1, 10)])
        self.assertEqual(med, 5.0)
        self.assertEqual(iv, (2.0, 8.0))  # P(Bin(9, 1/2) <= 1) = 0.0195

    def test_report(self):
        with tempfile.TemporaryDirectory() as d:
            def side(commit, runs, tokens):
                files = []
                for k, n_ok in enumerate(runs, 1):
                    path = os.path.join(d, f"{commit}-{k}.jsonl")
                    with open(path, "w") as f:
                        f.write(json.dumps({"summary": True, "task_set": "m1b", "agent": "openai:qwen3.8-27b",
                                            "commit": commit, "repeat": k}) + "\n")
                        for i in range(10):
                            f.write(json.dumps({"id": f"w{i}", "kind": "write", "success": i < n_ok,
                                                "model_usage": {"input_tokens": tokens, "output_tokens": 0}}) + "\n")
                        f.write(json.dumps({"id": "n0", "kind": "navigate", "success": True}) + "\n")
                    files.append(path)
                return files
            a, b = side("before", [2, 2, 3, 2], 1000), side("after", [9, 8, 9, 9], 500)
            text = repeat.diff(a, b)
            self.assertIn("B(m1b, 4회) − A(m1b, 4회), 작성 과제 10개", text)
            self.assertIn("| A | 2/10, 2/10, 3/10, 2/10 | 22.5% |", text)
            self.assertIn("성공률 차이: +65.0%p", text)
            self.assertIn("→ B가 높음\n", text)
            self.assertIn("과제별 토큰 비율 B/A(10과제): 중앙값 0.50, 95% 구간 0.50~0.50, 기하 평균 0.50", text)
            same = repeat.diff(a, a[:])  # the same runs on both sides
            self.assertIn("→ 변동 안", same)
            with self.assertRaises(SystemExit):  # two sets on one side
                repeat.diff(a + [run_file(d, 1, [True], task_set="bvb")], b)


if __name__ == "__main__":
    unittest.main()
